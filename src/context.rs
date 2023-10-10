use std::time::Duration;

use headless_chrome::{Browser, LaunchOptions};
use tetrio_api::http::cached_client::CachedClient;
use tokio::sync::Mutex;
use twilight_http::Client;
use twilight_model::{guild::Guild, oauth::Application, http::interaction::{InteractionResponse, InteractionResponseType, InteractionResponseData}, gateway::payload::incoming::InteractionCreate};

use crate::utils::box_commands::PhantomCommandTrait;

pub struct Context {
    pub http_client: Client,
    pub tetrio_client: CachedClient,
    pub application: Application,
    pub test_guild: Guild,
    pub local_server_url: String,
    pub tetrio_token: String,
    pub test_mode: Mutex<bool>,
    pub sql_connection: sqlx::postgres::PgPool,
    pub commands: Vec<Box<dyn PhantomCommandTrait>>
}



impl Context {

    pub async fn defer_response(&self, interaction: &InteractionCreate) -> Result<(), twilight_http::error::Error>  {
        let interaction_response = InteractionResponse {
            kind: InteractionResponseType::DeferredChannelMessageWithSource,
            data: None,
        };
    
        let interaction_client = self.http_client.interaction(self.application.id);
        interaction_client
            .create_response(interaction.id, &interaction.token, &interaction_response)
            .await.map(|_| ())
    }

    pub async fn response_to_interaction(&self, interaction:&InteractionCreate, content: InteractionResponseData) -> Result<(), twilight_http::error::Error> {
        let interaction_client = self.http_client.interaction(self.application.id);
        let response = InteractionResponse {
            data: Some(content),
            kind:
                twilight_model::http::interaction::InteractionResponseType::ChannelMessageWithSource,
        };
        return interaction_client
            .create_response(interaction.id, &interaction.token, &response)
            .await.map(|_| ());
    }

    pub async fn response_to_interaction_with_content(&self, interaction: &InteractionCreate, content: &str) -> Result<(), twilight_http::error::Error> {
        let response = twilight_model::http::interaction::InteractionResponseData {
                allowed_mentions: None,
                attachments: None,
                choices: None,
                components: None,
                content: Some(content.to_string()),
                custom_id: None,
                embeds: None,
                flags: None,
                title: None,
                tts: None,
            };
        return self.response_to_interaction(interaction, response).await;
    }
}

pub async fn create_browser() -> anyhow::Result<Browser> {
    let _browser_version = if cfg!(windows) { "830237" } else { "830288" };

    let launch_options = LaunchOptions::default_builder()
        .headless(true)
        // .path(Some("/var/www/taka_the_discord_bot/headless-chrome/chrome-linux/chrome".into()))
        // // .fetcher_options(FetcherOptions::default().with_revision(browser_version))
        .window_size(Some((1440, 1440)))
        .sandbox(false)
        .idle_browser_timeout(Duration::from_secs(15))
        .build()?;

    log::debug!("made configuration");

    let browser = headless_chrome::Browser::new(launch_options)?;
    log::debug!("launched browser");

    
    Ok(browser)
}
