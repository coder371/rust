use axum::{Json, extract::State};
use serde::Serialize;
use services::ServiceHealth;

use crate::context::RestCtx;

#[derive(Serialize)]
pub struct ServiceHealthDto {
    pub service: &'static str,
    pub status: &'static str,
    pub detail: String,
}

impl From<ServiceHealth> for ServiceHealthDto {
    fn from(h: ServiceHealth) -> Self {
        Self {
            service: h.service,
            status: h.status.as_str(),
            detail: h.detail,
        }
    }
}

#[derive(Serialize)]
pub struct Health {
    status: &'static str,
    services: Vec<ServiceHealthDto>,
}

/// بيفحص كل الخدمات المسجّلة في نفس اللحظة — مش واحدة ورا التانية.
pub async fn handler(State(ctx): State<RestCtx>) -> Json<Health> {
    let report = ctx.hub.platform_health().await;

    Json(Health {
        status: report.status.as_str(),
        services: report.services.into_iter().map(Into::into).collect(),
    })
}
