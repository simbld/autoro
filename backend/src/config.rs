// /backend/src/config.rs

#[derive(Clone)]
pub struct Config {
    pub etoro_base_url: String,
    pub etoro_api_key: String,
    pub bind_addr: String,
    pub cors_origin: Option<String>,
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
        .field("etoro_base_url", &self.etoro_base_url)
        .field("etoro_api_key", &self.etoro_api_key)
        .field("bind_addr", &self.bind_addr)
        .field("cors_origin", &self.cors_origin)
            .finish()

    }
}

impl Config {
    pub fn from_env() -> Self {
        let etoro_base_url = std::env::var("ETORO_BASE_URL").unwrap_or_else(|_| {
            panic!("Missing ETORO_BASE_URL (set it in backend/.env)")
        });
        let etoro_api_key = std::env::var("ETORO_API_KEY").unwrap_or_else(|_| panic!("Missing ETORO_API_KEY (set it in backend/.env)"));
        let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8000".into());
        let cors_origin = std::env::var("CORS_ORIGIN").ok();

        Self {
            etoro_base_url,
            etoro_api_key,
            bind_addr,
            cors_origin,
        }

    }
}