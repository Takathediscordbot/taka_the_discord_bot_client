use twilight_interactions::command::{CommandModel, CreateCommand, ResolvedUser};

#[derive(CreateCommand, CommandModel, Debug)]
#[command(name = "stats", desc = "Compare a discord user with stats")]
pub struct StatsSubCommand {
    /// the discord user to be compared
    pub discord_user: ResolvedUser,
    /// The pps to be compared
    pub pps: f64,
    /// The apm to be compared
    pub apm: f64,
    /// The vs to be compared
    pub vs: f64,
}
