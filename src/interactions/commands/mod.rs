use crate::{
    interactions::commands::{
        help::HelpCommand,
        ping_command::PingCommand, reload_commands::ReloadCommands, rng::RngCommand, eight_ball::EightBallCommand,
    },
    utils::box_commands::{PhantomCommand, PhantomCommandTrait},
};

#[cfg(feature = "tetrio")]
use crate::interactions::commands::tetrio_commands::{
    lb::LbCommand, psq::PsqCommand,
    rlb::RLbCommand, sq::SqCommand, 
    ts::TsCommand, vs::VsCommand, vsr::VsrCommand,
    vst::VstCommand,
};


pub mod help;
pub mod models;
pub mod options;
pub mod subcommands;
#[cfg(feature = "tetrio")]
pub mod tetrio_commands;
pub mod ping_command;
pub mod reload_commands;
pub mod rng;
pub mod eight_ball;
#[cfg(feature = "database")]
pub mod silly_commands_utils;

pub fn get_commands() -> Vec<Box<dyn PhantomCommandTrait>> {
    #[cfg(feature = "database")] 
    use silly_commands_utils::{
        add_preference::AddPreferenceCommand,
        add_silly_text::AddSillyText,
        add_silly_image::AddSillyImage,
        export_silly_commands::ExportSillyCommands,
        load_silly_command_images::LoadSillyCommandImages,
        silly_command::SillyCommand,
        create_silly_command::CreateSillyCommand,    
    };

    #[cfg(feature = "html_server_image_generation")]
    use crate::
        interactions::commands::
            tetrio_commands::{
                teto::TetoCommand,
                tetra_command::TetraCommand
            }
    ;

    vec![
        #[cfg(feature = "html_server_image_generation")]
        Box::new(PhantomCommand::<TetoCommand>::new()),
        #[cfg(feature = "html_server_image_generation")]
        Box::new(PhantomCommand::<TetraCommand>::new()),
        Box::new(PhantomCommand::<PingCommand>::new()),
        Box::new(PhantomCommand::<ReloadCommands>::new()),
        #[cfg(feature = "tetrio")]
        Box::new(PhantomCommand::<TsCommand>::new()),
        #[cfg(feature = "tetrio")]
        Box::new(PhantomCommand::<VstCommand>::new()),
        #[cfg(feature = "tetrio")]
        Box::new(PhantomCommand::<VsCommand>::new()),
        #[cfg(feature = "tetrio")]
        Box::new(PhantomCommand::<VsrCommand>::new()),
        #[cfg(feature = "tetrio")]
        Box::new(PhantomCommand::<SqCommand>::new()),
        #[cfg(feature = "tetrio")]
        Box::new(PhantomCommand::<PsqCommand>::new()),
        #[cfg(feature = "tetrio")]
        Box::new(PhantomCommand::<LbCommand>::new()),
        #[cfg(feature = "tetrio")]
        Box::new(PhantomCommand::<RLbCommand>::new()),
        Box::new(PhantomCommand::<HelpCommand>::new()),
        Box::new(PhantomCommand::<RngCommand>::new()),
        Box::new(PhantomCommand::<EightBallCommand>::new()),
        #[cfg(feature = "database")]
        Box::new(PhantomCommand::<ExportSillyCommands>::new()),
        #[cfg(feature = "database")]
        Box::new(PhantomCommand::<AddPreferenceCommand>::new()),
        #[cfg(feature = "database")]
        Box::new(PhantomCommand::<CreateSillyCommand>::new()),
        #[cfg(feature = "database")]
        Box::new(PhantomCommand::<SillyCommand>::new()),
        #[cfg(feature = "database")]
        Box::new(PhantomCommand::<AddSillyImage>::new()),
        #[cfg(feature = "database")]
        Box::new(PhantomCommand::<AddSillyText>::new()),
        #[cfg(feature = "database")]
        Box::new(PhantomCommand::<LoadSillyCommandImages>::new()),
    ]
}
