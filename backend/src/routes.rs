// /backend/src/routes.rs
// Example: `curl -X POST http://127.0.0.1:8080/api/orders -d ...`

use axum::{
    extract::{Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
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
        .route("/", get(dashboard))
        .route("/health", get(health))
        .route("/api/orders", post(create_order))
        .route("/api/instruments/search", get(search_instrument))
        .route("/api/instruments/rates", get(get_rates))
        .route("/api/portfolio", get(get_portfolio))
        .route("/api/positions/{id}/close", post(close_position))
        .route("/api/history", get(get_history))
        .with_state(state)
}

async fn dashboard() -> impl IntoResponse {
    let html = r#"<!DOCTYPE html>
<html lang="fr">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Autoro — Dashboard</title>
  <style>
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body { font-family: monospace; background: #0d0d0d; color: #e0e0e0; padding: 24px; }
    h1 { color: #00ff88; margin-bottom: 24px; font-size: 1.4rem; }
    .grid { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; margin-bottom: 16px; }
    .card { background: #1a1a1a; border: 1px solid #2a2a2a; border-radius: 8px; padding: 16px; }
    .card h2 { color: #888; font-size: 0.75rem; text-transform: uppercase; margin-bottom: 12px; }
    .balance { font-size: 2rem; color: #00ff88; }
    .positions-count { font-size: 2rem; color: #00aaff; }
    table { width: 100%; border-collapse: collapse; font-size: 0.85rem; }
    th { color: #555; text-align: left; padding: 6px 8px; border-bottom: 1px solid #2a2a2a; }
    td { padding: 8px; border-bottom: 1px solid #1f1f1f; }
    .buy { color: #00ff88; }
    .sell { color: #ff4455; }
    .pnl-pos { color: #00ff88; }
    .pnl-neg { color: #ff4455; }
    .refresh { color: #555; font-size: 0.75rem; margin-top: 16px; }
    .error { color: #ff4455; padding: 8px; }
  </style>
</head>
<body>
  <h1>⚡ Autoro — Trading Demo</h1>
  <div class="grid">
    <div class="card">
      <h2>Solde disponible</h2>
      <div class="balance" id="balance">…</div>
    </div>
    <div class="card">
      <h2>Positions ouvertes</h2>
      <div class="positions-count" id="pos-count">…</div>
    </div>
  </div>
  <div class="card">
    <h2>Positions</h2>
    <table>
      <thead>
        <tr>
          <th>ID</th><th>Instrument</th><th>Dir</th>
          <th>Entrée</th><th>Montant</th><th>Units</th>
          <th>SL</th><th>TP</th>
        </tr>
      </thead>
      <tbody id="positions-body"><tr><td colspan="8">Chargement…</td></tr></tbody>
    </table>
  </div>
  <div class="refresh" id="last-update"></div>

  <script>
    const INSTRUMENTS = { 100001: 'ETH', 100063: 'SOL', 100000: 'BTC' };

    async function refresh() {
      try {
        const r = await fetch('/api/portfolio');
        if (!r.ok) throw new Error(r.status);
        const p = await r.json();

        document.getElementById('balance').textContent = '$' + p.credit.toLocaleString('fr-FR', {minimumFractionDigits: 2});
        document.getElementById('pos-count').textContent = p.positions.length;

        const tbody = document.getElementById('positions-body');
        if (p.positions.length === 0) {
          tbody.innerHTML = '<tr><td colspan="8" style="color:#555">Aucune position ouverte</td></tr>';
        } else {
          tbody.innerHTML = p.positions.map(pos => {
            const symbol = INSTRUMENTS[pos.instrumentID] || pos.instrumentID;
            const dir = pos.isBuy ? '<span class="buy">LONG</span>' : '<span class="sell">SHORT</span>';
            return `<tr>
              <td>${pos.positionID}</td>
              <td>${symbol}</td>
              <td>${dir}</td>
              <td>${pos.openRate.toFixed(4)}</td>
              <td>$${pos.amount.toFixed(2)}</td>
              <td>${pos.units.toFixed(6)}</td>
              <td>${pos.stopLossRate ? pos.stopLossRate.toFixed(4) : '—'}</td>
              <td>${pos.takeProfitRate ? pos.takeProfitRate.toFixed(4) : '—'}</td>
            </tr>`;
          }).join('');
        }
        document.getElementById('last-update').textContent = 'Mis à jour : ' + new Date().toLocaleTimeString('fr-FR');
      } catch(e) {
        document.getElementById('last-update').textContent = 'Erreur : ' + e.message;
      }
    }

    refresh();
    setInterval(refresh, 10000);
  </script>
</body>
</html>"#;
    ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], html)
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

async fn close_position(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<i64>,
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