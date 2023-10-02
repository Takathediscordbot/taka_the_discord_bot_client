use std::sync::Arc;

use anyhow::anyhow;
use tetrio_api::models::users::user_rank::UserRank;

use crate::context::Context;

use super::stats::PlayerStatsUnwrapped;

pub async fn average_of_rank(
    rank: Option<UserRank>,
    country: Option<String>,
    context: Arc<Context>,
) -> anyhow::Result<anyhow::Result<(PlayerStatsUnwrapped, usize, f64)>> {
    let packet = context
        .tetrio_client
        .fetch_full_league_leaderboard(country)
        .await?;

    let  Some(data) = &packet.data else {
        return Ok(Err(anyhow!("Couldn't find tetra league data")));
    };

    let stats = data.users.iter().filter_map(|u| {
        if let (Some(pps), Some(apm), Some(vs), tr, Some(glicko), Some(rd), user_rank) = (
            u.league.pps,
            u.league.apm,
            u.league.vs,
            u.league.rating,
            u.league.glicko,
            u.league.rd,
            u.league.rank.clone(),
        ) {
            if let Some(rank) = &rank {
                if &u.league.rank == rank {
                    Some(PlayerStatsUnwrapped {
                        apm,
                        pps,
                        vs,
                        rd,
                        tr,
                        glicko,
                        rank: Some(user_rank),
                    })
                } else {
                    None
                }
            } else {
                Some(PlayerStatsUnwrapped {
                    apm,
                    pps,
                    vs,
                    rd,
                    tr,
                    glicko,
                    rank: Some(user_rank),
                })
            }
        } else {
            None
        }
    });

    let count = stats.clone().count() as f64;
    let mut acc = stats.clone().fold(
        PlayerStatsUnwrapped {
            apm: 0.0,
            pps: 0.0,
            vs: 0.0,
            rd: 0.0,
            tr: 0.0,
            glicko: 0.0,
            rank: rank.clone(),
        },
        |mut acc, stats| {
            acc.apm += stats.apm;
            acc.pps += stats.pps;
            acc.vs += stats.vs;
            acc.tr += stats.tr;
            acc.glicko += stats.glicko;
            acc.rd += stats.rd;

            acc
        },
    );

    if count != 0.0 {
        acc.apm /= count;
        acc.pps /= count;
        acc.vs /= count;
        acc.tr /= count;
        acc.glicko /= count;
        acc.rd /= count;
    }

    Ok(Ok((
        acc,
        count as usize,
        stats
            .filter(|stats| stats.rd < 65.0)
            .map(|a| a.tr)
            .fold(f64::INFINITY, |a, b| a.min(b)),
    )))
}
