use twilight_interactions::command::{CommandModel, CreateCommand};

use crate::interactions::commands::options::user_rank_option::UserRankOption;

#[derive(CreateCommand, CommandModel, Debug)]
#[command(name = "average", desc = "Compare two averages of ranks")]
pub struct AverageSubCommand {
    /// The first rank
    pub average_rank1: Option<UserRankOption>,
    /// The first country
    pub average_country1: Option<String>,

    /// The second rank
    pub average_rank2: Option<UserRankOption>,
    /// The second country
    pub average_country2: Option<String>,
}
