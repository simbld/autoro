// /backend/src/etoro.rs

use reqwest::Client;
use uuid::Uuid;
use crate::models::{
    ClientPortfolio, ClosePositionRequest, CreateOrderRequest, CreateOrderResponse,
    InstrumentRatesResponse, InstrumentSearchResponse, TradeHistoryItem,
};

#[derive(Clone)]
pub struct EtoroClient {
    pub base_url: String,
    pub api_key: String,
    pub user_key: String,
    pub http: Client,
}

impl std::fmt::Debug for EtoroClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EtoroClient")
            .field("base_url", &self.base_url)
            .field("api_key", &"[REDACTED]")
            .field("user_key", &"[REDACTED]")
            .field("http", &self.http)
            .finish()
    }
}

impl EtoroClient {
    pub fn new(base_url: &str, api_key: String, user_key: String) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            user_key,
            http: Client::new(),
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
            .json::<InstrumentSearchResponse>()
            .await
    }

    pub async fn get_rates(&self, instrument_ids: &[i64]) -> Result<InstrumentRatesResponse, reqwest::Error> {
        let ids_param = instrument_ids.iter().map(ToString::to_string).collect::<Vec<_>>().join(",");
        self.get("/api/v1/market-data/instruments/rates")
            .query(&[("instrumentIds", ids_param)])
            .send().await?
            .json::<InstrumentRatesResponse>()
            .await
    }

    pub async fn get_portfolio(&self) -> Result<ClientPortfolio, reqwest::Error> {
        self.get("/api/v1/trading/info/demo/pnl")
            .send().await?
            .json::<ClientPortfolio>()
            .await
    }

    pub async fn close_position(
        &self,
        position_id: i64,
        payload: ClosePositionRequest,
    ) -> Result<CreateOrderResponse, reqwest::Error> {
        self.post(&format!("/api/v1/trading/execution/market-close-orders/positions/{position_id}"))
            .json(&payload)
            .send().await?
            .json::<CreateOrderResponse>()
            .await
    }

    pub async fn get_history(&self, min_date: &str) -> Result<Vec<TradeHistoryItem>, reqwest::Error> {
        self.get("/api/v1/trading/info/trade/history")
            .query(&[("minDate", min_date)])
            .send().await?
            .json::<Vec<TradeHistoryItem>>()
            .await
    }

    pub async fn send_order(&self, payload: CreateOrderRequest) -> Result<CreateOrderResponse, reqwest::Error> {
        let endpoint = if payload.amount.is_some() {
            "market-open-orders/by-amount"
        } else {
            "market-open-orders/by-units"
        };
        self.post(&format!("/api/v1/trading/execution/{endpoint}"))
            .json(&payload)
            .send().await?
            .json::<CreateOrderResponse>()
            .await
    }
}
