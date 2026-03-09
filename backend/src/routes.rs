// /backend/src/routes.rs
// Example: `curl -X POST http://127.0.0.1:8080/api/orders -d ...`

use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;

use crate::etoro::EtoroClient;
use crate::models::{
    ClientPortfolio, ClosePositionRequest, CreateOrderRequest, CreateOrderResponse, Health,
    InstrumentRatesResponse, InstrumentSearchResponse, TradeHistoryItem,
};

#[derive(Clone)]
pub struct AppState {
    pub etoro_client: EtoroClient,
}

pub fn app_router(etoro_client: EtoroClient) -> Router {
    let state = AppState { etoro_client };

    Router::new()
        .route("/health", get(health))
        .route("/api/orders", post(create_order))
        .route("/api/instruments/search", get(search_instrument))
        .route("/api/instruments/rates", get(get_rates))
        .route("/api/portfolio", get(get_portfolio))
        .route("/api/positions/:id/close", post(close_position))
        .route("/api/history", get(get_history))
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
    match state.etoro_client.send_order(payload).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            tracing::error!("send_order failed: {:?}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

#[derive(Deserialize)]
struct SearchQuery {
    symbol: String,
}

async fn search_instrument(
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<InstrumentSearchResponse>, StatusCode> {
    match state.etoro_client.search_instrument(&params.symbol).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            tracing::error!("search_instrument failed: {:?}", e);
            Err(StatusCode::BAD_GATEWAY)
        }
    }
}

#[derive(Deserialize)]
struct RatesQuery {
    ids: String,
}

async fn get_rates(
    State(state): State<AppState>,
    Query(params): Query<RatesQuery>,
) -> Result<Json<InstrumentRatesResponse>, StatusCode> {
    let ids: Vec<i64> = params.ids
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    match state.etoro_client.get_rates(&ids).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            tracing::error!("get_rates failed: {:?}", e);
            Err(StatusCode::BAD_GATEWAY)
        }
    }
}

async fn get_portfolio(
    State(state): State<AppState>,
) -> Result<Json<ClientPortfolio>, StatusCode> {
    match state.etoro_client.get_portfolio().await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            tracing::error!("get_portfolio failed: {:?}", e);
            Err(StatusCode::BAD_GATEWAY)
        }
    }
}

#[derive(Deserialize)]
struct ClosePositionPath {
    id: i64,
}

async fn close_position(
    State(state): State<AppState>,
    axum::extract::Path(ClosePositionPath { id }): axum::extract::Path<ClosePositionPath>,
    Json(payload): Json<ClosePositionRequest>,
) -> Result<Json<CreateOrderResponse>, StatusCode> {
    match state.etoro_client.close_position(id, payload).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            tracing::error!("close_position failed: {:?}", e);
            Err(StatusCode::BAD_GATEWAY)
        }
    }
}

#[derive(Deserialize)]
struct HistoryQuery {
    min_date: String,
}

async fn get_history(
    State(state): State<AppState>,
    Query(params): Query<HistoryQuery>,
) -> Result<Json<Vec<TradeHistoryItem>>, StatusCode> {
    match state.etoro_client.get_history(&params.min_date).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            tracing::error!("get_history failed: {:?}", e);
            Err(StatusCode::BAD_GATEWAY)
        }
    }
}