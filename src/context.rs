use std::fmt::Display;
#[allow(unused_imports)]
use std::{time::Duration, future::IntoFuture};
#[cfg(feature = "ai")]
use chatgpt::prelude::ChatGPT;
#[cfg(feature = "tetrio")]
use tetrio_api::{http::clients::reqwest_client::RedisReqwestClient, models::{packet::Packet, users::user_leaderboard::LeaderboardUser}};
use twilight_http::Client;
use twilight_model::{guild::Guild, oauth::Application, http::interaction::{InteractionResponse, InteractionResponseType, InteractionResponseData}, gateway::payload::incoming::InteractionCreate, id::{marker::InteractionMarker, Id}};

use crate::utils::box_commands::PhantomCommandTrait;



pub struct Context<'a> {
    pub http_client: Client,
    #[cfg(feature = "tetrio")]
    pub tetrio_client: RedisReqwestClient<'a>,
    pub application: Application,
    pub test_guild: Guild,
    pub local_server_url: String,
    pub api_url: String,

    pub commands: Vec<Box<dyn PhantomCommandTrait>>,
    pub author_id: u64,
    #[cfg(feature = "database")]
    pub sql_connection: sqlx::postgres::PgPool,
    #[cfg(feature = "ai")]
    pub ai_channel: u64,
    #[cfg(feature = "ai")]
    pub openai_prompt: &'static str,
    #[cfg(feature = "ai")]
    pub chatgpt_client: ChatGPT
}


impl Context<'_> {

    pub async fn defer_response(&self, interaction: &InteractionCreate) -> Result<(), twilight_http::error::Error>  {
        self.defer_response_with(interaction.id, interaction.token.clone()).await
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

    pub async fn fetch_full_leaderboard<S: Display>(&self, country: Option<S>) -> Result<Packet<Vec<LeaderboardUser>>, reqwest::Error> {
        let url = match country {
            Some(country) => format!("{}/api/v1/fetch_full_leaderboard?country={country}", self.api_url),
            None => format!("{}/api/v1/fetch_full_leaderboard", self.api_url),
        };
        reqwest::get(url).await?.json::<Packet<Vec<LeaderboardUser>>>().await           

    }
}

