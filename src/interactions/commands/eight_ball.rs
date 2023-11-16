use std::borrow::Cow;
use rand::prelude::*;

use twilight_interactions::command::{CommandInputData, CommandModel, CreateCommand};
use twilight_model::{
    application::interaction::application_command::CommandData,
    gateway::payload::incoming::InteractionCreate,
};
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::{
    context::Context, utils::{box_commands::RunnableCommand, self},
};

#[derive(CreateCommand, CommandModel)]
#[command(name = "8ball", desc = "Get the real only answer to your question.")]
pub struct EightBallCommand {
    /// The question
    question: String
}

const YES_OPTIONS: [&str; 8] = 
    [
        "Well duh..", "Obviously yes", "Do you think I'm stupid? Yes!!!", "Omg ofc.", "Yes!!",
        "uwu yes", "not like the answer is yes or anything b-baka", "maybe... ok... yeh... if I think about it... ok the answer is........ yes"
    ];

const NO_OPTIONS: [&str; 8] = 
    [
        "Are you for real? Of course not.", "The answer is actually no", "Noooo", "nou", "No!!!",
        "b-baka, no!!", "Ok time to be edgy.. no...", "did you know 'I'm fine' is actually a very common lie? well saying the answer to this question is yes is also a lie."
    ];

#[async_trait::async_trait]
impl RunnableCommand for EightBallCommand {
    async fn run(
        _shard: u64,
        interaction: &InteractionCreate,
        data: Box<CommandData>,
        context: &Context,
    ) -> anyhow::Result<anyhow::Result<()>> {
        let model = Self::from_interaction(CommandInputData {
            options: data.options,
            resolved: data.resolved.map(Cow::Owned),
        })?;


        let question = model.question;
        let yes_no = rand::thread_rng().gen_bool(0.5);

        let answer = if yes_no {
            YES_OPTIONS[rand::thread_rng().gen_range(0..YES_OPTIONS.len())]
        }
        else {
            NO_OPTIONS[rand::thread_rng().gen_range(0..NO_OPTIONS.len())]
        };

        let embed = utils::create_embed::create_embed(None, &context).await?;

        let embed = embed.title(question).description(answer).build();
        context.response_to_interaction(interaction, InteractionResponseDataBuilder::new().embeds([embed]).build())
            .await?;

        Ok(Ok(()))
    }
}
