// /backend/src/trader.rs
// Orchestre la boucle de trading automatique — stratégie V3 :
// entrées long ET short filtrées par la tendance, SL fixe à l'ouverture,
// bascule en trailing stop (TSL côté serveur eToro) une fois le trade en profit.

use std::collections::{HashMap, VecDeque};
use tokio::time::{interval, Duration};

use crate::config::Config;
use crate::etoro::EtoroClient;
use crate::models::{
    ClosePositionRequest, CreateOrderRequest, EditPositionRequest, Position, StopLossType,
};
use crate::strategy::{compute_initial_sl, compute_signal, Signal};

/// Suivi local d'une position ouverte par le bot.
/// Nécessaire, car `get_portfolio` est capricieux en demo (souvent 0 positions
/// juste après une ouverture) — on garde donc notre propre état.
struct TrackedPosition {
    is_buy: bool,
    /// Prix d'entrée (ask pour un long, bid pour un short)
    entry: f64,
    /// True une fois le PATCH stopLossType=trailing accepté par eToro
    tsl_activated: bool,
}

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
    /// Profit (%) à partir duquel on bascule le SL en trailing.
    tsl_trigger_pct: f64,
    /// Écart (%) entre le prix courant et le SL trailing posé.
    tsl_gap_pct: f64,
    /// Autoriser les positions short sur signal Sell confirmé.
    allow_short: bool,
    /// Positions ouvertes par le bot (suivi local, réconcilié avec le portfolio).
    open_tracks: HashMap<i64, TrackedPosition>,
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
            tracing::error!("No instruments resolved, trader won't start");
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
            tsl_trigger_pct: cfg.trader_tsl_trigger_pct,
            tsl_gap_pct: cfg.trader_tsl_gap_pct,
            allow_short: cfg.trader_allow_short,
            open_tracks: HashMap::new(),
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

        // 2. Portfolio (positions ouvertes) — on réconcilie le suivi local quand il répond
        let open_positions: HashMap<i64, Position> = match self.client.get_portfolio().await {
            Ok(p) => {
                // En demo l'API ne renvoie souvent aucune position même après ouverture :
                // on ne réconcilie que si le portfolio contient des données.
                if !p.positions.is_empty() {
                    self.reconcile(&p.positions);
                }
                tracing::info!(
                    "Tick — solde: ${:.2} | positions ouvertes: {} (local: {})",
                    p.credit,
                    p.positions.len(),
                    self.open_tracks.len()
                );
                p.positions.into_iter().map(|p| (p.instrument_id, p)).collect()
            }
            Err(e) => {
                tracing::warn!("get_portfolio indisponible ({:?}), utilisation du suivi local ({} positions)", e, self.open_tracks.len());
                HashMap::new()
            }
        };

        for rate in &all_rates {
            let id = rate.instrument_id;
            let symbol = self.instruments.get(&id).cloned().unwrap_or_default();

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
            let count = *count;

            tracing::info!(
                "{} ask={:.4} bid={:.4}  signal={:?}  confirm={}/{}  window={}/{}",
                symbol, rate.ask, rate.bid, signal, count.abs(), self.confirm_required,
                window.len(), self.window_size
            );

            // 5. Gestion du trailing stop sur les positions en profit
            self.manage_trailing(id, &symbol, rate.bid, rate.ask, open_positions.get(&id)).await;

            // 6. Action sur signal confirmé
            let confirmed_buy = count >= self.confirm_required;
            let confirmed_sell = count <= -self.confirm_required;
            if !confirmed_buy && !confirmed_sell {
                continue;
            }
            let want_buy = confirmed_buy;

            match self.open_tracks.get(&id) {
                // Déjà positionné dans le même sens → rien à faire
                Some(track) if track.is_buy == want_buy => {}

                // Signal opposé à la position ouverte → fermeture
                Some(_) => {
                    if let Some(pos) = open_positions.get(&id) {
                        tracing::info!(
                            "CLOSE {} position_id={} open={:.4} bid={:.4} (signal opposé confirmé)",
                            symbol, pos.position_id, pos.open_rate, rate.bid
                        );
                        match self.client.close_position(pos.position_id, ClosePositionRequest { instrument_id: id, units_to_deduct: None }).await {
                            Ok(resp) => {
                                tracing::info!("Position fermée: {:?}", resp);
                                self.open_tracks.remove(&id);
                                self.confirm_counts.insert(id, 0);
                            }
                            Err(e) => tracing::error!("close_position failed: {:?}", e),
                        }
                    } else {
                        tracing::warn!(
                            "{} : signal opposé confirmé mais position_id inconnu (portfolio vide ce tick), nouvel essai au prochain tick",
                            symbol
                        );
                    }
                }

                // Pas de position → ouverture dans le sens du signal
                None => {
                    if !want_buy && !self.allow_short {
                        tracing::info!("{} : signal Sell confirmé mais TRADER_ALLOW_SHORT=false, on ignore", symbol);
                        continue;
                    }
                    // Un long s'achète au ask, un short se vend au bid
                    let entry = if want_buy { rate.ask } else { rate.bid };
                    let sl = compute_initial_sl(&prices, entry, want_buy);
                    tracing::info!(
                        "{} {} @ {:.4} SL={:.4} (pas de TP — sortie par TSL)",
                        if want_buy { "BUY" } else { "SHORT" }, symbol, entry, sl
                    );
                    let order = CreateOrderRequest {
                        instrument_id: id,
                        is_buy: want_buy,
                        leverage: self.leverage,
                        amount: Some(self.amount),
                        units: None,
                        stop_loss_rate: Some(sl),
                        take_profit_rate: None,
                        is_tsl_enabled: None,
                        is_no_take_profit: Some(true),
                    };
                    match self.client.send_order(order).await {
                        Ok(resp) => {
                            tracing::info!("Ordre placé: {:?}", resp);
                            self.open_tracks.insert(id, TrackedPosition {
                                is_buy: want_buy,
                                entry,
                                tsl_activated: false,
                            });
                            self.confirm_counts.insert(id, 0);
                        }
                        Err(e) => tracing::error!("send_order failed: {:?}", e),
                    }
                }
            }
        }
    }

    /// Aligne le suivi local sur le portfolio réel quand celui-ci répond.
    /// Les positions disparues (fermées par SL/TSL côté eToro) sont retirées,
    /// celles inconnues (ouvertes à la main par l'utilisateur) sont adoptées.
    fn reconcile(&mut self, positions: &[Position]) {
        let by_instrument: HashMap<i64, &Position> =
            positions.iter().map(|p| (p.instrument_id, p)).collect();

        self.open_tracks.retain(|id, _| {
            let still_open = by_instrument.contains_key(id);
            if !still_open {
                tracing::info!(
                    "{} : position absente du portfolio (fermée par SL/TSL), retrait du suivi local",
                    self.instruments.get(id).map_or("?", |s| s.as_str())
                );
            }
            still_open
        });

        for (&id, pos) in &by_instrument {
            if self.instruments.contains_key(&id) && !self.open_tracks.contains_key(&id) {
                tracing::info!(
                    "{} : position trouvée dans le portfolio (id={}), adoption dans le suivi local",
                    self.instruments.get(&id).map_or("?", |s| s.as_str()),
                    pos.position_id
                );
                self.open_tracks.insert(id, TrackedPosition {
                    is_buy: pos.is_buy,
                    entry: pos.open_rate,
                    tsl_activated: pos.is_tsl_enabled,
                });
            }
        }
    }

    /// Bascule le SL en trailing dès que le trade est en profit de `tsl_trigger_pct` %.
    /// Le PATCH exige le `positionId` (connu via le portfolio) — si le portfolio
    /// était vide ce tick, on réessaie simplement au suivant.
    async fn manage_trailing(
        &mut self,
        id: i64,
        symbol: &str,
        bid: f64,
        ask: f64,
        portfolio_pos: Option<&Position>,
    ) {
        let Some(track) = self.open_tracks.get(&id) else { return };
        if track.tsl_activated {
            return;
        }

        // Profit en % du prix d'entrée (un long se ferme au bid, un short à ask)
        let profit_pct = if track.is_buy {
            (bid - track.entry) / track.entry * 100.0
        } else {
            (track.entry - ask) / track.entry * 100.0
        };
        if profit_pct < self.tsl_trigger_pct {
            return;
        }

        let Some(pos) = portfolio_pos else {
            tracing::warn!(
                "{} : profit {:.2}% ≥ seuil TSL {:.2}% mais position_id inconnu (portfolio vide ce tick)",
                symbol, profit_pct, self.tsl_trigger_pct
            );
            return;
        };

        // Nouveau SL posé à gap% du prix courant, côté protecteur
        let new_sl = if track.is_buy {
            bid * (1.0 - self.tsl_gap_pct / 100.0)
        } else {
            ask * (1.0 + self.tsl_gap_pct / 100.0)
        };
        tracing::info!(
            "TSL {} position_id={} profit={:.2}% → SL trailing @{:.4} (gap {:.1}%)",
            symbol, pos.position_id, profit_pct, new_sl, self.tsl_gap_pct
        );

        let req = EditPositionRequest {
            stop_loss_rate: Some(new_sl),
            stop_loss_type: Some(StopLossType::Trailing),
            ..Default::default()
        };
        match self.client.edit_position(pos.position_id, req).await {
            // Réponse 202 : la présence d'opérationId confirme l'acceptation
            Ok(resp) if resp.0.get("operationId").is_some() => {
                tracing::info!("TSL activé sur {} : {:?}", symbol, resp);
                if let Some(track) = self.open_tracks.get_mut(&id) {
                    track.tsl_activated = true;
                }
            }
            Ok(resp) => tracing::error!("edit_position refusé pour {} : {:?}", symbol, resp),
            Err(e) => tracing::error!("edit_position failed pour {} : {:?}", symbol, e),
        }
    }
}
