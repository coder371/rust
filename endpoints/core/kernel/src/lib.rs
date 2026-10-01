//! ❶ الكور — الحقيقة الواحدة للمشروع:
//! الموديلات + عقود المستودعات (traits) + الأخطاء + هوية المنفّذ.
//!
//! القاعدة: الكريت ده مايعتمدش على async-graphql ولا axum ولا أي درايفر داتابيز،
//! ولا يعرف حاجة عن الخدمات ولا البوابات. كل الطبقات فوقه بتتكلم بلغته.

pub mod actor;
pub mod error;
pub mod ids;
pub mod money;
pub mod order;
pub mod page;
pub mod user;

pub use actor::{Actor, Role};
pub use error::DomainError;
pub use ids::{OrderId, UserId};
pub use money::Money;
pub use page::Page;

pub type DomainResult<T> = Result<T, DomainError>;
