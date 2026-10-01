//! خدمة الهوية: المستخدمين + التوكنات.
//!
//! كل يوزكيس في ملف لوحده وكلهم بيضيفوا ميثودز على نفس `IdentityService`.
//! الاسم بيوصف المجال مش البوابة — عشان أكتر من بوابة تستعملها في نفس الوقت.

use std::sync::Arc;

use async_trait::async_trait;
use kernel::user::UserRepo;
use service_runtime::{Service, ServiceHealth};

mod auth;
mod ban_user;
mod create_user;
mod public_view;
mod read;

pub use auth::AuthService;

pub struct IdentityService {
    pub(crate) users: Arc<dyn UserRepo>,
}

impl IdentityService {
    pub fn new(users: Arc<dyn UserRepo>) -> Self {
        Self { users }
    }
}

#[async_trait]
impl Service for IdentityService {
    fn name(&self) -> &'static str {
        "identity"
    }

    fn description(&self) -> &'static str {
        "المستخدمين والصلاحيات"
    }

    async fn health(&self) -> ServiceHealth {
        match self.users.count().await {
            Ok(n) => ServiceHealth::up(self.name(), format!("{n} users")),
            Err(err) => ServiceHealth::down(self.name(), &err),
        }
    }
}
