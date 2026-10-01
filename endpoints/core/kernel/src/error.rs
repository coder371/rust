#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("unauthorized")]
    Unauthorized,

    #[error("forbidden: {0}")]
    Forbidden(String),

    #[error("invalid input: {0}")]
    Invalid(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl DomainError {
    /// كود ثابت بتستعمله البوابات في `extensions` بتاعة الإيرور.
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "NOT_FOUND",
            Self::Unauthorized => "UNAUTHORIZED",
            Self::Forbidden(_) => "FORBIDDEN",
            Self::Invalid(_) => "INVALID_INPUT",
            Self::Conflict(_) => "CONFLICT",
            Self::Internal(_) => "INTERNAL",
        }
    }

    /// هل الرسالة آمنة إننا نطلّعها لبرّه؟ (البوابة العامة بتخفي التفاصيل الداخلية)
    pub fn is_public(&self) -> bool {
        !matches!(self, Self::Internal(_))
    }
}
