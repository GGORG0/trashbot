use chrono::Utc;
use poise::serenity_prelude::Timestamp;
use serde_json::json;
use std::time::Duration;

use crate::{Context, Error, HTTP_CLIENT};

/// Judges your attitude towards NixOS.
#[poise::command(slash_command, prefix_command)]
pub async fn nixos(
    ctx: Context<'_>,
    #[rest]
    #[description = "Your opinion about NixOS"]
    opinion: String,
) -> Result<(), Error> {
    let res = HTTP_CLIENT.post("https://api.groq.com/openai/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", std::env::var("GROQ_API_KEY").unwrap()))
        .json(&json!({
            "model": "llama-3.3-70b-versatile",
            "messages": [
                {
                    "role": "system",
                    "content": "The user will state their opinion about NixOS, a Linux distribution. You have to reply with just a single integer from 0 to 100, where 100 means an infinitely positive attitude and 0 means infinitely negative. Please don'\''t just say 0 or 100, try to position it on a scale."
                },
                {
                    "role": "user",
                    "content": opinion
                }
            ]
        })).send().await?.json::<serde_json::Value>().await?;

    let score = res["choices"][0]["message"]["content"]
        .as_str()
        .unwrap()
        .parse::<u8>()?;

    if score > 100 {
        ctx.say(":fire: You're too positive!").await?;
        return Ok(());
    }

    if score < 50 && !ctx.author().bot {
        let ts =
            Timestamp::from_unix_timestamp((Utc::now() + Duration::from_secs(69)).timestamp())?;

        if let Some(member) = ctx.author_member().await {
            let mut member = member.into_owned();

            member
                .disable_communication_until_datetime(&ctx.http(), ts)
                .await?;
        }
    }

    ctx.say(format!(":fire: Your attitude towards NixOS is: {}%", score))
        .await?;

    Ok(())
}
