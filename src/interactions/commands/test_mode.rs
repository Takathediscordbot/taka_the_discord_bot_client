use std::borrow::Cow;

use anyhow::anyhow;
use twilight_interactions::command::{CommandInputData, CommandModel, CreateCommand};
use twilight_model::{
    application::interaction::application_command::CommandData,
    gateway::payload::incoming::InteractionCreate,
};

use crate::{context::Context, utils::box_commands::RunnableCommand};

#[derive(CreateCommand, CommandModel)]
#[command(name = "test_mode", desc = "Enable test mode (taka only)")]
pub struct TestMode {
    /// Enable test mode
    enable: bool,
}

#[async_trait::async_trait]
impl RunnableCommand for TestMode {
    async fn run(
        _shard: u64,
        interaction: &InteractionCreate,
        data: Box<CommandData>,
        context: &Context<'_>,
    ) -> anyhow::Result<anyhow::Result<()>> {
        let model = Self::from_interaction(CommandInputData {
            options: data.options,
            resolved: data.resolved.map(Cow::Owned),
        })?;

        let Some(author) = interaction.author_id() else {
            return Ok(Err(anyhow!("❌ You're probably not taka")))
        };

        if author.get() != 434626996262273038 {
            return Ok(Err(anyhow!("❌ You're definitely not taka")));
        }

        let mut test_mode = context.test_mode.lock().await;
        *test_mode = model.enable;

        let str = if model.enable { "enabled" } else { "disabled" };

        context.response_to_interaction_with_content(interaction, &format!("Test mode has been {}!", str)).await?;

        Ok(Ok(()))
    }
}
