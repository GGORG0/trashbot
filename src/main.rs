mod fun;
mod misc;
mod moderation;

use fun::*;
use misc::*;
use moderation::*;

use dotenv::dotenv;
use once_cell::sync::{Lazy, OnceCell};
use poise::serenity_prelude::{self as serenity, ActivityData, CacheHttp, GuildId};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, (), Error>;

static STARTUP_TIME: OnceCell<std::time::Instant> = OnceCell::new();
static HTTP_CLIENT: Lazy<reqwest::Client> = Lazy::new(reqwest::Client::new);

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = std::env::var("TOKEN").expect("missing TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let commands = vec![help(), register(), uptime(), purge(), ping(), nixos()];

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

    tokio::spawn(async move {
        client.unwrap().start().await.unwrap();
    });

    tokio::signal::ctrl_c().await.unwrap();
}
