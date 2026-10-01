use crate::application::{MessageComposer, OrderSummary};
use crate::domain::NotifyError;

/// التنفيذ الافتراضي: قالب ثابت، بلا شبكة وبلا تكلفة.
pub struct TemplateComposer;

#[async_trait::async_trait]
impl MessageComposer for TemplateComposer {
    async fn order_confirmation(&self, o: &OrderSummary) -> Result<String, NotifyError> {
        // المبلغ مخزَّن بالوحدة الصغرى، والعرض فقط هو من يقسّم على ١٠٠
        let major = o.total_minor as f64 / 100.0;
        Ok(format!(
            "تم استلام طلبك رقم {} بمبلغ {:.2} {}. شكراً لتسوّقك معنا.",
            &o.order_id[..8.min(o.order_id.len())],
            major,
            o.currency
        ))
    }
}
