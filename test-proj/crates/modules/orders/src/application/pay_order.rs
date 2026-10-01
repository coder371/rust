use super::place_order::ApplicationError;
use crate::domain::{OrderError, OrderRepository};
use qumra_contracts::orders::{ORDER_PAID, OrderPaidV1};
use qumra_kernel::{Clock, OrderId, OutboxRecord, Permission, TenantContext};
use std::sync::Arc;

pub struct PayOrder {
    orders: Arc<dyn OrderRepository>,
    clock: Arc<dyn Clock>,
}

impl PayOrder {
    pub fn new(orders: Arc<dyn OrderRepository>, clock: Arc<dyn Clock>) -> Self {
        Self { orders, clock }
    }

    pub async fn exec(
        &self,
        ctx: &TenantContext,
        id: &OrderId,
    ) -> Result<crate::domain::Order, ApplicationError> {
        ctx.require(Permission::OrdersWrite)?;

        let mut order = self
            .orders
            .find(&ctx.store_id, id)
            .await?
            .ok_or(OrderError::NotFound)?;

        let expected = order.version;
        order.mark_paid()?; // آلة الحالات في الـ domain هي الحكم

        let event = OutboxRecord::new(
            ORDER_PAID,
            ctx.store_id.as_str(),
            self.clock.now_ms(),
            &OrderPaidV1 {
                order_id: order.id.to_string(),
                customer_id: order.customer_id.to_string(),
                total_minor: order.total.minor,
                currency: order.total.currency.code().to_string(),
            },
        )
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        self.orders.save(&order, expected, &[event]).await?;
        Ok(order)
    }
}
