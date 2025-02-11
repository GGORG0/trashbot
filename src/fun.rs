use async_openai::types::{
    ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
    CreateChatCompletionRequestArgs, FinishReason,
};
use chrono::Utc;
use poise::serenity_prelude::Timestamp;
use std::time::Duration;

use crate::{Context, Error, OPENAI_CLIENT};

/// Judges your attitude towards NixOS.
#[poise::command(slash_command, prefix_command)]
pub async fn nixos(
    ctx: Context<'_>,
    #[rest]
    #[description = "Your opinion about NixOS"]
    opinion: String,
) -> Result<(), Error> {
    const SYSTEM_PROMPT: &str = "The user will state their opinion about NixOS, a Linux distribution. You have to reply with just a single integer from 0 to 100, where 100 means an infinitely positive attitude and 0 means infinitely negative. Please don't just say 0 or 100, try to position it on a scale.";

    let request = CreateChatCompletionRequestArgs::default()
        .n(1)
        .max_tokens(5u32)
        .model("llama-3.3-70b-versatile")
        .messages([
            ChatCompletionRequestSystemMessageArgs::default()
                .content(SYSTEM_PROMPT)
                .build()?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(opinion)
                .build()?
                .into(),
        ])
        .build()?;

    let response = OPENAI_CLIENT.chat().create(request).await?;

    let res = response.choices.first().unwrap();

    if res.finish_reason != Some(FinishReason::Stop) {
        ctx.reply(":fire: The AI didn't understand your opinion. Please try again.")
            .await?;
        return Ok(());
    }

    let score = match res.message.content.clone().unwrap().parse::<u8>() {
        Ok(score) => score,
        Err(_) => {
            ctx.reply(":fire: The AI didn't understand your opinion. Please try again.")
                .await?;
            return Ok(());
        }
    };

    if score > 100 {
        ctx.reply(":fire: The AI didn't understand your opinion. Please try again.")
            .await?;
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

    ctx.reply(format!(":fire: Your attitude towards NixOS is: {}%", score))
        .await?;

    Ok(())
}
