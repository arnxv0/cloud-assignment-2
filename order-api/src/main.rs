mod db;
mod handlers;
mod models;

use handlers::AppState;

use axum::{
    routing::{get, post},
    Router,
};
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() {
    let conn = db::open();
    db::init(&conn).expect("DB init failed");
    println!("Database Ready!");

    let state = AppState {
        db: Arc::new(Mutex::new(conn)),
    };

    let app = Router::new()
        .route("/", get(async || "Orders server is running!"))
        .route("/orders", post(handlers::create_order))
        .route("/orders/{order_id}", get(handlers::get_order))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("Listening on http://0.0.0.0:8080");
    axum::serve(listener, app).await.unwrap();
}
