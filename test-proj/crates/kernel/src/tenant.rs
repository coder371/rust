use crate::ids::StoreId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    OrdersRead,
    OrdersWrite,
    InventoryRead,
    InventoryWrite,
    NotificationsRead,
}

#[derive(Debug, thiserror::Error)]
#[error("صلاحية غير كافية: {0:?}")]
pub struct AuthError(pub Permission);

/// هوية الطلب: أي متجر، وأي فاعل، وبأي صلاحيات.
/// كل حالة استخدام تبدأ بالتحقق منه، وكل استعلام تخزين مقيّد بـ store_id.
#[derive(Debug, Clone)]
pub struct TenantContext {
    pub store_id: StoreId,
    pub actor: String,
    pub permissions: Vec<Permission>,
}

impl TenantContext {
    pub fn require(&self, p: Permission) -> Result<(), AuthError> {
        if self.permissions.contains(&p) {
            Ok(())
        } else {
            Err(AuthError(p))
        }
    }
}
