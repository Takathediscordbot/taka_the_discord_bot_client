use std::{borrow::Cow, sync::Arc};

use anyhow::anyhow;
use twilight_interactions::command::{CommandInputData, CommandModel, CreateCommand};
use twilight_model::{
    application::interaction::application_command::CommandData,
    gateway::payload::incoming::InteractionCreate,
};

use crate::{
    context::Context, 
    services::silly_command::SillyCommandPDO, utils::box_commands::RunnableCommand,
};

#[derive(CreateCommand, CommandModel)]
#[command(name = "add_silly_text", desc = "Add a silly text (taka only)")]
pub struct AddSillyText {
    /// The name of the command
    name: String,
    /// Text
    text: String,
    /// Author
    author: bool,
}

#[async_trait::async_trait]
impl RunnableCommand for AddSillyText {
    async fn run(
        _shard: u64,
        interaction: &InteractionCreate,
        data: Box<CommandData>,
        context: Arc<Context>,
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

        let result = if model.author {
            SillyCommandPDO::add_text_author(Arc::clone(&context), &model.name, &model.text).await?
        } else {
            SillyCommandPDO::add_text(Arc::clone(&context), &model.name, &model.text).await?
        };

        let interaction_client = context.http_client.interaction(context.application.id);
        interaction_client
            .update_response(&interaction.token)
            .content(Some(&format!(
                " Text has been created with id {result} for command {}",
                model.name
            )))?
            .await?;

        Ok(Ok(()))
    }
}
