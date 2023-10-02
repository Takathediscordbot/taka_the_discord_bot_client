use twilight_interactions::command::{CommandOption, CreateOption};

#[allow(clippy::upper_case_acronyms)]
#[derive(CreateOption, CommandOption, Debug)]
pub enum UserStatOption {
    #[option(name = "apm", value = "apm")]
    APM,
    #[option(name = "pps", value = "pps")]
    PPS,
    #[option(name = "vs", value = "vs")]
    VS,
    #[option(name = "app", value = "app")]
    APP,
    #[option(name = "downstack per piece", value = "dspiece")]
    DSPIECE,
    #[option(name = "downstack per second", value = "dssecond")]
    DSSECOND,
    #[option(name = "cheese index", value = "cheese")]
    CHEESE,
    #[option(name = "garbage efficiency", value = "ge")]
    GE,
    #[option(name = "area", value = "area")]
    AREA,
    #[option(name = "weighted app", value = "wapp")]
    WAPP,
    #[option(name = "VS / APM", value = "vsapm")]
    VSAPM,
    #[option(name = "Downstack + APP / Piece", value = "dsapppiece")]
    DSAPPPIECE,
    #[option(name = "TR", value = "tr")]
    TR,
    #[option(name = "Estimated TR", value = "esttr")]
    ESTTR,
    #[option(name = "Accuracy of Estimated TR", value = "atr")]
    ATR,
    #[option(name = "opener", value = "opener")]
    OPENER,
    #[option(name = "plonk", value = "plonk")]
    PLONK,
    #[option(name = "stride", value = "stride")]
    STRIDE,
    #[option(name = "infds", value = "infds")]
    INFDS,
    #[option(name = "wins", value = "wins")]
    WINS,
    #[option(name = "games", value = "games")]
    GAMES,
    #[option(name = "win rate", value = "winrate")]
    WR,
}
