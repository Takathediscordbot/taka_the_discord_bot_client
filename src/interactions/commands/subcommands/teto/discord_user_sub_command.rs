use twilight_interactions::command::{CommandModel, CreateCommand, ResolvedUser};

#[derive(CreateCommand, CommandModel, Debug)]
#[command(name = "discord", desc = "Use a discord user")]
pub struct DiscordUserSubCommand {
    /// the discord user to be selected
    pub user: ResolvedUser,
}
