use axum::{Extension, Json, extract::{Path, State}};
use kernel::{Actor, UserId};

use crate::{context::RestCtx, dto::UserDto, error::RestError};

pub async fn handler(
    State(ctx): State<RestCtx>,
    Extension(actor): Extension<Actor>,
    Path(id): Path<String>,
) -> Result<Json<UserDto>, RestError> {
    let user = ctx.hub.identity().by_id(&actor, &UserId::new(id)).await?;
    Ok(Json(user.into()))
}
