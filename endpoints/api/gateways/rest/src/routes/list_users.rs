use axum::{Extension, Json, extract::{Query, State}};
use kernel::{Actor, Page};
use serde::Deserialize;

use crate::{context::RestCtx, dto::UserDto, error::RestError};

#[derive(Deserialize)]
pub struct Params {
    #[serde(default)]
    offset: u32,
    limit: Option<u32>,
}

pub async fn handler(
    State(ctx): State<RestCtx>,
    Extension(actor): Extension<Actor>,
    Query(params): Query<Params>,
) -> Result<Json<Vec<UserDto>>, RestError> {
    let page = Page::new(params.offset, params.limit.unwrap_or(20));
    let users = ctx.hub.identity().list(&actor, page).await?;
    Ok(Json(users.into_iter().map(UserDto::from).collect()))
}
