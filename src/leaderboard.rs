use crate::{models::voice_time::VoiceTime, mongo_connection_provider, Context, Error};
use mongodb::bson::doc;
use poise::{
    serenity_prelude::{Channel, CreateEmbed},
    CreateReply,
};

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

/// Set Leaderboard Channel
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

pub async fn increment_user_time(
    user_id: u64,
    guild_id: u64,
    time: u64,
) -> Result<(), mongodb::error::Error> {
    println!("incrementing user {} by {}", user_id, time);

    let db = mongo_connection_provider::get_db();

    let query = doc! {
        "user_id": user_id as i64,
        "guild_id": guild_id as i64,
    };

    let update = doc! {
        "$inc": {
            "time": time as i64,
        },
    };

    db.collection::<VoiceTime>("voice_time")
        .update_one(query, update)
        .upsert(true)
        .await?;

    Ok(())
}
