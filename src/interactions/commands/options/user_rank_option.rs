use tetrio_api::models::users::user_rank::UserRank;
use twilight_interactions::command::{CommandOption, CreateOption};

#[derive(CreateOption, CommandOption, Clone, Debug)]
pub enum UserRankOption {
    #[option(name = "X", value = "X")]
    X,
    #[option(name = "U", value = "U")]
    U,
    #[option(name = "SS", value = "SS")]
    SS,
    #[option(name = "S+", value = "S+")]
    SPlus,
    #[option(name = "S", value = "S")]
    S,
    #[option(name = "S-", value = "S-")]
    SMinus,
    #[option(name = "A+", value = "A+")]
    APlus,
    #[option(name = "A", value = "A")]
    A,
    #[option(name = "A-", value = "A-")]
    AMinus,
    #[option(name = "B+", value = "B+")]
    BPlus,
    #[option(name = "B", value = "B")]
    B,
    #[option(name = "B-", value = "B-")]
    BMinus,
    #[option(name = "C+", value = "C+")]
    CPlus,
    #[option(name = "C", value = "C")]
    C,
    #[option(name = "C-", value = "C-")]
    CMinus,
    #[option(name = "D+", value = "D+")]
    DPlus,
    #[option(name = "D", value = "D")]
    D,
}

impl From<UserRankOption> for UserRank {
    fn from(val: UserRankOption) -> Self {
        match val {
            UserRankOption::X => UserRank::X,
            UserRankOption::U => UserRank::U,
            UserRankOption::SS => UserRank::SS,
            UserRankOption::SPlus => UserRank::SPlus,
            UserRankOption::S => UserRank::S,
            UserRankOption::SMinus => UserRank::SMinus,
            UserRankOption::APlus => UserRank::APlus,
            UserRankOption::A => UserRank::A,
            UserRankOption::AMinus => UserRank::AMinus,
            UserRankOption::BPlus => UserRank::BPlus,
            UserRankOption::B => UserRank::B,
            UserRankOption::BMinus => UserRank::BMinus,
            UserRankOption::CPlus => UserRank::CPlus,
            UserRankOption::C => UserRank::C,
            UserRankOption::CMinus => UserRank::CMinus,
            UserRankOption::DPlus => UserRank::DPlus,
            UserRankOption::D => UserRank::D,
        }
    }
}
