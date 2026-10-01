use kernel::{Actor, DomainError, Role};

/// تحويل التوكن لـ Actor. كل بوابة بتستدعيه من الميدلوير بتاعها.
pub struct AuthService {
    admin_token: String,
    partner_token: String,
    customer_token: String,
}

impl AuthService {
    pub fn new(admin_token: String, partner_token: String, customer_token: String) -> Self {
        Self {
            admin_token,
            partner_token,
            customer_token,
        }
    }

    /// بياخد قيمة هيدر `Authorization` كاملة زي `Bearer xxx`.
    pub fn authenticate(&self, header: Option<&str>) -> Result<Actor, DomainError> {
        let Some(token) = header.and_then(|h| h.strip_prefix("Bearer ")) else {
            return Err(DomainError::Unauthorized);
        };

        if token == self.admin_token {
            Ok(Actor {
                id: "admin".into(),
                role: Role::Admin,
                tenant: None,
            })
        } else if let Some(tenant) = token.strip_prefix(&format!("{}:", self.partner_token)) {
            // شكل توكن الشريك: `Bearer <partner_token>:<tenant>`
            Ok(Actor {
                id: format!("partner:{tenant}"),
                role: Role::Partner,
                tenant: Some(tenant.to_owned()),
            })
        } else if token == self.customer_token {
            Ok(Actor {
                id: "customer".into(),
                role: Role::Customer,
                tenant: None,
            })
        } else {
            Err(DomainError::Unauthorized)
        }
    }

    /// للبوابة العامة: التوكن اختياري، ولو غلط بنرجّع زائر بدل ما نرفض.
    pub fn authenticate_optional(&self, header: Option<&str>) -> Actor {
        self.authenticate(header).unwrap_or_else(|_| Actor::anonymous())
    }
}
