use serenity::all::{
    Command, Context, CreateCommand, CreateInteractionResponse, CreateInteractionResponseMessage,
    EventHandler, GatewayIntents, Interaction, Ready,
};
use serenity::async_trait;
use serenity::client::Client;

use crate::settings::Discord;

const PING_COMMAND_NAME: &str = "ping";

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

    pub async fn run(self) -> anyhow::Result<()> {
        tracing::info!("starting discord bot");

        let mut client = Client::builder(self.token, GatewayIntents::empty())
            .event_handler(DiscordEventHandler)
            .await?;

        client.start().await?;
        Ok(())
    }
}

struct DiscordEventHandler;

#[async_trait]
impl EventHandler for DiscordEventHandler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        tracing::info!(
            user = %ready.user.name,
            "discord bot connected; registering commands"
        );

        if let Err(error) = Command::create_global_command(
            &ctx.http,
            CreateCommand::new(PING_COMMAND_NAME).description("Replies with pong."),
        )
        .await
        {
            tracing::error!(%error, "failed to register discord ping command");
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let Interaction::Command(command) = interaction else {
            return;
        };

        if command.data.name != PING_COMMAND_NAME {
            return;
        }

        if let Err(error) = command
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new().content("pong"),
                ),
            )
            .await
        {
            tracing::error!(%error, "failed to respond to discord ping command");
        }
    }
}
