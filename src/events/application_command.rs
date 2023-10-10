use std::sync::Arc;

use twilight_interactions::command::CreateCommand;
use twilight_model::{
    application::interaction::application_command::CommandData,
    gateway::payload::incoming::InteractionCreate,
};
use twilight_util::builder::embed::EmbedBuilder;

use crate::{
    context::Context,
    interactions::commands::test_mode::TestMode,
    services::silly_command::SillyCommandPDO,
    utils::box_commands::RunnableCommand,
};

use super::silly_command::handle_silly_command;

pub async fn handle_application_command(
    shard: u64,
    interaction: &InteractionCreate,
    data: Box<CommandData>,
    context: Arc<Context>,
) -> anyhow::Result<()> {
 

    let name = data.name.as_str();
    if name == TestMode::NAME {
        TestMode::run(shard, interaction, data, Arc::clone(&context)).await??;
        return Ok(());
    }

    {
        let test_mode = *context.test_mode.lock().await;
        if test_mode {
            if let Some(channel) = &interaction.channel {
                context.http_client.create_message(channel.id).content("❌ Test mode is enabled and the command will be ignored\nIf you think this is not intentional message @Taka#4011 on discord.")?.await?;
            }

            return Ok(());
        }
    }

    let command = context.commands.iter().find(|a| a.get_name() == name);

    let result = if let Some(command) = command {
        command
            .run(shard, interaction, data, Arc::clone(&context))
            .await
    } else if let Some(command) =
        SillyCommandPDO::fetch_silly_command_by_name(Arc::clone(&context), name).await
    {
        handle_silly_command(shard, interaction, data, command, Arc::clone(&context)).await
    } else {
        let Err(e) = context.http_client.interaction(context.application.id)
        .update_response(&interaction.token)
        .content(Some("❌ Unhandled command: this command has not yet been implemented"))
        ?.await else {
            return Ok(());
        };

        Err(anyhow::anyhow!(e))
    };

    if let Err(result) = result {
        log::error!("{result:?}");
        let embeds = &[EmbedBuilder::new()
            .title("❌ An error has occured, tell taka about it")
            .description(format!("{result:?}"))
            .build()];
        let interaction_client = context.http_client.interaction(context.application.id);
        interaction_client
            .update_response(&interaction.token)
            .content(None)?
            .embeds(Some(embeds))?
            .await?;
    } else if let Ok(Err(result)) = result {
        let interaction_client = context.http_client.interaction(context.application.id);
        interaction_client
            .update_response(&interaction.token)
            .content(Some(&result.to_string()))?
            .await?;
    }

    Ok(())
}
