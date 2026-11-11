pub mod models;
pub mod service;

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

        let metar = self.services.flight().metar(&normalized_icao).await;
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

#[derive(Debug, thiserror::Error)]
pub enum DiscordPublishError {
    #[error("Discord event publishing is not configured")]
    NotConfigured,
    #[error("invalid Discord message ID")]
    InvalidMessageId,
    #[error(transparent)]
    Request(#[from] serenity::Error),
}

/// Publishes website events as forum posts; the starter message has the thread's ID.
#[derive(Clone)]
pub struct DiscordEventPublisher {
    http: std::sync::Arc<serenity::http::Http>,
    forum: serenity::all::ChannelId,
    tags: Vec<serenity::all::ForumTagId>,
}

impl DiscordEventPublisher {
    pub fn from_settings(settings: &Discord) -> Option<Self> {
        if !settings.enabled || settings.token.trim().is_empty() {
            return None;
        }
        Some(Self {
            http: std::sync::Arc::new(serenity::http::Http::new(settings.token.trim())),
            forum: serenity::all::ChannelId::new(
                settings.event_forum_channel_id.filter(|id| *id != 0)?,
            ),
            tags: settings
                .event_forum_tag_ids
                .iter()
                .copied()
                .filter(|id| *id != 0)
                .map(serenity::all::ForumTagId::new)
                .collect(),
        })
    }

    pub async fn sync(
        &self,
        event: &mut crate::modules::event::models::Event,
    ) -> Result<(), DiscordPublishError> {
        use serenity::all::{
            ChannelId, CreateAllowedMentions, CreateForumPost, CreateMessage, EditMessage,
            EditThread, MessageId,
        };
        let title = forum_title(&event.title);
        let embed = event_embed(event);
        if let Some(message) = &event.discord_message {
            let id = message
                .message_id
                .parse::<u64>()
                .ok()
                .filter(|id| *id != 0)
                .ok_or(DiscordPublishError::InvalidMessageId)?;
            let channel = ChannelId::new(id);
            // Archived forum posts must be reopened before editing their starter message.
            channel
                .edit_thread(&self.http, EditThread::new().archived(false).name(title))
                .await?;
            channel
                .edit_message(
                    &self.http,
                    MessageId::new(id),
                    EditMessage::new()
                        .embed(embed)
                        .allowed_mentions(CreateAllowedMentions::new()),
                )
                .await?;
        } else {
            let thread = self
                .forum
                .create_forum_post(
                    &self.http,
                    CreateForumPost::new(
                        title,
                        CreateMessage::new()
                            .embed(embed)
                            .allowed_mentions(CreateAllowedMentions::new()),
                    )
                    .set_applied_tags(self.tags.clone()),
                )
                .await?;
            event.discord_message = Some(models::DiscordMessage {
                message_id: thread.id.to_string(),
                guild_id: thread.guild_id.to_string(),
                status: models::DiscordSyncStatus::Sync,
                synced_at: chrono::Utc::now(),
            });
        }
        Ok(())
    }
}

// Discord limits strings by UTF-16 code units. Keep astral characters intact.
fn discord_text(value: &str, limit: usize) -> String {
    if value.encode_utf16().count() <= limit {
        return value.to_owned();
    }
    let mut result = String::new();
    let mut remaining = limit.saturating_sub(1);
    for c in value.chars() {
        if c.len_utf16() > remaining {
            break;
        }
        result.push(c);
        remaining -= c.len_utf16();
    }
    result.push('…');
    result
}

fn forum_title(title: &str) -> String {
    let title = title.trim();
    if title.chars().count() < 2 {
        format!("活动 {title}")
    } else {
        discord_text(title, 100)
    }
}

fn event_embed(event: &crate::modules::event::models::Event) -> serenity::all::CreateEmbed {
    use serenity::all::CreateEmbed;
    let url = format!(
        "https://www.vatprc.net/events/{}",
        ulid::Ulid::from(event.id)
    );
    let mut embed = CreateEmbed::new()
        .title(discord_text(&event.title, 256))
        .url(&url)
        .description(discord_text(&event.description, 4096))
        .field(
            "活动时间",
            format!(
                "<t:{}:F> – <t:{}:t>",
                event.start_at.timestamp(),
                event.end_at.timestamp()
            ),
            false,
        )
        .field("活动详情 / 报名", format!("[在官网查看]({url})"), false);
    if let Some(title) = event.title_en.as_deref().filter(|s| !s.is_empty()) {
        embed = embed.field("English title", discord_text(title, 256), false);
    }
    if let Some(image) = event.image_url.as_deref().filter(|s| valid_public_url(s)) {
        embed = embed.image(image);
    }
    for (name, link) in [
        ("论坛", &event.community_link),
        ("VATSIM", &event.vatsim_link),
    ] {
        if let Some(link) = link
            .as_deref()
            .filter(|s| valid_public_url(s) && s.encode_utf16().count() <= 512)
        {
            embed = embed.field(name, link, false);
        }
    }
    embed
}

fn valid_public_url(value: &str) -> bool {
    url::Url::parse(value)
        .is_ok_and(|url| matches!(url.scheme(), "http" | "https") && url.host_str().is_some())
}

#[cfg(test)]
mod event_post_tests {
    use super::*;

    #[test]
    fn truncation_respects_discord_utf16_limits() {
        assert_eq!(discord_text("中文标题", 4), "中文标题");
        assert_eq!(discord_text("🚀🚀🚀", 4), "🚀…");
        assert_eq!(discord_text("abcde", 4), "abc…");
        assert!(forum_title(&"🚀".repeat(101)).encode_utf16().count() <= 100);
        assert!(forum_title(" ").chars().count() >= 2);
    }

    #[test]
    fn event_embed_includes_details_and_clears_removed_image() {
        let event = example_event();
        let value = serde_json::to_value(event_embed(&event)).unwrap();
        assert!(
            value["description"]
                .as_str()
                .unwrap()
                .encode_utf16()
                .count()
                <= 4096
        );
        assert_eq!(value["fields"][0]["value"], "<t:1000:F> – <t:2000:t>");
        assert!(value.get("image").is_none());
        assert!(
            value["url"]
                .as_str()
                .unwrap()
                .ends_with("/events/00000000000000000000000000")
        );
    }

    pub(super) fn example_event() -> crate::modules::event::models::Event {
        crate::modules::event::models::Event {
            id: uuid::Uuid::nil(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            title: "测试活动".into(),
            title_en: None,
            start_at: chrono::DateTime::from_timestamp(1000, 0).unwrap(),
            end_at: chrono::DateTime::from_timestamp(2000, 0).unwrap(),
            start_booking_at: None,
            end_booking_at: None,
            start_atc_booking_at: None,
            image_url: None,
            community_link: None,
            vatsim_link: None,
            description: "🚀".repeat(3000),
            discord_message: None,
        }
    }

    #[tokio::test]
    async fn publishes_once_then_updates_the_same_post_without_mentions() {
        use axum::{extract::Request, response::IntoResponse};
        use serenity::all::{ChannelId, GuildChannel, GuildId, Message};
        use std::sync::{Arc, Mutex};
        let requests = Arc::new(Mutex::new(Vec::new()));
        let captured = requests.clone();
        let app = axum::Router::new().fallback(move |request: Request| {
            let captured = captured.clone();
            async move {
                let method = request.method().clone();
                let path = request.uri().path().to_string();
                let body = axum::body::to_bytes(request.into_body(), 16384)
                    .await
                    .unwrap();
                captured.lock().unwrap().push((
                    method,
                    path.clone(),
                    serde_json::from_slice::<serde_json::Value>(&body).unwrap_or_else(|_| {
                        // Forum creation uses multipart/form-data even without attachments.
                        let body = std::str::from_utf8(&body).unwrap();
                        let payload = body
                            .split("\r\n\r\n")
                            .nth(1)
                            .unwrap()
                            .split("\r\n--")
                            .next()
                            .unwrap();
                        serde_json::from_str(payload).unwrap()
                    }),
                ));
                if captured.lock().unwrap().last().unwrap().2["name"] == "Forbidden" {
                    return (
                        axum::http::StatusCode::FORBIDDEN,
                        axum::Json(serde_json::json!({
                            "code": 50013, "message": "Missing Permissions"
                        })),
                    )
                        .into_response();
                }
                if path.ends_with("/messages/123") {
                    axum::Json(serde_json::to_value(Message::default()).unwrap()).into_response()
                } else {
                    let mut channel = GuildChannel::default();
                    channel.id = ChannelId::new(123);
                    channel.guild_id = GuildId::new(456);
                    axum::Json(serde_json::to_value(channel).unwrap()).into_response()
                }
            }
        });
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let publisher = DiscordEventPublisher {
            http: Arc::new(
                serenity::http::HttpBuilder::new("test-token")
                    .proxy(format!("http://{address}"))
                    .ratelimiter_disabled(true)
                    .build(),
            ),
            forum: ChannelId::new(789),
            tags: vec![],
        };
        let mut event = example_event();
        publisher.sync(&mut event).await.unwrap();
        assert_eq!(
            event
                .discord_message
                .as_ref()
                .map(|message| message.message_id.as_str()),
            Some("123")
        );
        assert_eq!(
            event
                .discord_message
                .as_ref()
                .map(|message| message.guild_id.as_str()),
            Some("456")
        );
        event.title = "Updated event".into();
        publisher.sync(&mut event).await.unwrap();
        let requests = requests.lock().unwrap().clone();
        assert_eq!(requests.len(), 3);
        assert_eq!(requests[0].0, "POST");
        assert!(requests[0].1.ends_with("/channels/789/threads"));
        assert_eq!(
            requests[0].2["message"]["allowed_mentions"]["parse"],
            serde_json::json!([])
        );
        assert_eq!(requests[1].0, "PATCH");
        assert!(requests[1].1.ends_with("/channels/123"));
        assert_eq!(requests[1].2["name"], "Updated event");
        assert_eq!(requests[1].2["archived"], false);
        assert!(requests[2].1.ends_with("/channels/123/messages/123"));
        assert_eq!(requests[2].2["embeds"][0]["title"], "Updated event");
        assert_eq!(
            requests[2].2["allowed_mentions"]["parse"],
            serde_json::json!([])
        );
        event.title = "Forbidden".into();
        let failure = publisher.sync(&mut event).await;
        assert!(failure.is_err());
        assert!(matches!(failure, Err(DiscordPublishError::Request(_))));
        assert_eq!(
            event
                .discord_message
                .as_ref()
                .map(|message| message.message_id.as_str()),
            Some("123")
        );
        let mut unpublished = example_event();
        unpublished.title = "Forbidden".into();
        assert!(publisher.sync(&mut unpublished).await.is_err());
        assert!(unpublished.discord_message.is_none());
        server.abort();
    }
}
