// /backend/src/trader.rs
// Orchestre la boucle de trading automatique

use std::collections::{HashMap, HashSet, VecDeque};
use tokio::time::{interval, Duration};

use crate::config::Config;
use crate::etoro::EtoroClient;
use crate::models::{ClosePositionRequest, CreateOrderRequest};
use crate::strategy::{compute_signal, compute_sl_tp, Signal};

pub struct Trader {
    client: EtoroClient,
    /// `instrument_id` → symbol
    instruments: HashMap<i64, String>,
    /// `instrument_id` → fenêtre glissante de prix ask
    price_windows: HashMap<i64, VecDeque<f64>>,
    amount: f64,
    leverage: i64,
    window_size: usize,
    /// Compteur de confirmations consécutives par instrument.
    /// Positif = signaux Buy accumulés, négatif = signaux Sell.
    confirm_counts: HashMap<i64, i8>,
    /// Nombre de confirmations consécutives requises avant d'agir.
    confirm_required: i8,
    /// Instruments pour lesquels on a déjà une position ouverte (suivi local).
    local_open: HashSet<i64>,
}

impl Trader {
    /// Résout les symboles en `instrument_id` au démarrage, puis lance la boucle.
    pub async fn start(client: EtoroClient, cfg: &Config) {
        let mut instruments = HashMap::new();

        for symbol in &cfg.trader_symbols {
            match client.search_instrument(symbol).await {
                Ok(resp) => {
                    if let Some(item) = resp.items.iter().find(|i| &i.internal_symbol_full == symbol) {
                        tracing::info!("Resolved {} → instrument_id={}", symbol, item.instrument_id);
                        instruments.insert(item.instrument_id, symbol.clone());
                    } else {
                        tracing::warn!("Symbol {} not found in search results", symbol);
                    }
                }
                Err(e) => tracing::error!("Failed to resolve symbol {}: {:?}", symbol, e),
            }
        }

        if instruments.is_empty() {
            tracing::error!("No instruments resolved, trader will not start");
            return;
        }

        let mut trader = Self {
            client,
            price_windows: instruments.keys().map(|&id| (id, VecDeque::new())).collect(),
            confirm_counts: instruments.keys().map(|&id| (id, 0i8)).collect(),
            instruments,
            amount: cfg.trader_amount,
            leverage: cfg.trader_leverage,
            window_size: cfg.trader_window_size,
            confirm_required: cfg.trader_confirm_ticks,
            local_open: HashSet::new(),
        };

        let mut ticker = interval(Duration::from_secs(cfg.trader_interval_secs));
        loop {
            ticker.tick().await;
            trader.tick().await;
        }
    }

    async fn tick(&mut self) {
        let ids: Vec<i64> = self.instruments.keys().copied().collect();

        // 1. Prix actuels — un appel par instrument (l'API ne renvoie qu'un résultat par requête multi-ID)
        let mut all_rates = Vec::new();
        for &id in &ids {
            match self.client.get_rates(&[id]).await {
                Ok(r) => all_rates.extend(r.rates),
                Err(e) => tracing::error!("get_rates({}) failed: {:?}", id, e),
            }
        }

        // 2. Portfolio (positions ouvertes) — optionnel, on utilise local_open en fallback
        let open_positions: HashMap<i64, crate::models::Position> = match self.client.get_portfolio().await {
            Ok(p) => {
                self.local_open = p.positions.iter().map(|pos| pos.instrument_id).collect();
                tracing::info!(
                    "Tick — solde: ${:.2} | positions ouvertes: {}",
                    p.credit,
                    p.positions.len()
                );
                p.positions.into_iter().map(|p| (p.instrument_id, p)).collect()
            }
            Err(e) => {
                tracing::warn!("get_portfolio indisponible ({:?}), utilisation du suivi local ({} positions)", e, self.local_open.len());
                HashMap::new()
            }
        };

        for rate in &all_rates {
            let id = rate.instrument_id;
            let symbol = self.instruments.get(&id).cloned().unwrap_or_default();
            // Pour la fermeture, on a besoin du position_id depuis le vrai portfolio
            let _ = &open_positions;

            // 3. Mise à jour fenêtre glissante
            let window = self.price_windows.entry(id).or_default();
            window.push_back(rate.ask);
            if window.len() > self.window_size {
                window.pop_front();
            }

            let prices: Vec<f64> = window.iter().copied().collect();
            let signal = compute_signal(&prices);

            // 4. Compteur de confirmations
            let count = self.confirm_counts.entry(id).or_insert(0);
            match signal {
                Signal::Buy => {
                    *count = if *count >= 0 { count.saturating_add(1) } else { 1 };
                }
                Signal::Sell => {
                    *count = if *count <= 0 { count.saturating_sub(1) } else { -1 };
                }
                Signal::Hold => {
                    *count = 0;
                }
            }

            tracing::info!(
                "{} ask={:.4}  signal={:?}  confirm={}/{}  window={}/{}",
                symbol, rate.ask, signal, count.abs(), self.confirm_required,
                window.len(), self.window_size
            );

            // 5. Action uniquement si confirmé
            if *count >= self.confirm_required {
                if !self.local_open.contains(&id) {
                    let (sl, tp) = compute_sl_tp(&prices, rate.ask);
                    tracing::info!(
                        "BUY {} @ {:.4}  SL={:.4}  TP={:.4}",
                        symbol, rate.ask, sl, tp
                    );
                    let order = CreateOrderRequest {
                        instrument_id: id,
                        is_buy: true,
                        leverage: self.leverage,
                        amount: Some(self.amount),
                        units: None,
                        stop_loss_rate: Some(sl),
                        take_profit_rate: Some(tp),
                        is_trailing_stop_loss: None,
                    };
                    match self.client.send_order(order).await {
                        Ok(resp) => {
                            tracing::info!("Ordre placé: {:?}", resp);
                            self.local_open.insert(id);
                        }
                        Err(e) => tracing::error!("send_order failed: {:?}", e),
                    }
                    *count = 0;
                }
            } else if *count <= -self.confirm_required {
                if let Some(pos) = open_positions.get(&id) {
                    tracing::info!(
                        "CLOSE {} position_id={} open={:.4} bid={:.4}",
                        symbol, pos.position_id, pos.open_rate, rate.bid
                    );
                    match self.client.close_position(pos.position_id, ClosePositionRequest { instrument_id: id, units_to_deduct: None }).await {
                        Ok(resp) => {
                            tracing::info!("Position fermée: {:?}", resp);
                            self.local_open.remove(&id);
                        }
                        Err(e) => tracing::error!("close_position failed: {:?}", e),
                    }
                    *count = 0;
                }
            }
        }
    }
}
