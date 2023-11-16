use std::{fs::File, io::Write};

use anyhow::anyhow;

use crate::{context::Context, models::silly_command::{RawSillyCommandData, SillyCommandData, Usages, SillyCommandType}};

use sqlx::FromRow;

#[derive(FromRow)]
struct CommandUsage {
    usages: i32
}

#[derive(FromRow)]
pub struct CommandUsageId {
    pub id_silly_command_usage: i32
}

#[derive(FromRow)]
struct CommandId {
    id_silly_command: i32
}

#[derive(FromRow)]
struct RandomImage {
    image: String
}

#[derive(FromRow)]
struct CommandTextId {
    id_silly_command_text: i32
}

#[derive(FromRow)]
struct CommandSelfActionTextId {
    id_silly_command_self_action_text: i32
}

#[derive(FromRow)]
struct CommandSelfActionImageId {
    id_silly_command_self_action: i32
}

#[derive(FromRow)]
struct CommandImageId {
    id_silly_command_images: i32
}


pub struct SillyCommandPDO;
impl SillyCommandPDO {
    
    pub async fn fetch_silly_commands(context: &Context) -> Vec<SillyCommandData> {
        let Ok(silly_commands) = sqlx::query(include_str!("../sql/silly_commands/fetch_silly_commands.sql"))
        .fetch_all(&context.sql_connection)
        .await else {
            return vec![]
        };

        silly_commands
            .into_iter()
            .filter_map(|row| RawSillyCommandData::from_row(&row).ok())
            .filter_map(RawSillyCommandData::into_silly_command_data)
            .collect()
    }

    pub async fn fetch_command_usage(
        context: &Context,
        command: i32,
        author: u64,
        user: u64,
    ) -> Option<i32> {
        let record = sqlx::query(include_str!("../sql/silly_commands/fetch_command_usage.sql"))
        .bind(author.to_string())
        .bind(user.to_string())
        .bind(command)
        .fetch_one(&context.sql_connection)
        .await
        .ok();

        record
        .map(|row| CommandUsage::from_row(&row).ok())
        .flatten()
        .and_then(|d| Some(d.usages))
    }

    pub async fn increment_command_usage(
        context: &Context,
        command: i32,
        author: u64,
        user: u64,
    ) -> anyhow::Result<Usages> {
        Ok(sqlx::query(include_str!("../sql/silly_commands/increment_command_usage.sql"))
        .bind(author.to_string())
        .bind(user.to_string()) 
        .bind(command)
        .fetch_one(&context.sql_connection)
        .await
        .map(|v| Usages::from_row(&v))??
        )
    }

    pub async fn create_command_usage(
        context: &Context,
        command: i32,
        author: u64,
        user: u64,
    ) -> anyhow::Result<i32> {
        let id = sqlx::query(include_str!("../sql/silly_commands/create_command_usage.sql"))
        .bind(command) 
        .bind(author.to_string())
        .bind(user.to_string())
        .fetch_one(&context.sql_connection)
        .await
        .map(|data| 
            CommandUsage::from_row(&data)
            .map(|usages| usages.usages))??;

        Ok(id)
    }

    pub async fn create_command(
        context: &Context,
        command_name: &str,
        description: &str,
        footer_text: &str,
        command_type: SillyCommandType,
    ) -> anyhow::Result<i32> {
        let id = sqlx::query(include_str!("../sql/silly_commands/create_command.sql"))
        .bind(command_name)
        .bind(description)
        .bind(command_type as i32)
        .bind(footer_text)
        .fetch_one(&context.sql_connection)
        .await
        .map(|row| 
            CommandId::from_row(&row)
            .map(|command| command)
        )??;

        Ok(id.id_silly_command)
    }

    pub async fn add_preference(
        context: &Context,
        preference: &str,
        command: &str
    )
    -> anyhow::Result<()> {
        sqlx::query(include_str!("../sql/silly_commands/add_preference.sql"))
        .bind(preference)
        .bind(command)
        .execute(&context.sql_connection)
        .await?;

        Ok(())
    }

    pub async fn fetch_silly_command_by_name(
        context: &Context,
        name: &str,
    ) -> Option<SillyCommandData> {
        let Ok(Ok(silly_commands)) = 
        sqlx::query(include_str!(
        "../sql/silly_commands/fetch_silly_command_by_name.sql"))
        .bind(name)
        .fetch_one(&context.sql_connection)
        .await
        .map(|row| RawSillyCommandData::from_row(&row)) else {
            return None
        };

        silly_commands.into_silly_command_data()
    }

    pub async fn fetch_random_silly_image_by_name_and_preference(
        context: &Context,
        command: i32,
        preference: &str
    ) -> anyhow::Result<String> {
        let result =
        RandomImage::from_row(& 
        sqlx::query(include_str!(
            "../sql/silly_commands/fetch_random_silly_image_by_name_and_preference.sql"))
        .bind(command)
        .bind( preference)
        .fetch_one(&context.sql_connection)
        .await?)?;

        Ok(result.image)
    }


    pub async fn add_text(
        context: &Context,
        command_name: &str,
        content: &str,
    ) -> anyhow::Result<i32> {
        let command = Self::fetch_silly_command_by_name(&context, command_name)
            .await
            .ok_or(anyhow!("Couldn't find command!"))?;
        
        let id = CommandTextId::from_row(&sqlx::query(include_str!("../sql/silly_commands/add_text.sql"))
        .bind(command.id_silly_command)
        .bind(content)
        .fetch_one(&context.sql_connection)
        .await?)?;

        Ok(id.id_silly_command_text)
    }

    pub async fn add_text_author(
        context: &Context,
        command_name: &str,
        content: &str,
    ) -> anyhow::Result<i32> {
        let command = Self::fetch_silly_command_by_name(&context, command_name)
            .await
            .ok_or(anyhow!("Couldn't find command!"))?;

        let id = CommandSelfActionTextId::from_row(&sqlx::query(include_str!("../sql/silly_commands/add_author_text.sql"))
        .bind(command.id_silly_command)
        .bind(content)
        .fetch_one(&context.sql_connection)
        .await?)?;

        Ok(id.id_silly_command_self_action_text)
    }

    pub async fn add_image(
        context: &Context,
        command_name: &str,
        image: Vec<u8>,
        extension: &str,
        preference: Option<String>
    ) -> anyhow::Result<i32> {
        let command = Self::fetch_silly_command_by_name(&context, command_name)
            .await
            .ok_or(anyhow!("Couldn't find command!"))?;

        if matches!(command.command_type, SillyCommandType::AuthorOnly) {
            return Self::add_image_author(context, command_name, image, extension).await;
        }

        let file_name = uuid::Uuid::new_v4().to_string();
        let file_path = format!("./assets/{file_name}.{extension}");
        let mut out = File::create(&file_path)?;
        let preference = preference.unwrap_or("ALL".to_string());

        out.write_all(&image[..])?;

        let id = CommandImageId::from_row(&sqlx::query(include_str!("../sql/silly_commands/add_image.sql"))
        .bind(command.id_silly_command)
        .bind(file_path) 
        .bind(preference)
        .fetch_one(&context.sql_connection)
        .await?)?;

        Ok(id.id_silly_command_images)
    }

    pub async fn add_image_author(
        context: &Context,
        command_name: &str,
        image: Vec<u8>,
        extension: &str,
    ) -> anyhow::Result<i32> {
        let command = Self::fetch_silly_command_by_name(&context, command_name)
            .await
            .ok_or(anyhow!("Couldn't find command!"))?;

        let file_name = uuid::Uuid::new_v4().to_string();
        let file_path = format!("./assets/{file_name}.{extension}");
        let mut out = File::create(&file_path)?;

        out.write_all(&image[..])?;

        let id = CommandSelfActionImageId::from_row( 
        &sqlx::query(include_str!("../sql/silly_commands/add_image_author.sql"))
        .bind(command.id_silly_command)
        .bind(file_path)
        .fetch_one(&context.sql_connection)
        .await?)?;

        Ok(id.id_silly_command_self_action)
    }
}
