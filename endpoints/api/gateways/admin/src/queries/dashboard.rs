use async_graphql::{Context, Object, Result};
use kernel::Actor;
use gw_shared::DomainResultExt;

use crate::{context::AdminCtx, guard::AdminGuard, types::Dashboard};

#[derive(Default)]
pub struct DashboardQuery;

#[Object]
impl DashboardQuery {
    /// 🔀 الإند بوينت ده بيلمس `identity` و `billing` و `orders` في نفس اللحظة.
    /// التجميع كله جوّه فلو في الهَب — البوابة بتحوّل الشكل بس.
    #[graphql(guard = AdminGuard)]
    async fn dashboard(&self, ctx: &Context<'_>) -> Result<Dashboard> {
        let c = ctx.data::<AdminCtx>()?;
        let actor = ctx.data::<Actor>()?;

        Ok(c.hub.admin_dashboard(actor).await.gql()?.into())
    }
}
