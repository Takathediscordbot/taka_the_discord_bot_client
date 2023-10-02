use std::sync::Arc;

use twilight_model::{
    application::interaction::{InteractionData, InteractionType},
    gateway::payload::incoming::InteractionCreate,
};

use crate::{context::Context, interactions::commands::help::HelpCommand};

use super::application_command;

pub async fn handle_interaction_create(
    shard: u64,
    it: Box<InteractionCreate>,
    context: Arc<Context>,
) {
    match (it.kind, it.data.clone()) {
        (InteractionType::ApplicationCommand, Some(InteractionData::ApplicationCommand(data))) => {
            let Err(e) = application_command::handle_application_command(shard, &it, data, context).await else {
                return;
            };
            log::error!("An error has occured: {e}");
        }
        (InteractionType::MessageComponent, Some(InteractionData::MessageComponent(data))) => {
            match data.custom_id.as_str() {
                "help_previous" => {
                    let Err(e) = HelpCommand::previous(shard, it, data, context).await else {
                        return;
                    };
                    log::error!("An error has occured: {e}");
                }
                "help_next" => {
                    let Err(e) = HelpCommand::next(shard, it, data, context).await else {
                        return;
                    };
                    log::error!("An error has occured: {e}");
                }
                _ => {}
            }
        }
        _ => {
            log::debug!("{:?} happened on shard {}", it.kind, shard);
        }
    }
}
