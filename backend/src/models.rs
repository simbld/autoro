// /backend/src/models.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderSide {
    Buy,
    Sell
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderStatus {
    Open,
    Closed,
    Cancelled,
    Rejected,
    Pending,
    Executed,
    Expired,
    Partial,
}

#[derive(Debug, Serialize)]
pub struct Health {
    pub ok: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrderRequest {
    pub symbol: String,
    pub order_side: OrderSide,
    pub amount: f64,
    pub take_profit: Option<f64>,
    pub stop_loss: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrderResponse {
    pub order_id: String,
    pub order_status: OrderStatus,
    pub order_message: Option<String>,
}