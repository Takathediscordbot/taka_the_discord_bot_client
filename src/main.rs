mod context;
pub mod events;
mod interactions;
pub mod models;
pub mod utils;
pub mod services;
use anyhow::anyhow;
use context::Context;
use events::handle_event;
use flexi_logger::{Logger, FileSpec, WriteMode};
use futures::StreamExt;
use sqlx::postgres::PgPoolOptions;
use twilight_http::Client;
use std::{str::FromStr, sync::Arc};

use tokio::sync::Mutex;
use twilight_gateway::{
    error::ReceiveMessageError,
    stream::{ShardEventStream, ShardRef},
    Event, Intents, Shard, ShardId,
};

use twilight_model::id::Id;
async fn run_bot() -> anyhow::Result<anyhow::Result<()>> {
    let tetrio_client = tetrio_api::http::cached_client::CachedClient::default();
        
        // let name = uuid::Uuid::new_v4();
        let _logger = Logger::try_with_str("warn, taka_the_discord_bot=info")?
        .log_to_file(FileSpec::default().directory("./logs"))
        .write_mode(WriteMode::BufferAndFlush)
        .start()?;

        let (discord_client, mut shards) = {
            let token = std::env::var("DISCORD_TOKEN")?;
            let http_client = twilight_http::Client::new(token.clone());
            
            let result = http_client.current_user().await.map(|c| async {
                c.model().await.map(|model| log::info!("Logged in as {}#{}", model.name,model.discriminator()))
            });

            match result {
                Ok(result) => {
                    let _ = result.await;
                },
                Err(err) => {
                    log::warn!("{}", err);
                }
            };
            
            let bot_connection_info = http_client
                .gateway()
                .authed()
                .await?
                .model()
                .await?;

            let shards: Vec<Shard> = (0..(bot_connection_info.shards))
                .map(|u| {
                    Shard::new(
                        ShardId::new(u, bot_connection_info.shards),
                        token.clone(),
                        Intents::all(),
                    )
                })
                .collect();
            (http_client, shards)
        };

        log::info!("Got number of shards required: {}", shards.len());

        let mut events = ShardEventStream::new(shards.iter_mut());

        let discord_application = discord_client
            .current_user_application()
            .await?
            .model()
            .await?;



        let test_guild = discord_client
            .guild(Id::from_str(&std::env::var("DISCORD_TEST_GUILD")?)?)
            .await?
            .model()
            .await?;
        // let discord_interaction_client = Arc::new(discord_interaction_client);
        // let _tetrio_bot_password =
        //     std::env::var("TETRIO_BOT_PASSWORD")?;
        // let _tetrio_bot_username =
        //     std::env::var("TETRIO_BOT_USERNAME")?;

        context::create_browser()
            .await?;

        let sql_connection_url =
            &std::env::var("DATABASE_URL")?;
        let sql_connection = PgPoolOptions::new()
            .max_connections(25)
            .connect(sql_connection_url)
            .await?;

        let row: (i64,) = sqlx::query_as("SELECT $1")
            .bind(150_i64)
            .fetch_one(&sql_connection)
            .await?;

        log::info!("{row:?}; SQL database initialized!");

        let context = Arc::new(Context {
            application: discord_application,
            http_client: discord_client,
            tetrio_client,
            test_guild,
            local_server_url: std::env::var("HTML_SERVER_URL")?,
            tetrio_token: std::env::var("TETRIO_TOKEN")?,
            test_mode: Mutex::new(false),
            sql_connection,
        });

        while let Some(data) = events.next().await {
            let (shard, event): (ShardRef<'_>, Result<Event, ReceiveMessageError>) = data;
            let id = shard.id().number();
            
            let event = match event {
                Ok(v) => v,
                Err(e) => {
                    if e.is_fatal() {
                        return Ok(Err(anyhow!(e)));
                    };
                    continue;
                }
            };
            tokio::spawn(handle_event(id, event, Arc::clone(&context)));
        };

        Ok(Ok(()))
}


#[tokio::main]
async fn main() -> ! {
    dotenvy::dotenv().expect("Couldn't find env vars");
    loop {
        match run_bot().await {
            Ok(v) => {
                match v {
                    Ok(_) => {},
                    Err(e) => {
                        log::error!("Bot crashed because {}", e);
                    }
                }
            }
            Err(e) => {
                log::error!("Error occured while initializing bot! {e}");
                panic!("Couldn't initilize bot");
            }
        }
    }
}
