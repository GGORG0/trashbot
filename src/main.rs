mod fun;
mod leaderboard;
mod misc;
mod models;
mod moderation;
mod mongo_connection_provider;
mod parental_control;
mod vcping;

use async_openai::config::OpenAIConfig;
use fun::*;
use leaderboard::*;
use misc::*;
use moderation::*;
use parental_control::*;
use vcping::*;

use dotenv::dotenv;
use once_cell::sync::{Lazy, OnceCell};
use poise::serenity_prelude::{self as serenity, ActivityData, CacheHttp, FullEvent, GuildId};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, (), Error>;

static STARTUP_TIME: OnceCell<std::time::Instant> = OnceCell::new();
static HTTP_CLIENT: Lazy<reqwest::Client> = Lazy::new(reqwest::Client::new);
static OPENAI_CLIENT: Lazy<async_openai::Client<OpenAIConfig>> = Lazy::new(|| {
    async_openai::Client::build(
        HTTP_CLIENT.clone(),
        OpenAIConfig::default().with_api_base(std::env::var("OPENAI_API_BASE").unwrap()),
        Default::default(),
    )
});

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = std::env::var("TOKEN").expect("missing TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let uri = "mongodb://localhost:27017";
    let db_name = "garbageDump";
    mongo_connection_provider::init(uri, db_name)
        .await
        .expect("Failed to initialize MongoDB connection");

    let commands = vec![
        help(),
        register(),
        uptime(),
        purge(),
        ping(),
        nixos(),
        vcping(),
        parental_control(),
        leaderboard(),
        set_leaderboard_channel(),
    ];

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands,
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
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
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

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .unwrap();

    tokio::spawn(async move {
        client.start().await.unwrap();
    });

    tokio::signal::ctrl_c().await.unwrap();
}

async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    framework: poise::FrameworkContext<'_, (), Error>,
    _data: &(),
) -> Result<(), Error> {
    match event {
        FullEvent::Ready { data_about_bot, .. } => {
            let user = data_about_bot.user.clone();
            println!("Bot is ready as: {}", user.tag());

            let guild_id: GuildId = std::env::var("GUILD_ID").unwrap().parse().unwrap();
            println!(
                "{} commands registered to guild {}",
                framework.options().commands.len(),
                guild_id.name(ctx.cache().unwrap()).unwrap()
            );

            ctx.set_activity(Some(ActivityData::custom("🗑️")));

            STARTUP_TIME.set(std::time::Instant::now()).unwrap();

            Ok(())
        }
        FullEvent::VoiceStateUpdate { old, new, .. } => {
            voice_state_update_handler(ctx, old, new).await
        }
        _ => Ok(()),
    }
}
