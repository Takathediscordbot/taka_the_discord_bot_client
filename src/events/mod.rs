use std::sync::Arc;

use twilight_gateway::Event;
use twilight_model::id::Id;

use crate::context::Context;

use self::interactions_create::handle_interaction_create;

pub mod application_command;
pub mod interactions_create;
pub mod silly_command;

pub async fn handle_event(shard: u64, event: Event, context: Arc<Context>) -> anyhow::Result<()> {
    log::debug!("{:?} happened on shard {}", event.kind(), shard);
    match event {
        Event::InteractionCreate(it) => {
            handle_interaction_create(shard, it, Arc::clone(&context)).await
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
