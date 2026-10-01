use axum::{
    Extension, Json,
    extract::{Path, State},
};
use kernel::{Actor, UserId};

use crate::{context::RestCtx, dto::AccountOverviewDto, error::RestError};

/// صورة الحساب كاملة — بتيجي من 3 خدمات بتشتغل على التوازي جوّه الهَب.
/// للأدمن بس (الشرط جوّه الفلو).
pub async fn handler(
    State(ctx): State<RestCtx>,
    Extension(actor): Extension<Actor>,
    Path(id): Path<String>,
) -> Result<Json<AccountOverviewDto>, RestError> {
    let overview = ctx.hub.account_overview(&actor, &UserId::new(id)).await?;
    Ok(Json(overview.into()))
}
