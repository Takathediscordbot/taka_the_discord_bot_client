use std::time::Instant;

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
        context: &Context<'_>,
    ) -> anyhow::Result<anyhow::Result<()>> {
        let interaction_client = context.http_client.interaction(context.application.id);

        context
                .response_to_interaction_with_content(interaction, "Pong!").await?;
        
        let it = interaction_client
            .response(&interaction.token)
            .await?
            .model()
            .await?;

        let snowflake = interaction.id.get();
        let interaction_sent_at = (snowflake >> 22) + 1420070400000;
        let snowflake = it.id.get();
        let response_sent_at = (snowflake >> 22) + 1420070400000;
        let ping = response_sent_at - interaction_sent_at;

        let gateway_ping = {
            let start = Instant::now();
            let url = context.http_client.gateway().await?;
            let duration = start.elapsed();
            log::debug!("{}", url.status());
            duration
        };

        interaction_client
            .update_response(&interaction.token)
            .content(Some(&format!("Pong!\nPing: {}ms\nGateway ping: {}ms", ping, gateway_ping.as_millis())))?
            .await?;

        Ok(Ok(()))
    }
}
