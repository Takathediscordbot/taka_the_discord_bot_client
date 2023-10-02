use std::{borrow::Cow, sync::Arc};
use rand::prelude::*;

use twilight_interactions::command::{CommandInputData, CommandModel, CreateCommand};
use twilight_model::{
    application::interaction::application_command::CommandData,
    gateway::payload::incoming::InteractionCreate,
};

use crate::{
    context::Context, utils::box_commands::RunnableCommand,
};

#[derive(CreateCommand, CommandModel)]
#[command(name = "random", desc = "Get a random number")]
pub struct RngCommand {
    /// The minimum value
    #[command(min_value = 0)]
    min: Option<i64>,
    /// The maximum value
    #[command(min_value = 0)]
    max: Option<i64>
}

#[async_trait::async_trait]
impl RunnableCommand for RngCommand {
    async fn run(
        _shard: u64,
        interaction: &InteractionCreate,
        data: Box<CommandData>,
        context: Arc<Context>,
    ) -> anyhow::Result<anyhow::Result<()>> {
        let model = Self::from_interaction(CommandInputData {
            options: data.options,
            resolved: data.resolved.map(Cow::Owned),
        })?;


        let min = model.min.unwrap_or(0);
        let max = model.max.unwrap_or(i64::MAX);

        let number = rand::thread_rng().gen_range(min..max);
        let interaction_client = context.http_client.interaction(context.application.id);
        interaction_client
            .update_response(&interaction.token)
            .content(Some(&number.to_string()))?
            .await?;

        Ok(Ok(()))
    }
}
