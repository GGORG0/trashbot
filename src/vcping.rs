use crate::{Context, Error};
use poise::serenity_prelude::{self as serenity, CacheHttp, Mentionable, RoleId};
use poise::serenity_prelude::{ChannelId, VoiceState};
use poise::CreateReply;

// TODO: add caching everywhere

/// (Un)subscribe to voice channel pings.
#[poise::command(slash_command, prefix_command)]
pub async fn vcping(ctx: Context<'_>) -> Result<(), Error> {
    if ctx.author().bot {
        ctx.reply(":fire: As a Matrix user, please add \"vc-ping\" to your notification keywords!")
            .await?;
        return Ok(());
    }

    let role: RoleId = std::env::var("VCPING_ROLE_ID").unwrap().parse().unwrap();

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
    let joined = match old {
        Some(old) => old.channel_id.is_none() && new.channel_id.is_some(),
        None => new.channel_id.is_some(),
    };

    if !joined {
        return Ok(());
    }

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

    let role: RoleId = std::env::var("VCPING_ROLE_ID").unwrap().parse().unwrap();

    let message = format!(
        ":fire: {}, {} joined empty voice channel: {}!",
        role.mention(),
        new.member.clone().unwrap().user.tag(),
        new.channel_id.unwrap().name(ctx.http()).await.unwrap()
    );

    let channel: ChannelId = std::env::var("VCPING_CHANNEL_ID").unwrap().parse().unwrap();

    let member = new.member.clone().unwrap();

    let channels = member.guild_id.channels(&ctx.http()).await.unwrap();

    let channel = channels.get(&channel).unwrap();

    channel.say(ctx.http(), message).await?;

    Ok(())
}
