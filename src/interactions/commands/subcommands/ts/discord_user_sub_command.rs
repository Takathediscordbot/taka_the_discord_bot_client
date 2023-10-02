use twilight_interactions::command::{CommandModel, CreateCommand, ResolvedUser};

#[derive(CreateCommand, CommandModel, Debug)]
#[command(name = "discord", desc = "Use a discord user")]
pub struct DiscordUserSubCommand {
    /// the discord user to be selected
    pub user: ResolvedUser,
    /// use detailed informations
    pub details: Option<bool>,
    /// tetra league game number
    pub tetra_league_game: Option<i64>,
    /// tetra league round number
    pub tetra_league_round: Option<i64>,
}
