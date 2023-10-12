use std::{borrow::Cow, sync::Arc};

use anyhow::anyhow;
use itertools::Itertools;
use twilight_interactions::command::{CommandInputData, CommandModel, CreateCommand, ResolvedUser};
use twilight_model::{
    application::interaction::application_command::CommandData,
    gateway::payload::incoming::InteractionCreate,
};

use crate::{
    context::Context,
    utils::{
        average_of_rank::average_of_rank,
        box_commands::{CommandBox, RunnableCommand},
        stats::{calculate_stats, calculate_win_chance, PlayerStats, Stats},
        timer::Timer,
    },
};

use crate::interactions::commands::{
    options::user_rank_option::UserRankOption,
    subcommands::vst::{average, discord, stats, tetrio},
};

#[derive(CreateCommand, CommandModel)]
#[command(name = "vst", desc = "Compare stats from two users")]
pub enum VstCommand {
    #[command(name = "discord")]
    Discord(discord::DiscordSubCommandGroup),
    #[command(name = "tetrio")]
    Tetrio(CommandBox<tetrio::TetrioSubCommandGroup>),
    #[command(name = "stats")]
    Stats(stats::StatsSubCommandGroup),
    #[command(name = "average")]
    Average(average::AverageSubCommandGroup),
}

impl VstCommand {
    pub async fn from_discord_user(
        user: &ResolvedUser,
        context: Arc<Context>,
    ) -> anyhow::Result<anyhow::Result<(String, Stats)>> {
        let tetrio_user = context
            .tetrio_client
            .search_user(&user.resolved.id.to_string())
            .await?;

        let Some(data) = &tetrio_user.data else {
            return Err(anyhow::anyhow!(
                "❌ Couldn't find user `@{}#{}`",
                user.resolved.name,
                user.resolved.discriminator()
            ));
        };

        Self::from_tetrio_user(data.user.username.as_ref(), context).await
    }

    pub async fn from_tetrio_user(
        user: &str,
        context: Arc<Context>,
    ) -> anyhow::Result<anyhow::Result<(String, Stats)>> {
        let tetrio_user = context.tetrio_client.fetch_user_info(user).await?;

        let Some(data) = &tetrio_user.data else {
            return Ok(Err(anyhow!(
                "❌ No data has been found for user {user}. User might be anonymous or banned."
            )));
        };

        let (_id, username, Some(apm), Some(pps), Some(vs), rank, tr, Some(glicko), Some(rd)) = (
            &data.user.id,
            &data.user.username,
            data.user.league.apm,
            data.user.league.pps,
            data.user.league.vs,
            &data.user.league.rank,
            data.user.league.rating,
            data.user.league.glicko,
            data.user.league.rd,
        ) else {
            return Ok(Err(anyhow::anyhow!(
                "❌ No tetra league stats have been found for user {user}."
            )));
        };

        Ok(Ok((
            username.to_string(),
            calculate_stats(PlayerStats {
                apm,
                pps,
                vs,
                rd: Some(rd),
                tr: Some(tr),
                glicko: Some(glicko),
                rank: Some(rank.clone()),
            }),
        )))
    }

    async fn from_average(
        rank: Option<UserRankOption>,
        country: Option<String>,
        context: Arc<Context>,
    ) -> anyhow::Result<anyhow::Result<(String, Stats)>> {
        let stats = average_of_rank(
            rank.clone().map(|r| r.into()),
            country.clone().map(|c| c.to_uppercase()),
            context,
        )
        .await?;

        let stats = match stats {
            Ok(stats) => stats,
            Err(err) => return Ok(Err(err)),
        };

        let name = format!(
            "$avg:{}:{}",
            rank.map(|r| r.value()).unwrap_or("*"),
            country.unwrap_or(String::from("*"))
        );

        Ok(Ok((name, calculate_stats(stats.0.into()))))
    }

    pub async fn from_stats(
        pps: f64,
        apm: f64,
        vs: f64,
    ) -> anyhow::Result<anyhow::Result<(String, Stats)>> {
        Ok(Ok((
            format!("{pps:.2},{apm:.1},{vs:.2}"),
            calculate_stats(PlayerStats {
                apm,
                pps,
                vs,
                rd: None,
                tr: None,
                glicko: None,
                rank: None,
            }),
        )))
    }
}

#[async_trait::async_trait]
impl RunnableCommand for VstCommand {
    async fn run(
        _shard: u64,
        interaction: &InteractionCreate,
        data: Box<CommandData>,
        context: Arc<Context>,
    ) -> anyhow::Result<anyhow::Result<()>> {
        log::info!("VST Command");
        let _command_timer = Timer::new("vst command");
        let thread = Context::threaded_defer_response(Arc::clone(&context), interaction);
        let model = Self::from_interaction(CommandInputData {
            options: data.options,
            resolved: data.resolved.map(Cow::Owned),
        })?;
        let (left, right) = {
            let _timer = crate::utils::timer::Timer::new("vst data fetching");
            let (left, right) = match model {
                VstCommand::Discord(discord) => match discord {
                    discord::DiscordSubCommandGroup::Discord(discord) => (
                        Self::from_discord_user(&discord.user1, Arc::clone(&context)).await,
                        Self::from_discord_user(&discord.user2, Arc::clone(&context)).await,
                    ),
                    discord::DiscordSubCommandGroup::Stats(stats) => (
                        Self::from_discord_user(&stats.discord_user, Arc::clone(&context)).await,
                        Self::from_stats(stats.pps, stats.apm, stats.vs).await,
                    ),
                },
                VstCommand::Tetrio(tetrio) => match tetrio.as_ref() {
                    tetrio::TetrioSubCommandGroup::Discord(discord) => (
                        Self::from_tetrio_user(&discord.tetrio_user, Arc::clone(&context)).await,
                        Self::from_discord_user(&discord.discord_user, Arc::clone(&context)).await,
                    ),
                    tetrio::TetrioSubCommandGroup::Tetrio(tetrio) => (
                        Self::from_tetrio_user(&tetrio.user1, Arc::clone(&context)).await,
                        Self::from_tetrio_user(&tetrio.user2, Arc::clone(&context)).await,
                    ),
                    tetrio::TetrioSubCommandGroup::Stats(stats) => (
                        Self::from_tetrio_user(&stats.tetrio_user, Arc::clone(&context)).await,
                        Self::from_stats(stats.pps, stats.apm, stats.vs).await,
                    ),
                },
                VstCommand::Stats(stats) => match stats {
                    stats::StatsSubCommandGroup::Stats(stats) => (
                        Self::from_stats(stats.pps1, stats.apm1, stats.vs1).await,
                        Self::from_stats(stats.pps2, stats.apm2, stats.vs2).await,
                    ),
                },
                VstCommand::Average(average) => match average {
                    average::AverageSubCommandGroup::Stats(data) => (
                        Self::from_stats(data.pps, data.apm, data.vs).await,
                        Self::from_average(
                            data.average_rank.clone(),
                            data.average_country.clone(),
                            Arc::clone(&context),
                        )
                        .await,
                    ),
                    average::AverageSubCommandGroup::Discord(data) => (
                        Self::from_discord_user(&data.discord_user, Arc::clone(&context)).await,
                        Self::from_average(
                            data.average_rank.clone(),
                            data.average_country.clone(),
                            Arc::clone(&context),
                        )
                        .await,
                    ),
                    average::AverageSubCommandGroup::Tetrio(data) => (
                        Self::from_tetrio_user(&data.tetrio_user, Arc::clone(&context)).await,
                        Self::from_average(
                            data.average_rank,
                            data.average_country,
                            Arc::clone(&context),
                        )
                        .await,
                    ),
                    average::AverageSubCommandGroup::Average(data) => (
                        Self::from_average(
                            data.average_rank1,
                            data.average_country1,
                            Arc::clone(&context),
                        )
                        .await,
                        Self::from_average(
                            data.average_rank2,
                            data.average_country2,
                            Arc::clone(&context),
                        )
                        .await,
                    ),
                },
            };

            let (left, right) = (left?, right?);

            let left = match left {
                Ok(result) => result,
                Err(err) => return Ok(Err(err)),
            };

            let right = match right {
                Ok(result) => result,
                Err(err) => return Ok(Err(err)),
            };

            (left, right)
        };

        let v = {
            let _timer = Timer::new(format!("vst data parsing {} {}", left.0, right.0));
            let v = [
                ["Names:".to_string(), left.0, right.0],
                [
                    "APM:".to_string(),
                    format!("{:.4}", left.1.apm),
                    format!("{:.4}", right.1.apm),
                ],
                [
                    "PPS:".to_string(),
                    format!("{:.4}", left.1.pps),
                    format!("{:.4}", right.1.pps),
                ],
                [
                    "VS:".to_string(),
                    format!("{:.4}", left.1.vs),
                    format!("{:.4}", right.1.vs),
                ],
                [
                    "APP:".to_string(),
                    format!("{:.4}", left.1.app),
                    format!("{:.4}", right.1.app),
                ],
                [
                    "DS/Piece:".to_string(),
                    format!("{:.4}", left.1.dspiece),
                    format!("{:.4}", right.1.dspiece),
                ],
                [
                    "APP+DS/Piece:".to_string(),
                    format!("{:.4}", left.1.dsapppiece),
                    format!("{:.4}", right.1.dsapppiece),
                ],
                [
                    "DS/Second:".to_string(),
                    format!("{:.4}", left.1.dssecond),
                    format!("{:.4}", right.1.dssecond),
                ],
                [
                    "VS/APM:".to_string(),
                    format!("{:.4}", left.1.vsapm),
                    format!("{:.4}", right.1.vsapm),
                ],
                [
                    "Cheese Index:".to_string(),
                    format!("{:.4}", left.1.cheese),
                    format!("{:.4}", right.1.cheese),
                ],
                [
                    "Garbage Effi:".to_string(),
                    format!("{:.4}", left.1.garbage_effi),
                    format!("{:.4}", right.1.garbage_effi),
                ],
                [
                    "Weighted APP:".to_string(),
                    format!("{:.4}", left.1.weighted_app),
                    format!("{:.4}", right.1.weighted_app),
                ],
                [
                    "Area:".to_string(),
                    format!("{:.4}", left.1.area),
                    format!("{:.4}", right.1.area),
                ],
                [
                    "Win Chance:".to_string(),
                    format!(
                        "{:.4}",
                        calculate_win_chance(
                            left.1.glicko.unwrap_or(left.1.estglicko),
                            right.1.glicko.unwrap_or(right.1.estglicko),
                            left.1.rd.unwrap_or(60.9),
                            right.1.rd.unwrap_or(60.9)
                        )
                    ),
                    format!(
                        "{:.4}",
                        calculate_win_chance(
                            right.1.glicko.unwrap_or(right.1.estglicko),
                            left.1.glicko.unwrap_or(left.1.estglicko),
                            right.1.rd.unwrap_or(60.9),
                            left.1.rd.unwrap_or(60.9)
                        )
                    ),
                ],
            ];

            // v.push(["Names:".to_string(), left.0, right.0]);
            // v.push([
            //     "APM:".to_string(),
            //     format!("{:.4}", left.1.apm),
            //     format!("{:.4}", right.1.apm),
            // ]);
            // v.push([
            //     "PPS:".to_string(),
            //     format!("{:.4}", left.1.pps),
            //     format!("{:.4}", right.1.pps),
            // ]);
            // v.push([
            //     "VS:".to_string(),
            //     format!("{:.4}", left.1.vs),
            //     format!("{:.4}", right.1.vs),
            // ]);
            // v.push([
            //     "APP:".to_string(),
            //     format!("{:.4}", left.1.app),
            //     format!("{:.4}", right.1.app),
            // ]);
            // v.push([
            //     "DS/Piece:".to_string(),
            //     format!("{:.4}", left.1.dspiece),
            //     format!("{:.4}", right.1.dspiece),
            // ]);
            // v.push([
            //     "APP+DS/Piece:".to_string(),
            //     format!("{:.4}", left.1.dsapppiece),
            //     format!("{:.4}", right.1.dsapppiece),
            // ]);
            // v.push([
            //     "DS/Second:".to_string(),
            //     format!("{:.4}", left.1.dssecond),
            //     format!("{:.4}", right.1.dssecond),
            // ]);
            // v.push([
            //     "VS/APM:".to_string(),
            //     format!("{:.4}", left.1.vsapm),
            //     format!("{:.4}", right.1.vsapm),
            // ]);
            // v.push([
            //     "Cheese Index:".to_string(),
            //     format!("{:.4}", left.1.cheese),
            //     format!("{:.4}", right.1.cheese),
            // ]);
            // v.push([
            //     "Garbage Effi:".to_string(),
            //     format!("{:.4}", left.1.garbage_effi),
            //     format!("{:.4}", right.1.garbage_effi),
            // ]);
            // v.push([
            //     "Weighted APP:".to_string(),
            //     format!("{:.4}", left.1.weighted_app),
            //     format!("{:.4}", right.1.weighted_app),
            // ]);
            // v.push([
            //     "Area:".to_string(),
            //     format!("{:.4}", left.1.area),
            //     format!("{:.4}", right.1.area),
            // ]);
            // v.push([
            //     "Win Chance:".to_string(),
            //     format!(
            //         "{:.4}",
            //         calculate_win_chance(
            //             left.1.glicko.unwrap_or(left.1.estglicko),
            //             right.1.glicko.unwrap_or(right.1.estglicko),
            //             left.1.rd.unwrap_or(60.9),
            //             right.1.rd.unwrap_or(60.9)
            //         )
            //     ),
            //     format!(
            //         "{:.4}",
            //         calculate_win_chance(
            //             right.1.glicko.unwrap_or(right.1.estglicko),
            //             left.1.glicko.unwrap_or(left.1.estglicko),
            //             right.1.rd.unwrap_or(60.9),
            //             left.1.rd.unwrap_or(60.9)
            //         )
            //     ),
            // ]);
            v
        };

        let final_str = {
            let _timer = Timer::new(format!("vst data formatting {} {}", v[0][1], v[0][2]));
            let mut columns_size = vec![0usize; v[0].len()];
            for i in 0..v[0].len() {
                columns_size[i] = v.iter().map(|array| array[i].len()).max().unwrap_or(0);
            }
            log::debug!("{:?}", columns_size);
            let top = String::from("╔")
                + &columns_size
                    .iter()
                    .map(|size| "═".repeat(size + 2))
                    .join("╤")
                + "╗\n";
            let bottom = String::from("╚")
                + &columns_size
                    .iter()
                    .map(|size| "═".repeat(size + 2))
                    .join("╧")
                + "╝";
            let separator = String::from("╟")
                + &columns_size
                    .iter()
                    .map(|size| "═".repeat(size + 2))
                    .join("┼")
                + "╢\n";

            let final_str = top
                + &v.into_iter()
                    .map(|mut array| {
                        for i in 0..array.len() {
                            array[i] = format!(" {:<width$}", array[i], width = columns_size[i] + 1)
                        }

                        String::from("║") + &array.join("│") + "║\n"
                    })
                    .join(&separator)
                + &bottom;

            let final_str = format!("```\n{}\n```", final_str);
            final_str
        };
        log::debug!("{}", final_str.len());

        thread.await??;

        let interaction_client = context.http_client.interaction(context.application.id);
        let r = interaction_client
            .update_response(&interaction.token)
            .content(Some(&final_str));
        match r {
            Ok(response) => response.await?,
            Err(_) => {
                interaction_client
                    .update_response(&interaction.token)
                    .content(Some("❌ Message was too long."))?
                    .await?
            }
        };
        Ok(Ok(()))
    }
}
