use async_graphql::{Context, Object, Result};
use kernel::Actor;
use gw_shared::{DomainResultExt, PageInput, pagination::page_or_default};

use crate::{context::AdminCtx, types::User};

#[derive(Default)]
pub struct ListUsersQuery;

#[Object]
impl ListUsersQuery {
    /// كل المستخدمين — بما فيهم المحظورين.
    async fn users(&self, ctx: &Context<'_>, page: Option<PageInput>) -> Result<Vec<User>> {
        let c = ctx.data::<AdminCtx>()?;
        let actor = ctx.data::<Actor>()?;

        let users = c
            .hub
            .identity()
            .list_all(actor, page_or_default(page))
            .await
            .gql()?;

        Ok(users.into_iter().map(User::from).collect())
    }
}
