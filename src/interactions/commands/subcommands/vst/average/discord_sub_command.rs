use twilight_interactions::command::{CommandModel, CreateCommand, ResolvedUser};

use crate::interactions::commands::options::user_rank_option::UserRankOption;

#[derive(CreateCommand, CommandModel, Debug)]
#[command(
    name = "discord",
    desc = "Compare the average of a rank with a discord user"
)]
pub struct DiscordSubCommand {
    /// The discord user
    pub discord_user: ResolvedUser,

    /// The average rank
    pub average_rank: Option<UserRankOption>,
    /// The average country
    pub average_country: Option<String>,
}
