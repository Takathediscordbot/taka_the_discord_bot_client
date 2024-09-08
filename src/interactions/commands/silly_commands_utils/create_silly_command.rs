use std::borrow::Cow;

use anyhow::anyhow;
use twilight_interactions::command::{
    CommandInputData, CommandModel, CommandOption, CreateCommand, CreateOption,
};
use twilight_model::{
    application::interaction::application_command::CommandData,
    gateway::payload::incoming::InteractionCreate,
};

use crate::{
    context::Context,
    models::silly_command::SillyCommandType,
    services::silly_command::SillyCommandPDO,
    utils::box_commands::RunnableCommand,
};

#[derive(CreateOption, CommandOption, Debug)]
pub enum SillyCommandTypeOption {
    #[option(name = "Author Only", value = "author_only")]
    AuthorOnly,
    #[option(name = "Single User", value = "single_user")]
    SingleUser,
}

#[derive(CreateCommand, CommandModel)]
#[command(
    name = "create_silly_command",
    desc = "Create a silly command (author only)"
)]
pub struct CreateSillyCommand {
    /// Type of command
    command_type: SillyCommandTypeOption,
    /// The name of the command
    name: String,
    /// description
    description: String,
    /// description
    footer_text: String,
}

#[async_trait::async_trait]
impl RunnableCommand for CreateSillyCommand {
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
            return Ok(Err(anyhow!("❌ You're probably not the author of this bot!")))
        };

        if author.get() != context.author_id {
            return Ok(Err(anyhow!("❌ You're definitely not the author of this bot!")));
        }

        let command_type = match model.command_type {
            SillyCommandTypeOption::AuthorOnly => SillyCommandType::AuthorOnly,
            SillyCommandTypeOption::SingleUser => SillyCommandType::SingleUser,
        };

        let result = SillyCommandPDO::create_command(
            &context,
            &model.name,
            &model.description,
            &model.footer_text,
            command_type,
        )
        .await?;

        context.response_to_interaction_with_content(interaction, 
            &format!("Command has been created with id {}\nYou should now try to use /reload_command to see it appear!", result)
        ).await?;
        
        Ok(Ok(()))
    }
}
