use async_graphql::{Context, Object, Result};
use kernel::Actor;
use gw_shared::{DomainResultExt, PageInput, pagination::page_or_default};

use crate::{context::PublicCtx, types::User};

#[derive(Default)]
pub struct BrowseUsersQuery;

#[Object]
impl BrowseUsersQuery {
    /// المستخدمين الظاهرين للعامة (المحظورين مش بيظهروا).
    async fn users(&self, ctx: &Context<'_>, page: Option<PageInput>) -> Result<Vec<User>> {
        let c = ctx.data::<PublicCtx>()?;
        let actor = ctx.data::<Actor>()?;

        let users = c
            .hub
            .identity()
            .browse(actor, page_or_default(page))
            .await
            .gql_public()?;

        Ok(users.into_iter().map(User::from).collect())
    }
}
