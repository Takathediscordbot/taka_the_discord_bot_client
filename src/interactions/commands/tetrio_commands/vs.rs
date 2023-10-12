use std::{borrow::Cow, sync::Arc};

use anyhow::anyhow;
use async_trait::async_trait;
use itertools::Itertools;
use serde_json::json;
use tetrio_api::models::users::user_rank::UserRank;
use twilight_interactions::command::{CommandInputData, CommandModel, CreateCommand};
use twilight_model::{
    application::interaction::application_command::CommandData,
    gateway::payload::incoming::InteractionCreate,
};

use crate::{
    context::Context,
    utils::{
        average_of_rank::average_of_rank,
        box_commands::RunnableCommand,
        stats::{
            calculate_stats, PlayerStats, APM_WEIGHT, APP_WEIGHT, CHEESE_WEIGHT, DSAPPPIECE_WEIGHT,
            DSPIECE_WEIGHT, DSSECOND_WEIGHT, GARBAGEEFFI_WEIGHT, PPS_WEIGHT, VSAPM_WEIGHT,
            VS_WEIGHT,
        },
        timer::Timer,
    },
};

#[derive(CommandModel, CreateCommand)]
#[command(name = "vs", desc = "Get a graph of player stats")]
pub struct VsCommand {
    /// A tetrio user, (pps, apm, vs), discord ping, $avgX where X is a rank, e.g S+ or $avgX:COUNTRY_CODE
    pub user_1: String,
    /// Get a dark mode chart
    pub dark_mode: bool,
    /// A tetrio user, (pps, apm, vs), discord ping $avgX where X is a rank, e.g S+ or $avgX:COUNTRY_CODE
    pub user_2: Option<String>,
}

impl VsCommand {
    fn parse_average_rank(rank: &str) -> anyhow::Result<Option<UserRank>> {
        let (_, rank) = match rank.split_once("$avg") {
            Some(data) => data,
            None => unreachable!(),
        };

        if rank.is_empty() {
            return Ok(None);
        }

        match serde_json::from_str::<UserRank>(&format!("\"{}\"", rank.to_lowercase())) {
            Ok(ok) => Ok(Some(ok)),
            Err(err) => Err(anyhow!("❌ Couldn't find rank in {rank} because {err}")),
        }
    }

    pub async fn parse_user(
        user: String,
        context: Arc<Context>,
    ) -> anyhow::Result<anyhow::Result<(String, PlayerStats)>> {
        let user = user.trim();
        if user.starts_with("$avg") {
            if user.contains(':') {
                let (left, right) = match user.split_once(':') {
                    Some(data) => data,
                    None => unreachable!(),
                };

                let rank = match Self::parse_average_rank(left) {
                    Ok(ok) => ok,
                    Err(err) => return Ok(Err(err)),
                };
                let country = right;
                let stats =
                    average_of_rank(rank.clone(), Some(country.to_uppercase()), context).await;

                let rank_str = format!(
                    "$avg{}:{country}",
                    rank.map(|r| r.to_string()).unwrap_or("".to_string())
                );

                stats.map(|result| result.map(|stats| (rank_str, stats.0.into())))
            } else {
                let rank = match Self::parse_average_rank(user) {
                    Ok(ok) => ok,
                    Err(err) => return Ok(Err(err)),
                };

                let stats = average_of_rank(rank.clone(), None, context).await;

                let rank_str = format!(
                    "$avg{}",
                    rank.map(|r| r.to_string()).unwrap_or("".to_string())
                );

                stats.map(|result| result.map(|stats| (rank_str, stats.0.into())))
            }
        } else if user.starts_with("<@") {
            let str = user.split(':').collect_vec();
            let user = str[0].trim();
            let user = {
                let user = user.chars().skip(2).collect::<String>();
                let user = user.chars().rev().skip(1).collect::<String>();
                user.chars().rev().collect::<String>()
            };

            let user_id: u64 = match user.parse() {
                Ok(ok) => ok,
                Err(_) => return Ok(Err(anyhow!("❌ Couldn't find discord user <@{user}>"))),
            };

            let discord_user = context
                .tetrio_client
                .search_user(&user_id.to_string())
                .await?;

            let data = match &discord_user.data {
                Some(data) => data,
                None => return Ok(Err(anyhow!("❌ Couldn't find tetrio user linked to discord user `<@{user}>`, they might have not linked their discord account to their tetrio account"))),
            };

            Self::parse_tetrio_user(&data.user.username, str, context).await
        } else {
            let strs = user.split(',');

            if strs.clone().count() == 3 {
                let Some((pps, apm, vs)) = strs.collect_tuple() else {
                    return Ok(Err(anyhow!("❌ Couldn't parse stats {user}")));
                };

                let pps = if pps.starts_with('(') {
                    pps.chars().skip(1).collect()
                } else {
                    pps.to_string()
                };

                let vs = if vs.ends_with('(') {
                    let vs = vs.chars().rev().skip(1).collect::<String>();
                    vs.chars().rev().collect()
                } else {
                    vs.to_string()
                };

                let (pps, apm, vs) = (pps.trim(), apm.trim(), vs.trim());

                let name = format!("{pps},{apm},{vs}");

                let (pps, apm, vs): (f64, f64, f64) = (pps.parse()?, apm.parse()?, vs.parse()?);

                return Ok(Ok((
                    name,
                    PlayerStats {
                        apm,
                        pps,
                        vs,
                        rd: None,
                        tr: None,
                        glicko: None,
                        rank: None,
                    },
                )));
            } else {
                let str = user.split(':').collect_vec();

                Self::parse_tetrio_user(str[0], str, context).await
            }
        }
    }

    async fn parse_tetrio_user(
        user_name: &str,
        params: Vec<&str>,
        context: Arc<Context>,
    ) -> anyhow::Result<anyhow::Result<(String, PlayerStats)>> {
        let user = context
            .tetrio_client
            .fetch_user_info(&user_name.to_lowercase())
            .await?;
        match &user.error {
            Some(err) => {
                return Ok(Err(anyhow!(
                    "❌ Couldn't find user data for {user_name} because {err}"
                )))
            }
            None => {}
        };

        let data = match &user.data {
            Some(data) => data,
            None => return Ok(Err(anyhow!("❌ Couldn't find user data for {user_name}"))),
        };

        let id = &data.user.id;
        let (Some(mut pps), Some(mut apm), Some(mut vs)) = (
            data.user.league.pps,
            data.user.league.apm,
            data.user.league.vs,
        ) else {
            return Ok(Err(anyhow!(
                "❌ {user_name} doesn't have a valid tetra league record "
            )));
        };

        let league_str = if let Some(Ok(mut tetra_league_game)) =
            params.get(1).map(|s| str::parse::<usize>(s))
        {
            if tetra_league_game == 0 {
                tetra_league_game = 1;
            }

            let game = context
                .tetrio_client
                .fetch_tetra_league_recent(id.as_ref())
                .await?;
            let Some(data) = &game.data else {
                return Ok(Err(anyhow::anyhow!("❌ Couldn't find tetra league game")));
            };
            let records = data.records.get(tetra_league_game - 1);
            let Some(records) = &records else {
                return Err(anyhow::anyhow!("❌ Couldn't find tetra league game"));
            };

            let Some(left) = records.endcontext.iter().find(|a| &a.user.id == id) else {
                return Err(anyhow::anyhow!("❌ Couldn't find tetra league game"));
            };

            if let Some(Ok(mut tetra_league_round)) = params.get(2).map(|s| str::parse::<usize>(s))
            {
                if tetra_league_round == 0 {
                    tetra_league_round = 1;
                }

                let index = tetra_league_round - 1;
                pps = left.points.tertiary_avg_tracking[index];
                apm = left.points.secondary_avg_tracking[index];
                vs = left.points.extra_avg_tracking.aggregate_stats_vs_score[index];

                format!(":{}:{}", tetra_league_game, tetra_league_round)
            } else {
                pps = left.points.tertiary;
                apm = left.points.secondary;
                vs = left.points.extra.vs;

                format!(":{}", tetra_league_game)
            }
        } else {
            String::new()
        };

        Ok(Ok((
            format!("{}{}", user_name, league_str),
            PlayerStats {
                apm,
                glicko: data.user.league.glicko,
                pps,
                vs,
                rd: data.user.league.rd,
                tr: Some(data.user.league.rating),
                rank: Some(data.user.league.rank.clone()),
            },
        )))
    }

    pub fn get_background_colors(dark_mode: bool) -> [&'static str; 2] {
        if dark_mode {
            ["rgba(254,190,9,0.7)", "rgba(123,124,132,0.7)"]
        } else {
            ["rgba(132,92,248,0.7)", "rgba(123,124,132,0.7)"]
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
impl RunnableCommand for VsCommand {
    async fn run(
        _shard: u64,
        interaction: &InteractionCreate,
        data: Box<CommandData>,
        context: Arc<Context>,
    ) -> anyhow::Result<anyhow::Result<()>> {
        log::info!("vs command");
        let _command_timer = Timer::new("vs command");
        let thread = Context::threaded_defer_response(Arc::clone(&context), interaction);
        let (dark_mode, new_vec) = {
            let _timer = Timer::new("vs command parsing input");
            let model = Self::from_interaction(CommandInputData {
                options: data.options,
                resolved: data.resolved.map(Cow::Owned),
            })?;

            let result = vec![Some(model.user_1), model.user_2]
                .into_iter()
                .filter_map(|c| {
                    c.map(|c| async { Self::parse_user(c, Arc::clone(&context)).await })
                })
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

            (model.dark_mode, new_vec)
        };

        let (response) = {
            let background_colors = Self::get_background_colors(dark_mode);
            let datasets = {
                let _timer2 = Timer::new("vs calculating stats");
                let datasets = new_vec
                    .into_iter()
                    .enumerate()
                    .map(|(i, v)| (i, v.0, calculate_stats(v.1)))
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
                                v.vsapm * VSAPM_WEIGHT,
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
                datasets
            };

            let _timer = Timer::new("vs generating graph");

            let json = json!({
                "type": "radar",
                "data": {
                    "labels": ["APM", "PPS", "VS", "APP", "DS/Second", "DS/Piece", "APP+DS/Piece", "VS/APM", "Cheese\nIndex", "Garbage\nEffi."],
                    "datasets": datasets
                },
                "options":{"legend": { "labels": { "fontColor": Self::get_font_color(dark_mode), "fontSize": 16}}, "scale":{"pointLabels":{"fontColor":Self::get_font_color(dark_mode), "fontSize": 16},"rAxis":{"ticks":{"display":false}},"ticks":{"min":0,"max":180,"stepSize":"30","fontColor":"blue","display":false},"gridLines":{"color":"gray"},"angleLines":{"color":"gray"}}}
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

            response
        };

        thread.await??;
        let interaction_client = context.http_client.interaction(context.application.id);
        interaction_client
            .update_response(&interaction.token)
            .content(Some(
                response
                    .get("url")
                    .ok_or(anyhow!("Couldn't find graph url"))?
                    .as_str()
                    .ok_or(anyhow!("Couldn't find graph url"))?,
            ))?
            .await?;

        Ok(Ok(()))
    }
}
