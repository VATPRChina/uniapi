use serenity::all::{
    Command, CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    CreateInteractionResponse, CreateInteractionResponseMessage, EditInteractionResponse,
    EventHandler, GatewayIntents, Interaction, Ready,
};
use serenity::async_trait;
use serenity::client::Client;

use crate::services::Services;
use crate::settings::Discord;

const METAR_COMMAND_NAME: &str = "metar";
const ICAO_OPTION_NAME: &str = "icao";

pub struct DiscordBot {
    token: String,
}

impl DiscordBot {
    pub fn from_settings(settings: &Discord) -> anyhow::Result<Option<Self>> {
        if !settings.enabled {
            tracing::info!("discord bot is disabled");
            return Ok(None);
        }

        let token = settings.token.trim();
        if token.is_empty() {
            anyhow::bail!("discord bot is enabled but discord.token is not configured");
        }

        Ok(Some(Self {
            token: token.to_owned(),
        }))
    }

    pub async fn run(self, services: Services) -> anyhow::Result<()> {
        tracing::info!("starting discord bot");

        let mut client = Client::builder(self.token, GatewayIntents::empty())
            .event_handler(DiscordEventHandler { services })
            .await?;

        client.start().await?;
        Ok(())
    }
}

struct DiscordEventHandler {
    services: Services,
}

#[async_trait]
impl EventHandler for DiscordEventHandler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        tracing::info!(
            user = %ready.user.name,
            "discord bot connected; registering commands"
        );

        if let Err(error) = Command::create_global_command(&ctx.http, metar_command()).await {
            tracing::error!(%error, "failed to register discord metar command");
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let Interaction::Command(command) = interaction else {
            return;
        };

        if command.data.name.as_str() == METAR_COMMAND_NAME {
            self.handle_metar_command(ctx, command).await;
        }
    }
}

impl DiscordEventHandler {
    async fn handle_metar_command(&self, ctx: Context, command: CommandInteraction) {
        let Some(icao) = command_icao(&command) else {
            respond_with_message(&ctx, &command, "ICAO is required.").await;
            return;
        };

        let normalized_icao = icao.trim().to_uppercase();
        if !is_valid_icao(&normalized_icao) {
            respond_with_message(
                &ctx,
                &command,
                "ICAO must be a four-character airport code.",
            )
            .await;
            return;
        }

        if let Err(error) = command
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Defer(CreateInteractionResponseMessage::new()),
            )
            .await
        {
            tracing::error!(%error, icao = %normalized_icao, "failed to defer discord metar command");
            return;
        }

        let metar = self.services.compat().get_metar(&normalized_icao).await;
        let content = if metar.is_empty() {
            format!("{normalized_icao} NO METAR")
        } else {
            metar
        };

        if let Err(error) = command
            .edit_response(&ctx.http, EditInteractionResponse::new().content(content))
            .await
        {
            tracing::error!(%error, icao = %normalized_icao, "failed to respond to discord metar command");
        }
    }
}

fn metar_command() -> CreateCommand {
    CreateCommand::new(METAR_COMMAND_NAME)
        .description("Returns the METAR for an airport.")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                ICAO_OPTION_NAME,
                "ICAO airport code",
            )
            .required(true)
            .min_length(4)
            .max_length(4),
        )
}

fn command_icao(command: &CommandInteraction) -> Option<&str> {
    command
        .data
        .options
        .iter()
        .find(|option| option.name == ICAO_OPTION_NAME)?
        .value
        .as_str()
}

fn is_valid_icao(icao: &str) -> bool {
    icao.len() == 4
        && icao
            .chars()
            .all(|character| character.is_ascii_alphanumeric())
}

async fn respond_with_message(ctx: &Context, command: &CommandInteraction, content: &str) {
    if let Err(error) = command
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().content(content),
            ),
        )
        .await
    {
        tracing::error!(%error, "failed to respond to discord command");
    }
}
