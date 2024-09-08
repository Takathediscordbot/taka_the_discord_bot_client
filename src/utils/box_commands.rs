use async_trait::async_trait;
use std::{marker::PhantomData, ops::Deref};
pub use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
    application::interaction::application_command::CommandData,
    gateway::payload::incoming::InteractionCreate,
};

use crate::context::Context;

pub struct CommandBox<T>(Box<T>);
#[async_trait]
pub trait RunnableCommand {
    async fn run(
        shard: u64,
        interaction: &InteractionCreate,
        data: Box<CommandData>,
        context: &Context<'_>,
    ) -> anyhow::Result<anyhow::Result<()>>;
}

#[derive(Clone, Copy)]
pub struct PhantomCommand<T> {
    data: PhantomData<T>,
}

impl<T> PhantomCommand<T> {
    pub fn new() -> Self {
        Self {
            data: PhantomData::<T> {},
        }
    }
}

impl<T> Default for PhantomCommand<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
pub trait PhantomCommandTrait: Send + Sync {
    fn get_name(&self) -> &'static str;
    fn create_command(&self) -> twilight_interactions::command::ApplicationCommandData;
    async fn run(
        &self,
        shard: u64,
        interaction: &InteractionCreate,
        data: Box<CommandData>,
        context: &Context<'_>,
    ) -> anyhow::Result<anyhow::Result<()>>;
}

#[async_trait]
impl<T: CreateCommand + RunnableCommand + Send + Sync> PhantomCommandTrait for PhantomCommand<T> {
    fn get_name(&self) -> &'static str {
        T::NAME
    }

    fn create_command(&self) -> twilight_interactions::command::ApplicationCommandData {
        T::create_command()
    }

    async fn run(
        &self,
        shard: u64,
        interaction: &InteractionCreate,
        data: Box<CommandData>,
        context: &Context<'_>,
    ) -> anyhow::Result<anyhow::Result<()>> {
        T::run(shard, interaction, data, context).await
    }
}

impl<T: CreateCommand> CreateCommand for PhantomCommand<T> {
    const NAME: &'static str = T::NAME;

    fn create_command() -> twilight_interactions::command::ApplicationCommandData {
        T::create_command()
    }
}

impl<T> Deref for CommandBox<T> {
    type Target = Box<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: CreateCommand> CreateCommand for CommandBox<T> {
    const NAME: &'static str = T::NAME;

    fn create_command() -> twilight_interactions::command::ApplicationCommandData {
        T::create_command()
    }
}

impl<T: CommandModel> CommandModel for CommandBox<T> {
    fn from_interaction(
        data: twilight_interactions::command::CommandInputData,
    ) -> Result<Self, twilight_interactions::error::ParseError> {
        Ok(CommandBox(Box::new(T::from_interaction(data)?)))
    }
}
