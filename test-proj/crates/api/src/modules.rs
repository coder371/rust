use crate::adapters::InProcessInventoryAdapter;
use qumra_inventory::InventoryModule;
use qumra_notifications::NotificationsModule;
use qumra_orders::OrdersModule;
use qumra_orders::application::InventoryPort;
use qumra_platform::AppState;
use std::sync::Arc;

/// كل الأسلاك بين الموديولات في هذا الملف وحده.
/// أي ملف آخر يذكر موديولين معاً هو خرق للقاعدة السادسة.
pub struct Modules {
    pub orders: OrdersModule,
    pub inventory: InventoryModule,
    pub notifications: NotificationsModule,
}

impl Modules {
    pub fn assemble(state: &AppState) -> Self {
        let inventory = InventoryModule::new(state);
        let notifications = NotificationsModule::new(state);

        // ← نقطة الاستخراج المستقبلية بالكامل: هذا السطر وحده
        let inventory_port: Arc<dyn InventoryPort> =
            Arc::new(InProcessInventoryAdapter::new(inventory.clone()));

        let orders = OrdersModule::new(state, inventory_port);

        Self { orders, inventory, notifications }
    }
}
