use std::borrow::Cow;

use anyhow::anyhow;

use common::LeagueRecord;
use serde_json::json;
use tetrio_api::models::packet::Packet;
use twilight_interactions::command::{CommandInputData, CommandModel, CreateCommand};

use twilight_model::application::interaction::application_command::CommandData;
use twilight_model::{gateway::payload::incoming::InteractionCreate, http::attachment::Attachment};

use crate::context::Context;
use crate::interactions::commands::subcommands::tetra::ttrm_replay_sub_command::TetrioReplaySubCommand;
use crate::utils::box_commands::{CommandBox, RunnableCommand};

use crate::interactions::commands::subcommands::tetra::discord_user_sub_command::DiscordUserSubCommand;
use crate::interactions::commands::subcommands::tetra::tetrio_user_sub_command::TetrioUserSubCommand;
use crate::utils::timer::Timer;

use serde::Deserialize;

#[derive(Deserialize)]
struct TetraData {
    replay_id: Option<String>, 
    buffer: Vec<u8>
}

#[derive(CreateCommand, CommandModel)]
#[command(name = "tetra", desc = "Fetch tetra league data for a user")]
pub enum TetraCommand {
    #[command(name = "discord")]
    /// Fetch data from a discord user
    Discord(CommandBox<DiscordUserSubCommand>),
    #[command(name = "tetrio")]
    /// Fetch data from a tetrio user
    Tetrio(TetrioUserSubCommand),
    #[command(name = "replay")]
    /// Fetch data from a ttrm replay
    Replay(TetrioReplaySubCommand),
}

#[async_trait::async_trait]
impl RunnableCommand for TetraCommand {
    async fn run(
        _shard: u64,
        interaction: &InteractionCreate,
        data: Box<CommandData>,
        context: &Context<'_>,
    ) -> anyhow::Result<anyhow::Result<()>> {
        log::info!("tetra command");
        let _command_timer = Timer::new("tetra command");
        Context::defer_response(&context, interaction).await?;

        let model = Self::from_interaction(CommandInputData {
            options: data.options,
            resolved: data.resolved.map(Cow::Owned),
        })?;
        let buffer = {
            let _timer = Timer::new("tetra command parsing input & fetching user");
            match model {
                TetraCommand::Discord(discord) => {
                    let packet = context
                        .tetrio_client
                        .search_discord_user(&discord.user.resolved.id.to_string())
                        .await?;

                    let Some(data) = &packet.data else {
                        return Ok(Err(anyhow!("❌ Couldn't find your tetrio id from the discord account, they might have not linked it publicly to their tetrio profile")));
                    };

                    let (id, game_num) = (data.user.id.clone(), discord.game_number.unwrap_or(1));

                    reqwest::get(format!("{}/api/v1/tetra?user_id={id}&game_num={game_num}", context.api_url)).await?.json::<Packet<TetraData>>().await?
                }
                TetraCommand::Tetrio(tetrio) => {
                    let packet = context
                        .tetrio_client
                        .fetch_user_info(&tetrio.tetrio_user.to_lowercase())
                        .await?;

                    let Some(data) = &packet.data else {
                        return Ok(Err(anyhow!("❌ Couldn't find tetrio user")));
                    };

                    let (id, game_num) = (data.id.clone(), tetrio.game_number.unwrap_or(1));

                    reqwest::get(format!("{}/api/v1/tetra?user_id={id}&game_num={game_num}", context.api_url)).await?.json::<Packet<TetraData>>().await?            
                },
                TetraCommand::Replay(replay) => {
                    
                    let attachment = &replay.replay;
                    // check that extension is ttrm
                    if !attachment.filename.ends_with("ttrm") {
                        return Ok(Err(anyhow!("❌ File type not supported: {:?}", attachment.filename)));
                    };

                    let bytes = reqwest::get(&attachment.url)
                        .await?
                        .bytes()
                        .await?;

                    let replay_data:common::replay::ttrm::models::Root = serde_json::from_slice(&bytes)
                    .map_err(|err| anyhow!("❌ Couldn't parse ttrm replay: {:?}", err))?;


                    let ts = replay_data.ts.clone();
                    let league_record = replay_data.into_league_record();
                    let league_record:LeagueRecord = match league_record {
                        Ok(league_record) => league_record,
                        Err(err) => {
                            return Ok(Err(anyhow!("❌ Couldn't parse ttrm replay: {:?}", err)));
                        }
                    };

                    if league_record.rounds.len() > 14 {
                        return Ok(Err(anyhow!("❌ Replay has more than 14 rounds")));
                    }
                    let reqwest_client = reqwest::Client::new();

                    reqwest_client.post(format!("{}/api/v1/tetra/replay", context.api_url))
                    .json(&json!{
                        {
                            "ts": ts,
                            "league_record": league_record
                        }
                    })
                    .send()
                    .await?
                    .json()
                    .await?
                }
            }
        };



        match buffer {
            Packet { success: true, data: Some(TetraData { replay_id: Some(replay_id), buffer }), .. } => {
                context
                .http_client
                .interaction(context.application.id)
                .update_response(&interaction.token)
                .content(Some(&format!(
                    "Replay url: <https://tetr.io/#r:{}>",
                    replay_id
                )))?
                .attachments(&[Attachment::from_bytes("tetra.png".to_string(), buffer, 1)])?
                .await?;
            },
            Packet { success: true, data: Some(TetraData { replay_id: _replay_id, buffer }), .. } => {
                context
                .http_client
                .interaction(context.application.id)
                .update_response(&interaction.token)
                .attachments(&[Attachment::from_bytes("tetra.png".to_string(), buffer, 1)])?
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
