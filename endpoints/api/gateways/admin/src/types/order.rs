use async_graphql::{Enum, ID, SimpleObject};
use gw_shared::MoneyView;

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
#[graphql(name = "OrderStatus")]
pub enum OrderStatus {
    Pending,
    Paid,
    Cancelled,
}

impl From<kernel::order::OrderStatus> for OrderStatus {
    fn from(s: kernel::order::OrderStatus) -> Self {
        match s {
            kernel::order::OrderStatus::Pending => Self::Pending,
            kernel::order::OrderStatus::Paid => Self::Paid,
            kernel::order::OrderStatus::Cancelled => Self::Cancelled,
        }
    }
}

/// شكل الأوردر في بوابة الإدارة — الـ tenant ظاهر هنا وبس.
#[derive(SimpleObject)]
#[graphql(name = "Order")]
pub struct Order {
    pub id: ID,
    pub customer_id: ID,
    pub total: MoneyView,
    pub status: OrderStatus,
    pub tenant: Option<String>,
}

impl From<kernel::order::Order> for Order {
    fn from(o: kernel::order::Order) -> Self {
        Self {
            id: ID(o.id.to_string()),
            customer_id: ID(o.user_id.to_string()),
            total: o.total.into(),
            status: o.status.into(),
            tenant: o.tenant,
        }
    }
}
