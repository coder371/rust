use async_graphql::SimpleObject;
use gw_shared::MoneyView;

#[derive(SimpleObject)]
#[graphql(name = "RevenueReport")]
pub struct RevenueReport {
    pub paid_orders: u64,
    pub pending_orders: u64,
    pub cancelled_orders: u64,
    pub total_paid: MoneyView,
}

impl From<services::RevenueReport> for RevenueReport {
    fn from(r: services::RevenueReport) -> Self {
        Self {
            paid_orders: r.paid_orders,
            pending_orders: r.pending_orders,
            cancelled_orders: r.cancelled_orders,
            total_paid: r.total_paid.into(),
        }
    }
}
