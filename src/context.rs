#[allow(unused_imports)]
use std::{time::Duration, future::IntoFuture};
#[cfg(feature = "ai")]
use chatgpt::prelude::ChatGPT;
#[cfg(feature = "html_server_image_generation")]
use headless_chrome::{Browser, LaunchOptions};
use tetrio_api::http::cached_client::CachedClient;
use tokio::sync::OnceCell;
use twilight_http::{Client, request::Request, routing::Route, response::marker::EmptyBody};
use twilight_model::{guild::Guild, oauth::Application, http::interaction::{InteractionResponse, InteractionResponseType, InteractionResponseData}, gateway::payload::incoming::InteractionCreate, id::{marker::InteractionMarker, Id}};

use crate::utils::box_commands::PhantomCommandTrait;

static UNAUTHED_CLIENT: OnceCell<Client> = OnceCell::const_new();

struct UnauthedClient;

impl UnauthedClient {
    pub async fn get() -> &'static Client  {
        return UNAUTHED_CLIENT.get_or_init(|| async {
            return Client::builder().build()
        }).await
    }
}


pub struct Context {
    pub http_client: Client,
    pub tetrio_client: CachedClient,
    pub application: Application,
    pub test_guild: Guild,
    pub local_server_url: String,
    pub commands: Vec<Box<dyn PhantomCommandTrait>>,
    #[cfg(feature = "database")]
    pub sql_connection: sqlx::postgres::PgPool,
    #[cfg(feature = "ai")]
    pub ai_channel: u64,
    #[cfg(feature = "ai")]
    pub openai_prompt: &'static str,
    #[cfg(feature = "ai")]
    pub chatgpt_client: ChatGPT
}


impl Context {

    pub async fn defer_response(&self, interaction: &InteractionCreate) -> Result<(), twilight_http::error::Error>  {
        self.defer_response_with(interaction.id, interaction.token.clone()).await
    }

    async fn __defer_response_with(id: Id<InteractionMarker>, token: String) -> Result<(), twilight_http::error::Error> {

        let interaction_response = InteractionResponse {
            kind: InteractionResponseType::DeferredChannelMessageWithSource,
            data: None,
        };

        let mut request = Request::builder(&Route::InteractionCallback {
            interaction_id: id.get(),
            interaction_token: &token,
        });

        request = request.use_authorization_token(false);
        
        request = request.json(&interaction_response)?;

        let request = request.build();
        
        UnauthedClient::get().await.request::<twilight_http::response::Response<EmptyBody>>(request).into_future().await.map(|_| ())
    }

    pub async fn defer_response_with(&self, id: Id<InteractionMarker>, token: String) -> Result<(), twilight_http::error::Error> {
        let interaction_client = self.http_client.interaction(self.application.id);

        let interaction_response = InteractionResponse {
            kind: InteractionResponseType::DeferredChannelMessageWithSource,
            data: None,
        };

        interaction_client
            .create_response(id, &token, &interaction_response)
            .await.map(|_| ())
    }

    pub fn threaded_defer_response(&self, interaction: &InteractionCreate)
        -> tokio::task::JoinHandle<Result<(), twilight_http::Error>>
     {

        let id = interaction.id;
        let token = interaction.token.clone();
        return tokio::spawn(
            Self::__defer_response_with(id, token)
        );
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
#[cfg(feature = "html_server_image_generation")]
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
