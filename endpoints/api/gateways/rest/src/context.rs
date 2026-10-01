use std::sync::Arc;

use services::ServiceHub;

#[derive(Clone)]
pub struct RestCtx {
    pub hub: Arc<ServiceHub>,
}
