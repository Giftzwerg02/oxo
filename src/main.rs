use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use dotenvy::dotenv;
use poise::{
    async_trait,
    serenity_prelude::{self as serenity, ChannelId, GuildId, Http, Mutex, RwLock, Guild},
    FrameworkError,
};
use songbird::{
    tracks::{Track, TrackHandle, TrackQueue},
    Call, Event, EventContext, EventHandler, SerenityInit, TrackEvent, input::Input, create_player,
};
mod error;
use error::{error_embed, log_unexpected_error, Error};
use tracing::debug;

type Context<'a> = poise::Context<'a, State, Error>;
type CmdRes = Result<(), Error>;

#[derive(Debug)]
struct State {
    queues: Arc<Mutex<HashMap<GuildId, TrackQueue>>>,
}

struct EndEventHandler {
    handler: Arc<Mutex<Call>>,
    http: Arc<Http>,
    channel_id: ChannelId,
    guild_id: GuildId,
}

#[async_trait]
impl EventHandler for EndEventHandler {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        debug!(
            channel_id = self.channel_id.to_string(),
            "Triggered leave-after-end-event-handler"
        );

        let EventContext::Track(tracks) = ctx else {
            return None;
        };

        // double de-ref LETS GO
        let [(state, handle)] = **tracks else {
            return None;
        };

        None
    }
}

/// Hol' up
#[poise::command(slash_command)]
async fn pause(ctx: Context<'_>) -> CmdRes {
    let guild = ctx.guild().unwrap();
    let queues = ctx.data().queues.lock().await;
    let queue = queues.get(&guild.id)
        .unwrap();

    queue.pause()?;

    Ok(())
}

/// Keep going
#[poise::command(slash_command)]
async fn resume(ctx: Context<'_>) -> CmdRes {
    let guild = ctx.guild().unwrap();
    let queues = ctx.data().queues.lock().await;
    let queue = queues.get(&guild.id)
        .unwrap();

    queue.resume()?;

    Ok(())
}

/// Don't care
#[poise::command(slash_command)]
async fn skip(ctx: Context<'_>) -> CmdRes {
    let guild = ctx.guild().unwrap();
    let queues = ctx.data().queues.lock().await;
    let queue = queues.get(&guild.id)
        .unwrap();

    queue.skip()?;

    Ok(())
}

/// Jamming
#[poise::command(slash_command)]
async fn play(ctx: Context<'_>, #[description = "URL"] url: String) -> CmdRes {
    let guild = ctx.guild().unwrap();
    let channel_id = guild
        .voice_states
        .get(&ctx.author().id)
        .and_then(|vs| vs.channel_id)
        .unwrap();

    let manager = songbird::get(ctx.serenity_context()).await.unwrap().clone();

    let (handler, _) = manager.join(guild.id, channel_id).await;
    let mut handler_lock = handler.lock().await;

    let mut queues = ctx.data().queues.lock().await;
    let queue = queues.entry(guild.id).or_default();

    let source = songbird::ytdl(&url).await?;
    let track = queue.add_source(source, &mut handler_lock);

    track.add_event(Event::Track(TrackEvent::End), EndEventHandler {
        handler: handler.clone(),
        channel_id: ctx.channel_id(),
        http: ctx.serenity_context().http.clone(),
        guild_id: guild.id,
    })?;

    Ok(())
}

async fn on_error(error: FrameworkError<'_, State, Error>) {
    let res = match error {
        FrameworkError::Setup {
            error: _,
            framework: _,
            data_about_bot: _,
            ctx: _,
        } => todo!(),
        FrameworkError::EventHandler {
            error: _,
            ctx: _,
            event: _,
            framework: _,
        } => todo!(),
        FrameworkError::Command { error, ctx } => {
            log_unexpected_error(&error);

            ctx.send(|create| create.embed(|e| error_embed(e, &error)))
                .await
        }
        FrameworkError::ArgumentParse {
            error: _,
            input: _,
            ctx: _,
        } => todo!(),
        FrameworkError::CommandStructureMismatch {
            description: _,
            ctx: _,
        } => todo!(),
        FrameworkError::CooldownHit {
            remaining_cooldown: _,
            ctx: _,
        } => todo!(),
        FrameworkError::MissingBotPermissions {
            missing_permissions: _,
            ctx: _,
        } => todo!(),
        FrameworkError::MissingUserPermissions {
            missing_permissions: _,
            ctx: _,
        } => todo!(),
        FrameworkError::NotAnOwner { ctx: _ } => todo!(),
        FrameworkError::GuildOnly { ctx: _ } => todo!(),
        FrameworkError::DmOnly { ctx: _ } => todo!(),
        FrameworkError::NsfwOnly { ctx: _ } => todo!(),
        FrameworkError::CommandCheckFailed { error: _, ctx: _ } => todo!(),
        FrameworkError::DynamicPrefix {
            error: _,
            ctx: _,
            msg: _,
        } => todo!(),
        FrameworkError::UnknownCommand {
            ctx: _,
            msg: _,
            prefix: _,
            msg_content: _,
            framework: _,
            invocation_data: _,
            trigger: _,
        } => todo!(),
        FrameworkError::UnknownInteraction {
            ctx: _,
            framework: _,
            interaction: _,
        } => todo!(),
        _ => todo!(),
    };

    if let Err(err_err) = res {
        log_unexpected_error(&err_err.to_string());
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![play(), pause(), resume(), skip()],
            on_error: |err| Box::pin(on_error(err)),
            ..Default::default()
        })
        .token(token)
        .intents(serenity::GatewayIntents::non_privileged())
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(State {
                    queues: Default::default()
                    // trackdata: Arc::new(Mutex::new(HashMap::new())),
                    // song_input_data: Arc::new(Mutex::new(HashMap::new())),
                })
            })
        })
        .client_settings(|builder| builder.register_songbird());

    framework.run().await.unwrap();
}
