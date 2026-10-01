//! إقلاع فقط: اقرأ الإعدادات، افتح الاتصالات، ركّب، استمع.
//! لا منطق عمل ولا معرفة بأي موديول بعينه.
use qumra_api::{Modules, build_schema, router};
use qumra_platform::{AppState, Config};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,qumra=debug".into()),
        )
        .init();

    let config = Config::from_env()?;

    // كل العملاء يُنشأون مرة واحدة هنا، ويُعاد استخدامهم لكل طلب
    let state = AppState::connect(&config).await?;
    let modules = Modules::assemble(&state);
    let schema = build_schema(&modules);

    let listener = tokio::net::TcpListener::bind(&config.bind_addr).await?;
    tracing::info!(addr = %config.bind_addr, "qumra api يستمع على /graphql");

    axum::serve(listener, router(schema))
        .with_graceful_shutdown(shutdown())
        .await?;

    Ok(())
}

async fn shutdown() {
    let _ = tokio::signal::ctrl_c().await;
    tracing::info!("إيقاف نظيف");
}
