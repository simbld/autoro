// /backend/src/models.rs

use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize)]
pub struct Health {
    pub ok: bool,
}

/// Requête pour ouvrir une position via l'API eToro.
/// Utilise `amount` OU `units`, pas les deux en même temps.
/// Endpoint by-amount : POST /trading/execution/market-open-orders/by-amount
/// Endpoint by-units  : POST /trading/execution/market-open-orders/by-units
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateOrderRequest {
    /// ID numérique de l'instrument (ex: 100000 pour BTC). Résoudre via /market-data/search
    pub instrument_id: i64,
    /// true = achat (long), false = vente (short)
    pub is_buy: bool,
    /// Levier (ex: 1, 2, 5, 10...)
    pub leverage: i64,
    /// Montant en cash à investir (utiliser avec by-amount)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<f64>,
    /// Nombre d'unités à trader (utiliser avec by-units)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub units: Option<f64>,
    /// Taux de stop-loss (optionnel)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_loss_rate: Option<f64>,
    /// Taux de take-profit (optionnel)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub take_profit_rate: Option<f64>,
    /// Stop-loss suiveur (optionnel)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_trailing_stop_loss: Option<bool>,
}

/// Réponse après ouverture d'un ordre (champs retournés par l'API eToro)
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateOrderResponse {
    pub order_id: i64,
    pub status: String,
    pub position_id: Option<i64>,
    pub error_code: Option<i64>,
    pub error_message: Option<String>,
}

/// Requête pour fermer une position (totale ou partielle)
/// Endpoint : POST /trading/execution/market-close-orders/positions/{positionId}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ClosePositionRequest {
    /// null = fermeture totale, valeur = fermeture partielle
    pub units_to_deduct: Option<f64>,
}

/// Position ouverte dans le portfolio
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[allow(clippy::struct_field_names)]
pub struct Position {
    pub position_id: i64,
    pub instrument_id: i64,
    pub is_buy: bool,
    pub leverage: i64,
    pub units: f64,
    pub amount: f64,
    pub open_rate: f64,
    pub open_timestamp: chrono::DateTime<chrono::Utc>,
    pub stop_loss_rate: Option<f64>,
    pub take_profit_rate: Option<f64>,
    pub is_trailing_stop_loss: Option<bool>,
    pub net_profit: f64,
    pub investment: f64,
}

/// Portfolio complet du client (réponse PnL)
/// Endpoint : GET /trading/info/{real|demo}/pnl
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientPortfolio {
    /// Solde total du compte
    pub credits: f64,
    /// Positions ouvertes
    pub positions: Vec<Position>,
    /// Ordres de marché en attente (Market Orders)
    pub orders_for_open: Vec<PendingOrder>,
    /// Ordres MIT en attente (Market-if-touched / Limit)
    pub orders: Vec<PendingOrder>,
}

/// Ordre en attente (utilisé dans le calcul du cash disponible)
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingOrder {
    pub amount: f64,
    /// 0 = ordre manuel, != 0 = ordre copy/mirror
    #[serde(rename = "mirrorID")]
    pub mirror_id: i64,
}

/// Item retourné par l'endpoint de recherche d'instruments
/// Endpoint : GET /market-data/search?internalSymbolFull=BTC
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstrumentSearchItem {
    pub instrument_id: i64,
    pub internal_symbol_full: String,
    pub instrument_display_name: Option<String>,
}

/// Réponse complète de la recherche d'instruments
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstrumentSearchResponse {
    pub items: Vec<InstrumentSearchItem>,
}

/// Prix en temps réel d'un instrument (bid/ask)
/// Endpoint : GET /market-data/instruments/rates?instrumentIds=100000
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct InstrumentRate {
    pub instrument_id: i64,
    /// Prix d'achat (ask)
    pub ask: f64,
    /// Prix de vente (bid)
    pub bid: f64,
    /// Prix de la dernière exécution
    pub last_execution: Option<f64>,
    pub date: Option<chrono::DateTime<chrono::Utc>>,
}

/// Réponse de l'endpoint des taux de marché
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct InstrumentRatesResponse {
    pub rates: Vec<InstrumentRate>,
}

/// Item d'historique de trading (position fermée)
/// Endpoint : GET /trading/history/real
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TradeHistoryItem {
    pub position_id: i64,
    pub instrument_id: i64,
    pub is_buy: bool,
    pub leverage: i64,
    pub units: f64,
    pub amount: f64,
    pub open_rate: f64,
    pub close_rate: f64,
    pub net_profit: f64,
    pub open_timestamp: chrono::DateTime<chrono::Utc>,
    pub close_timestamp: chrono::DateTime<chrono::Utc>,
}