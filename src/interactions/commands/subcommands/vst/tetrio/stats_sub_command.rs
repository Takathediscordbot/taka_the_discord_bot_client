use twilight_interactions::command::{CommandModel, CreateCommand};

#[derive(CreateCommand, CommandModel, Debug)]
#[command(name = "stats", desc = "Compare a tetrio user with stats")]
pub struct StatsSubCommand {
    /// the discord user to be compared
    pub tetrio_user: String,
    /// The pps to be compared
    pub pps: f64,
    /// The apm to be compared
    pub apm: f64,
    /// The vs to be compared
    pub vs: f64,
}
