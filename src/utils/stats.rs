#![cfg(feature = "tetrio")]


use std::f64::consts::{self, PI};

use tetrio_api::models::users::user_rank::UserRank;

#[cfg(feature = "tetrio")]
pub struct Stats {
    pub apm: f64,
    pub vs: f64,
    pub pps: f64,
    pub tr: Option<f64>,
    pub glicko: Option<f64>,
    pub rd: Option<f64>,
    pub rank: Option<UserRank>,
    pub vsapm: f64,
    pub app: f64,
    pub dssecond: f64,
    pub dspiece: f64,
    pub dsapppiece: f64,
    pub cheese: f64,
    pub garbage_effi: f64,
    pub weighted_app: f64,
    pub area: f64,
    pub srarea: f64,
    pub stat_rank: f64,
    pub estglicko: f64,
    pub esttr: f64,
    pub atr: Option<f64>,
    pub napm: f64,
    pub npps: f64,
    pub nvs: f64,
    pub napp: f64,
    pub ndss: f64,
    pub ndsp: f64,
    pub nge: f64,
    pub nvsapm: f64,
    pub opener: f64,
    pub plonk: f64,
    pub stride: f64,
    pub infds: f64,
}

#[derive(Clone)]
pub struct PlayerStats {
    pub apm: f64,
    pub pps: f64,
    pub vs: f64,
    pub rd: Option<f64>,
    pub tr: Option<f64>,
    pub glicko: Option<f64>,
    pub rank: Option<UserRank>,
}

#[derive(Clone)]
pub struct PlayerStatsUnwrapped {
    pub apm: f64,
    pub pps: f64,
    pub vs: f64,
    pub rd: f64,
    pub tr: f64,
    pub glicko: f64,
    pub rank: Option<UserRank>,
}

impl From<PlayerStatsUnwrapped> for PlayerStats {
    fn from(val: PlayerStatsUnwrapped) -> Self {
        PlayerStats {
            apm: val.apm,
            pps: val.pps,
            vs: val.vs,
            rd: Some(val.rd),
            tr: Some(val.tr),
            glicko: Some(val.glicko),
            rank: val.rank,
        }
    }
}

pub const APM_WEIGHT: f64 = 1.0;
pub const PPS_WEIGHT: f64 = 45.0;
pub const APP_WEIGHT: f64 = 185.0;
pub const VS_WEIGHT: f64 = 0.444;
pub const DSSECOND_WEIGHT: f64 = 175.0;
pub const DSPIECE_WEIGHT: f64 = 450.0;
pub const DSAPPPIECE_WEIGHT: f64 = 140.0;
pub const VSAPM_WEIGHT: f64 = 60.0;
pub const CHEESE_WEIGHT: f64 = 1.25;
pub const GARBAGEEFFI_WEIGHT: f64 = 315.0;
pub const PPS_SRW: f64 = 135.0;
pub const APP_SRW: f64 = 290.0;
pub const DSPIECE_SRW: f64 = 700.0;
pub const APM_SRW: f64 = 0.0;
pub const VS_SRW: f64 = 0.0;
pub const DSSECOND_SRW: f64 = 0.0;
pub const GARBAGEEFFI_SRW: f64 = 0.0;

pub fn calculate_stats(player_stats: PlayerStats) -> Stats {
    let PlayerStats {
        apm,
        pps,
        vs,
        rd,
        tr,
        glicko,
        rank,
    } = player_stats;
    let original_rd = rd;
    let rd = rd.unwrap_or(60.9);

    let vsapm = vs / apm;
    let app = apm / (pps * 60.0);
    let dssecond = (vs / 100.0) - (apm / 60.0);
    let dspiece = dssecond / pps;
    let dsapppiece = dspiece + app;
    let cheese = (dspiece * 150.0) + ((vsapm - 2.0) * 50.0) + (0.6 - app) * 125.0;
    let garbage_effi = ((app * dssecond) / pps) * 2.0;
    let weighted_app = app - 5.0 * f64::tan(f64::to_radians((cheese / -30.0) + 1.0));

    let area = apm * APM_WEIGHT
        + pps * PPS_WEIGHT
        + vs * VS_WEIGHT
        + app * APP_WEIGHT
        + dssecond * DSSECOND_WEIGHT
        + dspiece * DSPIECE_WEIGHT
        + garbage_effi * GARBAGEEFFI_WEIGHT;

    let srarea = (apm * APM_SRW)
        + (pps * PPS_SRW)
        + (vs * VS_SRW)
        + (app * APP_SRW)
        + (dssecond * DSSECOND_SRW)
        + (dspiece * DSPIECE_SRW)
        + (garbage_effi * GARBAGEEFFI_SRW);
    let stat_rank = (11.2 * f64::atan(((srarea) - 93.0) / 130.0)) + 1.0;

    let stat_rank = if stat_rank <= 0.0 { 0.001 } else { stat_rank };

    let estglicko = 4.0867 * srarea + 186.68;
    let esttr = {
        let temp = (1500.0 - estglicko) * consts::PI;
        let temp2 = f64::powf(15.9056943314 * (rd * rd) + 3527584.25978, 0.5);
        let temp3 = 1.0 + (f64::powf(10.0, temp / temp2));
        25000.0 / temp3
    };

    let atr = tr.map(|tr| esttr - tr);

    let napm = ((apm / srarea)
        / ((0.069 * f64::powf(1.0017, f64::powf(stat_rank, 5.0) / 4700.0)) + stat_rank / 360.0))
        - 1.0;

    let npps = ((pps / srarea)
        / (0.0084264 * (f64::powf(2.14, -2.0 * (stat_rank / 2.7 + 1.03))) - stat_rank / 5750.0
            + 0.0067))
        - 1.0;

    let nvs = ((vs / srarea)
        / (0.1333
            * f64::powf(
                1.0021,
                (f64::powf(stat_rank, 7.0) * (stat_rank / 16.5)) / 1400000.0,
            )
            + stat_rank / 133.0))
        - 1.0;

    let napp = (app
        / (0.1368803292 * f64::powf(1.0024, f64::powf(stat_rank, 5.0) / 2800.0)
            + stat_rank / 54.0))
        - 1.0;

    let ndss = (dssecond
        / (0.01436466667 * f64::powf(4.1, (stat_rank - 9.6) / 2.9) + stat_rank / 140.0 + 0.01))
        - 1.0;

    let ndsp = (dspiece
        / (0.02136327583 * (f64::powf(14.0, (stat_rank - 14.75) / 3.9))
            + stat_rank / 152.0
            + 0.022))
        - 1.0;

    let nge = (garbage_effi
        / (stat_rank / 350.0 + 0.005948424455 * f64::powf(3.8, (stat_rank - 6.1) / 4.0) + 0.006))
        - 1.0;

    let nvsapm = (vsapm / (-(f64::powi((stat_rank - 16.0) / 36.0, 2)) + 2.133)) - 1.0;

    let opener =
        ((napm + (npps * 0.75) + (nvsapm * -10.0) + (napp * 0.75) + (ndsp * -0.25)) / 3.5) + 0.5;

    let plonk = ((nge + napp + (ndsp * 0.75) + (npps * -1.0)) / 2.73) + 0.5;

    let stride = (((napm * -0.25) + npps + (napp * -2.0) + (ndsp * -0.5)) * 0.79) + 0.5;

    let infds =
        ((ndsp + (napp * -0.75) + (napm * 0.5) + (nvsapm * 1.5) + (npps * 0.5)) * 0.9) + 0.5;

    Stats {
        apm,
        vs,
        pps,
        tr,
        glicko,
        rd: original_rd,
        rank,
        vsapm,
        app,
        dssecond,
        dspiece,
        dsapppiece,
        cheese,
        garbage_effi,
        weighted_app,
        area,
        srarea,
        stat_rank,
        estglicko,
        esttr,
        atr,
        napm,
        npps,
        nvs,
        napp,
        ndss,
        ndsp,
        nge,
        nvsapm,
        opener,
        plonk,
        stride,
        infds,
    }
}

pub struct StringifiedStats {
    pub apm: String,
    pub vs: String,
    pub pps: String,
    pub tr: Option<String>,
    pub glicko: Option<String>,
    pub rd: Option<String>,
    pub vsapm: String,
    pub app: String,
    pub dssecond: String,
    pub dspiece: String,
    pub dsapppiece: String,
    pub cheese: String,
    pub garbage_effi: String,
    pub weighted_app: String,
    pub area: String,
    pub esttr: String,
    pub atr: Option<String>,
    pub opener: String,
    pub plonk: String,
    pub stride: String,
    pub infds: String,
}

pub fn stringified_stats(stats: PlayerStats) -> StringifiedStats {
    let stats = calculate_stats(stats);
    stringify_stats(stats)
}

pub fn stringify_stats(stats: Stats) -> StringifiedStats {
    let Stats {
        apm,
        pps,
        vs,
        rd,
        tr,
        glicko,
        vsapm,
        app,
        dssecond,
        dspiece,
        dsapppiece,
        cheese,
        garbage_effi,
        weighted_app,
        area,
        srarea: _,
        stat_rank: _,
        estglicko: _,
        esttr,
        atr,
        napm: _,
        npps: _,
        nvs: _,
        napp: _,
        ndss: _,
        ndsp: _,
        nge: _,
        nvsapm: _,
        opener,
        plonk,
        stride,
        infds,
        rank: _,
    } = stats;

    StringifiedStats {
        apm: format!("{apm:.2}"),
        pps: format!("{pps:.2}"),
        vs: format!("{vs:.2}"),
        app: format!("{app:.4}"),
        dssecond: format!("{dssecond:.4}"),
        dspiece: format!("{dspiece:.4}"),
        dsapppiece: format!("{dsapppiece:.4}"),
        vsapm: format!("{vsapm:.4}"),
        garbage_effi: format!("{garbage_effi:.4}"),
        cheese: format!("{cheese:.4}"),
        weighted_app: format!("{weighted_app:.4}"),
        area: format!("{area:.4}"),
        tr: tr.map(|tr| format!("{tr:.1}")),
        esttr: format!("{esttr:.1}"),
        atr: atr.map(|atr| format!("{}{atr:.2}", if atr >= 0.0 { "+" } else { "" })),
        glicko: glicko.map(|glicko| format!("{glicko:.2}")),
        rd: rd.map(|rd| format!("{rd:.2}")),
        opener: format!("{opener:.4}"),
        plonk: format!("{plonk:.4}"),
        stride: format!("{stride:.4}"),
        infds: format!("{infds:.4}"),
    }
}

pub fn q() -> f64 {
    f64::ln(10.0) / 400.0
}

pub fn calculate_win_chance(glicko0: f64, glicko1: f64, rd0: f64, rd1: f64) -> f64 {
    1.0 / (1.0
        + 10.0f64.powf(
            (glicko1 - glicko0)
                / (400.0
                    * f64::sqrt(1.0 + (3.0 * q() * q() * (rd0 * rd0 + rd1 * rd1)) / (PI * PI))),
        ))
}
