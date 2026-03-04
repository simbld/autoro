// /backend/src/etoro.rs

use reqwest::Client;
use crate::models::{CreateOrderRequest, CreateOrderResponse};

#[derive(Clone)]
pub struct EtoroClient {
    pub base_url: String,
    pub api_key: String,
    pub http: Client,
}

impl std::fmt::Debug for EtoroClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EtoroClient")
            .field("base_url", &self.base_url)
            .field("api_key", &"[REDACTED]")
            .field("http", &self.http)
            .finish()

    }
}

impl EtoroClient {
    pub fn new(base_url: &str, api_key: String) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            http: Client::new(),
        }
    }

    pub async fn send_order(&self, payload: CreateOrderRequest) -> Result<CreateOrderResponse, reqwest::Error> {
        self.http
		  .post(format!("{}/orders", self.base_url))
		  .json(&payload)
		  .header("Authorization", &self.api_key)
		  .send().await?
		  .json::<CreateOrderResponse>()
		  .await
    }
}
