use twilight_interactions::command::{CommandModel, CreateCommand};

#[derive(CreateCommand, CommandModel, Debug)]
#[command(name = "stats", desc = "Compare stats with other stats")]
pub struct StatsSubCommand {
    /// The first pps to be compared
    pub pps1: f64,
    /// The first apm to be compared
    pub apm1: f64,
    /// The first vs to be compared
    pub vs1: f64,

    /// The first pps to be compared
    pub pps2: f64,
    /// The first apm to be compared
    pub apm2: f64,
    /// The first vs to be compared
    pub vs2: f64,
}
