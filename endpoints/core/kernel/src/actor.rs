use crate::error::DomainError;

/// مين بينفّذ العملية. بتتحقن من الميدلوير بتاع كل بوابة.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Actor {
    pub id: String,
    pub role: Role,
    /// للشركاء: هو تابع لأنهي جهة.
    pub tenant: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Anonymous,
    Customer,
    Partner,
    Admin,
}

impl Actor {
    pub fn anonymous() -> Self {
        Self {
            id: "anonymous".to_owned(),
            role: Role::Anonymous,
            tenant: None,
        }
    }

    pub fn is_admin(&self) -> bool {
        self.role == Role::Admin
    }

    pub fn is_partner(&self) -> bool {
        self.role == Role::Partner
    }

    pub fn require_admin(&self) -> Result<(), DomainError> {
        if self.is_admin() {
            Ok(())
        } else {
            Err(DomainError::Forbidden("admin role required".into()))
        }
    }

    pub fn require_tenant(&self) -> Result<&str, DomainError> {
        self.tenant
            .as_deref()
            .ok_or_else(|| DomainError::Forbidden("actor has no tenant".into()))
    }
}
