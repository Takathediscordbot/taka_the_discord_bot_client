mod context;
pub mod events;
mod interactions;
pub mod models;
pub mod utils;
pub mod services;
use anyhow::anyhow;
use axum::{Router, response::IntoResponse};
use context::Context;
use events::handle_event;
use flexi_logger::{Logger, FileSpec, WriteMode, TS_DASHES_BLANK_COLONS_DOT_BLANK, DeferredNow};
use futures::StreamExt;
use itertools::Itertools;
use log::Record;
#[cfg(feature = "database")]
use sqlx::postgres::PgPoolOptions;
#[allow(unused_imports)]
use std::{str::FromStr, sync::Arc, time::Duration};
use tower_http::cors::CorsLayer;
#[cfg(feature = "ai")]
use chatgpt::prelude::{ChatGPT, ChatGPTEngine, ModelConfigurationBuilder};
use twilight_gateway::{
    error::ReceiveMessageError,
    stream::{ShardEventStream, ShardRef},
    Event, Intents, Shard, ShardId,
};



use twilight_model::id::Id;





use crate::interactions::commands::get_commands;
async fn run_bot() -> anyhow::Result<anyhow::Result<()>> {
    let tetrio_client = tetrio_api::http::cached_client::CachedClient::default();
        
        // let name = uuid::Uuid::new_v4();

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
        println!("creating browser");
        #[cfg(feature = "html_server_image_generation")]
        context::create_browser()
            .await?;

        #[cfg(feature = "ai")]
        let openai_prompt = include_str!("./assets/prompt");
        #[cfg(feature = "ai")]
        let openai_token =
            &std::env::var("OPENAI_TOKEN")?;        

 
        #[cfg(feature = "ai")]
        let chatgpt = ChatGPT::new_with_config(openai_token, 
            (&mut ModelConfigurationBuilder::default())
                .engine(ChatGPTEngine::Gpt35Turbo_0301)
                .timeout(Duration::from_secs(600))
                .build()?
        )?;

    #[cfg(feature = "database")]

    let sql_connection_url =
        &std::env::var("DATABASE_URL")?;
    #[cfg(feature = "database")]
    let sql_connection = PgPoolOptions::new()
        .max_connections(25)
        .connect(sql_connection_url)
        .await?;
    #[cfg(feature = "database")]
    let row: (i64,) = sqlx::query_as("SELECT $1")
        .bind(150_i64)
        .fetch_one(&sql_connection)
        .await?;


        #[cfg(feature = "ai")]
        let ai_channel: u64 = std::env::var("AI_CHANNEL")?.parse()?;
    

        let context = Arc::new(Context {
            application: discord_application,
            http_client: discord_client,
            tetrio_client,
            test_guild,
            local_server_url: std::env::var("HTML_SERVER_URL")?,
            #[cfg(feature = "database")]
            sql_connection,
            commands: get_commands(),
            author_id: std::env::var("AUTHOR_ID")?.parse()?,
            #[cfg(feature = "ai")]
            openai_prompt,
            #[cfg(feature = "ai")]
            chatgpt_client: chatgpt,
            #[cfg(feature = "ai")]
            ai_channel
        });


    #[cfg(feature = "database")]
    log::info!("{row:?}; SQL database initialized!");

        println!("Hello World!");

        tokio::spawn(async {
            let ip_bind = std::env::var("BIND_URL").unwrap_or("0.0.0.0:8080".to_string());
            println!("{ip_bind}");
            let origins = [
                "https://health.takathedinosaur.tech/".parse().unwrap()
            ];
            let cors = CorsLayer::new()
                // allow `GET` and `POST` when accessing the resource
                .allow_methods([reqwest::Method::GET])
                // allow requests from any origin
                .allow_origin(origins);
            // build our application with a route
            let app = Router::new()
                // `GET /` goes to `root`
                .route("/health", axum::routing::get(health_status))     
                .route("/logs", axum::routing::get(logs))            
                .layer(cors);
        
            // run our app with hyper
            let _ = axum::Server::bind(&ip_bind.parse().unwrap())
                .serve(app.into_make_service())
                .await;
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
            let context = Arc::clone(&context);
            tokio::spawn(async move {
                match handle_event(id, event, context).await {
                    Ok(_) => {},
                    Err(e) => log::error!("{e:?}")
                };


            });
        };

        Ok(Ok(()))
}

async fn health_status() -> impl IntoResponse {
    "OK"
}


async fn logs() -> impl IntoResponse {
    let last_modified_file = std::fs::read_dir("./logs")
    .expect("Couldn't access local directory")
    .flatten() // Remove failed
    .filter(|f| f.metadata().unwrap().is_file()) // Filter out directories (only consider files)
    .max_by_key(|x| x.metadata().unwrap().modified().unwrap()).unwrap(); // Get the most recently modified file

    let value = std::fs::read_to_string(format!("{}", last_modified_file.path().to_str().unwrap())).unwrap();
    let return_val: String = value.lines().map(|c| c.to_string()).join("\n");
    return_val
}



pub fn my_own_format(
    w: &mut dyn std::io::Write,
    now: &mut DeferredNow,
    record: &Record,
) -> Result<(), std::io::Error> {
    write!(
        w,
        "{} [Thread {}] Severity {}, Message: {}",
        now.format(TS_DASHES_BLANK_COLONS_DOT_BLANK),
        std::thread::current().name().unwrap_or("<unnamed>"),
        record.level(),
        &record.args()
    )
}

#[tokio::main]
async fn main() -> ! {
    dotenvy::dotenv().expect("Couldn't find env vars");
    let _logger = Logger::try_with_str("warn, taka_the_discord_bot=info").expect("Couldn't initialize logger")
    .log_to_file(FileSpec::default().directory("./logs"))
    .write_mode(WriteMode::BufferAndFlush)
    .format(my_own_format)
    .start().expect("Couldn't start logger");

    
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
