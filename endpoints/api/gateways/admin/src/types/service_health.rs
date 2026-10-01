use async_graphql::{Enum, SimpleObject};

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
#[graphql(name = "ServiceStatus")]
pub enum ServiceStatus {
    Up,
    Degraded,
    Down,
}

impl From<services::ServiceStatus> for ServiceStatus {
    fn from(s: services::ServiceStatus) -> Self {
        match s {
            services::ServiceStatus::Up => Self::Up,
            services::ServiceStatus::Degraded => Self::Degraded,
            services::ServiceStatus::Down => Self::Down,
        }
    }
}

/// حال خدمة واحدة من الخدمات المسجّلة في الهَب.
#[derive(SimpleObject)]
#[graphql(name = "ServiceHealth")]
pub struct ServiceHealth {
    pub service: String,
    pub status: ServiceStatus,
    pub detail: String,
}

impl From<services::ServiceHealth> for ServiceHealth {
    fn from(h: services::ServiceHealth) -> Self {
        Self {
            service: h.service.to_owned(),
            status: h.status.into(),
            detail: h.detail,
        }
    }
}
