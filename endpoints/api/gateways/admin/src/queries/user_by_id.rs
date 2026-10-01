use async_graphql::{Context, ID, Object, Result};
use kernel::{Actor, UserId};
use gw_shared::DomainResultExt;

use crate::{context::AdminCtx, types::User};

#[derive(Default)]
pub struct UserByIdQuery;

#[Object]
impl UserByIdQuery {
    /// مستخدم واحد بكل بياناته.
    async fn user_by_id(&self, ctx: &Context<'_>, id: ID) -> Result<User> {
        let c = ctx.data::<AdminCtx>()?;
        let actor = ctx.data::<Actor>()?;

        let user = c.hub.identity().by_id(actor, &UserId::new(id.0)).await.gql()?;
        Ok(user.into())
    }
}
