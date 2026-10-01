use async_graphql::{Context, ID, Object, Result};
use kernel::{Actor, UserId};
use gw_shared::DomainResultExt;

use crate::{context::PublicCtx, types::User};

#[derive(Default)]
pub struct UserProfileQuery;

#[Object]
impl UserProfileQuery {
    async fn user(&self, ctx: &Context<'_>, id: ID) -> Result<User> {
        let c = ctx.data::<PublicCtx>()?;
        let actor = ctx.data::<Actor>()?;

        let user = c
            .hub
            .identity()
            .public_profile(actor, &UserId::new(id.0))
            .await
            .gql_public()?;

        Ok(user.into())
    }
}
