use twilight_interactions::command::{CommandModel, CreateCommand};

#[derive(CreateCommand, CommandModel, Debug)]
#[command(name = "tetrio", desc = "Use a tetrio user")]
pub struct TetrioUserSubCommand {
    /// A tetrio username or id
    pub tetrio_user: String,
    /// use detailed informations
    pub details: Option<bool>,
    /// tetra league game number
    pub tetra_league_game: Option<i64>,
    /// tetra league round number
    pub tetra_league_round: Option<i64>,
}
