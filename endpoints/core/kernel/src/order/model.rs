use crate::{Money, OrderId, UserId};

#[derive(Debug, Clone)]
pub struct Order {
    pub id: OrderId,
    pub user_id: UserId,
    pub total: Money,
    pub status: OrderStatus,
    /// الشريك المسؤول عن الأوردر (لو موجود)
    pub tenant: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    Pending,
    Paid,
    Cancelled,
}
