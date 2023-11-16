use std::{borrow::Cow, ffi::OsStr, path::Path};

use anyhow::anyhow;
use twilight_interactions::command::{CommandInputData, CommandModel, CreateCommand};
use twilight_model::{
    application::interaction::application_command::CommandData, channel::Attachment,
    gateway::payload::incoming::InteractionCreate,
};

use crate::{
    context::Context, services::silly_command::SillyCommandPDO,
    utils::box_commands::RunnableCommand,
};

#[derive(CreateCommand, CommandModel)]
#[command(name = "add_silly_image", desc = "Add a silly image (taka only)")]
pub struct AddSillyImage {
    /// The name of the command
    name: String,
    /// The image to add
    attachment: Attachment,
    /// Author
    author: bool,
    /// preference
    preference: Option<String>,
}

#[async_trait::async_trait]
impl RunnableCommand for AddSillyImage {
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
            return Ok(Err(anyhow!("❌ You're probably not taka")));
        };

        if author.get() != 434626996262273038 {
            return Ok(Err(anyhow!("❌ You're definitely not taka")));
        }

        let Some(file_type) = Path::new(&model.attachment.filename)
            .extension()
            .and_then(OsStr::to_str)
        else {
            return Ok(Err(anyhow!("❌ Couldn't find file extension.")));
        };

        let bytes = reqwest::get(model.attachment.url)
            .await?
            .bytes()
            .await?
            .to_vec();
        let result = if model.author {
            SillyCommandPDO::add_image_author(&context, &model.name, bytes, file_type)
                .await?
        } else {
            SillyCommandPDO::add_image(
                &context,
                &model.name,
                bytes,
                file_type,
                model.preference.clone(),
            )
            .await?
        };

        context
            .response_to_interaction_with_content(
                interaction,
                &format!(
                    " Image has been created with id {result} for command {}",
                    model.name
                ),
            )
            .await?;

        Ok(Ok(()))
    }
}
