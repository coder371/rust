//! أدوات مشتركة للمشروع.
//!
//! كل ملف جوّه المجلد ده = موديول ابن، لازم يتعلن هنا بـ `mod`.
//! الـ `pub use` تحت بتقصّر المسار: بدل `utils::password::hash_password`
//! تقدر تكتب `utils::hash_password`.

pub mod config;
pub mod jwt;
pub mod password;

pub use jwt::{Claims, create_token, verify_token};
pub use password::{hash_password, verify_password};
