use chrono::Utc;
use dotenv::dotenv;
use once_cell::sync::{Lazy, OnceCell};
use poise::{
    samples::HelpConfiguration,
    serenity_prelude::{
        self as serenity, ActivityData, CacheHttp, GetMessages, GuildId, Timestamp,
    },
};
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, (), Error>;

static STARTUP_TIME: OnceCell<std::time::Instant> = OnceCell::new();
static HTTP_CLIENT: Lazy<reqwest::Client> = Lazy::new(reqwest::Client::new);

/// Show this menu
#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"] command: Option<String>,
) -> Result<(), Error> {
    poise::builtins::help(ctx, command.as_deref(), HelpConfiguration::default()).await?;
    Ok(())
}

/// Show a menu of slash command registration options.
#[poise::command(prefix_command, owners_only, hide_in_help)]
pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

/// Get the bot's uptime.
#[poise::command(slash_command, prefix_command)]
async fn uptime(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say(format!(
        ":fire: Uptime: {:?}",
        STARTUP_TIME.get().unwrap().elapsed()
    ))
    .await?;

    Ok(())
}

/// Get the bot's gateway latency.
#[poise::command(slash_command, prefix_command)]
async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say(format!(":fire: Ping: {:?}", ctx.ping().await))
        .await?;

    Ok(())
}

/// Purges a number of messages from the channel.
#[poise::command(
    slash_command,
    prefix_command,
    required_permissions = "MANAGE_MESSAGES",
    guild_only
)]
async fn purge(
    ctx: Context<'_>,
    #[description = "The number of messages to delete"] number: u8,
) -> Result<(), Error> {
    let message_ids = ctx
        .guild_channel()
        .await
        .unwrap()
        .messages(
            ctx.http(),
            GetMessages::new().limit(number + if ctx.prefix() == "/" { 0 } else { 1 }),
        )
        .await?
        .iter()
        .map(|msg| msg.id)
        .collect::<Vec<_>>();

    ctx.guild_channel()
        .await
        .unwrap()
        .delete_messages(ctx.http(), message_ids)
        .await?;

    let msg = ctx.say(":fire: Deleted messages!").await?;
    sleep(Duration::from_secs(5)).await;
    msg.delete(ctx).await?;

    Ok(())
}

/// Judges your attitude towards NixOS.
#[poise::command(slash_command, prefix_command)]
async fn nixos(
    ctx: Context<'_>,
    #[rest]
    #[description = "Your opinion about NixOS"]
    opinion: String,
) -> Result<(), Error> {
    let res = HTTP_CLIENT.post("https://api.groq.com/openai/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", std::env::var("GROQ_API_KEY").unwrap()))
        .json(&json!({
            "model": "llama-3.3-70b-versatile",
            "messages": [
                {
                    "role": "system",
                    "content": "The user will state their opinion about NixOS, a Linux distribution. You have to reply with just a single integer from 0 to 100, where 100 means an infinitely positive attitude and 0 means infinitely negative. Please don'\''t just say 0 or 100, try to position it on a scale."
                },
                {
                    "role": "user",
                    "content": opinion
                }
            ]
        })).send().await?.json::<serde_json::Value>().await?;

    let score = res["choices"][0]["message"]["content"]
        .as_str()
        .unwrap()
        .parse::<u8>()?;

    if score < 50 && !ctx.author().bot {
        let ts =
            Timestamp::from_unix_timestamp((Utc::now() + Duration::from_secs(69)).timestamp())?;

        if let Some(member) = ctx.author_member().await {
            let mut member = member.into_owned();

            member
                .disable_communication_until_datetime(&ctx.http(), ts)
                .await?;
        }
    }

    ctx.say(format!(":fire: Your attitude towards NixOS is: {}%", score))
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = std::env::var("TOKEN").expect("missing TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![help(), register(), uptime(), purge(), ping(), nixos()],
            prefix_options: poise::PrefixFrameworkOptions {
                ignore_bots: false, // We use a Matrix -> Discord bridge bot
                case_insensitive_commands: true,
                dynamic_prefix: Some(|ctx| {
                    Box::pin(async move {
                        // Mentions are <@USER_ID>, with Matrix clients appending "\:" to the end
                        Ok(Some(format!("<@{}>\\:", ctx.framework.bot_id)))
                    })
                }),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                let guild_id: GuildId = std::env::var("GUILD_ID").unwrap().parse().unwrap();
                poise::builtins::register_in_guild(ctx, &framework.options().commands, guild_id)
                    .await?;

                let user = ctx.http.get_current_user().await?;
                println!(
                    "Bot is ready as: {}#{}",
                    user.name,
                    user.discriminator.unwrap()
                );

                println!(
                    "{} commands registered to guild {}",
                    framework.options().commands.len(),
                    guild_id.name(ctx.cache().unwrap()).unwrap()
                );

                ctx.set_activity(Some(ActivityData::custom("🗑️")));

                STARTUP_TIME.set(std::time::Instant::now()).unwrap();

                Ok(())
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}
