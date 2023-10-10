use crate::{
    interactions::commands::{
        add_silly_image::AddSillyImage,
        add_silly_text::AddSillyText,
        create_silly_command::CreateSillyCommand,
        help::HelpCommand,
        test_mode::TestMode,
        tetrio_commands::{
            lb::LbCommand, psq::PsqCommand,
            rlb::RLbCommand, sq::SqCommand, teto::TetoCommand,
            tetra_command::TetraCommand, ts::TsCommand, vs::VsCommand, vsr::VsrCommand,
            vst::VstCommand,
        }, silly_command::SillyCommand, ping_command::PingCommand, reload_commands::ReloadCommands, rng::RngCommand, eight_ball::EightBallCommand, export_silly_commands::ExportSillyCommands, add_preference::AddPreferenceCommand, load_silly_command_images::LoadSillyCommandImages,
    },
    utils::box_commands::{PhantomCommand, PhantomCommandTrait},
};

pub mod add_silly_image;
pub mod add_silly_text;
pub mod create_silly_command;
pub mod help;
pub mod models;
pub mod options;
pub mod subcommands;
pub mod test_mode;
pub mod tetrio_commands;
pub mod silly_command;
pub mod ping_command;
pub mod reload_commands;
pub mod rng;
pub mod eight_ball;
pub mod export_silly_commands;
pub mod add_preference;
pub mod load_silly_command_images;

pub fn get_commands() -> Vec<Box<dyn PhantomCommandTrait>> {
    vec![
        Box::new(PhantomCommand::<PingCommand>::new()),
        Box::new(PhantomCommand::<ReloadCommands>::new()),
        Box::new(PhantomCommand::<TetraCommand>::new()),
        Box::new(PhantomCommand::<TetoCommand>::new()),
        Box::new(PhantomCommand::<TsCommand>::new()),
        Box::new(PhantomCommand::<VstCommand>::new()),
        Box::new(PhantomCommand::<VsCommand>::new()),
        Box::new(PhantomCommand::<VsrCommand>::new()),
        Box::new(PhantomCommand::<SqCommand>::new()),
        Box::new(PhantomCommand::<PsqCommand>::new()),
        Box::new(PhantomCommand::<LbCommand>::new()),
        Box::new(PhantomCommand::<RLbCommand>::new()),
        Box::new(PhantomCommand::<CreateSillyCommand>::new()),
        Box::new(PhantomCommand::<AddSillyImage>::new()),
        Box::new(PhantomCommand::<AddSillyText>::new()),
        Box::new(PhantomCommand::<HelpCommand>::new()),
        Box::new(PhantomCommand::<TestMode>::new()),
        Box::new(PhantomCommand::<SillyCommand>::new()),
        Box::new(PhantomCommand::<RngCommand>::new()),
        Box::new(PhantomCommand::<EightBallCommand>::new()),
        Box::new(PhantomCommand::<ExportSillyCommands>::new()),
        Box::new(PhantomCommand::<AddPreferenceCommand>::new()),
        Box::new(PhantomCommand::<LoadSillyCommandImages>::new()),
    ]
}
