use super::place_order::ApplicationError;
use crate::domain::{Order, OrderRepository};
use qumra_kernel::{OrderId, Permission, TenantContext};
use std::sync::Arc;

pub struct OrderQueries {
    orders: Arc<dyn OrderRepository>,
}

impl OrderQueries {
    pub fn new(orders: Arc<dyn OrderRepository>) -> Self {
        Self { orders }
    }

    pub async fn get(
        &self,
        ctx: &TenantContext,
        id: &OrderId,
    ) -> Result<Option<Order>, ApplicationError> {
        ctx.require(Permission::OrdersRead)?;
        Ok(self.orders.find(&ctx.store_id, id).await?)
    }

    pub async fn list(
        &self,
        ctx: &TenantContext,
        limit: i64,
    ) -> Result<Vec<Order>, ApplicationError> {
        ctx.require(Permission::OrdersRead)?;
        Ok(self.orders.list(&ctx.store_id, limit.clamp(1, 100)).await?)
    }
}
