use crate::{models::voice_time::VoiceTime, mongo_connection_provider, Context, Error};
use mongodb::bson::doc;
use poise::futures_util::TryStreamExt;
use poise::serenity_prelude::CreateMessage;
use poise::{
    serenity_prelude::{Channel, CreateEmbed},
    CreateReply,
};

pub async fn get_leaderboard(ctx: Context<'_>) -> Result<CreateEmbed, Error> {
    let db = mongo_connection_provider::get_db();

    let query = doc! {
        "guild_id": ctx.guild_id().unwrap().get() as i64
    };

    let data = db
        .collection::<VoiceTime>("voice_time")
        .find(query)
        .sort(doc! { "time": -1 })
        .limit(10)
        .await?;

    // while data.advance().await? {
    //     println!("{:?}", data.deserialize_current()?);
    // }

    let data_vec: Vec<VoiceTime> = data.try_collect().await?;

    dbg!(&data_vec);

    let podiuim = data_vec
        .iter()
        .enumerate()
        .map(|(i, user)| {
            if i < 3 {
                format!(
                    "{} {}. <@{}>\n 🔊{} h {} min",
                    "#".repeat(i + 1),
                    i + 1,
                    user.user_id,
                    user.time / 3600,
                    user.time / 60,
                )
            } else {
                format!(
                    "\n**{}. <@{}>** 🔊{} h {} min",
                    i + 1,
                    user.user_id,
                    user.time / 3600,
                    user.time / 60,
                )
            }
        })
        .collect::<Vec<String>>()
        .join("\n");

    let ranking = CreateEmbed::default().description(format!("# 📕Leaderboard\n{}", podiuim));

    Ok(ranking)
}

/// Print Leaderboard
#[poise::command(slash_command, prefix_command)]
pub async fn leaderboard(ctx: Context<'_>) -> Result<(), Error> {
    let ranking = get_leaderboard(ctx.clone()).await?;

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
    let channels = ctx.guild_id().unwrap().channels(ctx.http()).await.unwrap();

    let guild_channel = channels.get(&channel.id()).unwrap();

    let embed = get_leaderboard(ctx.clone()).await;

    let message = CreateMessage::default().embed(embed.unwrap());

    let sent_message = guild_channel.send_message(ctx.http(), message).await?;

    let db = mongo_connection_provider::get_db();

    let query = doc! {
        "guild_id": ctx.guild_id().unwrap().get() as i64
    };

    // db.collection("leaderboards")
    //     .find_one_and_update(query)

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
