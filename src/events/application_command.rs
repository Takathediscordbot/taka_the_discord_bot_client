use twilight_model::{
    application::{interaction::application_command::CommandData, command::CommandType},
    gateway::payload::incoming::InteractionCreate,
};
use twilight_util::builder::embed::EmbedBuilder;

use crate::{
    context::Context,
    utils::timer::Timer,
};

#[cfg(feature = "database")]
use crate::services::silly_command::SillyCommandPDO;

#[cfg(feature = "database")]
use super::silly_command::handle_silly_command;

pub async fn handle_chat_command(
    shard: u64,
    interaction: &InteractionCreate,
    data: Box<CommandData>,
    context: &Context,
) -> anyhow::Result<()> {
    let name = data.name.as_str();
    let command = context.commands.iter().find(|a| a.get_name() == name);


    let result = if let Some(command) = command {
        command
            .run(shard, interaction, data, &context)
            .await
    }
     else {
        #[cfg(feature = "database")]
        if let Some(command) =
        SillyCommandPDO::fetch_silly_command_by_name(&context, name).await
        {
            handle_silly_command(shard, interaction, data, command, &context).await
        }
        else {
            let Err(e) = context.response_to_interaction_with_content(interaction,"❌ Unhandled command: this command has not yet been implemented")
            .await else {
                return Ok(());
            };
    
            Err(anyhow::anyhow!(e))
        }

        #[cfg(not(feature = "database"))]
        let Err(e) = context.response_to_interaction_with_content(interaction,"❌ Unhandled command: this command has not yet been implemented")
        .await else {
            return Ok(());
        };

        #[cfg(not(feature = "database"))]
        Err(anyhow::anyhow!(e))
    };

    if let Err(result) = result {
        log::error!("{result:?}");
        let embeds = &[EmbedBuilder::new()
            .title("❌ An error has occured, tell taka about it")
            .description(format!("{result:?}"))
            .build()];
        let interaction_client = context.http_client.interaction(context.application.id);
        
            match interaction_client.response(&interaction.token).await {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        interaction_client
                        .update_response(&interaction.token)
                        .embeds(Some(embeds))?
                        .await?;
                    }
                    else {
                        match &interaction.channel {
                            Some(channel) => context.http_client.create_message(channel.id)
                            .embeds(embeds)?
                            .await?,
                            None => return Ok(())
                        };
                    }
                }
                Err(_) => {
                    match &interaction.channel {
                        Some(channel) => context.http_client.create_message(channel.id)
                        .embeds(embeds)?
                        .await?,
                        None => return Ok(())
                    };
                }
            };
    } else if let Ok(Err(result)) = result {
        let interaction_client = context.http_client.interaction(context.application.id);

        match interaction_client.response(&interaction.token).await {
            Ok(response) => {
                let status = response.status();
                if status.is_success() {
                    interaction_client
                    .update_response(&interaction.token)
                    .content(Some(&result.to_string()))?
                    .await?;
                }
                else {
                    match &interaction.channel {
                        Some(channel) => context.http_client.create_message(channel.id)
                        .content(&result.to_string())?
                        .await?,
                        None => return Ok(())
                    };
                }
            }
            Err(_) => {
                match &interaction.channel {
                    Some(channel) => context.http_client.create_message(channel.id)
                    .content(&result.to_string())?
                    .await?,
                    None => return Ok(())
                };
            }
        };
        

    }

    Ok(())
}

pub async fn handle_application_command(
    shard: u64,
    interaction: &InteractionCreate,
    data: Box<CommandData>,
    context: &Context,
) -> anyhow::Result<()> {
    let _timer = Timer::new("handle_application_command");
    
    match data.kind {
        CommandType::ChatInput => handle_chat_command(shard, interaction, data, context).await,
        _ => Ok(())
    }
}
