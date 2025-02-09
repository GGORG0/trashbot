use dotenv::dotenv;
use once_cell::sync::OnceCell;
use poise::serenity_prelude::{self as serenity, ActivityData, CacheHttp, GetMessages, GuildId};
use std::time::Duration;
use tokio::time::sleep;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, (), Error>;

static STARTUP_TIME: OnceCell<std::time::Instant> = OnceCell::new();

/// Show a menu of slash command registration options.
#[poise::command(prefix_command, owners_only)]
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

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = std::env::var("TOKEN").expect("missing TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![register(), uptime(), purge(), ping()],
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
