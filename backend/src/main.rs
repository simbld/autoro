// /backend/src/main.rs
// Example: `cargo run` then GET http://127.0.0.1:8080/health

mod config;
mod etoro;
mod models;
mod routes;

use axum::Router;
use config::Config;
use etoro::EtoroClient;
use routes::app_router;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cfg = Config::from_env();
    let etoro = EtoroClient::new(cfg.etoro_base_url.clone(), cfg.etoro_api_key.clone());

    let cors = CorsLayer::new()
        .allow_origin(
            cfg.cors_origin
                .as_deref()
                .map(|_| Any) // keep simple for dev; tighten in prod
                .unwrap_or(Any),
        )
        .allow_headers(Any)
        .allow_methods(Any);

    let app: Router = app_router(etoro).layer(cors);

    let addr: SocketAddr = cfg.bind_addr.parse().expect("Invalid BIND_ADDR");
    tracing::info!("listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}