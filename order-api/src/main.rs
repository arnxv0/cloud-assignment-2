mod db;

use axum::{routing::get, Router};

#[tokio::main]
async fn main() {
    let conn = db::open();
    db::init(&conn).expect("DB init failed");
    println!("Database Ready!");

    let app = Router::new().route("/", get(async || "Orders server is running!"));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("Listening on http://0.0.0.0:8080");
    axum::serve(listener, app).await.unwrap();
}
