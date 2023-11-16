use std::num::NonZeroU64;


use anyhow::anyhow;
use itertools::Itertools;
use twilight_interactions::command::CreateCommand;
use twilight_model::application::command::{
    CommandOptionChoice, CommandOptionChoiceValue, CommandOptionType, CommandType,
};
use twilight_model::application::{
    command::Command, command::CommandOption, interaction::application_command::CommandData,
};
use twilight_model::gateway::payload::incoming::InteractionCreate;
use twilight_model::id::marker::CommandVersionMarker;
use twilight_model::id::Id;
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::models::silly_command::SillyCommandType;
use crate::services::silly_command::SillyCommandPDO;
use crate::{context::Context, utils::box_commands::RunnableCommand};

#[derive(CreateCommand)]
#[command(name = "reload_commands", desc = "Reload commands (taka only)")]
pub struct ReloadCommands {}

impl ReloadCommands {
    async fn create_commands(context: &Context) -> anyhow::Result<Vec<Command>> {
        let mut v: Vec<Command> = context.commands
            .iter()
            .map(|a| a.create_command().into())
            .collect_vec();

        let commands = SillyCommandPDO::fetch_silly_commands(context).await;

        for command in commands.into_iter() {
            let gender_attributes = {
                let mut old = command.gender_attributes.clone();
                old.insert(0, "ALL".to_string());
                old.into_iter()
                    .map(|c| CommandOptionChoice {
                        name: c.clone(),
                        name_localizations: None,
                        value: CommandOptionChoiceValue::String(c),
                    })
                    .collect::<Vec<_>>()
            };

            let c = Command {
                application_id: None,
                default_member_permissions: None,
                dm_permission: None,
                description: command.description,
                description_localizations: None,
                guild_id: None,
                id: None,
                kind: CommandType::ChatInput,
                name: command.name.clone(),
                name_localizations: None,
                nsfw: None,
                options: match command.command_type {
                    SillyCommandType::AuthorOnly => vec![],
                    SillyCommandType::SingleUser => vec![
                        CommandOption {
                            autocomplete: None,
                            channel_types: None,
                            choices: None,
                            description: String::from("A user to target"),
                            description_localizations: None,
                            kind: CommandOptionType::User,
                            max_length: None,
                            max_value: None,
                            min_length: None,
                            min_value: None,
                            name: String::from("user"),
                            name_localizations: None,
                            options: None,
                            required: Some(true),
                        },
                        CommandOption {
                            autocomplete: None,
                            channel_types: None,
                            choices: Some(gender_attributes),
                            description: String::from(
                                "What kind of characters should be shown in the gif",
                            ),
                            description_localizations: None,
                            kind: CommandOptionType::String,
                            max_length: None,
                            max_value: None,
                            min_length: None,
                            min_value: None,
                            name: String::from("preference"),
                            name_localizations: None,
                            options: None,
                            required: Some(true),
                        },
                    ],
                },
                version: Id::<CommandVersionMarker>::from(
                    NonZeroU64::new(1).ok_or(anyhow!("Couldn't create command"))?,
                ),
            };
            v.push(c);


        }

        Ok(v)
    }
}

#[async_trait::async_trait]
impl RunnableCommand for ReloadCommands {
    async fn run(
        _shard: u64,
        interaction: &InteractionCreate,
        _data: Box<CommandData>,
        context: &Context,
    ) -> anyhow::Result<anyhow::Result<()>> {
        let Some(author) = interaction.author_id() else {
            return Ok(Err(anyhow!("❌ You're probably not taka")));
        };

        if author.get() != 434626996262273038 {
            return Ok(Err(anyhow!("❌ You're definitely not taka")));
        }

        let interaction_client = context.http_client.interaction(context.application.id);

        interaction_client
            .set_guild_commands(
                context.test_guild.id,
                &Self::create_commands(&context).await?,
            )
            .await?;

        context
            .response_to_interaction(
                interaction,
                InteractionResponseDataBuilder::new()
                    .content("✅ Commands have successfully been reloaded")                    
                    .build(),
            )
            .await?;


        Ok(Ok(()))
    }
}
