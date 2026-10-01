//! موديول الطلبات.
//!
//! الواجهة العامة المقصودة: `OrdersModule` + `graphql` + `application::ports`.
//! كل ما عدا ذلك تفاصيل داخلية لا يراها أحد.
pub mod application;
pub mod domain;
pub mod graphql;
pub mod infrastructure;

use application::{CancelOrder, InventoryPort, OrderQueries, PayOrder, PlaceOrder};
use domain::OrderRepository;
use infrastructure::MongoOrderRepository;
use qumra_platform::AppState;
use std::sync::Arc;

/// حالات الاستخدام مجمّعة. تُبنى مرة عند الإقلاع وتُسجَّل في سياق GraphQL.
#[derive(Clone)]
pub struct OrdersModule {
    pub place_order: Arc<PlaceOrder>,
    pub pay_order: Arc<PayOrder>,
    pub cancel_order: Arc<CancelOrder>,
    pub queries: Arc<OrderQueries>,
}

impl OrdersModule {
    /// لاحظ التوقيع: الموديول يستقبل الـ port جاهزاً من جذر التركيب،
    /// ولا يعرف من ينفّذه. هذا هو كل سرّ قابلية الاستخراج.
    pub fn new(state: &AppState, inventory: Arc<dyn InventoryPort>) -> Self {
        let repo: Arc<dyn OrderRepository> = Arc::new(MongoOrderRepository::new(state));

        Self {
            place_order: Arc::new(PlaceOrder::new(
                repo.clone(),
                inventory,
                state.clock.clone(),
            )),
            pay_order: Arc::new(PayOrder::new(repo.clone(), state.clock.clone())),
            cancel_order: Arc::new(CancelOrder::new(repo.clone())),
            queries: Arc::new(OrderQueries::new(repo)),
        }
    }
}
