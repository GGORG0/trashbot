use poise::{
    serenity_prelude::{Channel, CreateEmbed},
    CreateReply,
};

use crate::{Context, Error};

/// Print Leaderboard
#[poise::command(slash_command, prefix_command)]
pub async fn leaderboard(ctx: Context<'_>) -> Result<(), Error> {
    let ranking = CreateEmbed::default()
        .title("📕 Leaderboard:")
        .description("niger niger");

    ctx.send(CreateReply::default().reply(true).embed(ranking))
        .await?;

    Ok(())
}

///Set Leaderboard Channel
#[poise::command(slash_command, prefix_command)]
pub async fn set_leaderboard_channel(
    ctx: Context<'_>,
    #[description = "Specify a text channel for the leaderboard"]
    #[channel_types("Text")]
    channel: Channel,
) -> Result<(), Error> {
    ctx.send(
        CreateReply::default()
            .content(format!("Leaderboard channel set to {}", channel.id()))
            .reply(true)
            .ephemeral(true),
    )
    .await?;

    Ok(())
}
