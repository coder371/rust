use crate::application::{MessageComposer, OrderSummary};
use crate::domain::NotifyError;
use qumra_platform::OpenAiClient;
use std::sync::Arc;

/// نفس العقد، تنفيذ مختلف. استبداله لا يمسّ حالة الاستخدام إطلاقاً —
/// وهذا بالضبط معنى «فصل منطق العمل عن OpenAI».
pub struct AiComposer {
    ai: Arc<OpenAiClient>,
}

impl AiComposer {
    pub fn new(ai: Arc<OpenAiClient>) -> Self {
        Self { ai }
    }
}

#[async_trait::async_trait]
impl MessageComposer for AiComposer {
    async fn order_confirmation(&self, o: &OrderSummary) -> Result<String, NotifyError> {
        let major = o.total_minor as f64 / 100.0;
        self.ai
            .complete(
                "أنت كاتب رسائل قصيرة لمتجر إلكتروني عربي. اكتب جملة واحدة ودودة.",
                &format!("تأكيد طلب رقم {} بمبلغ {:.2} {}", o.order_id, major, o.currency),
            )
            .await
            .map_err(|e| NotifyError::Compose(e.to_string()))
    }
}
