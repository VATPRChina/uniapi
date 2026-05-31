use vatprc_uniapi::discord::DiscordBot;
use vatprc_uniapi::services::Services;
use vatprc_uniapi::{app, settings, telemetry};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let settings = settings::Settings::new().expect("failed to load settings");
    let _telemetry = telemetry::init(&settings.telemetry)?;

    let discord_bot = DiscordBot::from_settings(&settings.discord)?;
    let services = Services::connect(&settings).await?;
    let discord_services = services.clone();

    let listener = tokio::net::TcpListener::bind(&settings.bind_address).await?;

    tracing::info!("listening on http://{}", settings.bind_address);

    tokio::try_join!(
        async {
            axum::serve(listener, app::router(services))
                .with_graceful_shutdown(shutdown_signal())
                .await
                .inspect_err(|error| tracing::error!(%error, "failed to start HTTP server"))
                .map_err(anyhow::Error::from)
        },
        async {
            if let Some(discord_bot) = discord_bot {
                discord_bot
                    .run(discord_services)
                    .await
                    .inspect_err(|error| tracing::error!(%error, "failed to start Discord bot"))
            } else {
                Ok(())
            }
        },
    )
    .map(|_| ())
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
