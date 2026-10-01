use async_graphql::{Context, InputObject, Object, Result};
use kernel::{Actor, Money, user::NewUser};
use gw_shared::DomainResultExt;

use crate::{context::AdminCtx, guard::AdminGuard, types::User};

/// الإنبوت بتاع الإند بوينت ده ساكن معاه في نفس الملف.
#[derive(InputObject)]
pub struct CreateUserInput {
    pub name: String,
    pub email: String,
    pub salary_cents: i64,
}

#[derive(Default)]
pub struct CreateUserMutation;

#[Object]
impl CreateUserMutation {
    #[graphql(guard = AdminGuard)]
    async fn create_user(&self, ctx: &Context<'_>, input: CreateUserInput) -> Result<User> {
        let c = ctx.data::<AdminCtx>()?;
        let actor = ctx.data::<Actor>()?;

        let user = c
            .hub
            .identity()
            .create(
                actor,
                NewUser {
                    name: input.name,
                    email: input.email,
                    salary: Money::from_cents(input.salary_cents),
                },
            )
            .await
            .gql()?;

        Ok(user.into())
    }
}
