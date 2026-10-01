use async_graphql::{Context, Object, Result};
use kernel::Actor;
use gw_shared::{DomainResultExt, PageInput, pagination::page_or_default};

use crate::{context::PartnerCtx, guard::PartnerGuard, types::Order};

#[derive(Default)]
pub struct MyOrdersQuery;

#[Object]
impl MyOrdersQuery {
    /// أوردرات الشريك الحالي بس — الفلترة بالـ tenant جوّه اللوجيك.
    #[graphql(guard = PartnerGuard)]
    async fn my_orders(&self, ctx: &Context<'_>, page: Option<PageInput>) -> Result<Vec<Order>> {
        let c = ctx.data::<PartnerCtx>()?;
        let actor = ctx.data::<Actor>()?;

        let orders = c
            .hub
            .orders()
            .for_tenant(actor, page_or_default(page))
            .await
            .gql()?;

        Ok(orders.into_iter().map(Order::from).collect())
    }
}
