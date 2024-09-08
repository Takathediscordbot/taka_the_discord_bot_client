

    use twilight_interactions::command::{CommandModel, CreateCommand};
    use twilight_model::channel::Attachment;

    #[derive(CreateCommand, CommandModel, Debug)]
    #[command(name = "replay", desc = "Use a ttrm replay")]
    pub struct TetrioReplaySubCommand {
        /// The replay to analyze
        pub replay: Attachment,
        /// The user to analyze
        pub user: String,
        /// which game to choose from the record
        pub game_number: Option<i64>,
        /// show details
        pub show_details: Option<bool>,
    }
    