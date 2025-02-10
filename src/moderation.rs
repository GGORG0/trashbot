use crate::{Context, Error};
use poise::{serenity_prelude::GetMessages, CreateReply};

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

    ctx.send(
        CreateReply::default()
            .content(format!(":fire: Deleted {} messages!", number))
            .reply(true)
            .ephemeral(true),
    )
    .await?;

    Ok(())
}
