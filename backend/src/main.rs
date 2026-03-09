// /backend/src/main.rs
// Example: `cargo run` then GET http://127.0.0.1:8080/health

mod config;
mod etoro;
mod models;
mod routes;
mod strategy;
mod trader;

use axum::Router;
use config::Config;
use etoro::EtoroClient;
use routes::app_router;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    // Cherche .env dans le répertoire courant, puis dans backend/ (workspace root)
    dotenvy::dotenv()
        .or_else(|_| dotenvy::from_filename("backend/.env"))
        .ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cfg = Config::from_env().expect("Failed to load config");
    let etoro = EtoroClient::new(cfg.etoro_base_url.as_str(), cfg.etoro_api_key.clone(), cfg.etoro_user_key.clone(), cfg.trading_mode.clone());

    // Lancer le trader en arrière-plan
    let trader_client = etoro.clone();
    let trader_cfg = cfg.clone();
    tokio::spawn(async move {
        trader::Trader::start(trader_client, &trader_cfg).await;
    });

	let cors: CorsLayer = CorsLayer::new()
	  .allow_origin(Any)
	  .allow_headers(Any)
	  .allow_methods(Any);

    let app: Router = app_router(etoro).layer(cors);

    let addr: SocketAddr = cfg.bind_addr.parse().expect("Invalid BIND_ADDR");
    tracing::info!("listening on https://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}