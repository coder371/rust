//! أدوات GraphQL مشتركة بين البوابات (تحويل الأخطاء، الترقيم، عرض الفلوس).
//! ❗ الكريت ده مالوش أي علاقة باللوجيك ولا الداتابيز.

pub mod error;
pub mod money;
pub mod pagination;

pub use error::DomainResultExt;
pub use money::MoneyView;
pub use pagination::PageInput;
