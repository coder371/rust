use async_trait::async_trait;

use super::model::Order;
use crate::{DomainError, OrderId, Page, UserId};

#[async_trait]
pub trait OrderRepo: Send + Sync + 'static {
    async fn find(&self, id: &OrderId) -> Result<Option<Order>, DomainError>;
    async fn list(&self, page: Page) -> Result<Vec<Order>, DomainError>;
    async fn list_by_user(&self, user_id: &UserId, page: Page) -> Result<Vec<Order>, DomainError>;
    async fn list_by_tenant(&self, tenant: &str, page: Page) -> Result<Vec<Order>, DomainError>;
}
