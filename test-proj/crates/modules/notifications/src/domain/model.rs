use qumra_kernel::StoreId;

#[derive(Debug, Clone)]
pub struct Notification {
    pub id: String,
    pub store_id: StoreId,
    pub order_id: String,
    pub channel: String,
    pub body: String,
    pub sent_at_ms: i64,
}
