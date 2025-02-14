use crate::{Context, Error};

use std::collections::HashMap;

use poise::serenity_prelude::{Http, Member, Role, RoleId, Timestamp, UserId};

use chrono::{DateTime, Duration, Timelike, Utc};
use poise::CreateReply;
fn highest_role_pos(member: &Member, roles: &HashMap<RoleId, Role>) -> u16 {
    member
        .roles
        .iter()
        .map(|r| roles.get(r).unwrap().position)
        .max()
        .unwrap()
}

#[poise::command(slash_command, prefix_command)]
pub async fn parental_control(
    ctx: Context<'_>,
    #[rest]
    #[description = "Give someone (or youtself) parental control"]
    user: UserId,
) -> Result<(), Error> {
    let role: RoleId = std::env::var("PARENTAL_CONTROL_ROLE_ID")
        .unwrap()
        .parse()
        .unwrap();

    let author_member = ctx.author_member().await.unwrap().into_owned();
    let targeted_member = ctx.guild_id().unwrap().member(&ctx.http(), user).await?;

    if highest_role_pos(&author_member, &ctx.guild().unwrap().roles)
        <= highest_role_pos(&targeted_member, &ctx.guild().unwrap().roles)
        || !author_member.permissions.unwrap().manage_roles()
    {
        ctx.send(
            CreateReply::default()
                .content("Insufficient permissions")
                .reply(true)
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    let reply: &str = if targeted_member.roles.contains(&role) {
        match targeted_member.remove_role(&ctx.http(), role).await {
            Ok(()) => ":fire: Removed parental control!",
            Err(_) => "Unable to remove parental control",
        }
    } else {
        match targeted_member.add_role(&ctx.http(), role).await {
            Ok(()) => ":fire: Added parental control!",
            Err(_) => "Unable to add parental control",
        }
    };

    ctx.send(
        CreateReply::default()
            .content(reply)
            .reply(true)
            .ephemeral(true),
    )
    .await?;

    println!("parental control for {}", user);

    Ok(())
}

pub async fn parental_timeout(http: &Http, member: &mut Member) {
    let now: DateTime<Utc> = Utc::now();
    let mut planned_date = now.with_hour(6).unwrap().with_minute(0).unwrap().with_second(0).unwrap().with_nanosecond(0).unwrap();
    planned_date = if now > planned_date {
        planned_date + Duration::days(1)
    } else {
        planned_date
    };
    let _ = member
        .disable_communication_until_datetime(http, Timestamp::from(planned_date))
        .await;
}
