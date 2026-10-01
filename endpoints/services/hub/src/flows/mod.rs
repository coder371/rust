//! الفلوهات: يوزكيسات بتلمس **أكتر من خدمة في نفس الوقت**.
//!
//! القاعدة: لو اليوزكيس بيلمس خدمة واحدة، مكانه جوّه الخدمة نفسها.
//! لو بيلمس اتنين أو أكتر، مكانه هنا — كل واحد في ملف.

mod account_overview;
mod admin_dashboard;
mod order_customer;
mod platform_health;

pub use account_overview::AccountOverview;
pub use admin_dashboard::AdminDashboard;
pub use platform_health::PlatformHealth;
