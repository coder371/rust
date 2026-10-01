use super::{tenant, types::{OrderDto, PlaceOrderInput}};
use crate::OrdersModule;
use crate::application::{PlaceOrderCommand, PlaceOrderLine};
use async_graphql::{Context, MergedObject, Object, Result};
use qumra_kernel::{Currency, CustomerId, Money, OrderId, ProductId};

#[derive(Default)]
pub struct PlaceOrderMutation;

#[Object]
impl PlaceOrderMutation {
    /// يحوّل، ينادي، يحوّل. صفر منطق عمل هنا.
    async fn place_order(&self, ctx: &Context<'_>, input: PlaceOrderInput) -> Result<OrderDto> {
        let m = ctx.data_unchecked::<OrdersModule>();

        let cmd = PlaceOrderCommand {
            customer_id: CustomerId::new(input.customer_id),
            lines: input
                .lines
                .into_iter()
                .map(|l| PlaceOrderLine {
                    product_id: ProductId::new(l.product_id),
                    quantity: l.quantity,
                    unit_price: Money::new(l.unit_price_minor, Currency::Sar),
                })
                .collect(),
        };

        let order = m.place_order.exec(tenant(ctx)?, cmd).await?;
        Ok(OrderDto::from(order))
    }
}

#[derive(Default)]
pub struct PayOrderMutation;

#[Object]
impl PayOrderMutation {
    async fn pay_order(&self, ctx: &Context<'_>, id: String) -> Result<OrderDto> {
        let m = ctx.data_unchecked::<OrdersModule>();
        let order = m.pay_order.exec(tenant(ctx)?, &OrderId::new(id)).await?;
        Ok(OrderDto::from(order))
    }
}

#[derive(Default)]
pub struct CancelOrderMutation;

#[Object]
impl CancelOrderMutation {
    async fn cancel_order(&self, ctx: &Context<'_>, id: String) -> Result<OrderDto> {
        let m = ctx.data_unchecked::<OrdersModule>();
        let order = m.cancel_order.exec(tenant(ctx)?, &OrderId::new(id)).await?;
        Ok(OrderDto::from(order))
    }
}

#[derive(MergedObject, Default)]
pub struct OrdersMutation(PlaceOrderMutation, PayOrderMutation, CancelOrderMutation);
