use twilight_interactions::command::{CommandModel, CreateCommand};

use crate::interactions::commands::options::user_rank_option::UserRankOption;

#[derive(CreateCommand, CommandModel, Debug)]
#[command(name = "stats", desc = "Compare stats with other stats")]
pub struct StatsSubCommand {
    /// The pps to be compared
    pub pps: f64,
    /// The apm to be compared
    pub apm: f64,
    /// The vs to be compared
    pub vs: f64,

    /// The rank to be compared
    pub average_rank: Option<UserRankOption>,
    /// The average country
    pub average_country: Option<String>,
}
