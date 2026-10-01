use async_trait::async_trait;

use super::model::User;
use crate::{DomainError, Page, UserId};

/// عقد الوصول للداتا. التنفيذ بتاعه في كريت `infra`.
#[async_trait]
pub trait UserRepo: Send + Sync + 'static {
    async fn find(&self, id: &UserId) -> Result<Option<User>, DomainError>;
    async fn list(&self, page: Page) -> Result<Vec<User>, DomainError>;
    async fn insert(&self, user: User) -> Result<User, DomainError>;
    async fn set_banned(&self, id: &UserId, banned: bool) -> Result<User, DomainError>;
    async fn count(&self) -> Result<u64, DomainError>;
}
