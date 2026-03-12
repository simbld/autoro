// /backend/src/strategy.rs
// Stratégie V2 : vote majoritaire RSI + EMA croisée + Bollinger Bands + MACD

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Signal {
    Buy,
    Sell,
    Hold,
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// EMA classique : SMA sur les `period` premiers points, puis lissage exponentiel.
fn compute_ema(prices: &[f64], period: usize) -> Option<f64> {
    if prices.len() < period {
        return None;
    }
    let k = 2.0 / (period as f64 + 1.0);
    let mut ema = prices[..period].iter().sum::<f64>() / period as f64;
    for &price in &prices[period..] {
        ema = price * k + ema * (1.0 - k);
    }
    Some(ema)
}

// ── Indicateurs (retournent +1 buy, -1 sell, 0 neutre) ───────────────────────

/// RSI(14) — seuils élargis à 35/65 pour plus de réactivité.
#[allow(clippy::cast_precision_loss)]
fn rsi_vote(prices: &[f64]) -> i8 {
    const PERIOD: usize = 14;
    if prices.len() < PERIOD + 1 {
        return 0;
    }
    let slice = &prices[prices.len() - PERIOD - 1..];
    let mut avg_gain = 0.0_f64;
    let mut avg_loss = 0.0_f64;
    for w in slice.windows(2) {
        let diff = w[1] - w[0];
        if diff > 0.0 {
            avg_gain += diff;
        } else {
            avg_loss -= diff;
        }
    }
    avg_gain /= PERIOD as f64;
    avg_loss /= PERIOD as f64;
    if avg_loss == 0.0 {
        return if avg_gain > 0.0 { -1 } else { 0 };
    }
    let rsi = 100.0 - 100.0 / (1.0 + avg_gain / avg_loss);
    if rsi < 35.0 {
        1
    } else if rsi > 65.0 {
        -1
    } else {
        0
    }
}

/// EMA croisée EMA7 / EMA21 — trend direction (fast > slow = haussier).
fn ema_cross_vote(prices: &[f64]) -> i8 {
    const FAST: usize = 7;
    const SLOW: usize = 21;
    let ema_fast = match compute_ema(prices, FAST) {
        Some(v) => v,
        None => return 0,
    };
    let ema_slow = match compute_ema(prices, SLOW) {
        Some(v) => v,
        None => return 0,
    };
    if ema_fast > ema_slow {
        1
    } else if ema_fast < ema_slow {
        -1
    } else {
        0
    }
}

/// Bollinger Bands (20, ×2σ) — achat sous la bande basse, vente au-dessus de la haute.
#[allow(clippy::cast_precision_loss)]
fn bollinger_vote(prices: &[f64]) -> i8 {
    const PERIOD: usize = 20;
    if prices.len() < PERIOD {
        return 0;
    }
    let slice = &prices[prices.len() - PERIOD..];
    let mean = slice.iter().sum::<f64>() / PERIOD as f64;
    let std = (slice.iter().map(|p| (p - mean).powi(2)).sum::<f64>() / PERIOD as f64).sqrt();
    let last = *prices.last().unwrap();
    if last < mean - 2.0 * std {
        1
    } else if last > mean + 2.0 * std {
        -1
    } else {
        0
    }
}

/// MACD(12, 26, 9) — signe de l'histogramme (MACD - signal line).
#[allow(clippy::cast_precision_loss)]
fn macd_vote(prices: &[f64]) -> i8 {
    const FAST: usize = 12;
    const SLOW: usize = 26;
    const SIGNAL: usize = 9;
    if prices.len() < SLOW + SIGNAL {
        return 0;
    }
    let macd_series: Vec<f64> = (SLOW..=prices.len())
        .filter_map(|i| {
            let ema_fast = compute_ema(&prices[..i], FAST)?;
            let ema_slow = compute_ema(&prices[..i], SLOW)?;
            Some(ema_fast - ema_slow)
        })
        .collect();
    if macd_series.len() < SIGNAL {
        return 0;
    }
    let signal_line = match compute_ema(&macd_series, SIGNAL) {
        Some(v) => v,
        None => return 0,
    };
    let histogram = macd_series.last().unwrap() - signal_line;
    if histogram > 0.0 {
        1
    } else if histogram < 0.0 {
        -1
    } else {
        0
    }
}

// ── Signal composite ─────────────────────────────────────────────────────────

/// Vote 3/4 : signal retourné seulement si au moins 3 indicateurs sont d'accord.
/// Minimum 35 prix requis (contrainte MACD : 26 + 9).
pub fn compute_signal(prices: &[f64]) -> Signal {
    if prices.len() < 35 {
        return Signal::Hold;
    }
    let votes = [
        rsi_vote(prices),
        ema_cross_vote(prices),
        bollinger_vote(prices),
        macd_vote(prices),
    ];
    let buy_count = votes.iter().filter(|&&v| v == 1).count();
    let sell_count = votes.iter().filter(|&&v| v == -1).count();

    tracing::debug!(
        "votes RSI={} EMA={} BB={} MACD={} → buy={} sell={}",
        votes[0], votes[1], votes[2], votes[3], buy_count, sell_count
    );

    if buy_count >= 3 {
        Signal::Buy
    } else if sell_count >= 3 {
        Signal::Sell
    } else {
        Signal::Hold
    }
}

// ── Stop-Loss / Take-Profit ──────────────────────────────────────────────────

/// Calcule SL/TP basé sur la volatilité récente (écart-type sur les 20 derniers prix).
/// Ratio risque/rendement 1:2 → SL = entry − 1.5σ, TP = entry + 3σ.
/// Floor minimum : 0.3 % du prix d'entrée pour éviter des niveaux trop proches.
#[allow(clippy::cast_precision_loss)]
pub fn compute_sl_tp(prices: &[f64], entry: f64) -> (f64, f64) {
    let period = prices.len().min(20);
    let slice = &prices[prices.len() - period..];
    let mean = slice.iter().sum::<f64>() / period as f64;
    let std = (slice.iter().map(|p| (p - mean).powi(2)).sum::<f64>() / period as f64)
        .sqrt()
        .max(entry * 0.003); // floor à 0.3 % du prix
    let sl = (entry - 1.5 * std).max(entry * 0.90); // cap max perte à −10 %
    let tp = entry + 3.0 * std;
    (sl, tp)
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hold_insufficient_data() {
        assert_eq!(compute_signal(&[100.0; 10]), Signal::Hold);
        assert_eq!(compute_signal(&[100.0; 34]), Signal::Hold);
    }

    #[test]
    fn test_ema_basic() {
        let prices: Vec<f64> = (1..=10).map(|i| i as f64).collect();
        assert!(compute_ema(&prices, 5).is_some());
        assert!(compute_ema(&prices, 11).is_none());
    }

    #[test]
    fn test_sl_tp_ratio() {
        let prices = vec![100.0_f64; 20];
        let (sl, tp) = compute_sl_tp(&prices, 100.0);
        // Avec prix parfaitement plat, std = 0 → floor à 0.3 %
        // SL = 100 - 1.5*0.3 = 99.55, TP = 100 + 3*0.3 = 100.9
        assert!(sl < 100.0, "SL doit être sous le prix d'entrée");
        assert!(tp > 100.0, "TP doit être au-dessus du prix d'entrée");
        let risk = 100.0 - sl;
        let reward = tp - 100.0;
        assert!(reward > risk, "ratio R/R doit être > 1");
    }
}
