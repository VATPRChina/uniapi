mod app;
mod services;
mod settings;

use std::env;
use std::net::SocketAddr;

use services::Services;
use tokio::net::TcpListener;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "warn,vatprc_uniapi=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let settings = settings::Settings::new().expect("failed to load settings");

    let services = Services::connect(&settings.database.url).await?;

    let listener = tokio::net::TcpListener::bind(&settings.bind_address)
        .await
        .unwrap();

    tracing::info!("listening on http://{}", settings.bind_address);

    axum::serve(listener, app::router(services))
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
