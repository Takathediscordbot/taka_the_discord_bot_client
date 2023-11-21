use twilight_interactions::command::CreateCommand;
use twilight_model::{gateway::payload::incoming::InteractionCreate, application::interaction::application_command::CommandData, channel::message::{Embed, embed::EmbedFooter, Component, component::{Button, ActionRow}, ReactionType}, http::interaction::{InteractionResponse, InteractionResponseType}};
use twilight_util::builder::InteractionResponseDataBuilder;


use crate::{utils::{box_commands::RunnableCommand, 
    create_embed::create_embed}, context::Context, 
};

use super::tetrio_commands;




#[derive(CreateCommand)]
#[command(name = "help", desc = "Get more information about commands!")]
pub struct HelpCommand;

impl HelpCommand {
    async fn get_command_descriptions_embed(context: &Context) -> anyhow::Result<Vec<Embed>> {
        let default_embed = create_embed(None, &context).await?;
        let first_page_embed = default_embed.clone()
            
            .title("Thanks & Calculations".to_string())
            .description(
                "**Special thanks for @kerrmunism and @dimentio for the inspiration & the help to make this bot**\n\n**Calculations from sheetbot:**```\nAPP: APM/(PPS*60)\nDS/Second: (VS/100)-(APM/60)\nDS/Piece: ((VS/100)-(APM/60))/PPS\nAPP+DS/Piece: (((VS/100)-(APM/60))/PPS) + APM/(PPS*60)\nCheese Index: ((DS/Piece * 150) + (((VS/APM)-2)*50) + (0.6-APP)*125))\nGarbage Effi.: (attack*downstack)/pieces^2\nArea: apm + pps * 45 + vs * 0.444 + app * 185 + dssecond * 175 + dspiece * 450 + garbageEffi * 315\nWeighted APP: APP - 5 * tan((cheeseIndex/ -30) + 1)\nEst. TR: (25000/(1+(10^(((1500-(4.0867 * (pps * 90 + app * 290 + dspiece * 750) + 186.68))*3.14159)/((15.9056943314 * (rd^2) + 3527584.25978)^0.5)))))\n```".to_string())
            .footer(EmbedFooter { icon_url: None, proxy_icon_url: None, text: "Page 1 / 3".to_string()  })
            .build();
        
        let mut tetrio_commands_description = String::new();
        for (name,desc) in tetrio_commands::get_descriptions().iter() {
            tetrio_commands_description += &format!("\n**{name}**\n{desc}\n");
        }
        let tetrio_embed = default_embed.clone();
        let tetrio_embed = tetrio_embed
            .title("Tetrio commands")
            .description(tetrio_commands_description)
            .footer(EmbedFooter { icon_url: None, proxy_icon_url: None, text: "Page 2 / 3".to_string()  })
            .build();


        #[cfg(feature = "database")] 
        let silly_command_embed = {
        use crate::services::silly_command::SillyCommandPDO;
        let silly_commands = SillyCommandPDO::fetch_silly_commands(&context).await;
        let mut silly_command_description = String::new();
        for command in silly_commands.into_iter() {
            silly_command_description += &format!("\n**{}**\n{}\n", &command.name, &command.description);
        }
        let silly_command_embed = default_embed.clone();
        silly_command_embed
            .title("Silly commands")
            .description(silly_command_description)
            .footer(EmbedFooter { icon_url: None, proxy_icon_url: None, text: "Page 3 / 3".to_string()  })
            .build()

        };

            Ok(vec![
                first_page_embed, 
                tetrio_embed,
                #[cfg(feature = "database")]
                silly_command_embed
            ])
    }

    fn get_buttons() -> [Component; 1] {

        [Component::ActionRow(ActionRow { components: vec![Component::Button(Button { 
            custom_id: Some("help_previous".to_string()), 
            disabled: false, 
            emoji: Some(ReactionType::Unicode { name: "◀️".to_string() }), 
            label: Some("Previous".to_string()), 
            style: twilight_model::channel::message::component::ButtonStyle::Primary, 
            url: None 
        }), Component::Button(Button { 
            custom_id: Some("help_next".to_string()), 
            disabled: false, 
            emoji: Some(ReactionType::Unicode { name: "▶️".to_string() }), 
            label: Some("Next".to_string()), 
            style: twilight_model::channel::message::component::ButtonStyle::Primary, 
            url: None 
        })] }) ]
        
    }

    pub async fn previous(_shard: u64, it: Box<InteractionCreate>, _data: twilight_model::application::interaction::message_component::MessageComponentInteractionData, context: &Context) -> anyhow::Result<()> {
        let Some(message) = &it.message else {
            return Ok(())
       };

       let Some(footer) = &message.embeds[0].footer else {
            return Ok(())
       };

       let embeds = Self::get_command_descriptions_embed(&context).await?;
       let Some(current) = embeds.iter().position(|a| match &a.footer {
            None => false,
            Some(e) => e.text == footer.text 
       }) else {
            return Ok(())
       };

       let index = if current > 0 {
            current - 1
       } else {
            embeds.len() - 1
       };

       let Some(embed) = embeds.get(index) else {
            return Ok(())
       };
       let embeds_array = [embed.clone()];
       let buttons = Self::get_buttons();

       let interaction =

       context.http_client
     .interaction(context.application.id);
       
       interaction.create_response(it.id, &it.token, &InteractionResponse { 
        kind: InteractionResponseType::UpdateMessage, 
        data: Some(InteractionResponseDataBuilder::new().components(buttons).embeds(embeds_array).build())
        }).await?;
               
        Ok(())
    }

    pub async fn next(_shard: u64, it: Box<InteractionCreate>, _data: twilight_model::application::interaction::message_component::MessageComponentInteractionData, context: &Context) -> anyhow::Result<()> {
       
       let Some(message) = &it.message else {
            return Ok(())
       };

       let Some(footer) = &message.embeds[0].footer else {
            return Ok(())
       };

       let embeds = Self::get_command_descriptions_embed(&context).await?;
       let Some(current) = embeds.iter().position(|a| match &a.footer {
            None => false,
            Some(e) => e.text == footer.text 
       }) else {
            return Ok(())
       };

       let Some(embed) = embeds.get((current + 1) % embeds.len()) else {
            return Ok(())
       };
       let embeds_array = [embed.clone()];
       let buttons = Self::get_buttons();

       let interaction =

         context.http_client
       .interaction(context.application.id);
        
        interaction.create_response(it.id, &it.token, &InteractionResponse { 
        kind: InteractionResponseType::UpdateMessage, 
        data: Some(InteractionResponseDataBuilder::new().components(buttons).embeds(embeds_array).build())
        }).await?;
        Ok(())
    }


}

#[async_trait::async_trait]
impl RunnableCommand for HelpCommand {
    async fn run(
        _shard: u64,
        interaction: &InteractionCreate,
        _data: Box<CommandData>,
        context: &Context,
    ) -> anyhow::Result<anyhow::Result<()>>{
        let embeds = Self::get_command_descriptions_embed(&context).await?;
        let buttons = Self::get_buttons();
        let embeds_array = [embeds[0].clone()];
        context.response_to_interaction(interaction, 
            InteractionResponseDataBuilder::new().embeds(embeds_array).components(buttons).build()).await?;
        Ok(Ok(()))
    }
}


