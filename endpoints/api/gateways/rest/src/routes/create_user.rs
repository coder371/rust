use axum::{Extension, Json, extract::State, http::StatusCode};
use kernel::{Actor, Money, user::NewUser};

use crate::{context::RestCtx, dto::{CreateUserBody, UserDto}, error::RestError};

/// محتاج توكن أدمن — نفس القاعدة اللي في `IdentityService::create`.
pub async fn handler(
    State(ctx): State<RestCtx>,
    Extension(actor): Extension<Actor>,
    Json(body): Json<CreateUserBody>,
) -> Result<(StatusCode, Json<UserDto>), RestError> {
    let user = ctx
        .hub
        .identity()
        .create(
            &actor,
            NewUser {
                name: body.name,
                email: body.email,
                salary: Money::from_cents(body.salary_cents),
            },
        )
        .await?;

    Ok((StatusCode::CREATED, Json(user.into())))
}
