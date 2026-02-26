use rusqlite:: {Connection, Result};

pub fn open() -> Connection {
    Connection::open("orders.db").expect("Failed to open db - orders.db")
}


pub fn init(conn: &Connection) -> Result<()> {
    conn.execute_batch(
"
        CREATE TABLE IF NOT EXISTS orders (
            order_id    TEXT PRIMARY KEY,
            customer_id TEXT NOT NULL,
            item_id     TEXT NOT NULL,
            quantity    INTEGER NOT NULL,
            created_at  TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS ledger (
            ledger_id   TEXT PRIMARY KEY,
            order_id    TEXT NOT NULL,
            customer_id TEXT NOT NULL,
            amount      INTEGER NOT NULL,
            created_at  TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS idempotency_records (
            idempotency_key TEXT PRIMARY KEY,
            request_hash    TEXT NOT NULL,
            response_body   TEXT NOT NULL,
            status_code     INTEGER NOT NULL,
            created_at      TEXT NOT NULL
        );
"
    )?;

    Ok(())
}

