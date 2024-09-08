use std::borrow::Cow;


use anyhow::anyhow;
use common::Average;
use common::replay::ttrm::models::events::Event;
use tetrio_api::http::parameters::personal_user_records::{PersonalLeaderboard, PersonalRecordsQuery};
use twilight_interactions::command::{CommandInputData, CommandModel, CreateCommand};

use twilight_model::application::interaction::application_command::CommandData;
use twilight_model::channel::message::embed::EmbedField;
use twilight_model::gateway::payload::incoming::InteractionCreate;
use twilight_model::http::attachment::Attachment;

use twilight_util::builder::embed::{EmbedBuilder, ImageSource};

use crate::context::Context;

use crate::interactions::commands::subcommands::ts::ttrm_replay_sub_command::TetrioReplaySubCommand;
use crate::utils::average_of_rank::average_of_rank;
use crate::utils::box_commands::{CommandBox, RunnableCommand};
use crate::utils::create_embed::create_embed;

use crate::utils::stats::{stringified_stats, PlayerStats, StringifiedStats};

use crate::interactions::commands::subcommands::ts::average_sub_command::AverageSubCommand;
use crate::interactions::commands::subcommands::ts::discord_user_sub_command::DiscordUserSubCommand;
use crate::interactions::commands::subcommands::ts::stats_sub_command::StatsSubCommand;
use crate::interactions::commands::subcommands::ts::tetrio_user_sub_command::TetrioUserSubCommand;
use crate::utils::timer::Timer;

#[derive(CreateCommand, CommandModel)]
#[command(name = "ts", desc = "Calculate the tetrio stats for a user")]
pub enum TsCommand {
    #[command(name = "discord")]
    /// Fetch data from a discord user
    Discord(CommandBox<DiscordUserSubCommand>),
    #[command(name = "tetrio")]
    /// Fetch data from a tetrio user
    Tetrio(TetrioUserSubCommand),
    /// Use tetrio stats
    #[command(name = "stats")]
    Stats(StatsSubCommand),

    /// Use tetrio replay
    #[command(name = "replay")]
    Replay(TetrioReplaySubCommand),

    #[command(name = "average")]
    /// Use average stats
    Average(AverageSubCommand),
}

impl TsCommand {
    async fn with_user(
        id: String,
        interaction: &InteractionCreate,
        show_details: bool,
        tetra_league_game: Option<i64>,
        tetra_league_round: Option<i64>,
        context: &Context<'_>,
    ) -> anyhow::Result<anyhow::Result<()>> {

        let tetrio_user = context.tetrio_client.fetch_user_info(&id).await?;

        let Some(data) = &tetrio_user.data else {
            return Ok(Err(anyhow::anyhow!("❌ No data has been found. User might be anonymous or banned.")));
        };

        let tetrio_league_summary = context.tetrio_client.fetch_user_league_summaries(&id).await?;

        let Some(league_data) = &tetrio_league_summary.data else {
            return Ok(Err(anyhow::anyhow!("❌ No data has been found. User might be anonymous or banned.")));
        };

        let (id, username, Some(mut apm), Some(mut pps), Some(mut vs), rank, tr, Some(glicko), Some(rd)) = (&data.id, &data.username, league_data.apm, league_data.pps, league_data.vs, &league_data.rank, league_data.tr, league_data.glicko, league_data.rd) else {
            return Ok(Err(anyhow::anyhow!("❌ No tetra league stats have been found.")));
        };

        let tetra_league_game_str = if let Some(mut tetra_league_game) = tetra_league_game {
            if tetra_league_game <= 0 {
                tetra_league_game = 1;
            }

            let game = context.tetrio_client.fetch_user_personal_league_records(id, PersonalLeaderboard::Recent, PersonalRecordsQuery::None).await?;
            let Some(data) = game.data else {
                return Ok(Err(anyhow::anyhow!("❌ Couldn't find tetra league game")));
            };
            let records = data.entries.get((tetra_league_game - 1) as usize);
            let Some(records) = &records else {
                return Err(anyhow::anyhow!("❌ Couldn't find tetra league game"));
            };



            if let Some(mut tetra_league_round) = tetra_league_round {
                if tetra_league_round <= 0 {
                    tetra_league_round = 1;
                }

                let Some(round) = records.results.rounds.get((tetra_league_round - 1) as usize) else {
                    return Err(anyhow::anyhow!("❌ Invalid round!"));
                };

                let Some(round) = round.iter().find(|user| &user.id == id) else {
                    return Err(anyhow::anyhow!("❌ Couldn't find stats!"));
                };

                pps = round.stats.pps;
                apm = round.stats.apm;
                vs  = round.stats.vsscore;

                format!(
                    "[Tetra league game](https://tetr.io/#r:{})\nStats from round {}.",
                    records.replayid, tetra_league_round
                )
            } else {
                let Some(left) = records.results.leaderboard.iter().find(|user| &user.id == id) else {
                    return Err(anyhow::anyhow!("❌ Couldn't find tetra league game"));
                };
                pps = left.stats.pps.unwrap_or(0.0);
                apm = left.stats.apm.unwrap_or(0.0);
                vs  = left.stats.vsscore.unwrap_or(0.0);

                format!(
                    "[Tetra league game](https://tetr.io/#R:{})\nStats from Average.",
                    records.replayid
                )
            }
        } else {
            String::new()
        };

        let avatar_revision = data.avatar_revision.unwrap_or(0);
        let builder = create_embed(None, &context).await?
            .title(username.to_uppercase())
            .url(format!("https://ch.tetr.io/u/{username}"))
            .description(format!("Takathebot - A bot attempting to copy sheetBot and but hiyajo maho but somehow does things in a better yet worse way.\n{}", tetra_league_game_str))
            ;
        

        let builder = if avatar_revision != 0 {
            builder.thumbnail(ImageSource::url(format!(
                "https://tetr.io/user-content/avatars/{id}.jpg?rv={avatar_revision}"
            ))?)
        } else {
            builder.thumbnail(ImageSource::attachment("profile_picture.webp")?)
        };

        let builder = Self::embed_with_stats(
            builder,
            show_details,
            PlayerStats {
                apm,
                pps,
                vs,
                rd: Some(rd),
                tr,
                glicko: Some(glicko),
                rank: rank.clone(),
            },
            None,
            None,
        )
        .await;

        let embed = builder.build();

        if avatar_revision == 0 {
            context
                .http_client
                .interaction(context.application.id)
                .update_response(&interaction.token)
                .attachments(&[Attachment::from_bytes(
                    "profile_picture.webp".to_string(),
                    include_bytes!("../../../assets/unkown_avatar.webp").to_vec(),
                    1,
                )])?
                .embeds(Some(&[embed]))?
                .await?;
        } else {
            context
                .http_client
                .interaction(context.application.id)
                .update_response(&interaction.token)
                .embeds(Some(&[embed]))?
                .await?;
        }

        Ok(Ok(()))
    }

    pub async fn with_replay(
        replay: TetrioReplaySubCommand,
        interaction: &InteractionCreate,
        context: &Context<'_>,
    ) -> anyhow::Result<anyhow::Result<()>> {
        let attachment = &replay.replay;
        // check that extension is ttrm
        if !attachment.filename.ends_with("ttrm") {
            return Ok(Err(anyhow!("❌ File type not supported: {:?}", attachment.filename)));
        };

        let bytes = reqwest::get(&attachment.url)
            .await?
            .bytes()
            .await?;

        let replay_data:common::replay::ttrm::models::Root = serde_json::from_slice(&bytes)
        .map_err(|err| anyhow!("❌ Couldn't parse ttrm replay: {:?}", err))?;


        let username = replay.user.clone();
        let Some(_) = replay_data.endcontext.iter().find(move |endcontext| {
            endcontext.get_username() == Some(username.clone())
        }) else {
            return Ok(Err(anyhow!("❌ Couldn't find user in replay")));
        };

        let username = replay.user.clone();
        let mut end_frames = replay_data.data.into_iter().map(|round| {
            let username = username.clone();
            round.replays.into_iter().filter_map(move |replayd| replayd.events.into_iter().find(|event|{
                match event {
                    Event::End { data, .. } => data.export.options.username == username,
                    _ => false
                }
            }).and_then(|obj| {
                match obj {
                    Event::End { data, .. } => Some(data),
                    _ => None
                }
            }))
        }).flatten()
        ;        



        let Some((stats, tetra_league_game_str)) = ({
            if let Some(game_number) = replay.game_number {
            end_frames.nth(game_number as usize).and_then(|data| {

            Some((Average {
                username: replay.user.clone(),
                apm: data.export.aggregatestats.apm,
                pps: data.export.aggregatestats.pps,
                vs: data.export.aggregatestats.vsscore,
                score: 0
            }, format!("Stats from round {}.", game_number)))
        })

        } else {
            // get average
            Some((end_frames.fold(Average{
                username: replay.user.clone(),
                apm: 0.0,
                pps: 0.0,
                vs: 0.0,
                score: 0
            }, |acc,  event| {
                    let apm = event.export.aggregatestats.apm;
                    let pps = event.export.aggregatestats.pps;
                    let vs = event.export.aggregatestats.vsscore;
                    Average {
                        username: replay.user.clone(),
                        apm: acc.apm + apm,
                        pps: acc.pps + pps,
                        vs: acc.vs + vs,
                        score: 0
                    }
                }
            ), "Stats from Average.".to_string()))
        }
        }) else {
            return Ok(Err(anyhow!("❌ Couldn't find user in replay")));
        };




        let builder = create_embed(None, &context).await?
        .title(replay.user.to_uppercase())
        .url(format!("https://ch.tetr.io/u/{}", replay.user))
        .description(format!("Takathebot - A bot attempting to copy sheetBot and but hiyajo maho but somehow does things in a better yet worse way.\n{}", tetra_league_game_str))
        ;
            
        let embed = Self::embed_with_stats(
            builder,
            replay.show_details.unwrap_or(false),
            PlayerStats {
                apm: stats.apm,
                pps: stats.pps,
                vs: stats.vs,
                rd: None,
                tr: None,
                glicko: None,
                rank: None,
            },
            None,
            None,
        )
        .await
        .build();

        context
            .http_client
            .interaction(context.application.id)
            .update_response(&interaction.token)
            .embeds(Some(&[embed]))?
            .await?;

        Ok(Ok(()))
    }

    pub async fn with_stats(
        stats: StatsSubCommand,
        interaction: &InteractionCreate,
        context: &Context<'_>,
    ) -> anyhow::Result<anyhow::Result<()>> {
        let builder = create_embed(None, &context).await?
            .title(format!("ADVANCED STATS FOR VALUES OF [APM = {}, PPS = {}, VS = {}]", stats.apm, stats.pps, stats.vs))
            .description("Takathebot - A bot attempting to copy sheetBot and but hiyajo maho but somehow does things in a better yet worse way.")
            ;

        let embed = Self::embed_with_stats(
            builder,
            stats.show_details.unwrap_or(false),
            PlayerStats {
                apm: stats.apm,
                pps: stats.pps,
                vs: stats.vs,
                rd: None,
                tr: None,
                glicko: None,
                rank: None,
            },
            None,
            None,
        )
        .await
        .build();

        context
            .http_client
            .interaction(context.application.id)
            .update_response(&interaction.token)
            .embeds(Some(&[embed]))?
            .await?;

        Ok(Ok(()))
    }

    pub async fn with_average_stats(
        average: AverageSubCommand,
        interaction: &InteractionCreate,
        context: &Context<'_>,
    ) -> anyhow::Result<anyhow::Result<()>> {

        let stats = average_of_rank(
            average.rank.clone().map(|rank| rank.into()),
            &context,
        )
        .await?;

        let stats = match stats {
            Ok(stats) => stats,
            Err(err) => return Ok(Err(err)),
        };

        let (avg, count, lowest) = stats;

        let rank = average
            .rank
            .clone()
            .map(|rank| rank.value())
            .unwrap_or("ALL");

        let builder = create_embed(None, &context).await?
            .title(format!("AVERAGE STATS OF RANK {}", rank))
            .description("Takathebot - A bot attempting to copy sheetBot and but hiyajo maho but somehow does things in a better yet worse way.")
            ;

        let builder = if let Some(rank) = average.rank.clone() {
            builder.thumbnail(ImageSource::url(format!(
                "https://tetr.io/res/league-ranks/{}.png",
                rank.value().to_lowercase()
            ))?)
        } else {
            builder.thumbnail(ImageSource::url("https://tetr.io/res/logo.png")?)
        };

        let embed = Self::embed_with_stats(
            builder,
            average.details.unwrap_or(false),
            PlayerStats {
                apm: avg.apm,
                pps: avg.pps,
                vs: avg.vs,
                rd: Some(avg.rd),
                tr: Some(avg.tr),
                glicko: Some(avg.glicko),
                rank: avg.rank,
            },
            Some(lowest),
            Some(count),
        )
        .await
        .build();

        context
            .http_client
            .interaction(context.application.id)
            .update_response(&interaction.token)
            .embeds(Some(&[embed]))?
            .await?;

        Ok(Ok(()))
    }

    pub async fn embed_with_stats(
        embed: EmbedBuilder,
        show_details: bool,
        PlayerStats {
            apm,
            pps,
            vs,
            rd,
            tr,
            glicko,
            rank,
        }: PlayerStats,
        required_tr: Option<f64>,
        members: Option<usize>,
    ) -> EmbedBuilder {
        let StringifiedStats {
            apm,
            vs,
            pps,
            dssecond,
            dspiece,
            dsapppiece,
            app,
            vsapm,
            cheese,
            garbage_effi,
            weighted_app,
            area,
            esttr,
            atr,
            opener,
            stride,
            infds,
            plonk,
            glicko,
            rd,
            tr,
            ..
        } = stringified_stats(PlayerStats {
            apm,
            pps,
            vs,
            rd,
            tr,
            glicko,
            rank: None,
        });

        if !show_details {
            let builder = embed
                .field(EmbedField {
                    inline: true,
                    name: "APM".to_string(),
                    value: apm,
                })
                .field(EmbedField {
                    inline: true,
                    name: "PPS".to_string(),
                    value: pps,
                })
                .field(EmbedField {
                    inline: true,
                    name: "VS".to_string(),
                    value: vs,
                })
                .field(EmbedField {
                    inline: true,
                    name: "DS/Second".to_string(),
                    value: dssecond,
                })
                .field(EmbedField {
                    inline: true,
                    name: "DS/Piece".to_string(),
                    value: dspiece,
                })
                .field(EmbedField {
                    inline: true,
                    name: "APP+DS/Piece".to_string(),
                    value: dsapppiece,
                })
                .field(EmbedField {
                    inline: true,
                    name: "APP".to_string(),
                    value: app,
                })
                .field(EmbedField {
                    inline: true,
                    name: "VS/APM".to_string(),
                    value: vsapm,
                })
                .field(EmbedField {
                    inline: true,
                    name: "Cheese Index".to_string(),
                    value: cheese,
                });

            let builder = if let Some(glicko) = glicko {
                builder.field(EmbedField {
                    inline: true,
                    name: "Glicko".to_string(),
                    value: glicko,
                })
            } else {
                builder
            };
            let builder = if let Some(tr) = tr {
                builder.field(EmbedField {
                    inline: true,
                    name: "TR".to_string(),
                    value: tr,
                })
            } else {
                builder
            };
            let builder = if let Some(rank) = rank {
                builder.field(EmbedField {
                    inline: true,
                    name: "Rank".to_string(),
                    value: rank.to_string(),
                })
            } else {
                builder
            };

            let builder = builder
                .field(EmbedField {
                    inline: true,
                    name: "Garbage Effi.".to_string(),
                    value: garbage_effi,
                })
                .field(EmbedField {
                    inline: true,
                    name: "Weighted APP".to_string(),
                    value: weighted_app,
                })
                .field(EmbedField {
                    inline: true,
                    name: "Area".to_string(),
                    value: area,
                });

            let builder = if let Some(tr) = required_tr {
                builder.field(EmbedField {
                    inline: true,
                    name: "TR Needed".to_string(),
                    value: format!("{:.2}", tr),
                })
            } else {
                builder.field(EmbedField {
                    inline: true,
                    name: "".to_string(),
                    value: "".to_string(),
                })
            };

            if let Some(members) = members {
                builder.field(EmbedField {
                    inline: true,
                    name: "Members".to_string(),
                    value: format!("{}", members),
                })
            } else {
                builder.field(EmbedField {
                    inline: true,
                    name: "".to_string(),
                    value: "".to_string(),
                })
            }
        } else {
            let builder = embed
                .field(EmbedField {
                    inline: true,
                    name: "APM".to_string(),
                    value: apm,
                })
                .field(EmbedField {
                    inline: true,
                    name: "PPS".to_string(),
                    value: pps,
                })
                .field(EmbedField {
                    inline: true,
                    name: "VS".to_string(),
                    value: vs,
                })
                .field(EmbedField {
                    inline: true,
                    name: "DS/Piece".to_string(),
                    value: dspiece,
                })
                .field(EmbedField {
                    inline: true,
                    name: "APP".to_string(),
                    value: app,
                })
                .field(EmbedField {
                    inline: true,
                    name: "APP+DS/Piece".to_string(),
                    value: dsapppiece,
                });

            let builder = if let Some(tr) = required_tr {
                builder.field(EmbedField {
                    inline: true,
                    name: "TR Needed".to_string(),
                    value: format!("{:.2}", tr),
                })
            } else {
                builder.field(EmbedField {
                    inline: true,
                    name: "".to_string(),
                    value: "".to_string(),
                })
            };

            let builder = if let Some(rank) = rank {
                builder.field(EmbedField {
                    inline: true,
                    name: "Rank".to_string(),
                    value: rank.to_string(),
                })
            } else {
                builder.field(EmbedField {
                    inline: true,
                    name: "Rank".to_string(),
                    value: "Global".to_string(),
                })
            };

            let builder = if let Some(members) = members {
                builder.field(EmbedField {
                    inline: true,
                    name: "Members".to_string(),
                    value: format!("{}", members),
                })
            } else {
                builder.field(EmbedField {
                    inline: true,
                    name: "".to_string(),
                    value: "".to_string(),
                })
            };

            let builder = builder.field(EmbedField { inline: true, name: "Advanced:".to_string(), value: 
                    format!("➤DS/Second:\n**{dssecond}**\n➤VS/APM:\n**{vsapm}**\n➤Garbage Effi.:\n**{garbage_effi}**\n➤Cheese Index:\n**{cheese}**\n➤Weighted APP:\n**{weighted_app}**")
             });
            if let (Some(rd), Some(glicko), Some(tr), Some(atr)) = (rd, glicko, tr, atr) {
                    builder.field(EmbedField { inline: true, name: "Ranking:".to_string(), value: 
                    format!("➤Area:**{area}**\n➤TR:**{tr}**\n➤Est. of TR:\n**{esttr}**\n➤Acc. of TR Est.:\n**{atr}**\n➤Glicko:\n**{glicko}**±{rd}")
                 })
                }
                else {
                    builder.field(EmbedField { inline: true, name: "Ranking:".to_string(), value: 
                        format!("➤Area:**{area}**\n➤Est. of TR:\n**{esttr}**\n")
                    })
                }

                .field(EmbedField { inline: true, name: "Playstyle:".to_string(), value: 
                    format!("➤Opener:**{opener}**\n➤Plonk:**{plonk}**\n➤Stride:**{stride}**\n➤Inf DS:**{infds}**")
                })
        }
    }
}

#[async_trait::async_trait]
impl RunnableCommand for TsCommand {
    async fn run(
        _shard: u64,
        interaction: &InteractionCreate,
        data: Box<CommandData>,
        context: &Context<'_>,
    ) -> anyhow::Result<anyhow::Result<()>> {
        log::info!("ts command");
        let _command_timer = Timer::new("ts command");
        context.defer_response(interaction).await?;        
        let model = Self::from_interaction(CommandInputData {
            options: data.options,
            resolved: data.resolved.map(Cow::Owned),
        })?;

        match model {
            TsCommand::Discord(discord) => {
                let packet = context
                    .tetrio_client
                    .search_discord_user(&discord.user.resolved.id.to_string())
                    .await?;

                if let Some(data) = &packet.data {
                    Self::with_user(
                        data.user.id.to_string(),
                        interaction,
                        discord.details.unwrap_or(false),
                        discord.tetra_league_game,
                        discord.tetra_league_round,
                        &context,
                    )
                    .await
                } else {
                    return Ok(Err(anyhow!("❌ Couldn't find your tetrio id from the discord account, they might have not linked it publicly to their tetrio profile")));
                }
            }
            TsCommand::Tetrio(tetrio) => {
                Self::with_user(
                    tetrio.tetrio_user,
                    interaction,
                    tetrio.details.unwrap_or(false),
                    tetrio.tetra_league_game,
                    tetrio.tetra_league_round,
                    &context,
                )
                .await
            }
            TsCommand::Stats(stats) => {
                Self::with_stats(stats, interaction, &context).await
            }
            TsCommand::Average(average) => {
                Self::with_average_stats(average, interaction, &context).await
            }
            TsCommand::Replay(replay) => {
                Self::with_replay(replay, interaction, &context).await
            }
        }
    }
}
