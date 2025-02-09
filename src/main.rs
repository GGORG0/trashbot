use dotenv::dotenv;
use once_cell::sync::OnceCell;
use poise::serenity_prelude::{self as serenity, GuildId};

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
    let response = format!(":fire: Uptime: {:?}", STARTUP_TIME.get().unwrap().elapsed());
    ctx.say(response).await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    STARTUP_TIME.set(std::time::Instant::now()).unwrap();

    let token = std::env::var("TOKEN").expect("missing TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![register(), uptime()],
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
                Ok(())
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}
