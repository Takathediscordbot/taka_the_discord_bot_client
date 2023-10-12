use anyhow::{anyhow, Result};
use rand::prelude::*;
use rand::seq::IteratorRandom;
use std::{ffi::OsStr, path::Path, sync::Arc};

use twilight_model::{
    application::interaction::application_command::{CommandData, CommandOptionValue},
    channel::message::embed::EmbedFooter,
    gateway::payload::incoming::InteractionCreate,
    http::attachment::Attachment,
};
use twilight_util::builder::embed::{EmbedBuilder, ImageSource};

use crate::{
    context::Context, services::silly_command::SillyCommandPDO, utils::create_embed::create_embed,
};

async fn handle_author_silly_command(
    interaction: &InteractionCreate,
    _data: Box<CommandData>,
    command: crate::models::silly_command::SillyCommandData,
    context: Arc<Context>,
) -> Result<Result<(), anyhow::Error>, anyhow::Error> {
    let thread = Context::threaded_defer_response(Arc::clone(&context), interaction);
    let Some(author_id) = interaction.author_id() else {
        return Ok(Err(anyhow!("❌ Couldn't find command author.")))
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

    let (embed, attachment) = create_embed_image(Arc::clone(&context), image, &text).await?;

    let image_bytes = { std::fs::read(image)? };

    let embed = embed.build();

    thread.await??;
    context
        .http_client
        .interaction(context.application.id)
        .update_response(&interaction.token)
        .attachments(&[Attachment::from_bytes(attachment, image_bytes, 1)])?
        .embeds(Some(&[embed]))?
        .await?;

    Ok(Ok(()))
}

async fn handle_single_user_silly_command(
    interaction: &InteractionCreate,
    data: Box<CommandData>,
    command: crate::models::silly_command::SillyCommandData,
    context: Arc<Context>,
) -> std::result::Result<std::result::Result<(), anyhow::Error>, anyhow::Error> {
    let thread = Context::threaded_defer_response(Arc::clone(&context), interaction);
    
    let Some(a) = data.options.iter().find(|a| &a.name == "user") else {
        return Ok(Err(anyhow!("❌ Command has to be reloaded, tell taka.")))
    };

    let CommandOptionValue::User(user) = a.value else {
        return Ok(Err(anyhow!("❌ Command has to be reloaded, tell taka.")))
    };

    let Some(CommandOptionValue::String(preference)) = 
        data.options.iter().find(|a| a.name == "preference").map(|c| c.value.clone()) else {
            return Ok(Err(anyhow!("❌ Command has to be reloaded, tell taka.")))
    };



    let Some(author_id) = interaction.author_id() else {
        return Ok(Err(anyhow!("❌ Couldn't find command author.")))
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

        let (embed, attachment) = create_embed_image(Arc::clone(&context), image, &text).await?;

        let image_bytes = { std::fs::read(image)? };

        let embed = embed.build();

        context
            .http_client
            .interaction(context.application.id)
            .update_response(&interaction.token)
            .attachments(&[Attachment::from_bytes(attachment, image_bytes, 1)])?
            .embeds(Some(&[embed]))?
            .await?;
    } else {
        let image = if preference == "ALL" {
            if !command.images.is_empty() {
                command.images[rand::thread_rng().gen_range(0..command.images.len())].clone()
            } else {
                return Ok(Err(anyhow!("❌ No images have been added yet.")));
            }
        }
        else {

            let mut rng = rand::thread_rng(); 
            match command.images.iter().filter(|image| {
                image.as_str() == preference
            }).choose(&mut rng) {
                Some(image) => image.clone(),
                None => return Ok(Err(anyhow!("❌ No images have been added yet."))),
            }
        };


        let text = if !command.texts.is_empty() {
            &command.texts[rand::thread_rng().gen_range(0..command.texts.len())]
        } else {
            ""
        }
        .replace("{author}", &format!("<@{}>", author_id.get()))
        .replace("{user}", &format!("<@{}>", user.get()));

        let (embed, attachment) = create_embed_image(Arc::clone(&context), &image, &text).await?;

        let image_bytes = { std::fs::read(image)? };
        
        let _ = {
            if let None = SillyCommandPDO::fetch_command_usage(
                Arc::clone(&context),
                command.id_silly_command,
                author_id.get(),
                user.get(),
            )
            .await {
                let _ = SillyCommandPDO::create_command_usage(
                    Arc::clone(&context),
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
            Arc::clone(&context),
            command.id_silly_command,
            author_id.get(),
            user.get(),
        )
        .await?;

        let embed = embed
            .footer(EmbedFooter {
                icon_url: None,
                proxy_icon_url: None,
                text: 
                command.footer_text
                    .replace("{author}", &author_name)
                    .replace("{user}", &user_name)
                    .replace("{count}", &format!("{}", usages.usages))
            })
            .build();

        
        thread.await??;
        let interaction_client = &context.http_client.interaction(context.application.id);
        interaction_client
            .update_response(&interaction.token)
            .attachments(&[Attachment::from_bytes(attachment, image_bytes, 1)])?
            .embeds(Some(&[embed]))?
            .await?;

        let _response = interaction_client
            .create_followup(&interaction.token)
            .content(&format!("<@{}>", user.get()))?
            .await?
            .model()
            .await?;


    };

    Ok(Ok(()))
}

async fn create_embed_image(
    context: Arc<Context>,
    image: &str,
    text: &str,
) -> anyhow::Result<(EmbedBuilder, String)> {
    let embed = create_embed(None, Arc::clone(&context)).await?;
    let img = Path::new(&image)
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or(anyhow!("Couldn't find image."))?;
    Ok((
        embed.image(ImageSource::attachment(img)?).description(text),
        img.to_string(),
    ))
}

pub async fn handle_silly_command(
    _shard: u64,
    interaction: &InteractionCreate,
    data: Box<CommandData>,
    command: crate::models::silly_command::SillyCommandData,
    context: Arc<Context>,
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
