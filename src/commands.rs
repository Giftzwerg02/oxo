use std::{time::Duration, sync::Arc};

use poise::{
    serenity_prelude::{Colour, Timestamp, Mutex},
    Command,
};


use rand::{seq::SliceRandom, thread_rng};
use songbird::{Event, TrackEvent};

use crate::{
    bot::{Context, LoopMode, State},
    error::Error,
    events::EndEventHandler,
};

pub type CmdRes = Result<(), Error>;

macro_rules! commands {
    ( $($(#[$header:meta])* async fn $name:ident ($($args:tt)*) -> CmdRes $blk:block)* ) => {
        $(
            $(#[$header])* async fn $name ($($args)*) -> CmdRes $blk
        )*

        pub type PoiseCommand = Command<Arc<Mutex<State>>, Box<(dyn std::error::Error + Send + Sync + 'static)>>;

        pub fn commands<'a>() -> Result<Vec<PoiseCommand>, Error> {
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
                    track.metadata().title.clone().unwrap_or_else(|| "N/A".into())
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

        let colour = Colour::BLITZ_BLUE;
        let title = metadata.title.clone().unwrap_or_else(|| "N/A".into());
        let author = format!("By: {}", metadata.artist.clone().unwrap_or_else(|| "N/A".into()));

        let thumbnail = metadata.thumbnail.clone().unwrap_or_else(||
            "https://www.keil.com/support/man/docs/ulinkplus/could_not_load_file.png".into(),
        );

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

        let description = format!(
            "Duration: `{} / {}`",
            duration_format(&already_played),
            duration_format(&total_duration)
        );
        let url = metadata
            .source_url
            .clone()
            .unwrap_or_else(|| "https://www.youtube.com/watch?v=dQw4w9WgXcQ".into());

        ctx.send(|create| {
            create.embed(|e| {
                e.colour(colour)
                    .title(title)
                    .author(|a| {
                        a.name(author)
                            .icon_url("https://cdn-icons-png.flaticon.com/512/2995/2995101.png")
                    })
                    .thumbnail(thumbnail)
                    .description(description)
                    .url(url)
                    .footer(|f| f.text("XOXO"))
                    .timestamp(Timestamp::now())
            })
        })
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

        ctx.say("Loading your track...").await.ok();

        let manager = songbird::get(ctx.serenity_context()).await.unwrap().clone();

        let (handler, _) = manager.join(guild.id, channel_id).await;
        let mut handler_lock = handler.lock().await;

        let mut queues = state.queues.lock().await;
        let queue = queues.entry(guild.id).or_default();

        let source = songbird::ytdl(&url).await?;
        let track = queue.add_source(source, &mut handler_lock);

        ctx.send(|create| {
            create.content(format!(
                "Started playing `{}`",
                track.metadata().title.clone().unwrap_or_else(|| "N/A".into())
            ))
        })
        .await
        .ok();

        track.add_event(
            Event::Track(TrackEvent::End),
            EndEventHandler::new(ctx, &state, handler.clone(), guild.id),
        )?;

        Ok(())
    }
}
