use anyhow::{anyhow, Result};
use rand::prelude::*;
use std::{ffi::OsStr, path::Path};

use twilight_model::{
    application::interaction::application_command::{CommandData, CommandOptionValue},
    channel::message::embed::EmbedFooter,
    gateway::payload::incoming::InteractionCreate,
    http::attachment::Attachment,
};
use twilight_util::builder::{
    embed::{EmbedBuilder, ImageSource},
    InteractionResponseDataBuilder,
};

use crate::{
    context::Context, services::silly_command::SillyCommandPDO, utils::create_embed::create_embed,
};

#[cfg(feature = "database")]
async fn handle_author_silly_command(
    interaction: &InteractionCreate,
    _data: Box<CommandData>,
    command: crate::models::silly_command::SillyCommandData,
    context: &Context<'_>,
) -> Result<Result<(), anyhow::Error>, anyhow::Error> {
    let Some(author_id) = interaction.author_id() else {
        return Ok(Err(anyhow!("❌ Couldn't find command author.")));
    };

    let mut images = command.self_images.clone();
    images.append(&mut command.images.clone());
    let image = if !images.is_empty() {
        &images[rand::thread_rng().gen_range(0..images.len())]
    } else {
        return Ok(Err(anyhow!("❌ No images have been added yet.")));
    };

    let mut texts = command.self_texts.clone();
    texts.append(&mut command.texts.clone());
    let text = if !texts.is_empty() {
        &texts[rand::thread_rng().gen_range(0..texts.len())]
    } else {
        ""
    }
    .replace("{author}", &format!("<@{}>", author_id.get()));

    let (embed, attachment) = create_embed_image(&context, image, &text).await?;

    let image_bytes = { std::fs::read(image)? };

    let embed = embed.build();

    let content = InteractionResponseDataBuilder::new()
        .attachments([Attachment::from_bytes(attachment, image_bytes, 1)])
        .embeds([embed]);

    context
        .response_to_interaction(&interaction, content.build())
        .await?;

    Ok(Ok(()))
}

#[cfg(feature = "database")]
async fn handle_single_user_silly_command(
    interaction: &InteractionCreate,
    data: Box<CommandData>,
    command: crate::models::silly_command::SillyCommandData,
    context: &Context<'_>,
) -> std::result::Result<std::result::Result<(), anyhow::Error>, anyhow::Error> {
    let Some(a) = data.options.iter().find(|a| &a.name == "user") else {
        return Ok(Err(anyhow!("❌ Command has to be reloaded, tell taka.")));
    };

    let CommandOptionValue::User(user) = a.value else {
        return Ok(Err(anyhow!("❌ Command has to be reloaded, tell taka.")));
    };

    let Some(CommandOptionValue::String(preference)) = data
        .options
        .iter()
        .find(|a| a.name == "preference")
        .map(|c| c.value.clone())
    else {
        return Ok(Err(anyhow!("❌ Command has to be reloaded, tell taka.")));
    };

    let Some(author_id) = interaction.author_id() else {
        return Ok(Err(anyhow!("❌ Couldn't find command author.")));
    };

    if user == author_id {
        let image = if !command.self_images.is_empty() {
            &command.self_images[rand::thread_rng().gen_range(0..command.self_images.len())]
        } else if !command.images.is_empty() {
            &command.images[rand::thread_rng().gen_range(0..command.images.len())]
        } else {
            return Ok(Err(anyhow!("❌ No images have been added yet.")));
        };

        let text = if !command.self_texts.is_empty() {
            &command.self_texts[rand::thread_rng().gen_range(0..command.self_texts.len())]
        } else if !command.texts.is_empty() {
            &command.texts[rand::thread_rng().gen_range(0..command.texts.len())]
        } else {
            ""
        }
        .replace("{author}", &format!("<@{}>", author_id.get()))
        .replace("{user}", &format!("<@{}>", user.get()));

        let (embed, attachment) = create_embed_image(&context, image, &text).await?;

        let image_bytes = { std::fs::read(image)? };

        let embed = embed.build();

        let content = InteractionResponseDataBuilder::new()
            .attachments([Attachment::from_bytes(attachment, image_bytes, 1)])
            .embeds([embed]);

        context
            .response_to_interaction(&interaction, content.build())
            .await?;
    } else {
        let image = if preference == "ALL" {
            if !command.images.is_empty() {
                command.images[rand::thread_rng().gen_range(0..command.images.len())].clone()
            } else {
                return Ok(Err(anyhow!("❌ No images have been added yet.")));
            }
        } else {
            SillyCommandPDO::fetch_random_silly_image_by_name_and_preference(
                &context,
                command.id_silly_command,
                &preference,
            )
            .await?
        };

        let text = if !command.texts.is_empty() {
            &command.texts[rand::thread_rng().gen_range(0..command.texts.len())]
        } else {
            ""
        }
        .replace("{author}", &format!("<@{}>", author_id.get()))
        .replace("{user}", &format!("<@{}>", user.get()));

        let (embed, attachment) = create_embed_image(&context, &image, &text).await?;

        let image_bytes = { std::fs::read(image)? };

        let _ = {
            if let None = SillyCommandPDO::fetch_command_usage(
                &context,
                command.id_silly_command,
                author_id.get(),
                user.get(),
            )
            .await
            {
                let _ = SillyCommandPDO::create_command_usage(
                    &context,
                    command.id_silly_command,
                    author_id.get(),
                    user.get(),
                )
                .await;
            }
        };

        let author_name = &interaction
            .author()
            .ok_or(anyhow!("❌ Couldn't find author"))?
            .name;
        let user_name = context.http_client.user(user).await?.model().await?.name;

        let usages = SillyCommandPDO::increment_command_usage(
            &context,
            command.id_silly_command,
            author_id.get(),
            user.get(),
        )
        .await?;

        let embed = embed
            .footer(EmbedFooter {
                icon_url: None,
                proxy_icon_url: None,
                text: command
                    .footer_text
                    .replace("{author}", &author_name)
                    .replace("{user}", &user_name)
                    .replace("{count}", &format!("{}", usages.usages)),
            })
            .build();

        let content = InteractionResponseDataBuilder::new()
            .attachments([Attachment::from_bytes(attachment, image_bytes, 1)])
            .embeds([embed])
            .content(format!("<@{}>", user.get()));

        context
            .response_to_interaction(&interaction, content.build())
            .await?;

        #[cfg(feature = "ai")]
        {
            if user.get() != context.application.id.get() {
                return Ok(Ok(()));
            }

            if let Some(channel) = &interaction.channel {
                let _ = context.http_client.create_typing_trigger(channel.id).await;
            }

            let author_name_2 = interaction
                .author()
                .map(|author| author.name.clone())
                .unwrap_or(String::from("An unknown user"));

            let author_nickname = interaction
                .member
                .as_ref()
                .map(|member| member.nick.clone())
                .flatten()
                .unwrap_or(author_name_2.clone());

            let prompt = include_str!("../assets/silly_command_prompt");

            let Ok(ai_message) = context
                .chatgpt_client
                .new_conversation()
                .send_message(format!(
                    "You are currently talking to {} (Also referred to as {}). {}. ",
                    author_nickname,
                    author_name_2,
                    prompt.replacen("{command_name}", &command.name, 1)
                ))
                .await
            else {
                log::error!("Couldn't generate ai message to respond to silly command.");
                return Ok(Ok(()));
            };

            let Some(message) = ai_message.message_choices.get(0) else {
                log::error!("Couldn't generate ai message to respond to silly command.");
                return Ok(Ok(()));
            };

            let message_content = &message.message.content;

            let image = if preference == "ALL" {
                if !command.images.is_empty() {
                    command.images[rand::thread_rng().gen_range(0..command.images.len())].clone()
                } else {
                    return Ok(Err(anyhow!("❌ No images have been added yet.")));
                }
            } else {
                SillyCommandPDO::fetch_random_silly_image_by_name_and_preference(
                    &context,
                    command.id_silly_command,
                    &preference,
                )
                .await?
            };

            let text = if !command.texts.is_empty() {
                &command.texts[rand::thread_rng().gen_range(0..command.texts.len())]
            } else {
                ""
            }
            .replace("{author}", &format!("<@{}>", user.get()))
            .replace("{user}", &format!("<@{}>", author_id.get()));

            let (embed, attachment) = create_embed_image(&context, &image, &text).await?;

            let image_bytes = { std::fs::read(image)? };

            let embed = embed
                .footer(EmbedFooter {
                    icon_url: None,
                    proxy_icon_url: None,
                    text: command
                        .footer_text
                        .replace("{author}", &user_name)
                        .replace("{user}", &author_name)
                        .replace("{count}", &format!("{}", usages.usages)),
                })
                .build();

            context
                .http_client
                .interaction(context.application.id)
                .create_followup(&interaction.token)
                .embeds(&[embed])?
                .content(&message_content)?
                .attachments(&[Attachment::from_bytes(attachment, image_bytes, 1)])?
                .await?;
        }
    };

    Ok(Ok(()))
}

async fn create_embed_image(
    context: &Context<'_>,
    image: &str,
    text: &str,
) -> anyhow::Result<(EmbedBuilder, String)> {
    let embed = create_embed(None, &context).await?;
    let img = Path::new(&image)
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or(anyhow!("Couldn't find image."))?;
    Ok((
        embed.image(ImageSource::attachment(img)?).description(text),
        img.to_string(),
    ))
}
#[cfg(feature = "database")]
pub async fn handle_silly_command(
    _shard: u64,
    interaction: &InteractionCreate,
    data: Box<CommandData>,
    command: crate::models::silly_command::SillyCommandData,
    context: &Context<'_>,
) -> Result<Result<(), anyhow::Error>, anyhow::Error> {
    match command.command_type {
        crate::models::silly_command::SillyCommandType::AuthorOnly => {
            handle_author_silly_command(interaction, data, command, context).await
        }
        crate::models::silly_command::SillyCommandType::SingleUser => {
            handle_single_user_silly_command(interaction, data, command, context).await
        }
    }
}
