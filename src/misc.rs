use crate::{Context, Error, STARTUP_TIME};
use poise::samples::HelpConfiguration;

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
pub async fn uptime(ctx: Context<'_>) -> Result<(), Error> {
    ctx.reply(format!(
        ":fire: Uptime: {:?}",
        STARTUP_TIME.get().unwrap().elapsed()
    ))
    .await?;

    Ok(())
}

/// Get the bot's gateway latency.
#[poise::command(slash_command, prefix_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.reply(format!(":fire: Ping: {:?}", ctx.ping().await))
        .await?;

    Ok(())
}
