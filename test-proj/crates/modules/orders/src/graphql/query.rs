use super::{tenant, types::OrderDto};
use crate::OrdersModule;
use async_graphql::{Context, MergedObject, Object, Result};
use qumra_kernel::OrderId;

#[derive(Default)]
pub struct OrderQuery;

#[Object]
impl OrderQuery {
    /// طلب واحد بالمعرّف، ضمن المتجر الحالي فقط
    async fn order(&self, ctx: &Context<'_>, id: String) -> Result<Option<OrderDto>> {
        let m = ctx.data_unchecked::<OrdersModule>();
        let order = m.queries.get(tenant(ctx)?, &OrderId::new(id)).await?;
        Ok(order.map(OrderDto::from))
    }
}

#[derive(Default)]
pub struct OrdersListQuery;

#[Object]
impl OrdersListQuery {
    async fn orders(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 20)] limit: i64,
    ) -> Result<Vec<OrderDto>> {
        let m = ctx.data_unchecked::<OrdersModule>();
        let orders = m.queries.list(tenant(ctx)?, limit).await?;
        Ok(orders.into_iter().map(OrderDto::from).collect())
    }
}

/// كل resolver في ملفه المنطقي، والدمج هنا — نفس نمط MergedObject
#[derive(MergedObject, Default)]
pub struct OrdersQuery(OrderQuery, OrdersListQuery);
