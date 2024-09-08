use twilight_interactions::command::{CommandModel, CreateCommand};

#[derive(CreateCommand, CommandModel, Debug)]
#[command(name = "stats", desc = "Use tetrio stats")]
pub struct StatsSubCommand {
    /// pieces per second
    pub pps: f64,
    /// attacks per minute
    pub apm: f64,
    /// vs score
    pub vs: f64,
    /// rd, defaults to 60
    #[allow(unused)]
    pub rd: Option<f64>,
    /// Show details
    pub show_details: Option<bool>,
}
