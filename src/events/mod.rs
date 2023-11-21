use std::sync::Arc;

use twilight_gateway::Event;
use twilight_model::id::Id;

use crate::context::Context;

use self::interactions_create::handle_interaction_create;

pub mod application_command;
pub mod interactions_create;
#[cfg(feature = "database")]
pub mod silly_command;

pub async fn handle_event(shard: u64, event: Event, context: Arc<Context>) -> anyhow::Result<()> {
    log::debug!("{:?} happened on shard {}", event.kind(), shard);
    match event {
        Event::InteractionCreate(it) => handle_interaction_create(shard, it, &context).await,
        #[cfg(feature = "ai")]
        Event::MessageCreate(message) => {
            if message.author.bot {
                return Ok(());
            }



            if message.content.to_lowercase().contains("takathebot")
                || message
                    .content
                    .contains(&format!("<@{}>", context.application.id))
            {   
                let Some(_guild_id) = message.guild_id else {
                    log::warn!("guild was undefined");
                    return Ok(());
                };


                

                let channel = context.http_client.channel(message.channel_id).await?.model().await?;

                let Some(parent_id) = channel.parent_id else {
                    log::warn!("parentid was undefined");
                    return Ok(());
                };

                if parent_id != context.ai_channel {
                    log::warn!("parentid wasnt the right one");
                    return Ok(());
                }

                let author_nickname = message
                    .member
                    .as_ref()
                    .map(|member| member.nick.clone())
                    .flatten()
                    .unwrap_or(message.author.name.clone());

                let _ = context
                    .http_client
                    .create_typing_trigger(message.channel_id)
                    .await;
                // Respond as AI.
                let text = format!("You are currently talking to {} (Also referred to as {}). Answer this message: {}", author_nickname, message.author.name, message.content.replace(&format!("<@{}>", 
                context.application.id), "takathebot"));

                let file = format!("./conversations/{}.json", channel.id);

                let mut conversation = context
                    .chatgpt_client
                    .restore_conversation_json(&file)
                    .await
                    .ok()
                    .unwrap_or(
                        context
                            .chatgpt_client
                            .new_conversation_directed(context.openai_prompt),
                    );

                let response = conversation.send_message(text).await?;

                context
                    .http_client
                    .create_message(channel.id)
                    .reply(message.id)
                    .content(&response.message_choices[0].message.content)?
                    .await?;

                conversation.save_history_json(&file).await?;
            }
        }
        Event::Ready(..) => {
            let channel = context
                .http_client
                .create_private_channel(Id::new(434626996262273038))
                .await?
                .model()
                .await?;
            context
                .http_client
                .create_message(channel.id)
                .content("✅ Bot has logged in!")?
                .await?;
        }
        _ => {}
    };

    Ok(())
}
