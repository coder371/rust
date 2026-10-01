use async_graphql::{Context, ID, Object, Result};
use kernel::{Actor, UserId};
use gw_shared::DomainResultExt;

use crate::{context::AdminCtx, guard::AdminGuard, types::User};

#[derive(Default)]
pub struct BanUserMutation;

#[Object]
impl BanUserMutation {
    #[graphql(guard = AdminGuard)]
    async fn set_user_banned(&self, ctx: &Context<'_>, id: ID, banned: bool) -> Result<User> {
        let c = ctx.data::<AdminCtx>()?;
        let actor = ctx.data::<Actor>()?;

        let user = c
            .hub
            .identity()
            .set_banned(actor, &UserId::new(id.0), banned)
            .await
            .gql()?;

        Ok(user.into())
    }
}
