use std::borrow::Cow;
use std::sync::Arc;

use anyhow::anyhow;
use twilight_interactions::command::{CommandInputData, CommandModel, CreateCommand};

use twilight_model::application::interaction::application_command::CommandData;
use twilight_model::channel::message::embed::EmbedField;
use twilight_model::gateway::payload::incoming::InteractionCreate;
use twilight_model::http::attachment::Attachment;

use twilight_util::builder::embed::{EmbedBuilder, ImageSource};

use crate::context::Context;

use crate::utils::average_of_rank::average_of_rank;
use crate::utils::box_commands::{CommandBox, RunnableCommand};
use crate::utils::create_embed::create_embed;

use crate::utils::stats::{stringified_stats, PlayerStats, StringifiedStats};

use crate::interactions::commands::subcommands::ts::average_sub_command::AverageSubCommand;
use crate::interactions::commands::subcommands::ts::discord_user_sub_command::DiscordUserSubCommand;
use crate::interactions::commands::subcommands::ts::stats_sub_command::StatsSubCommand;
use crate::interactions::commands::subcommands::ts::tetrio_user_sub_command::TetrioUserSubCommand;

#[derive(CreateCommand, CommandModel)]
#[command(name = "ts", desc = "Calculate the tetrio stats for a user")]
pub enum TsCommand {
    #[command(name = "discord")]
    /// Fetch data from a discord user
    Discord(CommandBox<DiscordUserSubCommand>),
    #[command(name = "tetrio")]
    /// Fetch data from a tetrio user
    Tetrio(TetrioUserSubCommand),
    #[command(name = "stats")]
    /// Use tetrio stats
    Stats(StatsSubCommand),

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
        context: Arc<Context>,
    ) -> anyhow::Result<anyhow::Result<()>> {
        let tetrio_user = context.tetrio_client.fetch_user_info(&id).await?;

        let Some(data) = &tetrio_user.data else {
            return Ok(Err(anyhow::anyhow!("❌ No data has been found. User might be anonymous or banned.")));
        };

        let (id, username, Some(mut apm), Some(mut pps), Some(mut vs), rank, tr, Some(glicko), Some(rd)) = (&data.user.id, &data.user.username, data.user.league.apm, data.user.league.pps, data.user.league.vs, &data.user.league.rank, data.user.league.rating, data.user.league.glicko, data.user.league.rd) else {
            return Ok(Err(anyhow::anyhow!("❌ No tetra league stats have been found.")));
        };

        let tetra_league_game_str = if let Some(mut tetra_league_game) = tetra_league_game {
            if tetra_league_game <= 0 {
                tetra_league_game = 1;
            }

            let game = tetrio_api::http::client::fetch_tetra_league_recent(id).await?;
            let Some(data) = game.data else {
                return Ok(Err(anyhow::anyhow!("❌ Couldn't find tetra league game")));
            };
            let records = data.records.get((tetra_league_game - 1) as usize);
            let Some(records) = &records else {
                return Err(anyhow::anyhow!("❌ Couldn't find tetra league game"));
            };

            let Some(left) = records.endcontext.iter().find(|a| {
                &a.user.id == id
            }) else {
                return Err(anyhow::anyhow!("❌ Couldn't find tetra league game"));
            };

            if let Some(mut tetra_league_round) = tetra_league_round {
                if tetra_league_round <= 0 {
                    tetra_league_round = 1;
                }

                let index = (tetra_league_round - 1) as usize;
                pps = left.points.tertiary_avg_tracking[index];
                apm = left.points.secondary_avg_tracking[index];
                vs = left.points.extra_avg_tracking.aggregate_stats_vs_score[index];

                format!(
                    "[Tetra league game](https://tetr.io/#r:{})\nStats from round {}.",
                    records.replay_id, tetra_league_round
                )
            } else {
                pps = left.points.tertiary;
                apm = left.points.secondary;
                vs = left.points.extra.vs;

                format!(
                    "[Tetra league game](https://tetr.io/#R:{})\nStats from Average.",
                    records.replay_id
                )
            }
        } else {
            String::new()
        };

        let avatar_revision = data.user.avatar_revision.unwrap_or(0);
        let builder = create_embed(None, Arc::clone(&context)).await?
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
                tr: Some(tr),
                glicko: Some(glicko),
                rank: Some(rank.clone()),
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

    pub async fn with_stats(
        stats: StatsSubCommand,
        interaction: &InteractionCreate,
        context: Arc<Context>,
    ) -> anyhow::Result<anyhow::Result<()>> {
        let builder = create_embed(None, Arc::clone(&context)).await?
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
        context: Arc<Context>,
    ) -> anyhow::Result<anyhow::Result<()>> {
        let country_str = average
            .country
            .clone()
            .map(|a| format!("IN COUNTRY {}", a))
            .unwrap_or(String::new());
        let stats = average_of_rank(
            average.rank.clone().map(|rank| rank.into()),
            average.country.map(|c| c.to_uppercase()),
            Arc::clone(&context),
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

        let builder = create_embed(None, Arc::clone(&context)).await?
            .title(format!("AVERAGE STATS OF RANK {} {country_str}", rank))
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
        context: Arc<Context>,
    ) -> anyhow::Result<anyhow::Result<()>> {
        let model = Self::from_interaction(CommandInputData {
            options: data.options,
            resolved: data.resolved.map(Cow::Owned),
        })?;

        match model {
            TsCommand::Discord(discord) => {
                let packet = context
                    .tetrio_client
                    .search_user(&discord.user.resolved.id.to_string())
                    .await?;

                let Some(data) = &packet.data else {
                    return Ok(Err(anyhow!("❌ Couldn't find your tetrio id from the discord account, they might have not linked it publicly to their tetrio profile")));
                };

                Self::with_user(
                    data.user.id.to_string(),
                    interaction,
                    discord.details.unwrap_or(false),
                    discord.tetra_league_game,
                    discord.tetra_league_round,
                    Arc::clone(&context),
                )
                .await
            }
            TsCommand::Tetrio(tetrio) => {
                Self::with_user(
                    tetrio.tetrio_user,
                    interaction,
                    tetrio.details.unwrap_or(false),
                    tetrio.tetra_league_game,
                    tetrio.tetra_league_round,
                    Arc::clone(&context),
                )
                .await
            }
            TsCommand::Stats(stats) => {
                Self::with_stats(stats, interaction, Arc::clone(&context)).await
            }
            TsCommand::Average(average) => {
                Self::with_average_stats(average, interaction, Arc::clone(&context)).await
            }
        }
    }
}
