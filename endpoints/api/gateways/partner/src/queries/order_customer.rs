use async_graphql::{Context, ID, Object, Result};
use kernel::{Actor, OrderId};
use gw_shared::DomainResultExt;

use crate::{context::PartnerCtx, guard::PartnerGuard, types::Customer};

#[derive(Default)]
pub struct OrderCustomerQuery;

// بيعدّي على فلو في الهَب لأنه بيلمس `orders` و `identity` مع بعض.

#[Object]
impl OrderCustomerQuery {
    #[graphql(guard = PartnerGuard)]
    async fn order_customer(&self, ctx: &Context<'_>, order_id: ID) -> Result<Customer> {
        let c = ctx.data::<PartnerCtx>()?;
        let actor = ctx.data::<Actor>()?;

        let customer = c
            .hub
            .order_customer(actor, &OrderId::new(order_id.0))
            .await
            .gql()?;

        Ok(customer.into())
    }
}
