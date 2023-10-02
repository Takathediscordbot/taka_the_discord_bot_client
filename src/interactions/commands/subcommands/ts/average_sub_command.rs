use twilight_interactions::command::{CommandModel, CreateCommand};

use crate::interactions::commands::options::user_rank_option::UserRankOption;

#[derive(CreateCommand, CommandModel, Debug)]
#[command(name = "average", desc = "Use the average stats")]
pub struct AverageSubCommand {
    /// the rank to get the average from
    pub rank: Option<UserRankOption>,
    /// use detailed informations
    pub details: Option<bool>,
    /// Country to limit the stats to
    pub country: Option<String>,
}
