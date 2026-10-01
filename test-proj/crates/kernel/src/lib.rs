//! المفردات المشتركة بين كل الموديولات: معرّفات، مبالغ، هوية المستأجر، زمن.
//! القاعدة: أي شيء يدخل هنا يجب أن يكون محايداً تقنياً تماماً.
pub mod clock;
pub mod ids;
pub mod money;
pub mod outbox;
pub mod tenant;

pub use clock::{Clock, SystemClock};
pub use ids::{CustomerId, OrderId, ProductId, ReservationId, StoreId};
pub use money::{Currency, Money, MoneyError};
pub use outbox::OutboxRecord;
pub use tenant::{AuthError, Permission, TenantContext};
