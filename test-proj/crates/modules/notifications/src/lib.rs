//! موديول الإشعارات. يستهلك أحداث الطلبات ولا يعرف موديول الطلبات.
pub mod application;
pub mod domain;
pub mod graphql;
pub mod infrastructure;

use application::{MessageComposer, NotificationQueries, SendOrderConfirmation};
use domain::NotificationRepository;
use infrastructure::{AiComposer, MongoNotificationRepository, TemplateComposer};
use qumra_platform::{AppState, Idempotency};
use std::sync::Arc;

#[derive(Clone)]
pub struct NotificationsModule {
    pub send_confirmation: Arc<SendOrderConfirmation>,
    pub queries: Arc<NotificationQueries>,
    pub idempotency: Arc<Idempotency>,
}

impl NotificationsModule {
    pub fn new(state: &AppState) -> Self {
        let repo: Arc<dyn NotificationRepository> =
            Arc::new(MongoNotificationRepository::new(state));

        // اختيار التنفيذ يحدث هنا مرة واحدة. حالة الاستخدام لا تعلم أيهما اختير.
        let composer: Arc<dyn MessageComposer> =
            if state.config.use_ai_composer && state.ai.is_configured() {
                tracing::info!("مؤلّف الرسائل: OpenAI");
                Arc::new(AiComposer::new(state.ai.clone()))
            } else {
                tracing::info!("مؤلّف الرسائل: قالب ثابت");
                Arc::new(TemplateComposer)
            };

        Self {
            send_confirmation: Arc::new(SendOrderConfirmation::new(
                repo.clone(),
                composer,
                state.clock.clone(),
            )),
            queries: Arc::new(NotificationQueries::new(repo)),
            idempotency: state.idempotency.clone(),
        }
    }
}
