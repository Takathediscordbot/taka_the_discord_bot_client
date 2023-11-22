use std::borrow::Cow;

use anyhow::anyhow;
use twilight_interactions::command::{CommandInputData, CommandModel, CreateCommand};
use twilight_model::{
    application::interaction::application_command::CommandData,
    gateway::payload::incoming::InteractionCreate,
};

use crate::{
    context::Context, services::silly_command::SillyCommandPDO,
    utils::box_commands::RunnableCommand,
};

#[derive(CreateCommand, CommandModel)]
#[command(
    name = "add_preference",
    desc = "Add a preference to a silly command (author only)"
)]
pub struct AddPreferenceCommand {
    /// The name of the command
    name: String,
    /// a
    preference: String,
}

#[async_trait::async_trait]
impl RunnableCommand for AddPreferenceCommand {
    async fn run(
        _shard: u64,
        interaction: &InteractionCreate,
        data: Box<CommandData>,
        context: &Context,
    ) -> anyhow::Result<anyhow::Result<()>> {
        let model = Self::from_interaction(CommandInputData {
            options: data.options,
            resolved: data.resolved.map(Cow::Owned),
        })?;

        let Some(author) = interaction.author_id() else {
            return Ok(Err(anyhow!("❌ You're probably not the author of this bot!")));
        };

        if author.get() != context.author_id {
            return Ok(Err(anyhow!("❌ You're definitely not the author of this bot!")));
        }

        SillyCommandPDO::add_preference(&context, &model.preference, &model.name)
            .await?;

        context.response_to_interaction_with_content(interaction, &format!(
            "✅Done! You should now try to use /reload_command to see it appear"
        ))
        .await?;
    

        Ok(Ok(()))
    }
}
