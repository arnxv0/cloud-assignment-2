use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone)]
pub struct CreateOrderRequest {
    pub customer_id: String,
    pub item_id: String,
    pub quantity: i32,
}
