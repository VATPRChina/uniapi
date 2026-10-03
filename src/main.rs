use std::io::Write;
use std::path::Path;

use clap::Parser;
use vatprc_uniapi::discord::DiscordBot;
use vatprc_uniapi::modules::flight::{dto::FlightLeg, flight_plan::v2};
use vatprc_uniapi::modules::navdata::service::NavdataService;
use vatprc_uniapi::services::Services;
use vatprc_uniapi::{app, command, openapi, repository, settings, telemetry};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    match command::Cli::parse().command() {
        command::Command::Run => run().await,
        command::Command::Openapi { output } => save_openapi(&output),
        command::Command::Migrate => migrate().await,
        command::Command::RouteV2 {
            route,
            navdata,
            preferred_routes,
        } => route_v2(&route, navdata.as_deref(), preferred_routes.as_deref()).await,
    }
}

async fn run() -> Result<(), anyhow::Error> {
    let settings = settings::Settings::new()?;
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

fn save_openapi(output: &Path) -> Result<(), anyhow::Error> {
    let file = std::fs::File::create(output)?;
    let mut writer = std::io::BufWriter::new(file);

    serde_json::to_writer_pretty(&mut writer, &openapi::openapi())?;
    writeln!(writer)?;

    Ok(())
}

async fn migrate() -> Result<(), anyhow::Error> {
    let settings = settings::Settings::new()?;
    repository::migration::migrate(&settings.database.url).await?;

    Ok(())
}

async fn route_v2(
    route: &str,
    navdata: Option<&str>,
    preferred_routes: Option<&Path>,
) -> Result<(), anyhow::Error> {
    let configured = settings::Settings::new()?.navdata;
    let navdata = NavdataService::with_preferred_routes_path(
        navdata.unwrap_or(&configured.local_data_path),
        preferred_routes.unwrap_or_else(|| Path::new(&configured.preferred_routes_path)),
    )
    .await?;
    let segments: Vec<FlightLeg> = v2::parse_route(&navdata, route)
        .await?
        .into_iter()
        .map(Into::into)
        .collect();
    serde_json::to_writer_pretty(std::io::stdout().lock(), &segments)?;
    std::io::stdout().lock().write_all(b"\n")?;
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
