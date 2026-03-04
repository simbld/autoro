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
use crate::models::{CreateOrderRequest, CreateOrderResponse, Health};

#[derive(Clone)]
pub struct AppState {
    pub etoro_client: EtoroClient,
}

pub fn app_router(etoro_client: EtoroClient) -> Router {
    let state = AppState { etoro_client };

    Router::new()
        .route("/health", get(health))
        .route("/api/orders", post(create_order))
        .with_state(state)
}

async fn health() -> Json<Health> {
    Json(Health {
        ok: true,
    })
}

async fn create_order(
    State(state): State<AppState>,
    Json(payload): Json<CreateOrderRequest>,
) -> Result<Json<CreateOrderResponse>, StatusCode> {
    let order = state.etoro_client.send_order(payload).await;

  match order {
	Ok(response) => Ok(Json(response)),
	Err(_) => Err(StatusCode::BAD_REQUEST),
  }
}