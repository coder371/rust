use crate::modules::Modules;
use async_graphql::{EmptySubscription, MergedObject, Schema};
use qumra_inventory::graphql::{InventoryMutation, InventoryQuery};
use qumra_notifications::graphql::NotificationsQuery;
use qumra_orders::graphql::{OrdersMutation, OrdersQuery};

/// سكيمة واحدة على /graphql، مدموجة من كل موديول.
/// إضافة موديول جديد = سطر واحد هنا، وصفر تعديل في الموديولات الأخرى.
#[derive(MergedObject, Default)]
pub struct QueryRoot(OrdersQuery, InventoryQuery, NotificationsQuery);

#[derive(MergedObject, Default)]
pub struct MutationRoot(OrdersMutation, InventoryMutation);

pub type QumraSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub fn build_schema(modules: &Modules) -> QumraSchema {
    Schema::build(QueryRoot::default(), MutationRoot::default(), EmptySubscription)
        .data(modules.orders.clone())
        .data(modules.inventory.clone())
        .data(modules.notifications.clone())
        .limit_depth(12)       // حدود ضد الاستعلامات المتوحّشة
        .limit_complexity(400)
        .finish()
}
