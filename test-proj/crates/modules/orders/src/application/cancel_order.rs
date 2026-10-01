use super::place_order::ApplicationError;
use crate::domain::{OrderError, OrderRepository, OrderStatus};
use qumra_kernel::{OrderId, Permission, TenantContext};
use std::sync::Arc;

pub struct CancelOrder {
    orders: Arc<dyn OrderRepository>,
}

impl CancelOrder {
    pub fn new(orders: Arc<dyn OrderRepository>) -> Self {
        Self { orders }
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

        if order.status == OrderStatus::Cancelled {
            return Err(OrderError::AlreadyCancelled.into());
        }

        let expected = order.version;
        order.status = OrderStatus::Cancelled;
        order.version += 1;

        self.orders.save(&order, expected, &[]).await?;
        Ok(order)
    }
}
