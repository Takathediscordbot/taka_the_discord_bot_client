use std::time::Duration;

use headless_chrome::{Browser, LaunchOptions};
use tetrio_api::http::cached_client::CachedClient;
use tokio::sync::Mutex;
use twilight_http::Client;
use twilight_model::{guild::Guild, oauth::Application};

pub struct Context {
    pub http_client: Client,
    pub tetrio_client: CachedClient,
    pub application: Application,
    pub test_guild: Guild,
    pub local_server_url: String,
    pub tetrio_token: String,
    pub test_mode: Mutex<bool>,
    pub sql_connection: sqlx::postgres::PgPool,
}

impl Context {}

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
