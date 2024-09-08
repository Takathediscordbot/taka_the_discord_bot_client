
#![cfg(feature = "tetrio")]


use anyhow::anyhow;
use tetrio_api::models::{labs::league_ranks::LeagueRank, packet::Packet, users::user_rank::UserRank};

use crate::context::Context;

use super::stats::PlayerStatsUnwrapped;

pub async fn average_of_rank(
    rank: Option<UserRank>, 
    context: &Context<'_>,
) -> anyhow::Result<anyhow::Result<(PlayerStatsUnwrapped, usize, f64)>> {
    let Packet { data: Some(data), .. } = context.tetrio_client.fetch_leagueranks().await? else {
        return Err(anyhow!("Couldn't find user ranks data!"));
    };

    let data = match &rank {
        Some(rank) => data.data.ranks.get(&rank).ok_or(anyhow!("Couldn't find stats for rank {}", &rank))?.clone(),
        None => {
            let sum = data.data.ranks.values().fold(LeagueRank { pos: 0, percentile: 0.0, tr: 0.0, targettr: 0.0, apm: None, pps: None, vs: None, count: 0 },|acc: LeagueRank, value: &LeagueRank| LeagueRank {
            pos: 0,
            percentile: 0.0,
            tr: acc.tr + value.tr,
            targettr: 0.0,
            apm: Some(acc.apm.unwrap_or(0.0) + value.apm.unwrap_or(0.0)),
            pps: Some(acc.pps.unwrap_or(0.0) + value.pps.unwrap_or(0.0)),
            vs:  Some( acc.vs.unwrap_or(0.0) + value.vs.unwrap_or(0.0)),
            count: acc.count + value.count,
        });
        
        LeagueRank {
            pos: sum.pos,
            percentile: sum.percentile,
            tr: sum.tr / data.data.ranks.len() as f64,
            targettr: sum.targettr,
            apm: sum.apm.map(|v| v / data.data.ranks.len() as f64),
            pps: sum.pps.map(|v| v / data.data.ranks.len() as f64),
            vs: sum.vs.map(|v| v / data.data.ranks.len() as f64),
            count: sum.count,
        }
    }

    };

    Ok(Ok((PlayerStatsUnwrapped {
        apm: data.apm.unwrap_or(0.),
        pps: data.pps.unwrap_or(0.),
        vs: data.vs.unwrap_or(0.),
        rd: 60.0,
        tr: data.tr,
        glicko: 0.0,
        rank
    }, data.count as usize, data.tr)))

}

// pub async fn average_of_rank_in_country(
//     rank: UserRank,
//     country: String,
//     context: &Context<'_>,
// ) -> anyhow::Result<anyhow::Result<(PlayerStatsUnwrapped, usize, f64)>> {
//     let packet = context
//         .fetch_full_leaderboard(country.as_deref())
//         .await?;

//     let  Some(data) = &packet.data else {
//         return Ok(Err(anyhow!("Couldn't find tetra league data")));
//     };

//     let stats = data.users.iter().filter_map(|u| {
//         if let (Some(pps), Some(apm), Some(vs), tr, Some(glicko), Some(rd), user_rank) = (
//             u.league.pps,
//             u.league.apm,
//             u.league.vs,
//             u.league.rating,
//             u.league.glicko,
//             u.league.rd,
//             u.league.rank.clone(),
//         ) {
//             if let Some(rank) = &rank {
//                 if &u.league.rank == rank {
//                     Some(PlayerStatsUnwrapped {
//                         apm,
//                         pps,
//                         vs,
//                         rd,
//                         tr,
//                         glicko,
//                         rank: Some(user_rank),
//                     })
//                 } else {
//                     None
//                 }
//             } else {
//                 Some(PlayerStatsUnwrapped {
//                     apm,
//                     pps,
//                     vs,
//                     rd,
//                     tr,
//                     glicko,
//                     rank: Some(user_rank),
//                 })
//             }
//         } else {
//             None
//         }
//     });

//     let count = stats.clone().count() as f64;
//     let mut acc = stats.clone().fold(
//         PlayerStatsUnwrapped {
//             apm: 0.0,
//             pps: 0.0,
//             vs: 0.0,
//             rd: 0.0,
//             tr: 0.0,
//             glicko: 0.0,
//             rank: rank.clone(),
//         },
//         |mut acc, stats| {
//             acc.apm += stats.apm;
//             acc.pps += stats.pps;
//             acc.vs += stats.vs;
//             acc.tr += stats.tr;
//             acc.glicko += stats.glicko;
//             acc.rd += stats.rd;

//             acc
//         },
//     );

//     if count != 0.0 {
//         acc.apm /= count;
//         acc.pps /= count;
//         acc.vs /= count;
//         acc.tr /= count;
//         acc.glicko /= count;
//         acc.rd /= count;
//     }

//     Ok(Ok((
//         acc,
//         count as usize,
//         stats
//             .filter(|stats| stats.rd < 65.0)
//             .map(|a| a.tr)
//             .fold(f64::INFINITY, |a, b| a.min(b)),
//     )))
// }
