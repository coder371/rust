use kernel::{Actor, DomainResult, Money, Page, order::OrderStatus};

#[derive(Debug, Clone)]
pub struct RevenueReport {
    pub paid_orders: u64,
    pub pending_orders: u64,
    pub cancelled_orders: u64,
    pub total_paid: Money,
}

impl super::BillingService {
    pub async fn revenue(&self, actor: &Actor) -> DomainResult<RevenueReport> {
        actor.require_admin()?;

        let orders = self.orders.list(Page::new(0, Page::MAX_LIMIT)).await?;

        let mut report = RevenueReport {
            paid_orders: 0,
            pending_orders: 0,
            cancelled_orders: 0,
            total_paid: Money::ZERO,
        };

        for order in &orders {
            match order.status {
                OrderStatus::Paid => {
                    report.paid_orders += 1;
                    report.total_paid = report.total_paid + order.total;
                }
                OrderStatus::Pending => report.pending_orders += 1,
                OrderStatus::Cancelled => report.cancelled_orders += 1,
            }
        }

        Ok(report)
    }
}
