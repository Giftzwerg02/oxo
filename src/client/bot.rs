use std::{collections::HashMap, sync::Arc};

use enum_assoc::Assoc;
use poise::serenity_prelude::{self as serenity, GuildId, Mutex};
use songbird::Songbird;
use songbird::{tracks::TrackQueue, SerenityInit};
use tracing::error;

use crate::client::commands::commands;
use crate::error::{on_error, Error};

pub type Context<'a> = poise::Context<'a, Arc<Mutex<State>>, Error>;

pub type Queues = Arc<Mutex<HashMap<GuildId, TrackQueue>>>;
#[derive(Debug)]
pub struct State {
    pub queues: Queues,
    pub loop_mode: Arc<Mutex<LoopMode>>,
    pub songbird_instance: Arc<Songbird>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            queues: Default::default(),
            loop_mode: Default::default(),
            songbird_instance: Songbird::serenity(),
        }
    }
}

#[derive(Debug, poise::ChoiceParameter, Clone, Default)]
pub enum LoopMode {
    #[default]
    Off,
    Track,
    Queue,
}

#[derive(Debug, poise::ChoiceParameter, Clone, Default)]
#[derive(Assoc)]
#[func(pub fn is_24_7(&self) -> bool)]
#[func(pub fn url(&self) -> &str)]
pub enum LofiSong {
    #[default]
    #[assoc(is_24_7 = true)]
    #[assoc(url = "https://www.youtube.com/watch?v=jfKfPfyJRdk")]
    LofiGirl,

    #[assoc(is_24_7 = false)]
    #[assoc(url = "https://www.youtube.com/watch?v=A7vMrjsBMTI")]
    Undertale,

    #[assoc(is_24_7 = false)]
    #[assoc(url = "https://www.youtube.com/watch?v=-z3RRwk2rdU")]
    Zelda,

    #[assoc(is_24_7 = false)]
    #[assoc(url = "https://www.youtube.com/watch?v=GNWLILeztaI")]
    AnimeOps,

    #[assoc(is_24_7 = false)]
    #[assoc(url = "https://www.youtube.com/watch?v=83PnFc6eh-4")]
    Metal,

    #[assoc(is_24_7 = false)]
    #[assoc(url = "https://www.youtube.com/watch?v=1XFtipo7v0Y")]
    Djent,
}


pub async fn start_bot(state: Arc<Mutex<State>>) {
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");

    let state_clone = state.clone();
    let state_clone = state_clone.lock().await;
    let songbird_instance = state_clone.songbird_instance.clone();

    drop(state_clone);

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
                Ok(state.clone())
            })
        })
        .client_settings(move |builder| builder.register_songbird_with(songbird_instance))
        .build()
        .await
        .unwrap();

    let status = framework.clone().start();
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Could not register ctrl+c handler");
        let shard_manager = framework.shard_manager();
        shard_manager.lock().await.shutdown_all().await;
    });

    if let Err(why) = status.await {
        error!("Client error: {:?}", why);
    }
}
