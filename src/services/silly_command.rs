use std::{sync::Arc, fs::File, io::Write};

use anyhow::anyhow;

use crate::{context::Context, models::silly_command::{RawSillyCommandData, SillyCommandData, Usages, SillyCommandType}};


pub struct SillyCommandPDO;
impl SillyCommandPDO {
    
    pub async fn fetch_silly_commands(context: Arc<Context>) -> Vec<SillyCommandData> {
        let Ok(silly_commands) = sqlx::query_file_as!(RawSillyCommandData, "./src/sql/silly_commands/fetch_silly_commands.sql")
        .fetch_all(&context.sql_connection)
        .await else {
            return vec![]
        };

        silly_commands
            .into_iter()
            .filter_map(RawSillyCommandData::into_silly_command_data)
            .collect()
    }

    pub async fn fetch_command_usage(
        context: Arc<Context>,
        command: i32,
        author: u64,
        user: u64,
    ) -> Option<i32> {
        let record = sqlx::query_file!("./src/sql/silly_commands/fetch_command_usage.sql"
        , author.to_string(), user.to_string(), command)
        .fetch_one(&context.sql_connection)
        .await;

        record.map(|d| Some(d.usages)).unwrap_or(None)
    }

    pub async fn increment_command_usage(
        context: Arc<Context>,
        command: i32,
        author: u64,
        user: u64,
    ) -> anyhow::Result<Usages> {
        Ok(sqlx::query_file_as!(Usages, "./src/sql/silly_commands/increment_command_usage.sql", author.to_string(), user.to_string(), command)
        .fetch_one(&context.sql_connection)
        .await?)
    }

    pub async fn create_command_usage(
        context: Arc<Context>,
        command: i32,
        author: u64,
        user: u64,
    ) -> anyhow::Result<i32> {
        let id = sqlx::query_file!("./src/sql/silly_commands/create_command_usage.sql", command, author.to_string(), user.to_string())
        .fetch_one(&context.sql_connection)
        .await?;

        Ok(id.id_silly_command_usage)
    }

    pub async fn create_command(
        context: Arc<Context>,
        command_name: &str,
        description: &str,
        footer_text: &str,
        command_type: SillyCommandType,
    ) -> anyhow::Result<i32> {
        let id = sqlx::query_file!("./src/sql/silly_commands/create_command.sql", command_name, description, command_type as i32, footer_text)
        .fetch_one(&context.sql_connection)
        .await?;

        Ok(id.id_silly_command)
    }

    pub async fn add_preference(
        context: Arc<Context>,
        preference: &str,
        command: &str
    )
    -> anyhow::Result<()> {
        sqlx::query_file!("./src/sql/silly_commands/add_preference.sql", preference, command)
        .execute(&context.sql_connection)
        .await?;

        Ok(())
    }

    pub async fn fetch_silly_command_by_name(
        context: Arc<Context>,
        name: &str,
    ) -> Option<SillyCommandData> {
        let Ok(silly_commands) = 
        sqlx::query_file_as!(RawSillyCommandData,
        "./src/sql/silly_commands/fetch_silly_command_by_name.sql", name)
        .fetch_one(&context.sql_connection)
        .await else {
            return None
        };

        silly_commands.into_silly_command_data()
    }

    pub async fn fetch_random_silly_image_by_name_and_preference(
        context: Arc<Context>,
        command: i32,
        preference: &str
    ) -> anyhow::Result<String> {
        let result = 
        sqlx::query_file!(
            "./src/sql/silly_commands/fetch_random_silly_image_by_name_and_preference.sql", 
            command, preference)
        .fetch_one(&context.sql_connection)
        .await?;

        Ok(result.image)
    }


    pub async fn add_text(
        context: Arc<Context>,
        command_name: &str,
        content: &str,
    ) -> anyhow::Result<i32> {
        let command = Self::fetch_silly_command_by_name(Arc::clone(&context), command_name)
            .await
            .ok_or(anyhow!("Couldn't find command!"))?;

        let id = sqlx::query_file!("./src/sql/silly_commands/add_text.sql", 
        command.id_silly_command, content)
        .fetch_one(&context.sql_connection)
        .await?;

        Ok(id.id_silly_command_text)
    }

    pub async fn add_text_author(
        context: Arc<Context>,
        command_name: &str,
        content: &str,
    ) -> anyhow::Result<i32> {
        let command = Self::fetch_silly_command_by_name(Arc::clone(&context), command_name)
            .await
            .ok_or(anyhow!("Couldn't find command!"))?;

        let id = sqlx::query_file!("./src/sql/silly_commands/add_author_text.sql", command.id_silly_command, content)
        .fetch_one(&context.sql_connection)
        .await?;

        Ok(id.id_silly_command_self_action_text)
    }

    pub async fn add_image(
        context: Arc<Context>,
        command_name: &str,
        image: Vec<u8>,
        extension: &str,
        preference: Option<String>
    ) -> anyhow::Result<i32> {
        let command = Self::fetch_silly_command_by_name(Arc::clone(&context), command_name)
            .await
            .ok_or(anyhow!("Couldn't find command!"))?;

        if matches!(command.command_type, SillyCommandType::AuthorOnly) {
            return Self::add_image_author(context, command_name, image, extension).await;
        }

        let file_name = uuid::Uuid::new_v4().to_string();
        let file_path = format!("./assets/{file_name}.{extension}");
        let mut out = File::create(&file_path)?;

        out.write_all(&image[..])?;

        let id = sqlx::query_file!("./src/sql/silly_commands/add_image.sql", command.id_silly_command, file_path, preference)
        .fetch_one(&context.sql_connection)
        .await?;

        Ok(id.id_silly_command_images)
    }

    pub async fn add_image_author(
        context: Arc<Context>,
        command_name: &str,
        image: Vec<u8>,
        extension: &str,
    ) -> anyhow::Result<i32> {
        let command = Self::fetch_silly_command_by_name(Arc::clone(&context), command_name)
            .await
            .ok_or(anyhow!("Couldn't find command!"))?;

        let file_name = uuid::Uuid::new_v4().to_string();
        let file_path = format!("./assets/{file_name}.{extension}");
        let mut out = File::create(&file_path)?;

        out.write_all(&image[..])?;

        let id = sqlx::query_file!("./src/sql/silly_commands/add_image_author.sql", command.id_silly_command, file_path)
        .fetch_one(&context.sql_connection)
        .await?;

        Ok(id.id_silly_command_self_action)
    }
}
