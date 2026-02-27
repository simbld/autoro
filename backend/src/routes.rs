// /backend/src/routes.rs
// Example: `curl -X POST http://127.0.0.1:8080/api/orders -d ...`

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;

use crate::etoro::EtoroClient;
use crate::models::{CreateOrderRequest, Health};

#[derive(Clone)]
pub struct AppState {
    pub etoro: EtoroClient,
}

pub fn app_router(etoro: EtoroClient) -> Router {
    let state = AppState { etoro };

    Router::new()
        .route("/health", get(health))
        .route("/api/orders", post(create_order))
        .with_state(state)
}

async fn health() -> Json<Health> {
    Json(Health { ok: true })
}
