use std::{borrow::Cow, sync::Arc};

use anyhow::anyhow;
use itertools::Itertools;

use twilight_interactions::command::{CommandInputData, CommandModel, CreateCommand};
use twilight_model::{
    application::interaction::application_command::CommandData,
    gateway::payload::incoming::InteractionCreate,
};

use crate::{
    context::Context,
    interactions::commands::options::{
        user_rank_option::UserRankOption, user_stat_options::UserStatOption,
    },
    utils::box_commands::RunnableCommand,
};

use super::lb::LbCommand;

#[derive(CreateCommand, CommandModel)]
#[command(name = "rlb", desc = "Get the reverse leaderboard")]
pub struct RLbCommand {
    /// The type of stat to use for the leaderboard,
    leaderboard_stat: UserStatOption,

    #[command(max_value = 50)]
    /// How many places to display
    limit: i64,

    /// Where to start searching for players
    position: Option<i64>,

    /// Only played in this rank will be displayed
    rank: Option<UserRankOption>,

    /// country to limit the placements from
    country_code: Option<String>,
}

#[async_trait::async_trait]
impl RunnableCommand for RLbCommand {
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

        let leaderboard = context
            .tetrio_client
            .fetch_full_league_leaderboard(model.country_code)
            .await?;

        let Some(data) = &leaderboard.data else {
            return Ok(Err(anyhow!("Couldn't fetch leaderboard data because {}", leaderboard.error.clone().unwrap_or("Unknwon error".to_string()))));
        };

        let iter = LbCommand::filter_rank(data, &model.rank);

        let v = LbCommand::get_stats(&model.leaderboard_stat, iter)
            .into_iter()
            .sorted_by(|(_, _, a), (_, _, b)| a.total_cmp(b));

        let skip = model
            .position
            .map(|position| (position as usize).saturating_sub(model.limit as usize))
            .unwrap_or(0);

        let content = v
            .skip(skip)
            .take(model.limit as usize)
            .enumerate()
            .map(|(index, data)| {
                format!(
                    "#{}: {} (Rank: #{}): ({:.4}) ",
                    index + 1,
                    data.1,
                    data.0 + 1,
                    data.2
                )
            })
            .join("\n");
        let content = format!("```\n{content}\n```");

        let interaction_client = context.http_client.interaction(context.application.id);
        let r = interaction_client
            .update_response(&interaction.token)
            .content(Some(&content));
        match r {
            Ok(response) => response.await?,
            Err(_) => {
                return Ok(Err(anyhow!(
                    "❌ Message was too long, try lowering the limit!"
                )));
            }
        };

        Ok(Ok(()))
    }
}
