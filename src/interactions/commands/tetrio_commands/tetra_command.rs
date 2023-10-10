use std::borrow::Cow;

use std::sync::Arc;

use anyhow::anyhow;
use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption;
use headless_chrome::LaunchOptions;

use twilight_interactions::command::{CommandInputData, CommandModel, CreateCommand};

use twilight_model::application::interaction::application_command::CommandData;
use twilight_model::{gateway::payload::incoming::InteractionCreate, http::attachment::Attachment};

use crate::context::Context;
use crate::utils::box_commands::{CommandBox, RunnableCommand};

use crate::interactions::commands::subcommands::tetra::discord_user_sub_command::DiscordUserSubCommand;
use crate::interactions::commands::subcommands::tetra::tetrio_user_sub_command::TetrioUserSubCommand;
use crate::utils::timer::Timer;

#[derive(CreateCommand, CommandModel)]
#[command(name = "tetra", desc = "Fetch tetra league data for a user")]
pub enum TetraCommand {
    #[command(name = "discord")]
    /// Fetch data from a discord user
    Discord(CommandBox<DiscordUserSubCommand>),
    #[command(name = "tetrio")]
    /// Fetch data from a tetrio user
    Tetrio(TetrioUserSubCommand),
}

#[async_trait::async_trait]
impl RunnableCommand for TetraCommand {
    async fn run(
        _shard: u64,
        interaction: &InteractionCreate,
        data: Box<CommandData>,
        context: Arc<Context>,
    ) -> anyhow::Result<anyhow::Result<()>> {
        log::info!("tetra command");
        let _command_timer = Timer::new("tetra command");

        let model = Self::from_interaction(CommandInputData {
            options: data.options,
            resolved: data.resolved.map(Cow::Owned),
        })?;
        let (id, game_num) = {
            let _timer = Timer::new("tetra command parsing input & fetching user");
            let (id, game_num) = match model {
                TetraCommand::Discord(discord) => {
                    let packet = context
                        .tetrio_client
                        .search_user(&discord.user.resolved.id.to_string())
                        .await?;

                    let Some(data) = &packet.data else {
                        return Ok(Err(anyhow!("❌ Couldn't find your tetrio id from the discord account, they might have not linked it publicly to their tetrio profile")));
                    };

                    (data.user.id.clone(), discord.game_number.unwrap_or(1))
                }
                TetraCommand::Tetrio(tetrio) => {
                    let packet = context
                        .tetrio_client
                        .fetch_user_info(&tetrio.tetrio_user.to_lowercase())
                        .await?;

                    let Some(data) = &packet.data else {
                        return Ok(Err(anyhow!("❌ Couldn't find tetrio user")));
                    };

                    (data.user.id.clone(), tetrio.game_number.unwrap_or(1))
                }
            };
            (id, game_num)
        };

        let packet = tetrio_api::http::client::fetch_tetra_league_recent(&id).await?;

        let Some(data) = &packet.data else {
            return Ok(Err(anyhow!("❌ Couldn't find tetra league records")));
        };

        let game_num = if game_num <= 0 { 1 } else { game_num };

        let Some(record) = data.records.get((game_num - 1) as usize) else {
            return Ok(Err(anyhow!("❌ Couldn't find tetra league game")));
        };

        let (Some(left), Some(_right)) = (record.endcontext.get(0), record.endcontext.get(1))
        else {
            return Ok(Err(anyhow!("❌ Couldn't find tetra league game")));
        };

        let buffer = {
            let _timer = Timer::new("tetra command taking screenshot");
            let launch_options = LaunchOptions::default_builder()
                .headless(true)
                // .fetcher_options(FetcherOptions::default().with_revision(browser_version))
                .window_size(Some((
                    1185,
                    350 + 60 * (left.points.secondary_avg_tracking.len() - 1) as u32,
                )))
                .sandbox(false)
                .build()?;

            log::debug!("made configuration");

            let browser = headless_chrome::Browser::new(launch_options)?;
            log::debug!("launched browser");
            let tab = browser.new_tab()?;
            log::debug!("opened tab");

            tab.navigate_to(&format!(
                "{}/league_replay?user_id={}&replay_id={}",
                context.local_server_url, id, record.replay_id
            ))?;
            log::debug!("navigated to tab");

            let _element = tab.wait_for_element("#multilog")?;
            log::debug!("waited for element");
            let buffer =
                tab.capture_screenshot(CaptureScreenshotFormatOption::Png, None, None, true)?;
            log::debug!("took screenshot");

            tab.close(true)?;
            buffer
        };

        context
            .http_client
            .interaction(context.application.id)
            .update_response(&interaction.token)
            .content(Some(&format!(
                "Replay url: <https://tetr.io/#r:{}>",
                record.replay_id
            )))?
            .attachments(&[Attachment::from_bytes("tetra.png".to_string(), buffer, 1)])?
            .await?;

        Ok(Ok(()))
    }
}
