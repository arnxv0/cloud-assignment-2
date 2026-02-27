# cloud-assignment-2

## Deployment

**EC2:** t3.micro, Amazon Linux 2023

**Security Group Inbound Rules:**

| Port | Source |
|---|---|
| 22 (SSH) | My IP |
| 8080 (HTTP) | 0.0.0.0/0 |

### First Time Setup on EC2

```bash
ssh -i ~/.ssh/your-key.pem ec2-user@44.200.32.181

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
sudo dnf install gcc -y
```

### Deploy

```bash
# From your local machine
scp -i ~/.ssh/your-key.pem -r ./order-api ec2-user@44.200.32.181:~/order-api

# On EC2
cd order-api
cargo build --release
nohup ./target/release/order-api > app.log 2>&1 &
cat app.log
```

### Redeploy After Changes

```bash
./deploy.sh
```

---

## Endpoints

### `POST /orders`

**Required headers:**
- `Content-Type: application/json`
- `Idempotency-Key: <unique-string>`

**Optional debug header:**
- `X-Debug-Fail-After-Commit: true` — commits to DB then returns 500 (simulates lost response)

**Body:**
```json
{
  "customer_id": "cust-42",
  "item_id": "item-9",
  "quantity": 1
}
```

**Response codes:**

| Code | Reason |
|---|---|
| 201 | Order created |
| 200 | Duplicate request, original response replayed |
| 400 | Missing `Idempotency-Key` |
| 409 | Same key, different payload |
| 500 | Server error |

---

### `GET /orders/:order_id`

Returns order by ID. 404 if not found.

---

## Verification

**Step 1 — Create order**
```bash
curl -X POST http://44.200.32.181:8080/orders \
  -H "Content-Type: application/json" \
  -H "Idempotency-Key: test-1" \
  -d '{"customer_id":"cust3","item_id":"item1","quantity":1}'
```
Expected: `201` with `order_id` and `"status":"created"`

**Step 2 — Retry same key + payload**
```bash
curl -X POST http://44.200.32.181:8080/orders \
  -H "Content-Type: application/json" \
  -H "Idempotency-Key: test-1" \
  -d '{"customer_id":"cust3","item_id":"item1","quantity":1}'
```
Expected: same `201`, same `order_id`, no duplicate DB row

**Step 3 — Same key, different payload**
```bash
curl -X POST http://44.200.32.181:8080/orders \
  -H "Content-Type: application/json" \
  -H "Idempotency-Key: test-1" \
  -d '{"customer_id":"cust3","item_id":"item1","quantity":5}'
```
Expected: `409 Conflict`

**Step 4 — Simulate failure after commit**
```bash
curl -X POST http://44.200.32.181:8080/orders \
  -H "Content-Type: application/json" \
  -H "Idempotency-Key: test-fail-3" \
  -H "X-Debug-Fail-After-Commit: true" \
  -d '{"customer_id":"cust4","item_id":"item2","quantity":1}'
```
Expected: `500`, but order is committed to DB

**Step 5 — Retry after failure**
```bash
curl -X POST http://44.200.32.181:8080/orders \
  -H "Content-Type: application/json" \
  -H "Idempotency-Key: test-fail-3" \
  -d '{"customer_id":"cust4","item_id":"item2","quantity":1}'
```
Expected: `201`, same `order_id`, no duplicate rows

**Step 6 — Fetch order**
```bash
curl http://44.200.32.181:8080/orders/5cc5e97d-b24f-409a-ac8a-5293e7350b16
```
Expected: full order JSON

**Check for duplicates (on EC2)**
```bash
sqlite3 ~/order-api/orders.db "SELECT COUNT(*) FROM orders WHERE customer_id='cust3';"
```

```bash
sqlite3 ~/order-api/orders.db "SELECT COUNT(*) FROM ledger WHERE customer_id='cust3';"
```
