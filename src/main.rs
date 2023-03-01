use std::{collections::HashMap, sync::Arc};

use dotenvy::dotenv;
use poise::serenity_prelude::{self as serenity, GuildId, Mutex};
use songbird::{tracks::TrackQueue, SerenityInit};
mod error;

use error::{on_error, Error};

mod commands;
use crate::commands::commands;

pub type Context<'a> = poise::Context<'a, State, Error>;

pub type Queues = Arc<Mutex<HashMap<GuildId, TrackQueue>>>;

mod events;

#[derive(Debug)]
pub struct State {
    queues: Queues,
    loop_mode: Arc<Mutex<LoopMode>>,
}

#[derive(Debug, poise::ChoiceParameter, Clone)]
pub enum LoopMode {
    Off,
    Track,
    Queue,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

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
                Ok(State {
                    queues: Default::default(),
                    loop_mode: Arc::new(Mutex::new(LoopMode::Off)), // trackdata: Arc::new(Mutex::new(HashMap::new())),
                                                                    // song_input_data: Arc::new(Mutex::new(HashMap::new())),
                })
            })
        })
        .client_settings(|builder| builder.register_songbird());

    framework.run().await.unwrap();
}
