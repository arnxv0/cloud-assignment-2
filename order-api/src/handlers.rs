use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use rusqlite::params;
use serde_json::json;

use chrono::Utc;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::models::CreateOrderRequest;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<rusqlite::Connection>>,
}

fn hash_body(s: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    hex::encode(hasher.finalize())
}

pub async fn create_order(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateOrderRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let idem_key = match headers
        .get("idempotency-key")
        .and_then(|v| v.to_str().ok())
        .map(String::from)
    {
        Some(key) => key,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Missing idempotency-key header"})),
            )
        }
    };

    let body_str = serde_json::to_string(&json!({
        "customer_id": payload.customer_id,
        "item_id": payload.item_id,
        "quantity": payload.quantity,
    }))
    .unwrap();

    let req_hash = hash_body(&body_str);
    let db = state.db.lock().unwrap();

    let existing = db.query_row(
        "SELECT request_hash, response_body, status_code
         FROM idempotency_records WHERE idempotency_key = ?1",
        params![idem_key],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i32>(2)?,
            ))
        },
    );

    if let Ok((stored_hash, stored_response, stored_status)) = existing {
        if stored_hash != req_hash {
            return (
                StatusCode::CONFLICT,
                Json(json!({"error": "Idempotency key reused with different payload"})),
            );
        }

        let cached: serde_json::Value = serde_json::from_str(&stored_response).unwrap();
        let status = StatusCode::from_u16(stored_status as u16).unwrap_or(StatusCode::OK);

        return (status, Json(cached));
    }

    let order_id = Uuid::new_v4().to_string();
    let ledger_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let response = json!({ "order_id": order_id, "status": "created" });
    let resp_str = response.to_string();
    db.execute_batch("BEGIN;").unwrap();

    let result = (|| -> rusqlite::Result<()> {
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
        )?;
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
        )?;
        db.execute(
            "INSERT INTO idempotency_records
             (idempotency_key, request_hash, response_body, status_code, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![idem_key, req_hash, resp_str, 201i32, now],
        )?;

        Ok(())
    })();

    match result {
        Ok(_) => {
            db.execute_batch("COMMIT;").unwrap();
        }
        Err(e) => {
            db.execute_batch("ROLLBACK;").ok();
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            );
        }
    }
    (StatusCode::CREATED, Json(response))
}

pub async fn get_order(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    let db = state.db.lock().unwrap();

    let result = db.query_row(
        "SELECT order_id, customer_id, item_id, quantity, created_at
         FROM orders WHERE order_id = ?1",
        params![order_id],
        |row| {
            Ok(json!({
                "order_id":    row.get::<_, String>(0)?,
                "customer_id": row.get::<_, String>(1)?,
                "item_id":     row.get::<_, String>(2)?,
                "quantity":    row.get::<_, i32>(3)?,
                "created_at":  row.get::<_, String>(4)?,
            }))
        },
    );

    match result {
        Ok(order) => (StatusCode::OK, Json(order)),
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Order not found"})),
        ),
    }
}
