use twilight_interactions::command::{CommandModel, CreateCommand};

#[derive(CreateCommand, CommandModel, Debug)]
#[command(name = "tetrio", desc = "Use a tetrio user")]
pub struct TetrioUserSubCommand {
    /// A tetrio username or id
    pub tetrio_user: String,
    /// which game to choose from the record
    pub game_number: Option<i64>,
}
