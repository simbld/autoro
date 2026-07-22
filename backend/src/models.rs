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
    /// ID numérique de l'instrument (ex: 100 000 pour BTC). Résoudre via /market-data/search
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
    /// Stop-loss suiveur dès l'ouverture (optionnel).
    /// L'API attend `IsTslEnabled`, pas `IsTrailingStopLoss`.
    #[serde(rename = "IsTslEnabled", skip_serializing_if = "Option::is_none")]
    pub is_tsl_enabled: Option<bool>,
    /// true = pas de take-profit sur la position (laisser courir les gagnants)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_no_take_profit: Option<bool>,
}

/// Type de stop-loss pour la modification d'une position ouverte (API v2)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StopLossType {
    Fixed,
    Trailing,
}

/// Requête de modification SL/TP d'une position ouverte.
/// Endpoint : PATCH /api/v2/trading/{demo/}positions/{positionId}
/// Au moins un champ doit être renseigné. Réponse 202 (asynchrone).
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditPositionRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_loss_rate: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub take_profit_rate: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_loss_type: Option<StopLossType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clear_stop_loss: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clear_take_profit: Option<bool>,
}

/// Réponse brute de l'API eToro pour les ordres (format variable selon l'endpoint).
/// On utilise `serde_json::Value` pour accepter n'importe quelle structure.
#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CreateOrderResponse(pub serde_json::Value);

/// Requête pour fermer une position (totale ou partielle)
/// Endpoint : POST /trading/execution/{mode}/market-close-orders/positions/{positionId}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ClosePositionRequest {
    /// Obligatoire en mode demo
    #[serde(rename = "InstrumentID")]
    pub instrument_id: i64,
    /// null = fermeture totale, valeur = fermeture partielle
    pub units_to_deduct: Option<f64>,
}

/// Position ouverte dans le portfolio
#[derive(Debug, Serialize, Deserialize)]
#[allow(clippy::struct_field_names)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    #[serde(rename = "positionID")]
    pub position_id: i64,
    #[serde(rename = "instrumentID")]
    pub instrument_id: i64,
    pub is_buy: bool,
    pub leverage: i64,
    pub units: f64,
    pub amount: f64,
    pub open_rate: f64,
    pub stop_loss_rate: Option<f64>,
    pub take_profit_rate: Option<f64>,
    #[serde(rename = "isTslEnabled", default)]
    pub is_tsl_enabled: bool,
    #[serde(rename = "tslRate", default)]
    pub tsl_rate: Option<f64>,
}

/// Portfolio complet du client (réponse PnL)
/// Endpoint : GET /trading/info/{real|demo}/pnl
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientPortfolio {
    /// Solde disponible du compte
    pub credit: f64,
    /// Positions ouvertes
    pub positions: Vec<Position>,
    /// Ordres de marché en attente
    pub orders_for_open: Vec<PendingOrder>,
    /// Ordres limit/stop en attente
    pub orders: Vec<PendingOrder>,
}

/// Wrapper de la réponse /pnl
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioResponse {
    pub client_portfolio: ClientPortfolio,
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
    pub instrument_display_name: Option<String>,
    pub internal_symbol_full: String,
	pub daily_price_change: Option<f64>,
	pub abs_daily_price_change: Option<f64>,
}

/// Réponse complète de la recherche d'instruments
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstrumentSearchResponse {
    pub items: Vec<InstrumentSearchItem>,
}

/// Item du catalog d'instruments (mapping ID -> symbole court)
/// Endpoint : GET /api/instruments/catalog
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstrumentCatalogItem {
    pub instrument_id: i64,
    pub symbol: String,
}

/// Prix en temps réel d'un instrument (bid/ask)
/// Endpoint : GET /market-data/instruments/rates?instrumentIds=100000
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstrumentRate {
		#[serde(rename = "instrumentID")]
		pub instrument_id: i64,
		pub ask: f64,
		pub bid: f64,
		pub last_execution: Option<f64>,
		pub date: Option<chrono::DateTime<chrono::Utc>>,
}

/// Réponse de l'endpoint des taux de marché
#[derive(Debug, Serialize, Deserialize)]
pub struct InstrumentRatesResponse {
    pub rates: Vec<InstrumentRate>,
}

/// Article d'actualité (réponse NewsAPI.org)
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewsArticle {
    pub title: String,
    pub description: Option<String>,
    pub url: String,
    pub image_url: Option<String>,
    pub published_at: String,
    pub source_name: String,
}

/// Réponse de la route /api/instruments/news
#[derive(Debug, Serialize, Deserialize)]
pub struct NewsResponse {
    pub articles: Vec<NewsArticle>,
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstrumentImage {
	#[serde(rename = "instrumentID")]
	pub instrument_id: i64,
	pub width: f64,
	pub height: f64,
	pub uri: String,
	pub background_color: Option<String>,
	pub text_color: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstrumentDisplayData {
	#[serde(rename = "instrumentID")]
	pub instrument_id: i64,
	pub instrument_display_name: Option<String>,
	pub symbol_full: String,
	pub images: Vec<InstrumentImage>
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstrumentDisplayResponse {
	pub instrument_display_datas: Vec<InstrumentDisplayData>
}