use std::time::Duration;

use poise::serenity_prelude::GetMessages;
use tokio::time::sleep;

use crate::{Context, Error};

/// Purges a number of messages from the channel.
#[poise::command(
    slash_command,
    prefix_command,
    required_permissions = "MANAGE_MESSAGES",
    guild_only
)]
pub async fn purge(
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
