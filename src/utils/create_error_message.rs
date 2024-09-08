
use twilight_model::gateway::payload::incoming::InteractionCreate;

use crate::context::Context;

pub async fn create_error_message(
    arg: &str,
    interaction: &InteractionCreate,
    context: &Context<'_>,
) -> anyhow::Result<()> {
    context
        .http_client
        .interaction(context.application.id)
        .update_response(&interaction.token)
        .content(Some(arg))?
        .await?;
    Ok(())
}
