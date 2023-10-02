use twilight_interactions::command::{CommandModel, CreateCommand};

use crate::utils::box_commands::CommandBox;

pub mod discord_sub_command;
pub mod stats_sub_command;
pub mod tetrio_sub_command;

#[derive(CreateCommand, CommandModel)]
#[command(name = "tetrio", desc = "Use a tetrio user for the first user")]
pub enum TetrioSubCommandGroup {
    #[command(name = "discord")]
    Discord(CommandBox<discord_sub_command::DiscordUserSubCommand>),
    #[command(name = "tetrio")]
    Tetrio(tetrio_sub_command::TetrioSubCommand),
    #[command(name = "stats")]
    Stats(stats_sub_command::StatsSubCommand),
}
