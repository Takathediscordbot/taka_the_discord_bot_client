use std::borrow::Cow;

use anyhow::anyhow;
use twilight_interactions::command::{CommandInputData, CommandModel, CreateCommand};
use twilight_model::{
    application::interaction::application_command::CommandData, channel::Attachment,
    gateway::payload::incoming::InteractionCreate,
};
use mime::Mime;
use crate::{
    context::Context, services::silly_command::SillyCommandPDO,
    utils::box_commands::RunnableCommand,
};

#[derive(CreateCommand, CommandModel)]
#[command(name = "add_silly_image", desc = "Add a silly image (author only)")]
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
        context: &Context<'_>,
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

        let Some(file_type) = &model.attachment.content_type else {
            return Ok(Err(anyhow!("❌ No file type found!")));
        };

        // parse mime type
        let file_type: Mime = file_type.parse()?;

        let extension = match (file_type.type_(), file_type.subtype()) {
            (_, mime::GIF | mime::JPEG | mime::PNG) | (mime::IMAGE, _) => {
                match file_type.subtype() {
                    mime::GIF => Some("gif"),
                    mime::JPEG => Some("jpg"),
                    mime::PNG => Some("png"),
                    _ => None,
                }
            }
            _ => {
                return Ok(Err(anyhow!(
                    "❌ File type not supported: {:?}",
                    file_type
                )));
            }

        };

        let extension = match extension {
            Some(extension) => extension,
            None => {
                return Ok(Err(anyhow!(
                    "❌ File type not supported: {:?}",
                    file_type
                )));
            }
        };

 
        let bytes = reqwest::get(model.attachment.url)
            .await?
            .bytes()
            .await?
            .to_vec();
        let result = if model.author {
            SillyCommandPDO::add_image_author(&context, &model.name, bytes, extension)
                .await?
        } else {
            SillyCommandPDO::add_image(
                &context,
                &model.name,
                bytes,
                extension,
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
