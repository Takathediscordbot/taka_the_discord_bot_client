use twilight_interactions::command::{CommandModel, CreateCommand};

use crate::utils::box_commands::CommandBox;

pub mod discord_sub_command;
pub mod stats_sub_command;

#[derive(CreateCommand, CommandModel)]
#[command(name = "discord", desc = "Use a discord user for the first user")]
pub enum DiscordSubCommandGroup {
    #[command(name = "discord")]
    Discord(CommandBox<discord_sub_command::DiscordUserSubCommand>),

    #[command(name = "stats")]
    Stats(CommandBox<stats_sub_command::StatsSubCommand>),
}
