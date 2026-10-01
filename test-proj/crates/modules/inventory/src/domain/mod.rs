pub mod model;
pub mod repository;
pub use model::{Reservation, StockItem};
pub use repository::{StockError, StockRepository};
