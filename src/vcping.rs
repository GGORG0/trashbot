use crate::models::vcping_settings::VcpingSettings;
use crate::{leaderboard, mongo_connection_provider, Context, Error};
use mongodb::bson::doc;
use once_cell::sync::Lazy;
use poise::serenity_prelude::{self as serenity, CacheHttp, Mentionable, RoleId};
use poise::serenity_prelude::{ChannelId, VoiceState};
use poise::CreateReply;
use std::collections::HashMap;
use tokio::sync::Mutex;

pub static INTERACTION_HISTORY: Lazy<Mutex<HashMap<u64, std::time::Instant>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub static JOIN_HISTORY: Lazy<Mutex<HashMap<u64, std::time::Instant>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

// TODO: add caching everywhere

/// (Un)subscribe to voice channel pings.
#[poise::command(slash_command, prefix_command)]
pub async fn vcping(ctx: Context<'_>) -> Result<(), Error> {
    if ctx.author().bot {
        ctx.reply(":fire: As a Matrix user, please add \"vc-ping\" to your notification keywords!")
            .await?;
        return Ok(());
    }

    let db = mongo_connection_provider::get_db();

    let query = doc! {
        "guild_id": ctx.guild_id().unwrap().get() as i64,
    };

    let dbdata = db
        .collection::<VcpingSettings>("vcping_settings")
        .find_one(query)
        .await?;

    // let role: RoleId = std::env::var("VCPING_ROLE_ID").unwrap().parse().unwrap();
    let role: RoleId = RoleId::from(dbdata.as_ref().unwrap().role_id as u64);

    let member = ctx.author_member().await.unwrap().into_owned();

    let reply = if member.roles.contains(&role) {
        member.remove_role(&ctx.http(), role).await?;
        ":fire: Unsubscribed from voice channel pings!"
    } else {
        member.add_role(&ctx.http(), role).await?;
        ":fire: Subscribed to voice channel pings!"
    };

    ctx.send(
        CreateReply::default()
            .content(reply)
            .reply(true)
            .ephemeral(true),
    )
    .await?;

    Ok(())
}

pub async fn voice_state_update_handler(
    ctx: &serenity::Context,
    old: &Option<VoiceState>,
    new: &VoiceState,
) -> Result<(), Error> {
    let db = mongo_connection_provider::get_db();

    let vcping_settings = db
        .collection::<VcpingSettings>("vcping_settings")
        .find_one(doc! {
            "guild_id": new.guild_id.clone().unwrap().get() as i64,
        })
        .await?;

    let last_interaction = INTERACTION_HISTORY
        .lock()
        .await
        .get(&new.member.clone().unwrap().user.id.get())
        .copied();

    let leave = match old {
        Some(old) => old.channel_id.is_some() && new.channel_id.is_none(),
        None => false,
    };

    let role: RoleId = RoleId::from(vcping_settings.as_ref().unwrap().role_id as u64);

    if leave {
        let history = JOIN_HISTORY.lock();

        if let Some(joined_at) = history
            .await
            .get(&new.member.clone().unwrap().user.id.get())
        {
            leaderboard::increment_user_time(
                new.member.clone().unwrap().user.id.get(),
                new.guild_id.clone().unwrap().get(),
                joined_at.elapsed().as_secs(),
                ctx.http(),
            )
            .await?;
        }
    } else {
        JOIN_HISTORY.lock().await.insert(
            new.member.clone().unwrap().user.id.get(),
            std::time::Instant::now(),
        );

        let members = new
            .channel_id
            .unwrap()
            .to_channel(ctx.http())
            .await
            .unwrap()
            .guild()
            .unwrap()
            .members(ctx.cache().unwrap())
            .unwrap();

        if members.len() > 1 {
            return Ok(());
        }

        let message = format!(
            ":fire: {}, {} joined empty voice channel: {}!",
            role.mention(),
            new.member.clone().unwrap().user.tag(),
            new.channel_id.unwrap().name(ctx.http()).await.unwrap()
        );

        if let Some(last_interaction) = last_interaction {
            if last_interaction.elapsed() < std::time::Duration::from_secs(30) {
                return Ok(());
            }
        }

        if !vcping_settings.is_none() {
            let channel: ChannelId =
                ChannelId::from(vcping_settings.as_ref().unwrap().channel_id as u64);

            let member = new.member.clone().unwrap();

            let channels = member.guild_id.channels(&ctx.http()).await.unwrap();
            let channel = channels.get(&channel).unwrap();

            channel.say(ctx.http(), message).await?;
        }
    };

    INTERACTION_HISTORY.lock().await.insert(
        new.member.clone().unwrap().user.id.get(),
        std::time::Instant::now(),
    );

    Ok(())
}
