use async_graphql::{Context, Object, Result};
use kernel::Actor;
use gw_shared::DomainResultExt;

use crate::{context::AdminCtx, guard::AdminGuard, types::RevenueReport};

#[derive(Default)]
pub struct RevenueQuery;

#[Object]
impl RevenueQuery {
    /// تقرير الإيرادات — محمي بجارد على مستوى الحقل كمان.
    #[graphql(guard = AdminGuard)]
    async fn revenue(&self, ctx: &Context<'_>) -> Result<RevenueReport> {
        let c = ctx.data::<AdminCtx>()?;
        let actor = ctx.data::<Actor>()?;

        Ok(c.hub.billing().revenue(actor).await.gql()?.into())
    }
}
