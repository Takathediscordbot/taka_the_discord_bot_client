use std::borrow::Cow;
use std::sync::Arc;

use anyhow::anyhow;
use async_trait::async_trait;
use serde_json::json;
use twilight_interactions::command::{CommandInputData, CommandModel, CreateCommand};

use twilight_model::application::interaction::application_command::CommandData;
use twilight_model::gateway::payload::incoming::InteractionCreate;

use crate::context::Context;

use crate::utils::box_commands::{CommandBox, RunnableCommand};
use crate::utils::stats::{calculate_stats, PlayerStats};

use crate::interactions::commands::models::graph_user_model::{
    AverageSubCommand, DiscordUserSubCommand, GraphUser, StatsSubCommand, TetrioUserSubCommand,
};

#[derive(CreateCommand)]
#[command(name = "sq", desc = "Get a small graph of player stats")]
#[allow(unused)]
pub enum SqCommand {
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

impl SqCommand {
    pub fn get_background_colors(dark_mode: bool) -> &'static str {
        if dark_mode {
            "rgba(254,190,9,0.7)"
        } else {
            "rgba(132,92,248,0.7)"
        }
    }

    pub fn get_font_color(dark_mode: bool) -> &'static str {
        if dark_mode {
            "#F5F5F5"
        } else {
            "#000000"
        }
    }

    async fn graph_with_stats(
        username: &str,
        dark_mode: bool,
        stats: PlayerStats,
    ) -> anyhow::Result<String> {
        let stats = calculate_stats(stats);
        let attack = (stats.apm / 60.0) * 0.4;
        let speed = stats.pps / 3.75;
        let defense = stats.dssecond * 1.05;
        let cheese = stats.cheese / 135.0;

        log::debug!("{attack} {speed} {defense} {cheese}");

        let json = json!({
            "type": "radar",
            "data": {
                "labels": ["ATTACK", "SPEED", "DEFENSE", "CHEESE"],
                "datasets": [{
                    "label": username,
                    "data": [
                        attack, speed, defense, cheese
                    ],
                    "backgroundColor": Self::get_background_colors(dark_mode),
                    "borderColor": Self::get_background_colors(dark_mode),
                    "borderWidth": 0,
                    "pointRadius": 0
                }]
            },
            "options":{"legend": { "labels": { "fontColor": Self::get_font_color(dark_mode), "fontSize": 16}}, "scale":{"pointLabels":{"fontColor":Self::get_font_color(dark_mode), "fontSize": 16},"rAxis":{"ticks":{"display":false}},"ticks":{"min":0,"max":1.2,"stepSize":"0.2","fontColor":"blue","display":false},"gridLines":{"color":"gray"},"angleLines":{"color":"gray"}}}
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
impl RunnableCommand for SqCommand {
    async fn run(
        _shard: u64,
        interaction: &InteractionCreate,
        data: Box<CommandData>,
        context: Arc<Context>,
    ) -> anyhow::Result<anyhow::Result<()>> {
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

        context
            .http_client
            .interaction(context.application.id)
            .update_response(&interaction.token)
            .content(Some(&content))?
            .await?;

        Ok(Ok(()))
    }
}
