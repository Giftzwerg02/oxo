use std::{collections::HashMap, sync::Arc};

use poise::serenity_prelude::{self as serenity, GuildId, Mutex};
use songbird::{tracks::TrackQueue, SerenityInit};
use tokio::sync::OwnedMutexGuard;

use crate::error::{on_error, Error};
use crate::commands::commands;

pub type Context<'a> = poise::Context<'a, OwnedMutexGuard<State>, Error>;

pub type Queues = Arc<Mutex<HashMap<GuildId, TrackQueue>>>;
#[derive(Debug, Default)]
pub struct State {
    pub queues: Queues,
    pub loop_mode: Arc<Mutex<LoopMode>>,
}

#[derive(Debug, poise::ChoiceParameter, Clone, Default)]
pub enum LoopMode {
    #[default]
    Off,
    Track,
    Queue,
}

pub async fn start_bot(state: Arc<Mutex<State>>) {
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands().unwrap(),
            on_error: |err| Box::pin(on_error(err)),
            ..Default::default()
        })
        .token(token)
        .intents(serenity::GatewayIntents::non_privileged())
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(state.clone().lock_owned().await)
            })
        })
        .client_settings(|builder| builder.register_songbird());

    framework.run().await.expect("Could not start discord bot")
}