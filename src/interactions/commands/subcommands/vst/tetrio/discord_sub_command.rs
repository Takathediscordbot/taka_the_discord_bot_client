use twilight_interactions::command::{CommandModel, CreateCommand, ResolvedUser};

#[derive(CreateCommand, CommandModel, Debug)]
#[command(name = "discord", desc = "Compare a discord user with a tetrio user")]
pub struct DiscordUserSubCommand {
    /// the tetrio user to be compared
    pub tetrio_user: String,
    /// the second discord user to be compared
    pub discord_user: ResolvedUser,
}
