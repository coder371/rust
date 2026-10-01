use std::sync::RwLock;

use async_trait::async_trait;
use kernel::{DomainError, OrderId, Page, UserId, order::{Order, OrderRepo}};

#[derive(Default)]
pub struct InMemoryOrderRepo {
    orders: RwLock<Vec<Order>>,
}

impl InMemoryOrderRepo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn seeded() -> Self {
        Self {
            orders: RwLock::new(super::seed::orders()),
        }
    }

    fn read(&self) -> Result<std::sync::RwLockReadGuard<'_, Vec<Order>>, DomainError> {
        self.orders
            .read()
            .map_err(|_| DomainError::Internal("order store poisoned".into()))
    }

    fn page<'a>(
        items: impl Iterator<Item = &'a Order>,
        page: Page,
    ) -> Vec<Order> {
        items
            .skip(page.offset as usize)
            .take(page.limit as usize)
            .cloned()
            .collect()
    }
}

#[async_trait]
impl OrderRepo for InMemoryOrderRepo {
    async fn find(&self, id: &OrderId) -> Result<Option<Order>, DomainError> {
        Ok(self.read()?.iter().find(|o| &o.id == id).cloned())
    }

    async fn list(&self, page: Page) -> Result<Vec<Order>, DomainError> {
        Ok(Self::page(self.read()?.iter(), page))
    }

    async fn list_by_user(&self, user_id: &UserId, page: Page) -> Result<Vec<Order>, DomainError> {
        Ok(Self::page(
            self.read()?.iter().filter(|o| &o.user_id == user_id),
            page,
        ))
    }

    async fn list_by_tenant(&self, tenant: &str, page: Page) -> Result<Vec<Order>, DomainError> {
        Ok(Self::page(
            self.read()?
                .iter()
                .filter(|o| o.tenant.as_deref() == Some(tenant)),
            page,
        ))
    }
}
