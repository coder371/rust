//! موديول المخزون. واجهته العامة `InventoryModule` + `graphql`.
pub mod application;
pub mod domain;
pub mod graphql;
pub mod infrastructure;

use application::StockOps;
use domain::StockRepository;
use infrastructure::MongoStockRepository;
use qumra_platform::AppState;
use std::sync::Arc;

#[derive(Clone)]
pub struct InventoryModule {
    pub ops: Arc<StockOps>,
}

impl InventoryModule {
    pub fn new(state: &AppState) -> Self {
        let repo: Arc<dyn StockRepository> = Arc::new(MongoStockRepository::new(state));
        Self { ops: Arc::new(StockOps::new(repo)) }
    }
}
