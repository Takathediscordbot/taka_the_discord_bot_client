use std::borrow::Cow;

use anyhow::anyhow;
use headless_chrome::{protocol::cdp::Page::CaptureScreenshotFormatOption, LaunchOptions};
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
        context: &Context,
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
                        .search_user(&discord.user.resolved.id.to_string())
                        .await?;

                    let Some(data) = &packet.data else {
                        return Ok(Err(anyhow!("❌ Couldn't find your tetrio id from the discord account, they might have not linked it publicly to their tetrio profile")));
                    };

                    data.user.username.clone()
                }
                TetoCommand::Tetrio(tetrio) => {
                    let packet = context
                        .tetrio_client
                        .fetch_user_info(&tetrio.tetrio_user.to_lowercase())
                        .await?;

                    let Some(data) = &packet.data else {
                        return Ok(Err(anyhow!("❌ Couldn't find tetrio user")));
                    };

                    data.user.username.clone()
                }
            };
            username
        };

        let buffer = {
            let _timer = Timer::new("teto taking screenshot");
            let launch_options = LaunchOptions::default_builder()
            .headless(true)
            // .fetcher_options(FetcherOptions::default().with_revision(browser_version))
            .window_size(Some((
                900,
                500,
            )))
            .sandbox(false)
            .build()?;


            log::debug!("made configuration");

            let browser = headless_chrome::Browser::new(launch_options)?;

            let tab = browser.new_tab()?;

            tab.set_transparent_background_color()?;

            tab.navigate_to(&format!(
                "{}/teto_test/{}",
                context.local_server_url,
                username.to_lowercase()
            ))?;
            log::debug!("navigated to tab");

            let element = tab.wait_for_element(".tetra_modal")?;
            log::debug!("waited for element");

            let viewport = element.get_box_model()?;
            let mut viewport = viewport.border_viewport();
            viewport.x -= 16.0;
            viewport.y -= 16.0;
            viewport.width += 32.0;
            viewport.height += 32.0;

            let buffer = tab.capture_screenshot(
                CaptureScreenshotFormatOption::Png,
                None,
                Some(viewport),
                true,
            )?;
            log::debug!("took screenshot");
            buffer
        };

        context
            .http_client
            .interaction(context.application.id)
            .update_response(&interaction.token)
            .content(Some(&format!(
                "Profile link: <https://ch.tetr.io/u/{}>",
                username
            )))?
            .attachments(&[Attachment::from_bytes("tetra.png".to_string(), buffer, 1)])?
            .await?;

        Ok(Ok(()))
    }
}
