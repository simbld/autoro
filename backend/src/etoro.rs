// /backend/src/etoro.rs

use reqwest::Client;

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
    pub fn new(base_url: String, api_key: String) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            http: Client::new(),
        }
    }
}