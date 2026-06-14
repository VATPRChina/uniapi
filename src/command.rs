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
}
