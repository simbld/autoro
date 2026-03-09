// /backend/src/config.rs

#[derive(Clone)]
pub struct Config {
    pub etoro_base_url: String,
    pub etoro_api_key: String,
    pub etoro_user_key: String,
    pub bind_addr: String,
    pub cors_origin: Option<String>,
    /// "demo" ou "real"
    pub trading_mode: String,
    /// Symboles suivis, ex: "BTC,ETH,AAPL"
    pub trader_symbols: Vec<String>,
    /// Montant USD investi par trade
    pub trader_amount: f64,
    /// Levier par défaut
    pub trader_leverage: i64,
    /// Intervalle entre chaque tick (secondes)
    pub trader_interval_secs: u64,
    /// Taille de la fenêtre de prix pour le calcul SMA
    pub trader_window_size: usize,
    /// Nombre de confirmations consécutives requises avant d'agir
    pub trader_confirm_ticks: i8,
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("etoro_base_url", &self.etoro_base_url)
            .field("etoro_api_key", &"[REDACTED]")
            .field("etoro_user_key", &"[REDACTED]")
            .field("bind_addr", &self.bind_addr)
            .field("cors_origin", &self.cors_origin)
            .finish()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing environment variable: {0}")]
    MissingVar(&'static str),
}

impl Config {
    pub fn from_env() -> Result<Config, ConfigError> {
        let etoro_base_url = std::env::var("ETORO_BASE_URL")
            .map_err(|_| ConfigError::MissingVar("ETORO_BASE_URL"))?;
        let etoro_api_key = std::env::var("ETORO_API_KEY")
            .map_err(|_| ConfigError::MissingVar("ETORO_API_KEY"))?;
        let etoro_user_key = std::env::var("ETORO_USER_KEY")
            .map_err(|_| ConfigError::MissingVar("ETORO_USER_KEY"))?;
        let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8000".into());
        let cors_origin = std::env::var("CORS_ORIGIN").ok();

        let trading_mode = std::env::var("TRADING_MODE").unwrap_or_else(|_| "demo".into());
        let trader_symbols = std::env::var("TRADER_SYMBOLS")
            .unwrap_or_else(|_| "BTC".into())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        let trader_amount = std::env::var("TRADER_AMOUNT")
            .unwrap_or_else(|_| "100".into())
            .parse::<f64>()
            .unwrap_or(100.0);
        let trader_leverage = std::env::var("TRADER_LEVERAGE")
            .unwrap_or_else(|_| "1".into())
            .parse::<i64>()
            .unwrap_or(1);
        let trader_interval_secs = std::env::var("TRADER_INTERVAL_SECS")
            .unwrap_or_else(|_| "30".into())
            .parse::<u64>()
            .unwrap_or(30);
        let trader_window_size = std::env::var("TRADER_WINDOW_SIZE")
            .unwrap_or_else(|_| "50".into())
            .parse::<usize>()
            .unwrap_or(50);
        let trader_confirm_ticks = std::env::var("TRADER_CONFIRM_TICKS")
            .unwrap_or_else(|_| "3".into())
            .parse::<i8>()
            .unwrap_or(3);

        Ok(Self {
            etoro_base_url,
            etoro_api_key,
            etoro_user_key,
            bind_addr,
            cors_origin,
            trading_mode,
            trader_symbols,
            trader_amount,
            trader_leverage,
            trader_interval_secs,
            trader_window_size,
            trader_confirm_ticks,
        })
    }
}