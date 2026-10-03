use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

impl Cli {
    pub fn command(self) -> Command {
        self.command.unwrap_or_default()
    }
}

#[derive(Debug, Default, PartialEq, Eq, Subcommand)]
pub enum Command {
    /// Start the web application.
    #[default]
    Run,
    /// Save the OpenAPI specification to a file.
    Openapi {
        #[arg(short, long, default_value = "openapi.json")]
        output: PathBuf,
    },
    /// Apply pending database migrations.
    Migrate,
    /// Parse and expand a complete flight route with v2, printing JSON to stdout.
    RouteV2 {
        /// Complete route text, including departure and arrival (quote spaces).
        route: String,
        /// Override navdata.local_data_path from settings.
        #[arg(long)]
        navdata: Option<String>,
        /// Override navdata.preferred_routes_path from settings.
        #[arg(long)]
        preferred_routes: Option<PathBuf>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn command(arguments: &[&str]) -> Command {
        Cli::try_parse_from(arguments).unwrap().command()
    }

    #[test]
    fn defaults_to_run() {
        assert_eq!(command(&["vatprc-uniapi"]), Command::Run);
    }

    #[test]
    fn parses_run() {
        assert_eq!(command(&["vatprc-uniapi", "run"]), Command::Run);
    }

    #[test]
    fn parses_openapi_output() {
        assert_eq!(
            command(&["vatprc-uniapi", "openapi", "--output", "api.json"]),
            Command::Openapi {
                output: "api.json".into()
            }
        );
        assert_eq!(
            command(&["vatprc-uniapi", "openapi", "-o", "short.json"]),
            Command::Openapi {
                output: "short.json".into()
            }
        );
    }

    #[test]
    fn defaults_openapi_output() {
        assert_eq!(
            command(&["vatprc-uniapi", "openapi"]),
            Command::Openapi {
                output: "openapi.json".into()
            }
        );
    }

    #[test]
    fn parses_migrate() {
        assert_eq!(command(&["vatprc-uniapi", "migrate"]), Command::Migrate);
    }

    #[test]
    fn parses_route_v2_text() {
        assert_eq!(
            command(&["vatprc-uniapi", "route-v2", "ZBAA ELKUR W40 YQG ZSPD"]),
            Command::RouteV2 {
                route: "ZBAA ELKUR W40 YQG ZSPD".to_owned(),
                navdata: None,
                preferred_routes: None,
            }
        );
    }

    #[test]
    fn parses_route_v2_data_overrides() {
        assert_eq!(
            command(&[
                "vatprc-uniapi",
                "route-v2",
                "ZBAA ZSPD",
                "--navdata",
                "navdata.db?mode=ro",
                "--preferred-routes",
                "routes.csv"
            ]),
            Command::RouteV2 {
                route: "ZBAA ZSPD".to_owned(),
                navdata: Some("navdata.db?mode=ro".to_owned()),
                preferred_routes: Some("routes.csv".into()),
            }
        );
    }

    #[test]
    fn route_v2_requires_route_text() {
        assert_eq!(
            Cli::try_parse_from(["vatprc-uniapi", "route-v2"])
                .unwrap_err()
                .kind(),
            clap::error::ErrorKind::MissingRequiredArgument
        );
    }
}
