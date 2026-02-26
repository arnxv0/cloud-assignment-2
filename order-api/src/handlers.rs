use axum::{extract::State, http::StatusCode, Json};
use rusqlite::params;
use serde_json::json;

use chrono::Utc;
use uuid::Uuid;

use crate::models::CreateOrderRequest;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<rusqlite::Connection>>,
}

pub async fn create_order(
    State(state): State<AppState>,
    Json(payload): Json<CreateOrderRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let order_id = Uuid::new_v4().to_string();
    let ledger_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let db = state.db.lock().unwrap();

    db.execute(
        "INSERT INTO orders (order_id, customer_id, item_id, quantity, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            order_id,
            payload.customer_id,
            payload.item_id,
            payload.quantity,
            now
        ],
    )
    .unwrap();

    db.execute(
        "INSERT INTO ledger (ledger_id, order_id, customer_id, amount, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            ledger_id,
            order_id,
            payload.customer_id,
            payload.quantity,
            now
        ],
    )
    .unwrap();

    println!("Created order: {}", order_id);
    (
        StatusCode::CREATED,
        Json(json!({
            "order_id": order_id,
            "status": "created",
        })),
    )
}
