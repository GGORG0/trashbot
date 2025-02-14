use crate::{models::vcping_settings::VcpingSettings, mongo_connection_provider, Context, Error};
use mongodb::bson::doc;
use poise::{
    serenity_prelude::{Channel, ChannelId, GetMessages, RoleId},
    CreateReply,
};

/// Sets up voice channel pings.
#[poise::command(
    slash_command,
    prefix_command,
    // required_permissions = "MANAGE_MESSAGES",
    guild_only
)]
pub async fn vcping_setup(
    ctx: Context<'_>,
    #[description = "Channel for pings"]
    #[channel_types("Text")]
    channel: ChannelId,
    #[description = "Role for pings"] role: RoleId,
) -> Result<(), Error> {
    let db = mongo_connection_provider::get_db();

    let query = doc! {
        "guild_id": ctx.guild_id().unwrap().get() as i64,
    };

    let update = doc! {
        "$set": {
            "channel_id": channel.get() as i64,
            "role_id": role.get() as i64,
        },
    };

    db.collection::<VcpingSettings>("vcping_settings")
        .update_one(query, update)
        .upsert(true)
        .await?;

    ctx.send(
        CreateReply::default()
            .content(format!("VC ping channel set to <#{}>", channel.get()))
            .reply(true)
            .ephemeral(true),
    )
    .await?;

    Ok(())
}
