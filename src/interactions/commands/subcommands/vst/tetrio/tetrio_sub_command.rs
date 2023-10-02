use twilight_interactions::command::{CommandModel, CreateCommand};

#[derive(CreateCommand, CommandModel, Debug)]
#[command(
    name = "tetrio",
    desc = "Compare a tetrio user with another tetrio user"
)]
pub struct TetrioSubCommand {
    /// the first tetrio user to be compared
    pub user1: String,
    /// the second tetrio user to be compared
    pub user2: String,
}
