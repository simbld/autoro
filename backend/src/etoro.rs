// /backend/src/etoro.rs
use reqwest::Client;
use uuid::Uuid;
use crate::models::{CandlesResponse, ClientPortfolio, ClosePositionRequest, CreateOrderRequest, CreateOrderResponse, EditPositionRequest, HistoryResponse, InstrumentRatesResponse, InstrumentSearchResponse, PortfolioResponse, TradeHistoryItem};

#[derive(Clone)]
pub struct EtoroClient {
    pub base_url: String,
    pub api_key: String,
    pub user_key: String,
    pub http: Client,
    /// "demo" ou "real"
    pub mode: String,
}

#[derive(Debug, Clone, Copy)]
pub enum CandleInterval {
	FiveMinutes,
	FifteenMinutes,
	ThirtyMinutes,
}

impl CandleInterval {
	/// Seul endroit du code où les chaînes exactes de l'API existent.
	fn as_str(self) -> &'static str {
		match self {
			Self::FiveMinutes => "FiveMinutes",
			Self::FifteenMinutes => "FifteenMinutes",
			Self::ThirtyMinutes => "ThirtyMinutes",
		}
	}
}

impl std::fmt::Debug for EtoroClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EtoroClient")
            .field("base_url", &self.base_url)
            .field("api_key", &"[REDACTED]")
            .field("user_key", &"[REDACTED]")
            .field("http", &self.http)
            .field("mode", &self.mode)
            .finish()
    }
}

impl EtoroClient {
    pub fn new(base_url: &str, api_key: String, user_key: String, mode: String) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            user_key,
            http: Client::new(),
            mode,
        }
    }

    fn get(&self, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        self.http
            .get(url)
            .header("x-api-key", &self.api_key)
            .header("x-user-key", &self.user_key)
            .header("x-request-id", Uuid::new_v4().to_string())
    }

    fn post(&self, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        self.http
            .post(url)
            .header("x-api-key", &self.api_key)
            .header("x-user-key", &self.user_key)
            .header("x-request-id", Uuid::new_v4().to_string())
    }

    fn patch(&self, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        self.http
            .patch(url)
            .header("x-api-key", &self.api_key)
            .header("x-user-key", &self.user_key)
            .header("x-request-id", Uuid::new_v4().to_string())
    }

    pub async fn search_instrument(&self, symbol: &str) -> Result<InstrumentSearchResponse, reqwest::Error> {
        self.get("/api/v1/market-data/search")
            .query(&[("internalSymbolFull", symbol)])
            .send().await?
            .error_for_status()?
            .json::<InstrumentSearchResponse>()
            .await
    }

    pub async fn get_rates(&self, instrument_ids: &[i64]) -> Result<InstrumentRatesResponse, reqwest::Error> {
        let params: Vec<(&str, String)> = instrument_ids
            .iter()
            .map(|id| ("instrumentIds", id.to_string()))
            .collect();
        self.get("/api/v1/market-data/instruments/rates")
            .query(&params)
            .send().await?
            .error_for_status()?
            .json::<InstrumentRatesResponse>()
            .await
    }

    pub async fn get_portfolio(&self) -> Result<ClientPortfolio, reqwest::Error> {
        self.get(&format!("/api/v1/trading/info/{}/pnl", self.mode))
            .send().await?
            .error_for_status()?
            .json::<PortfolioResponse>()
            .await
            .map(|r| r.client_portfolio)
    }

    pub async fn close_position(
        &self,
        position_id: i64,
        payload: ClosePositionRequest,
    ) -> Result<CreateOrderResponse, reqwest::Error> {
        let resp = self.post(&format!("/api/v1/trading/execution/{}/market-close-orders/positions/{position_id}", self.mode))
            .json(&payload)
            .send().await?;
        let status = resp.status();
        let text = resp.text().await?;
        tracing::debug!("close_position status={} body={}", status, text);
        let value: serde_json::Value = serde_json::from_str(&text)
            .unwrap_or(serde_json::json!({"raw": text, "status": status.as_u16()}));
        Ok(CreateOrderResponse(value))
    }

    /// Modifie le SL/TP d'une position ouverte (API v2, réponse 202 asynchrone).
    /// Path demo : /api/v2/trading/demo/positions/{id} — réel : /api/v2/trading/positions/{id}
    pub async fn edit_position(
        &self,
        position_id: i64,
        payload: EditPositionRequest,
    ) -> Result<CreateOrderResponse, reqwest::Error> {
        let mode_segment = if self.mode == "demo" { "demo/" } else { "" };
        let resp = self.patch(&format!("/api/v2/trading/{mode_segment}positions/{position_id}"))
            .json(&payload)
            .send().await?;
        let status = resp.status();
        let text = resp.text().await?;
        tracing::debug!("edit_position status={} body={}", status, text);
        let value: serde_json::Value = serde_json::from_str(&text)
            .unwrap_or(serde_json::json!({"raw": text, "status": status.as_u16()}));
        Ok(CreateOrderResponse(value))
    }

    pub async fn get_history(&self, min_date: &str) -> Result<Vec<TradeHistoryItem>, reqwest::Error> {
		let mode_segment = if self.mode == "demo" { "demo/" } else { "" };
		let resp =
        self.get(&format!("/api/v1/trading/info/trade/{mode_segment}history"))
            .query(&[("minDate", min_date)])
            .send().await?
            .error_for_status()?
            .json::<HistoryResponse>()
            .await?;
		Ok(resp.items)
    }

    pub async fn send_order(&self, payload: CreateOrderRequest) -> Result<CreateOrderResponse, reqwest::Error> {
        let endpoint = if payload.amount.is_some() {
            "market-open-orders/by-amount"
        } else {
            "market-open-orders/by-units"
        };
        self.post(&format!("/api/v1/trading/execution/{}/{endpoint}", self.mode))
            .json(&payload)
            .send().await?
            .error_for_status()?
            .json::<CreateOrderResponse>()
            .await
    }

	/// Get candles for a given instrument (API v3
	pub async fn get_candles(
		&self,
		instrument_id: i64,
		interval: CandleInterval,
		count: u32,
	) -> Result<CandlesResponse, reqwest::Error> {
		self.get(&format!(
			"/api/v1/market-data/instruments/{instrument_id}/history/candles/asc/{}/{count}",
			interval.as_str()
		))
			.send().await?
			.error_for_status()?
			.json::<CandlesResponse>()
			.await
	}
}

