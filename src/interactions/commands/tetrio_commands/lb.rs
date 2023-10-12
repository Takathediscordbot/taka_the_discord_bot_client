use std::{borrow::Cow, sync::Arc};

use anyhow::anyhow;
use itertools::Itertools;
use tetrio_api::models::users::lists::league_full::LeagueFullPacketData;
use twilight_interactions::command::{CommandInputData, CommandModel, CreateCommand};
use twilight_model::{
    application::interaction::application_command::CommandData,
    gateway::payload::incoming::InteractionCreate,
};

use crate::{
    context::Context,
    utils::{
        box_commands::RunnableCommand,
        stats::{calculate_stats, PlayerStats},
    },
};

use crate::interactions::commands::options::{
    user_rank_option::UserRankOption, user_stat_options::UserStatOption,
};

use crate::utils::timer::Timer;


#[derive(CreateCommand, CommandModel)]
#[command(name = "lb", desc = "Get the leaderboard")]
pub struct LbCommand {
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

impl LbCommand {
    pub fn filter_rank<'a>(
        data: &'a LeagueFullPacketData,
        rank: &'a Option<UserRankOption>,
    ) -> std::iter::FilterMap<
        std::iter::Enumerate<
            std::slice::Iter<'a, tetrio_api::models::users::lists::league_full::LeagueFullUser>,
        >,
        impl FnMut(
            (
                usize,
                &'a tetrio_api::models::users::lists::league_full::LeagueFullUser,
            ),
        ) -> Option<(
            usize,
            &'a tetrio_api::models::users::lists::league_full::LeagueFullUser,
        )>,
    > {
        let rank = rank.clone().map(|f| f.into());
        data.users
            .iter()
            .enumerate()
            .filter_map(move |(pos, user)| match &rank {
                Some(rank) => {
                    if &user.league.rank == rank {
                        Some((pos, user))
                    } else {
                        None
                    }
                }
                None => Some((pos, user)),
            })
    }

    pub fn get_stats<'a>(
        leaderboart_stat: &UserStatOption,
        iter: impl Iterator<
            Item = (
                usize,
                &'a tetrio_api::models::users::lists::league_full::LeagueFullUser,
            ),
        >,
    ) -> Vec<(usize, Arc<str>, f64)> {
        match leaderboart_stat {
            UserStatOption::APM => iter
                .filter_map(|(rank, user)| {
                    user.league
                        .apm
                        .map(|stat| (rank, user.username.clone(), stat))
                })
                .collect(),
            UserStatOption::PPS => iter
                .filter_map(|(rank, user)| {
                    user.league
                        .pps
                        .map(|stat| (rank, user.username.clone(), stat))
                })
                .collect(),
            UserStatOption::VS => iter
                .filter_map(|(rank, user)| {
                    user.league
                        .vs
                        .map(|stat| (rank, user.username.clone(), stat))
                })
                .collect(),
            UserStatOption::WR => iter
                .map(|(rank, user)| {
                    (
                        rank,
                        user.username.clone(),
                        user.league.gameswon as f64 / user.league.gamesplayed as f64,
                    )
                })
                .collect(),
            UserStatOption::WINS => iter
                .map(|(rank, user)| (rank, user.username.clone(), user.league.gameswon as f64))
                .collect(),
            UserStatOption::GAMES => iter
                .map(|(rank, user)| (rank, user.username.clone(), user.league.gamesplayed as f64))
                .collect(),
            UserStatOption::TR => iter
                .map(|(rank, user)| (rank, user.username.clone(), user.league.rating))
                .collect(),
            e => iter
                .filter_map(|(rank, user)| {
                    if let (Some(pps), Some(apm), Some(vs)) =
                        (user.league.pps, user.league.apm, user.league.vs)
                    {
                        let stats = calculate_stats(PlayerStats {
                            apm,
                            pps,
                            vs,
                            rd: user.league.rd,
                            tr: Some(user.league.rating),
                            glicko: user.league.glicko,
                            rank: Some(user.league.rank.clone()),
                        });
                        match e {
                            UserStatOption::APP => Some((rank, user.username.clone(), stats.app)),
                            UserStatOption::DSPIECE => {
                                Some((rank, user.username.clone(), stats.dspiece))
                            }
                            UserStatOption::DSSECOND => {
                                Some((rank, user.username.clone(), stats.dssecond))
                            }
                            UserStatOption::CHEESE => {
                                Some((rank, user.username.clone(), stats.cheese))
                            }
                            UserStatOption::GE => {
                                Some((rank, user.username.clone(), stats.garbage_effi))
                            }
                            UserStatOption::AREA => Some((rank, user.username.clone(), stats.area)),
                            UserStatOption::WAPP => {
                                Some((rank, user.username.clone(), stats.weighted_app))
                            }
                            UserStatOption::VSAPM => {
                                Some((rank, user.username.clone(), stats.vsapm))
                            }
                            UserStatOption::DSAPPPIECE => {
                                Some((rank, user.username.clone(), stats.dsapppiece))
                            }
                            UserStatOption::ESTTR => {
                                Some((rank, user.username.clone(), stats.esttr))
                            }
                            UserStatOption::ATR => {
                                stats.atr.map(|atr| (rank, user.username.clone(), atr))
                            }
                            UserStatOption::OPENER => {
                                Some((rank, user.username.clone(), stats.opener))
                            }
                            UserStatOption::PLONK => {
                                Some((rank, user.username.clone(), stats.plonk))
                            }
                            UserStatOption::STRIDE => {
                                Some((rank, user.username.clone(), stats.stride))
                            }
                            UserStatOption::INFDS => {
                                Some((rank, user.username.clone(), stats.infds))
                            }
                            _ => None,
                        }
                    } else {
                        None
                    }
                })
                .collect(),
        }
    }
}

#[async_trait::async_trait]
impl RunnableCommand for LbCommand {
    async fn run(
        _shard: u64,
        interaction: &InteractionCreate,
        data: Box<CommandData>,
        context: Arc<Context>,
    ) -> anyhow::Result<anyhow::Result<()>> {
        log::info!("lb command");
        let _command_timer = Timer::new("lb command");
        let thread = Context::threaded_defer_response(Arc::clone(&context), interaction);
        
        let model = Self::from_interaction(CommandInputData {
            options: data.options,
            resolved: data.resolved.map(Cow::Owned),
        })?;

        let leaderboard = context
            .tetrio_client
            .fetch_full_league_leaderboard(model.country_code)
            .await?;

        let Some(data) = &leaderboard.data else {
            return Ok(Err(anyhow!("❌ Couldn't fetch leaderboard data because {}", leaderboard.error.clone().unwrap_or("Unknwon error".to_string()))));
        };

        let iter = Self::filter_rank(data, &model.rank);

        let v = Self::get_stats(&model.leaderboard_stat, iter)
            .into_iter()
            .sorted_by(|(_, _, a), (_, _, b)| b.total_cmp(a));

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

        thread.await??;

        let interaction_client = context.http_client.interaction(context.application.id);
        let r = interaction_client
            .update_response(&interaction.token)
            .content(Some(&content));
        match r {
            Ok(response) => response.await?,
            Err(_) => return Ok(Err(anyhow!("❌ Message content was too long!"))),
        };

        Ok(Ok(()))
    }
}
