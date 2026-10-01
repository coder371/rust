use std::sync::Arc;

use services::ServiceHub;

/// البوابة بتاخد الهَب كله، مش خدمة بعينها — عشان أي إند بوينت جديد
/// يقدر يجمّع أكتر من خدمة من غير ما نغيّر التركيب.
#[derive(Clone)]
pub struct PartnerCtx {
    pub hub: Arc<ServiceHub>,
}
