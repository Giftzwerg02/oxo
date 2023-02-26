use std::{
    collections::{HashMap, VecDeque},
    sync::Arc, time::Duration,
};

use dotenvy::dotenv;
use poise::{
    async_trait,
    serenity_prelude::{self as serenity, ChannelId, GuildId, Http, Mutex, Colour, Timestamp},
    FrameworkError,
};
use rand::thread_rng;
use rand::seq::SliceRandom;
use songbird::{
    tracks::{TrackQueue, TrackCommand},
    Call, Event, EventContext, EventHandler, SerenityInit, TrackEvent, create_player,
};
mod error;
use error::{error_embed, log_unexpected_error, Error};
use tracing::debug;


type Context<'a> = poise::Context<'a, State, Error>;
type CmdRes = Result<(), Error>;

type Queues = Arc<Mutex<HashMap<GuildId, TrackQueue>>>;

#[derive(Debug)]
struct State {
    queues: Queues,
}

struct EndEventHandler {
    http: Arc<Http>,
    channel_id: ChannelId,
    queues: Queues,
    call: Arc<Mutex<Call>>,  
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
        let [(_, handle)] = **tracks else {
            return None;
        };

        let fallback_title = "No title found,  no seriously this is not the name of the track - for some reason there just isn't one".to_string();
        let title = handle.metadata().title.as_ref().unwrap_or(&fallback_title);

        let _ = self.channel_id.say(&self.http, format!("Finished playing `{}` uwu", title))
            .await;

        let queues = self
            .queues
            .lock()
            .await;

        let queue = queues 
            .get(&self.guild_id)
            .unwrap(); 

        if queue.is_empty() {
            let _ = self.channel_id.say(&self.http, "No more songs to play OwO, guess I'll go...")
                .await;

            let mut call = self.call.lock().await;
            let _ = call.leave().await;
        }

        None
    }
}

struct QueueLoopHandler {
    guild_id: GuildId,
    queues: Queues,
    handler: Arc<Mutex<Call>>,
    loop_mode: LoopMode,
}

#[async_trait]
impl EventHandler for QueueLoopHandler {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        let queues = self.queues
            .lock()
            .await;
        
        let queue = queues 
            .get(&self.guild_id)
            .unwrap();

        let mut handler = self.handler
            .lock()
            .await;

        let EventContext::Track(tracks) = ctx else {
            return None;
        };

        // double de-ref LETS GO
        let [(_, track)] = **tracks else {
            return None;
        };
    
        let input = track.metadata().source_url.as_ref().unwrap();

        match self.loop_mode {
            LoopMode::Off => {},
            // track-looping is handled on command-execution once
            LoopMode::Track => {
                let input = songbird::ytdl(&input).await.unwrap();
                queue.add_source(input, &mut handler);
                queue.modify_queue(|q| q.rotate_right(1));
            },
            LoopMode::Queue => {
                let input = songbird::ytdl(&input).await.unwrap();
                queue.add_source(input, &mut handler);        
            },
        }
        None
    }
}

#[derive(Debug, poise::ChoiceParameter, Clone)]
enum LoopMode {
    Off,
    Track,
    Queue
}

/// And it goes on and on and on and on and ...
#[poise::command(slash_command, rename = "loop")]
async fn loop_mode(ctx: Context<'_>, loop_mode: LoopMode) -> CmdRes {
    let guild = ctx.guild().unwrap();
    let queues = ctx.data().queues.lock().await;
    let queue = queues.get(&guild.id)
        .unwrap();

    let manager = songbird::get(ctx.serenity_context()).await.unwrap().clone();
    let handler = manager.get(guild.id)
        .unwrap();

    for track in queue.current_queue() {
        track.add_event(Event::Track(TrackEvent::End), QueueLoopHandler {
            queues: ctx.data().queues.clone(),
            guild_id: guild.id,
            handler: handler.clone(),
            loop_mode: loop_mode.clone()
        })?;
    }

    Ok(())
}


/// In the beninging
#[poise::command(slash_command)]
async fn playtop(
    ctx: Context<'_>,
    #[description = "The track to send to the front"]
    track_number: usize,
) -> CmdRes {
    let guild = ctx.guild().unwrap();
    let queues = ctx.data().queues.lock().await;
    let queue = queues.get(&guild.id)
        .unwrap();

    if track_number <= 1 {
        return Ok(())
    }

    queue.pause()?;
    queue.modify_queue(|q| q.swap(0, track_number - 1));
    queue.resume()?;

    Ok(())
}


/// Harlem shake
#[poise::command(slash_command)]
async fn shuffle(ctx: Context<'_>) -> CmdRes {
    let guild = ctx.guild().unwrap();
    let queues = ctx.data().queues.lock().await;
    let queue = queues.get(&guild.id)
        .unwrap();

    queue.modify_queue(|q| {
        let mut rng = thread_rng();
        let [_, rest@..] = q.make_contiguous() else {
            return;
        };

        rest.shuffle(&mut rng);        
    });

    Ok(())
}


/// I WANT 'EM ALL - I WANT 'EM NOW
#[poise::command(slash_command)]
async fn queue(ctx: Context<'_>) -> CmdRes {
    let guild = ctx.guild().unwrap();
    let queues = ctx.data().queues.lock().await;
    let queue = queues.get(&guild.id)
        .unwrap();

    let list = queue.current_queue();

    let formatted_list = list
        .iter()
        .map(|track| format!("▷ {}", track.metadata().title.clone().unwrap_or("N/A".into())))
        .collect::<Vec<_>>()
        .join("\n");

    ctx.send(|create| create.embed(|e| e
        .title("Current Playlist")
        .description(format!("{} songs are currently queued", list.len()))
        .field("Queue", formatted_list, false)
        .colour(Colour::BLITZ_BLUE)
        .footer(|f| f.text("XOXO"))
        .timestamp(Timestamp::now())
    )).await?;

    Ok(())
}

/// Who asked?
#[poise::command(slash_command)]
async fn now_playing(ctx: Context<'_>) -> CmdRes {
    let guild = ctx.guild().unwrap();
    let queues = ctx.data().queues.lock().await;
    let queue = queues.get(&guild.id)
        .unwrap();

    let current = queue.current()
        .unwrap();


    let metadata = current.metadata();
    let track_info = current.get_info().await?;

    let colour = Colour::BLITZ_BLUE; 
    let title = metadata.title.clone().unwrap_or("N/A".into()); 
    let author = format!("By: {}", metadata.artist.clone().unwrap_or("N/A".into()));

    let thumbnail = metadata.thumbnail.clone().unwrap_or("https://www.keil.com/support/man/docs/ulinkplus/could_not_load_file.png".into());

    let total_duration = metadata.duration.unwrap_or_default();
    let already_played = track_info.position;


    fn duration_format(duration: &Duration) -> String {
        let seconds = duration.as_secs();
        let minutes = seconds / 60;
        let hours = minutes / 60;
        let minutes = minutes % 60;
        let seconds = seconds % 60;

        if hours > 0 {
            format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
        } else {
            format!("{:02}:{:02}", minutes, seconds)
        }
    }

    let description = format!("Duration: `{} / {}`", duration_format(&already_played), duration_format(&total_duration)); 
    let url = metadata.source_url.clone().unwrap_or("https://www.youtube.com/watch?v=dQw4w9WgXcQ".into());

    ctx.send(|create| 
        create
            .embed(|e| e
                .colour(colour)
                .title(title)
                .author(|a| a
                        .name(author)
                        .icon_url("https://cdn-icons-png.flaticon.com/512/2995/2995101.png"))
                .thumbnail(thumbnail)
                .description(description)
                .url(url)
                .footer(|f| f.text("XOXO"))
                .timestamp(Timestamp::now())
            )
        ).await?;

    Ok(())
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

    // track.add_event(Event::Track(TrackEvent::End), EndEventHandler {
    //     channel_id: ctx.channel_id(),
    //     http: ctx.serenity_context().http.clone(),
    //     call: handler.clone(),
    //     guild_id: guild.id,
    //     queues: ctx.data().queues.clone()
    // })?;

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
            commands: vec![play(), pause(), resume(), skip(), now_playing(), queue(), loop_mode(), shuffle(), playtop()],
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
