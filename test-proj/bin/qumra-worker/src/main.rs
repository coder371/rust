//! نفس الكود ونفس AppState، جذر تركيب مختلف.
//! المهام الخلفية لا تشارك عملية الـ API في نفس الـ runtime.
use qumra_api::Modules;
use qumra_platform::{AppState, Config, outbox};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,qumra=debug".into()),
        )
        .init();

    let config = Config::from_env()?;
    let state = AppState::connect(&config).await?;
    let modules = Modules::assemble(&state);

    // مُرحِّل الـ outbox: مونجو → RabbitMQ
    tokio::spawn(outbox::relay(
        state.clone(),
        vec!["orders_outbox".to_string()],
    ));

    qumra_worker::run(state, modules).await
}
