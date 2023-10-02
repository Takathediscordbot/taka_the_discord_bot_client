use twilight_interactions::command::{CommandModel, CreateCommand};

pub mod stats_sub_command;
#[derive(CreateCommand, CommandModel, Debug)]
#[command(name = "stats", desc = "Compare two set of stats")]
pub enum StatsSubCommandGroup {
    #[command(name = "stats")]
    Stats(stats_sub_command::StatsSubCommand),
}
