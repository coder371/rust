use kernel::DomainError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ServiceStatus {
    Up,
    /// شغّالة بس فيها حاجة مش مظبوطة — مش سبب كافي نوقّع الريكوست.
    Degraded,
    Down,
}

impl ServiceStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Up => "up",
            Self::Degraded => "degraded",
            Self::Down => "down",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ServiceHealth {
    pub service: &'static str,
    pub status: ServiceStatus,
    /// تفصيلة مفيدة زي عدد الصفوف — مش رسالة خطأ داخلية.
    pub detail: String,
}

impl ServiceHealth {
    pub fn up(service: &'static str, detail: impl Into<String>) -> Self {
        Self {
            service,
            status: ServiceStatus::Up,
            detail: detail.into(),
        }
    }

    pub fn degraded(service: &'static str, detail: impl Into<String>) -> Self {
        Self {
            service,
            status: ServiceStatus::Degraded,
            detail: detail.into(),
        }
    }

    /// الخطأ الداخلي مابيتسرّبش — بنطلّع الكود بس.
    pub fn down(service: &'static str, err: &DomainError) -> Self {
        Self {
            service,
            status: ServiceStatus::Down,
            detail: err.code().to_owned(),
        }
    }

    pub fn is_up(&self) -> bool {
        self.status == ServiceStatus::Up
    }
}
