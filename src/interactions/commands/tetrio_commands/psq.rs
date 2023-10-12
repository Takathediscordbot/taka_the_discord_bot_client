use std::borrow::Cow;
use std::sync::Arc;

use anyhow::anyhow;
use async_trait::async_trait;
use serde_json::json;
use twilight_interactions::command::{CommandInputData, CommandModel, CreateCommand};

use twilight_model::application::interaction::application_command::CommandData;
use twilight_model::gateway::payload::incoming::InteractionCreate;

use crate::context::Context;

use super::sq::SqCommand;
use crate::utils::box_commands::{CommandBox, RunnableCommand};
use crate::utils::stats::{calculate_stats, PlayerStats};

use crate::interactions::commands::models::graph_user_model::{
    AverageSubCommand, DiscordUserSubCommand, GraphUser, StatsSubCommand, TetrioUserSubCommand,
};

use crate::utils::timer::Timer;


#[derive(CreateCommand)]
#[command(name = "psq", desc = "Get a graph of the playerstyle")]
#[allow(unused)]
pub enum PsqCommand {
    #[command(name = "discord")]
    /// Fetch data from a discord user
    Discord(CommandBox<DiscordUserSubCommand>),
    #[command(name = "tetrio")]
    /// Fetch data from a tetrio user
    Tetrio(TetrioUserSubCommand),
    #[command(name = "stats")]
    /// Use tetrio stats
    Stats(StatsSubCommand),
    #[command(name = "average")]
    /// Use average stats
    Average(AverageSubCommand),
}

impl PsqCommand {
    async fn graph_with_stats(
        username: &str,
        dark_mode: bool,
        stats: PlayerStats,
    ) -> anyhow::Result<String> {
        let stats = calculate_stats(stats);
        let infds = stats.infds;
        let stride = stats.stride;
        let plonk = stats.plonk;
        let opener = stats.opener;

        let json = json!({
            "type": "radar",
            "data": {
                "labels": ["OPENER", "STRIDE", "INF DS", "PLONK"],
                "datasets": [{
                    "label": username,
                    "data": [
                        opener, stride, infds, plonk
                    ],
                    "backgroundColor": SqCommand::get_background_colors(dark_mode),
                    "borderColor": SqCommand::get_background_colors(dark_mode),
                    "borderWidth": 0,
                    "pointRadius": 0
                }]
            },
            "options":{"legend": { "labels": { "fontColor": SqCommand::get_font_color(dark_mode), "fontSize": 16}}, "scale":{"pointLabels":{"fontColor":SqCommand::get_font_color(dark_mode), "fontSize": 16},"rAxis":{"ticks":{"display":false}},"ticks":{"min":0,"max":1.2,"stepSize":"0.2","fontColor":"blue","display":false},"gridLines":{"color":"gray"},"angleLines":{"color":"gray"}}}
        });

        let json = json!({
            "width": 500,
            "height": 300,
            "format": "webp",
            "background": "transparent",
            "version": 2,
            "chart": json
        });

        log::debug!("{json}");

        let response = reqwest::Client::builder()
            .build()?
            .post("https://quickchart.io/chart/create")
            .header("Content-Type", "application/json")
            .body(json.to_string())
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;
        Ok(response
            .get("url")
            .ok_or(anyhow!("Expected a string for the url of the chart"))?
            .as_str()
            .ok_or(anyhow!("Expected a string"))?
            .to_string())
    }
}

#[async_trait]
impl RunnableCommand for PsqCommand {
    async fn run(
        _shard: u64,
        interaction: &InteractionCreate,
        data: Box<CommandData>,
        context: Arc<Context>,
    ) -> anyhow::Result<anyhow::Result<()>> {

        log::info!("psq command");
        let _command_timer = Timer::new("psq command");
        let thread = Context::threaded_defer_response(Arc::clone(&context), interaction);

        let model = GraphUser::from_interaction(CommandInputData {
            options: data.options,
            resolved: data.resolved.map(Cow::Owned),
        })?;

        let data = model.get_data(Arc::clone(&context)).await?;

        let data = match data {
            Ok(data) => data,
            Err(e) => return Ok(Err(e)),
        };

        let replay_str = if let Some(url) = data.replay_url {
            let round_str = if let Some(round) = data.round {
                format!("Round {}", round)
            } else {
                String::from("Average stats")
            };

            format!("Tetra league game: <{url}>\nStats from: {round_str}")
        } else {
            String::new()
        };

        let url = Self::graph_with_stats(&data.name, data.dark_mode, data.stats).await?;

        let content = format!("{replay_str}\n{url}");
        thread.await??;
        context
            .http_client
            .interaction(context.application.id)
            .update_response(&interaction.token)
            .content(Some(&content))?
            .await?;

        Ok(Ok(()))
    }
}
