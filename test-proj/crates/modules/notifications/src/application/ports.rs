use super::send_confirmation::OrderSummary;
use crate::domain::NotifyError;

/// مؤلّف نص الرسالة. له تنفيذان: قالب ثابت، وآخر عبر OpenAI.
///
/// وجود هذا الـ trait هو ما يفصل منطق الإشعارات عن OpenAI:
/// حالة الاستخدام لا تعرف أن هناك نموذج لغة في الصورة أصلاً.
#[async_trait::async_trait]
pub trait MessageComposer: Send + Sync + 'static {
    async fn order_confirmation(&self, order: &OrderSummary) -> Result<String, NotifyError>;
}
