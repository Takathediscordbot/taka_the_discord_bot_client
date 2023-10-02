use twilight_interactions::command::{CommandModel, CreateCommand};

use crate::utils::box_commands::CommandBox;

pub mod average_sub_command;
pub mod discord_sub_command;
pub mod stats_sub_command;
pub mod tetrio_sub_command;
#[derive(CreateCommand, CommandModel)]
#[command(name = "average", desc = "Compare average stats of a rank")]
pub enum AverageSubCommandGroup {
    #[command(name = "stats")]
    Stats(CommandBox<stats_sub_command::StatsSubCommand>),
    #[command(name = "discord")]
    Discord(CommandBox<discord_sub_command::DiscordSubCommand>),
    #[command(name = "tetrio")]
    Tetrio(tetrio_sub_command::TetrioSubCommand),
    #[command(name = "average")]
    Average(average_sub_command::AverageSubCommand),
}
