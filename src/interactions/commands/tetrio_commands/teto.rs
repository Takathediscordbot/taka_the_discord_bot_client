use std::borrow::Cow;

use anyhow::anyhow;
use tetrio_api::models::packet::Packet;
use twilight_interactions::command::{CommandInputData, CommandModel, CreateCommand};
use twilight_model::{
    application::interaction::application_command::CommandData,
    gateway::payload::incoming::InteractionCreate, http::attachment::Attachment,
};

use crate::{
    context::Context,
    utils::{
        box_commands::{CommandBox, RunnableCommand},
        timer::Timer,
    },
};

use crate::interactions::commands::subcommands::teto::{
    discord_user_sub_command::DiscordUserSubCommand, tetrio_user_sub_command::TetrioUserSubCommand,
};

#[derive(CreateCommand, CommandModel)]
#[command(name = "teto", desc = "Fetch the tetrio profile")]
pub enum TetoCommand {
    #[command(name = "discord")]
    /// Fetch data from a discord user
    Discord(CommandBox<DiscordUserSubCommand>),
    #[command(name = "tetrio")]
    /// Fetch data from a tetrio user
    Tetrio(TetrioUserSubCommand),
}

#[async_trait::async_trait]
impl RunnableCommand for TetoCommand {
    async fn run(
        _shard: u64,
        interaction: &InteractionCreate,
        data: Box<CommandData>,
        context: &Context<'_>,
    ) -> anyhow::Result<anyhow::Result<()>> {
        log::info!("teto command");
        let _command_timer = Timer::new("teto command");
        context.defer_response(interaction).await?;
        let model = Self::from_interaction(CommandInputData {
            options: data.options,
            resolved: data.resolved.map(Cow::Owned),
        })?;

        let username = {
            let _timer = Timer::new("teto fetching username & parsing input");
          
            let username = match &model {
                TetoCommand::Discord(discord) => {
                    let packet = context
                        .tetrio_client
                        .search_discord_user(&discord.user.resolved.id.to_string())
                        .await?;

                    let Some(data) = &packet.data else {
                        return Ok(Err(anyhow!("❌ Couldn't find your tetrio id from the discord account, they might have not linked it publicly to their tetrio profile")));
                    };

                    data.user.username.clone()
                }
                TetoCommand::Tetrio(tetrio) => {
                    tetrio.tetrio_user.clone().into()
                }
            };
            username
        };

        context.tetrio_client.fetch_user_summaries(&username).await?;

        let buffer = 
            reqwest::get(format!("{}/api/v1/teto/{username}", context.api_url)).await?.json::<Packet<Box<[u8]>>>().await?;

        match buffer {
            Packet { success: true, data: Some(data), .. } => {
                context
                .http_client
                .interaction(context.application.id)
                .update_response(&interaction.token)
                .content(Some(&format!(
                    "Profile link: <https://ch.tetr.io/u/{}>",
                    username
                )))?
                .attachments(&[Attachment::from_bytes("tetra.png".to_string(), data.to_vec(), 1)])?
                .await?;
            }
            Packet { error: Some(error), .. } => {
                return Ok(Err(anyhow!("❌ {}", error.msg)));
            }
            _ => {
                return Ok(Err(anyhow!("❌ Unknown error!")));

            }
        }



        Ok(Ok(()))
    }
}
