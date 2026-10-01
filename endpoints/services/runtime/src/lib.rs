//! ❷ البيزنس لوجيك — الأرضية المشتركة اللي بتخلّي أكتر من خدمة تشتغل مع بعض.
//!
//! كل خدمة (`svc-*`) بتنفّذ `Service`، والـ [`ServiceRegistry`] بيمسكهم كلهم
//! ويقدر يكلّمهم في نفس اللحظة بدل ما يعدّي عليهم واحد ورا التاني.
//! الكريت ده مايعرفش خدمة بعينها — عشان كده أي خدمة جديدة بتتضاف من غير ما يتلمس.

mod health;
mod registry;

pub use health::{ServiceHealth, ServiceStatus};
pub use registry::ServiceRegistry;

use async_trait::async_trait;

/// أي حاجة بتتسجّل في الهَب لازم تنفّذ العقد ده.
#[async_trait]
pub trait Service: Send + Sync + 'static {
    /// الاسم اللي بيظهر في الـ health وفي اللوجز.
    fn name(&self) -> &'static str;

    /// سطر واحد بيوصف الخدمة — بيظهر في `/rest/health`.
    fn description(&self) -> &'static str {
        ""
    }

    /// فحص سريع. بيتنادى على كل الخدمات في نفس الوقت، فخلّيه رخيص.
    async fn health(&self) -> ServiceHealth;
}
