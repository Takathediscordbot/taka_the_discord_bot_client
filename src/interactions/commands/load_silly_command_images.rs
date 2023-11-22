use std::{borrow::Cow, fs::DirEntry};

use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
    application::interaction::application_command::CommandData,
    gateway::payload::incoming::InteractionCreate,
};

use crate::{
    context::Context,     
    services::silly_command::SillyCommandPDO,
    utils::box_commands::RunnableCommand,
};

#[derive(CreateCommand, CommandModel)]
#[command(
    name = "load_silly_command_images",
    desc = "Create preferences and images (author only)"
)]
pub struct LoadSillyCommandImages {}

impl LoadSillyCommandImages {
    fn check_is_directory(file: &DirEntry, warnings: &mut String) -> bool {
        let Ok(file_name) = &file.file_name().into_string() else {
            *warnings += "Couldn't get the filename from a file.";
            return false;
        };

        if let Ok(file_type) = file.file_type() {
            if !file_type.is_dir() {
                *warnings += &format!("Found file that wasn't a directory {} \n", file_name);
                return false;
            }
        } else {
            *warnings += &format!("Couldn't find the file type of {} \n", file_name);
            return false;
        };

        return true;
    }

    fn check_is_file(file: &DirEntry, warnings: &mut String) -> bool {
        let Ok(file_name) = &file.file_name().into_string() else {
            *warnings += "Couldn't get the filename from a file.";
            return false;
        };

        if let Ok(file_type) = file.file_type() {
            if file_type.is_dir() {
                *warnings += &format!("Found file that was a directory {} \n", file_name);
                return false;
            }
        } else {
            *warnings += &format!("Couldn't find the file type of {} \n", file_name);
            return false;
        };

        return true;
    }
}

#[async_trait::async_trait]
impl RunnableCommand for LoadSillyCommandImages {
    async fn run(
        _shard: u64,
        interaction: &InteractionCreate,
        _data: Box<CommandData>,
        context: &Context,
    ) -> anyhow::Result<anyhow::Result<()>> {
        context.defer_response(interaction).await?;
        let interaction_client = context.http_client.interaction(context.application.id);

        let Some(author) = interaction.author_id() else {
            return Ok(Err(anyhow::anyhow!("❌ You're probably not the author of this bot!")))
        };

        if author.get() != context.author_id {
            return Ok(Err(anyhow::anyhow!("❌ You're definitely not the author of this bot!")));
        }

        let commands = SillyCommandPDO::fetch_silly_commands(&context).await;

        let Ok(silly_command_folder) = std::fs::read_dir("./assets/silly_commands") else {
            return Ok(Err(anyhow::anyhow!("❌ Couldn't read silly_commands folder")));
        };

        let mut warnings = String::new();

        for file in silly_command_folder {
            let Ok(file) = file else {
                warnings += "Couldn't read a file in the directory ./assets/silly_commands";
                continue;
            };

            if !Self::check_is_directory(&file, &mut warnings) {
                continue;
            };

            let command_name = file.file_name();
            let command_name = command_name.to_string_lossy();

            let command_name = match command_name {
                Cow::Borrowed(f) => f.to_owned(),
                Cow::Owned(f) => f.clone(),
            };

            let Some(command) = commands.iter().find(|c| {
                c.name == command_name
            }) else {
                warnings += &format!("Found file without an associated command {}\n", command_name);
                continue;
            };

            let path = file.path();
            let Ok(dir) = std::fs::read_dir(path.clone()) else {
                warnings += &format!("Couldn't read directory {}\n", path.to_string_lossy());
                continue;
            };

            for file in dir {
                let Ok(file) = file else {
                    warnings += &format!("Couldn't read file in directory {}\n", path.to_string_lossy());
                    continue;
                };

                let file_name = file.file_name();
                let file_name = file_name.to_string_lossy();

                let file_name = match file_name {
                    Cow::Borrowed(f) => f.to_owned(),
                    Cow::Owned(f) => f,
                };

                let preference_name = match file_name.as_str() {
                    "BB" => "Male x Male".to_owned(),
                    "BG" => "Male x Female".to_owned(),
                    "GG" => "Female x Female".to_owned(),
                    name => name.to_owned(),
                };

                if !command.gender_attributes.contains(&preference_name) {
                    SillyCommandPDO::add_preference(
                        &context,
                        &preference_name,
                        &command.name,
                    )
                    .await?;
                };

                let path = file.path();
                let Ok(dir) = std::fs::read_dir(&path) else {
                    warnings += &format!("Couldn't read directory {}\n", path.to_string_lossy());
                    continue;
                };

                for file in dir {
                    let Ok(file) = file else {
                        warnings += &format!("Couldn't read file in directory {}\n", path.to_string_lossy());
                        continue;
                    };

                    if !Self::check_is_file(&file, &mut warnings) {
                        continue;
                    }
                    let path = file.path();
                    let Some(extension) = path.extension() else {
                        warnings += &format!("Couldn't find extension of {}\n", file.path().to_string_lossy());
                        continue;
                    };

                    let Ok(file_content) = std::fs::read(file.path()) else {
                        warnings += &format!("Couldn't read file {}\n", file.path().to_string_lossy());
                        continue;
                    };

                    let extension = extension.to_string_lossy();

                    SillyCommandPDO::add_image(
                        &context,
                        &command.name,
                        file_content,
                        &extension,
                        Some(preference_name.clone()),
                    )
                    .await?;
                }
            }
        }

        // let buf = buf.to_vec();
        interaction_client
            .update_response(&interaction.token)
            .content(Some(if warnings.is_empty() {
                "✅ Finished with no warnings!"
            } else {
                &warnings
            }))?
            .await?;

        Ok(Ok(()))
    }
}
