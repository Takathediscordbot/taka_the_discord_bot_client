

    use twilight_interactions::command::{CommandModel, CreateCommand};
    use twilight_model::channel::Attachment;

    #[derive(CreateCommand, CommandModel, Debug)]
    #[command(name = "replay", desc = "Use a ttrm replay")]
    pub struct TetrioReplaySubCommand {
        /// The replay to analyze
        pub replay: Attachment,

    }
    