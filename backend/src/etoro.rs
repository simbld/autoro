// /backend/src/etoro.rs

use reqwest::Client;
use uuid::Uuid;
use crate::models::{
    ClientPortfolio, ClosePositionRequest, CreateOrderRequest, CreateOrderResponse,
    InstrumentRatesResponse, InstrumentSearchResponse, PortfolioResponse, TradeHistoryItem,
};

#[derive(Clone)]
pub struct EtoroClient {
    pub base_url: String,
    pub api_key: String,
    pub user_key: String,
    pub http: Client,
    /// "demo" ou "real"
    pub mode: String,
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
        let resp = self.get(&format!("/api/v1/trading/info/{}/pnl", self.mode))
            .send().await?
            .error_for_status()?
            .json::<PortfolioResponse>()
            .await?;
        Ok(resp.client_portfolio)
    }

    pub async fn close_position(
        &self,
        position_id: i64,
        payload: ClosePositionRequest,
    ) -> Result<CreateOrderResponse, reqwest::Error> {
        self.post(&format!("/api/v1/trading/execution/{}/market-close-orders/positions/{position_id}", self.mode))
            .json(&payload)
            .send().await?
            .error_for_status()?
            .json::<CreateOrderResponse>()
            .await
    }

    pub async fn get_history(&self, min_date: &str) -> Result<Vec<TradeHistoryItem>, reqwest::Error> {
        self.get("/api/v1/trading/info/trade/history")
            .query(&[("minDate", min_date)])
            .send().await?
            .error_for_status()?
            .json::<Vec<TradeHistoryItem>>()
            .await
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
}
