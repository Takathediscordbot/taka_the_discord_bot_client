use twilight_interactions::command::{CommandModel, CreateCommand};

use crate::interactions::commands::options::user_rank_option::UserRankOption;

#[derive(CreateCommand, CommandModel, Debug)]
#[command(
    name = "tetrio",
    desc = "Compare the average of a rank with a tetrio user"
)]
pub struct TetrioSubCommand {
    /// The tetrio user
    pub tetrio_user: String,

    /// The average rank
    pub average_rank: Option<UserRankOption>,
    /// The average country
    pub average_country: Option<String>,
}
