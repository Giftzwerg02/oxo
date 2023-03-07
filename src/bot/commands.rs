use std::sync::Arc;

use poise::{
    serenity_prelude::{Colour, Mutex, Timestamp},
    Command,
};

use rand::{seq::SliceRandom, thread_rng};
use songbird::{Event, TrackEvent};
use tracing::warn;

use crate::error::Error;

use crate::bot::{
    bot::{Context, LoopMode, State},
    embed_ext::CreateEmbedExt,
    events::EndEventHandler,
};

pub type CmdRes = Result<(), Error>;

macro_rules! commands {
    ( $($(#[$header:meta])* async fn $name:ident ($($args:tt)*) -> CmdRes $blk:block)* ) => {
        $(
            $(#[$header])* async fn $name ($($args)*) -> CmdRes $blk
        )*

        pub type PoiseCommand = Command<Arc<Mutex<State>>, Box<(dyn std::error::Error + Send + Sync + 'static)>>;

        pub fn commands() -> Result<Vec<PoiseCommand>, Error> {
            return Ok(vec![
                $(
                    $name(),
                )*
            ])
        }
    };
}

commands! {
/// And it goes on and on and on and on and ...
#[poise::command(slash_command, rename = "loop")]
async fn loop_mode(ctx: Context<'_>, loop_mode: LoopMode) -> CmdRes {
    let state = ctx.data().lock().await;
    let mut global_loop_mode = state.loop_mode.lock().await;
    *global_loop_mode = loop_mode;

    ctx.send(|create| create.embed(|e| e.info_embed(format!("Set loop-mode to {global_loop_mode}"))))
        .await?;

    Ok(())
}

/// In the beninging
#[poise::command(slash_command)]
async fn playtop(
    ctx: Context<'_>,
    #[description = "The track to send to the front"] track_number: usize,
) -> CmdRes {
    let state = ctx.data().lock().await;
    let guild = ctx.guild().unwrap();
    let queues = state.queues.lock().await;
    let queue = queues.get(&guild.id).unwrap();

    if track_number <= 1 {
        return Ok(());
    }

    queue.pause()?;
    queue.modify_queue(|q| q.swap(0, track_number - 1));
    queue.resume()?;

    Ok(())
}

/// Harlem shake
#[poise::command(slash_command)]
async fn shuffle(ctx: Context<'_>) -> CmdRes {
    let state = ctx.data().lock().await;
    let guild = ctx.guild().unwrap();
    let queues = state.queues.lock().await;
    let queue = queues.get(&guild.id).unwrap();

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
    let state = ctx.data().lock().await;
    let guild = ctx.guild().unwrap();
    let queues = state.queues.lock().await;
    let queue = queues.get(&guild.id).unwrap();

    let list = queue.current_queue();

    let formatted_list = list
        .iter()
        .map(|track| {
            format!(
                "▷ {}",
                track
                    .metadata()
                    .title
                    .clone()
                    .unwrap_or_else(|| "N/A".into())
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    ctx.send(|create| {
        create.embed(|e| {
            e.title("Current Playlist")
                .description(format!("{} songs are currently queued", list.len()))
                .field("Queue", formatted_list, false)
                .colour(Colour::BLITZ_BLUE)
                .footer(|f| f.text("XOXO"))
                .timestamp(Timestamp::now())
        })
    })
    .await?;

    Ok(())
}

/// Who asked?
#[poise::command(slash_command)]
async fn now_playing(ctx: Context<'_>) -> CmdRes {
    let state = ctx.data().lock().await;
    let guild = ctx.guild().unwrap();
    let queues = state.queues.lock().await;
    let queue = queues.get(&guild.id).unwrap();

    let current = queue.current().unwrap();

    let metadata = current.metadata();
    let track_info = current.get_info().await?;

    ctx.send(|create| create.embed(|e| e.song_embed(metadata, &track_info)))
        .await?;

    Ok(())
}

/// Hol' up
#[poise::command(slash_command)]
async fn pause(ctx: Context<'_>) -> CmdRes {
    let state = ctx.data().lock().await;
    let guild = ctx.guild().unwrap();
    let queues = state.queues.lock().await;
    let queue = queues.get(&guild.id).unwrap();

    queue.pause()?;

    Ok(())
}

/// Keep going
#[poise::command(slash_command)]
async fn resume(ctx: Context<'_>) -> CmdRes {
    let state = ctx.data().lock().await;
    let guild = ctx.guild().unwrap();
    let queues = state.queues.lock().await;
    let queue = queues.get(&guild.id).unwrap();

    queue.resume()?;

    Ok(())
}

/// Don't care
#[poise::command(slash_command)]
async fn skip(ctx: Context<'_>) -> CmdRes {
    let state = ctx.data().lock().await;
    let guild = ctx.guild().unwrap();
    let queues = state.queues.lock().await;
    let queue = queues.get(&guild.id).unwrap();

    queue.skip()?;

    Ok(())
}

/// Jamming
#[poise::command(slash_command)]
async fn play(ctx: Context<'_>, #[description = "URL"] url: String) -> CmdRes {
    let state = ctx.data().lock().await;
    let guild = ctx.guild().unwrap();
    let channel_id = guild
        .voice_states
        .get(&ctx.author().id)
        .and_then(|vs| vs.channel_id)
        .unwrap();

    if let Err(warn) = ctx.say("Loading your track...").await {
        warn!("{warn}");
    }

    let manager = state.songbird_instance.clone();
    
    let source = songbird::ytdl(&url).await?;

    let (handler, _) = manager.join(guild.id, channel_id).await;
    let mut handler_lock = handler.lock().await;

    let mut queues = state.queues.lock().await;
    let queue = queues.entry(guild.id).or_default();

    let track = queue.add_source(source, &mut handler_lock);

    let metadata = track.metadata();
    let track_info = track.get_info().await?;

    ctx.send(|create| create.embed(|e| e.song_embed(metadata, &track_info)))
        .await?;

    track.add_event(
        Event::Track(TrackEvent::End),
        EndEventHandler::new(ctx, &state, handler.clone(), guild.id),
    )?;

    Ok(())
}
}
