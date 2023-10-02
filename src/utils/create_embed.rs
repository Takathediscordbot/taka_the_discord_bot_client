use std::{fmt::Display, sync::Arc, time::UNIX_EPOCH};

use twilight_model::{channel::message::embed::EmbedAuthor, util::Timestamp};
use twilight_util::builder::embed::{EmbedBuilder, ImageSource};

use crate::context::Context;

pub fn get_avatar(user_id: impl Display, avatar: impl Display) -> String {
    format!(
        "https://cdn.discordapp.com/avatars/{}/{}.webp",
        user_id, avatar
    )
}

pub fn set_image_if_exists(embed: EmbedBuilder, avatar: Option<ImageSource>) -> EmbedBuilder {
    log::debug!("{:?}", avatar);

    if let Some(avatar) = avatar {
        embed.image(avatar)
    } else {
        embed
    }
}

pub async fn create_embed(
    color: Option<u32>,
    context: Arc<Context>,
) -> anyhow::Result<EmbedBuilder> {
    let bot_user = context.http_client.current_user().await?.model().await?;

    Ok(EmbedBuilder::new()
        .author(EmbedAuthor {
            icon_url: bot_user
                .avatar
                .map(|avatar| get_avatar(bot_user.id, avatar)),
            name: format!("{}#{}", bot_user.name, bot_user.discriminator()),
            proxy_icon_url: None,
            url: None,
        })
        .timestamp(Timestamp::from_secs(UNIX_EPOCH.elapsed()?.as_secs() as i64)?)
        .color(color.unwrap_or(0xe6aabb)))
}
