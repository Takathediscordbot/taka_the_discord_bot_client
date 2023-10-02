use std::{borrow::Cow, sync::Arc};

use anyhow::anyhow;
use async_trait::async_trait;
use serde_json::json;
use twilight_interactions::command::{CommandInputData, CommandModel, CreateCommand};
use twilight_model::{
    application::interaction::application_command::CommandData,
    gateway::payload::incoming::InteractionCreate,
};

use crate::{
    context::Context,
    interactions::commands::tetrio_commands::vs::VsCommand,
    utils::{
        box_commands::RunnableCommand,
        stats::{
            calculate_stats, APM_WEIGHT, APP_WEIGHT, CHEESE_WEIGHT, DSAPPPIECE_WEIGHT,
            DSPIECE_WEIGHT, DSSECOND_WEIGHT, GARBAGEEFFI_WEIGHT, PPS_WEIGHT, VSAPM_WEIGHT,
            VS_WEIGHT,
        },
    },
};

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "vsr",
    desc = "Get a graph of player stats relative to the highest stat"
)]
pub struct VsrCommand {
    /// A tetrio user, (pps, apm, vs), discord ping, $avgX where X is a rank, e.g S+ or $avgX:COUNTRY_CODE
    pub user_1: String,
    /// Get a dark mode chart
    pub dark_mode: bool,
    /// A tetrio user, (pps, apm, vs), discord ping $avgX where X is a rank, e.g S+ or $avgX:COUNTRY_CODE
    pub user_2: Option<String>,
    /// A tetrio user, (pps, apm, vs), discord ping $avgX where X is a rank, e.g S+ or $avgX:COUNTRY_CODE
    pub user_3: Option<String>,
    /// A tetrio user, (pps, apm, vs), discord ping $avgX where X is a rank, e.g S+ or $avgX:COUNTRY_CODE
    pub user_4: Option<String>,
    /// A tetrio user, (pps, apm, vs), discord ping $avgX where X is a rank, e.g S+ or $avgX:COUNTRY_CODE
    pub user_5: Option<String>,
}

impl VsrCommand {
    pub fn get_background_colors(dark_mode: bool) -> [&'static str; 5] {
        if dark_mode {
            [
                "rgba(254,190,9,0.7)",
                "rgba(123,124,132,0.7)",
                "rgba(132,92,248,0.7)",
                "rgba(0,0,0,0)",
                "rgba(0,0,0,0)",
            ]
        } else {
            [
                "rgba(132,92,248,0.7)",
                "rgba(123,124,132,0.7)",
                "rgba(254,148,9,0.7)",
                "rgba(0,0,0,0)",
                "rgba(0,0,0,0)",
            ]
        }
    }

    pub fn get_font_color(dark_mode: bool) -> &'static str {
        if dark_mode {
            "#F5F5F5"
        } else {
            "#000000"
        }
    }
}

#[async_trait]
impl RunnableCommand for VsrCommand {
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

        let result = vec![
            Some(model.user_1),
            model.user_2,
            model.user_3,
            model.user_4,
            model.user_5,
        ]
        .into_iter()
        .filter_map(|c| c.map(|c| async { VsCommand::parse_user(c, Arc::clone(&context)).await }))
        .rev()
        .collect::<Vec<_>>();
        let mut result = futures::future::join_all(result).await;

        let mut new_vec = vec![];

        loop {
            let r = match result.pop() {
                Some(r) => r,
                None => break,
            };

            new_vec.push(match r? {
                Ok(ok) => ok,
                Err(err) => return Ok(Err(err)),
            })
        }

        let background_colors = Self::get_background_colors(model.dark_mode);

        let data = new_vec
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i, v.0, calculate_stats(v.1)))
            .collect::<Vec<_>>();

        let max_stat = data
            .iter()
            .map(|(_, _, v)| {
                [
                    v.apm * APM_WEIGHT,
                    v.pps * PPS_WEIGHT,
                    v.vs * VS_WEIGHT,
                    v.app * APP_WEIGHT,
                    v.dssecond * DSSECOND_WEIGHT,
                    v.dspiece * DSPIECE_WEIGHT,
                    v.dsapppiece * DSAPPPIECE_WEIGHT,
                    ((v.vsapm - 2.0).abs() * VSAPM_WEIGHT),
                    v.cheese * CHEESE_WEIGHT,
                    v.garbage_effi * GARBAGEEFFI_WEIGHT,
                ]
                .into_iter()
                .fold(f64::NEG_INFINITY, |a, b| a.max(b))
            })
            .fold(f64::NEG_INFINITY, |a, b| a.max(b));

        let datasets = data
            .into_iter()
            .map(|(i, label, v)| {
                json!({
                    "label": label,
                    "data": [
                        v.apm * APM_WEIGHT,
                        v.pps * PPS_WEIGHT,
                        v.vs * VS_WEIGHT,
                        v.app * APP_WEIGHT,
                        v.dssecond * DSSECOND_WEIGHT,
                        v.dspiece * DSPIECE_WEIGHT,
                        v.dsapppiece * DSAPPPIECE_WEIGHT,
                        ((v.vsapm - 2.0).abs() * VSAPM_WEIGHT),
                        v.cheese * CHEESE_WEIGHT,
                        v.garbage_effi * GARBAGEEFFI_WEIGHT,
                    ],
                    "backgroundColor": background_colors[i],
                    "borderColor": background_colors[i],
                    "borderWidth": 0,
                    "pointRadius": 0
                })
            })
            .collect::<Vec<_>>();

        let json = json!({
            "type": "radar",
            "data": {
                "labels": ["APM", "PPS", "VS", "APP", "DS/Second", "DS/Piece", "APP+DS/Piece", "VS/APM", "Cheese\nIndex", "Garbage\nEffi."],
                "datasets": datasets
            },
            "options":{"legend": { "labels": { "fontColor": Self::get_font_color(model.dark_mode), "fontSize": 16}}, "scale":{"pointLabels":{"fontColor":Self::get_font_color(model.dark_mode), "fontSize": 16},"rAxis":{"ticks":{"display":false}},"ticks":{"min":0,"max":max_stat,"stepSize":"30","fontColor":"blue","display":false},"gridLines":{"color":"gray"},"angleLines":{"color":"gray"}}}
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

        let interaction_client = context.http_client.interaction(context.application.id);
        interaction_client
            .update_response(&interaction.token)
            .content(Some(
                response
                    .get("url").ok_or(anyhow!("Couldn't find graph url"))?
                    .as_str().ok_or(anyhow!("Couldn't find graph url"))?
            ))?
            .await?;

        Ok(Ok(()))
    }
}