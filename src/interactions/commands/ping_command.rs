use std::{sync::Arc, time::Instant};

use anyhow::anyhow;
use twilight_interactions::command::CreateCommand;
use twilight_model::{
    application::interaction::application_command::CommandData,
    gateway::payload::incoming::InteractionCreate,
};

use crate::{context::Context, utils::box_commands::RunnableCommand};

#[derive(CreateCommand)]
#[command(name = "ping", desc = "Get the current ping")]
pub struct PingCommand {}

#[async_trait::async_trait]
impl RunnableCommand for PingCommand {
    async fn run(
        _shard: u64,
        interaction: &InteractionCreate,
        _data: Box<CommandData>,
        context: Arc<Context>,
    ) -> anyhow::Result<anyhow::Result<()>> {
        let interaction_client = context.http_client.interaction(context.application.id);

        let message = if let Some(channel) = &interaction.channel {
            context
                .http_client
                .create_message(channel.id)
                .content("Pong!")?
                .await?
                .model()
                .await?
        } else {
            return Ok(Err(anyhow!("❌ Couldn't send message")));
        };

        let it = interaction_client
            .response(&interaction.token)
            .await?
            .model()
            .await?;

        let snowflake = interaction.id.get();
        let interaction_sent_at = (snowflake >> 22) + 1420070400000;
        let snowflake = it.id.get();
        let response_sent_at = (snowflake >> 22) + 1420070400000;
        let snowflake = message.id.get();
        let message_sent_at = (snowflake >> 22) + 1420070400000;
        let ping = response_sent_at - interaction_sent_at;
        let response_time = message_sent_at - response_sent_at;
        let total_time = message_sent_at - interaction_sent_at;

        let gateway_ping = {
            let start = Instant::now();
            let url = context.http_client.gateway().authed().await?;
            let duration = start.elapsed();
            log::debug!("{}", url.status());
            duration
        };

        interaction_client
            .update_response(&interaction.token)
            .content(Some(&format!("Pong!\nPing: {}ms\nResponse updated after: {}ms\nTotal Response time: {}ms\nGateway ping: {}ms", ping, response_time, total_time, gateway_ping.as_millis())))?
            .await?
            .model()
            .await?;

        Ok(Ok(()))
    }
}
