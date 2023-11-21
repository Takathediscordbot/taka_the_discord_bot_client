use std::borrow::Cow;

use anyhow::anyhow;
use twilight_interactions::command::{
    CommandInputData, CommandModel, CommandOption, CreateCommand, CreateOption,
};
use twilight_model::{
    application::interaction::application_command::CommandData,
    gateway::payload::incoming::InteractionCreate,
};
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::{
    context::Context,
    models::silly_command::SillyCommandType,
    services::silly_command::SillyCommandPDO,
    utils::{self, box_commands::RunnableCommand},
};

#[derive(CreateOption, CommandOption, Debug)]
pub enum SillyCommandTypeOption {
    #[option(name = "Author Only", value = "author_only")]
    AuthorOnly,
    #[option(name = "Single User", value = "single_user")]
    SingleUser,
}

#[derive(CreateCommand, CommandModel)]
#[command(name = "silly_command", desc = "Get all data of a silly command")]
pub struct SillyCommand {
    /// The name of the command
    name: String,
}

#[async_trait::async_trait]
impl RunnableCommand for SillyCommand {
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

        let result =
            match SillyCommandPDO::fetch_silly_command_by_name(&context, &model.name)
                .await
            {
                Some(e) => e,
                None => return Ok(Err(anyhow!("❌ This command does not exist"))),
            };
        

        let description = format!(
            "Command type: {}\nImages: {}\nSelf Images: {}\n Texts: {}\n Self Texts: {}",
            match result.command_type {
                SillyCommandType::AuthorOnly => "Author only",
                SillyCommandType::SingleUser => "Single User",
            },
            result.images.len(),
            result.self_images.len(),
            result.texts.len(),
            result.self_texts.len()
        );

        let embed = utils::create_embed::create_embed(None, &context).await?;
        let embed = embed.description(description).build();

        context
            .response_to_interaction(
                interaction,
                InteractionResponseDataBuilder::new().embeds([embed]).build(),
            )
            .await?;

        Ok(Ok(()))
    }
}
