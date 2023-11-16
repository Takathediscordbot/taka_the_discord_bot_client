

use anyhow::anyhow;
use tetrio_api::models::users::user_rank::UserRank;
use twilight_interactions::command::{CommandModel, CreateCommand, ResolvedUser};

use crate::{
    context::Context,
    interactions::commands::options::user_rank_option::UserRankOption,
    utils::{average_of_rank::average_of_rank, box_commands::CommandBox, stats::PlayerStats},
};

#[derive(CreateCommand, CommandModel, Debug)]
#[command(name = "average", desc = "Use the average stats")]
pub struct AverageSubCommand {
    /// dark mode
    pub dark_mode: bool,
    /// the rank to get the average from
    pub rank: Option<UserRankOption>,
    /// Country to limit the stats to
    pub country: Option<String>,
}

#[derive(CreateCommand, CommandModel, Debug)]
#[command(name = "discord", desc = "Use a discord user")]
pub struct DiscordUserSubCommand {
    /// the discord user to be selected
    pub user: ResolvedUser,
    /// dark mode
    pub dark_mode: bool,
    /// tetra league game number
    pub tetra_league_game: Option<i64>,
    /// tetra league round number
    pub tetra_league_round: Option<i64>,
}

#[derive(CreateCommand, CommandModel, Debug)]
#[command(name = "tetrio", desc = "Use a tetrio user")]
pub struct TetrioUserSubCommand {
    /// A tetrio username or id
    pub tetrio_user: String,
    /// dark mode
    pub dark_mode: bool,
    /// tetra league game number
    pub tetra_league_game: Option<i64>,
    /// tetra league round number
    pub tetra_league_round: Option<i64>,
}

#[derive(CreateCommand, CommandModel, Debug)]
#[command(name = "stats", desc = "Use tetrio stats")]
pub struct StatsSubCommand {
    /// pieces per second
    pub pps: f64,
    /// attacks per minute
    pub apm: f64,
    /// vs score
    pub vs: f64,
    /// dark mode
    pub dark_mode: bool,
}

#[derive(CommandModel)]
pub enum GraphUser {
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

pub struct GraphUserData {
    pub name: String,
    pub replay_url: Option<String>,
    pub round: Option<i64>,
    pub stats: PlayerStats,
    pub dark_mode: bool,
}

struct User {
    username: String,
    tetra_league_game: Option<i64>,
    tetra_league_round: Option<i64>,
}

impl GraphUser {
    async fn from_discord_user(
        discord_data: &CommandBox<DiscordUserSubCommand>,
        context: &Context,
    ) -> anyhow::Result<anyhow::Result<GraphUserData>> {
        let user = context
            .tetrio_client
            .search_user(&discord_data.user.resolved.id.get().to_string())
            .await?;

        let Some(data) = &user.data else {
            return Ok(Err(anyhow!("❌ Couldn't find your tetrio id from the discord account, they might have not linked it publicly to their tetrio profile")));
        };

        Self::from_user(
            User {
                username: data.user.id.to_string(),
                tetra_league_game: discord_data.tetra_league_game,
                tetra_league_round: discord_data.tetra_league_round,
            },
            discord_data.dark_mode,
            &context,
        )
        .await
    }

    async fn from_tetrio_user(
        request_data: &TetrioUserSubCommand,
        context: &Context,
    ) -> anyhow::Result<anyhow::Result<GraphUserData>> {
        Self::from_user(
            User {
                username: request_data.tetrio_user.clone(),
                tetra_league_game: request_data.tetra_league_game,
                tetra_league_round: request_data.tetra_league_round,
            },
            request_data.dark_mode,
            &context,
        )
        .await
    }

    async fn from_user(
        User {
            username,
            tetra_league_game,
            tetra_league_round,
        }: User,
        dark_mode: bool,
        context: &Context,
    ) -> anyhow::Result<anyhow::Result<GraphUserData>> {
        let tetrio_user = context.tetrio_client.fetch_user_info(&username).await?;

        let Some(data) = &tetrio_user.data else {
            return Ok(Err(anyhow::anyhow!("❌ No data has been found. User might be anonymous or banned.")));
        };

        let (id, _username, Some(mut apm), Some(mut pps), Some(mut vs), rank, tr, Some(glicko), Some(rd)) = (&data.user.id, &data.user.username, data.user.league.apm, data.user.league.pps, data.user.league.vs, &data.user.league.rank, data.user.league.rating, data.user.league.glicko, data.user.league.rd) else {
            return Ok(Err(anyhow::anyhow!("❌ No tetra league stats have been found.")));
        };

        let mut replay_url = None;
        let mut round = None;

        if let Some(mut tetra_league_game) = tetra_league_game {
            if tetra_league_game <= 0 {
                tetra_league_game = 1;
            }

            let game = tetrio_api::http::client::fetch_tetra_league_recent(id).await?;
            let Some(data) = game.data else {
                return Ok(Err(anyhow::anyhow!("❌ Couldn't find tetra league game")));
            };
            let records = data.records.get((tetra_league_game - 1) as usize);
            let Some(records) = &records else {
                return Err(anyhow::anyhow!("❌ Couldn't find tetra league game"));
            };

            let Some(left) = records.endcontext.iter().find(|a| {
                &a.get_id().unwrap_or("".into()) == id
            }) else {
                return Err(anyhow::anyhow!("❌ Couldn't find tetra league game"));
            };

            replay_url = Some(format!("https://tetr.io/#r:{}", records.replay_id));

            if let Some(mut tetra_league_round) = tetra_league_round {
                if tetra_league_round <= 0 {
                    tetra_league_round = 1;
                }

                round = Some(tetra_league_round);

                let index = (tetra_league_round - 1) as usize;
                pps = left.points.tertiary_avg_tracking[index];
                apm = left.points.secondary_avg_tracking[index];
                vs = left.points.extra_avg_tracking.aggregate_stats_vs_score[index];
            } else {
                pps = left.points.tertiary;
                apm = left.points.secondary;
                vs = left.points.extra.vs;
            }
        }

        Ok(Ok(GraphUserData {
            name: data.user.username.to_string(),
            replay_url,
            round,
            stats: PlayerStats {
                apm,
                pps,
                vs,
                rd: Some(rd),
                tr: Some(tr),
                glicko: Some(glicko),
                rank: Some(rank.clone()),
            },
            dark_mode,
        }))
    }

    async fn from_stats(
        data: &StatsSubCommand,
        _context: &Context,
    ) -> anyhow::Result<anyhow::Result<GraphUserData>> {
        Ok(Ok(GraphUserData {
            name: format!("{},{},{}", data.pps, data.apm, data.vs),
            replay_url: None,
            round: None,
            stats: PlayerStats {
                apm: data.apm,
                pps: data.pps,
                vs: data.vs,
                rd: None,
                tr: None,
                glicko: None,
                rank: None,
            },
            dark_mode: data.dark_mode,
        }))
    }

    async fn from_average(
        average: &AverageSubCommand,
        context: &Context,
    ) -> anyhow::Result<anyhow::Result<GraphUserData>> {
        let country_str = average
            .country
            .clone()
            .map(|a| format!(":{}", a))
            .unwrap_or(String::new());
        let rank_str = average
            .rank
            .clone()
            .map(|r| {
                let user_rank: UserRank = r.into();
                format!(":{}", user_rank)
            })
            .unwrap_or("".to_string());
        let stats = average_of_rank(
            average.rank.clone().map(|rank| rank.into()),
            average.country.clone().map(|c| c.to_uppercase()),
            &context,
        )
        .await?;
        let rank_str = format!("$avg{}{}", rank_str, country_str);

        let stats = match stats {
            Ok(stats) => stats,
            Err(err) => return Ok(Err(err)),
        };

        let (avg, _count, _lowest) = stats;

        Ok(Ok(GraphUserData {
            name: rank_str,
            replay_url: None,
            round: None,
            stats: avg.into(),
            dark_mode: average.dark_mode,
        }))
    }

    pub async fn get_data(
        &self,
        context: &Context,
    ) -> anyhow::Result<anyhow::Result<GraphUserData>> {
        match self {
            GraphUser::Discord(discord) => Self::from_discord_user(discord, context).await,
            GraphUser::Tetrio(tetrio) => Self::from_tetrio_user(tetrio, context).await,
            GraphUser::Stats(stats) => Self::from_stats(stats, context).await,
            GraphUser::Average(average) => Self::from_average(average, context).await,
        }
    }
}
