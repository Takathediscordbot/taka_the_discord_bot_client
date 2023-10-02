use twilight_interactions::command::{CommandModel, CreateCommand, ResolvedUser};

#[derive(CreateCommand, CommandModel, Debug)]
#[command(
    name = "discord",
    desc = "Compare a discord user with another discord user"
)]
pub struct DiscordUserSubCommand {
    /// the first discord user to be compared
    pub user1: ResolvedUser,
    /// the second discord user to be compared
    pub user2: ResolvedUser,
}
