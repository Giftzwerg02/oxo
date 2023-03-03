use std::sync::Arc;

use poise::{
    async_trait,
    serenity_prelude::{ChannelId, GuildId, Http, Mutex},
};

use songbird::{create_player, Call, Event, EventContext, EventHandler, TrackEvent};

use tracing::debug;

use crate::bot::{Context, LoopMode, Queues};

#[derive(Clone)]
pub struct EndEventHandler {
    http: Arc<Http>,
    channel_id: ChannelId,
    queues: Queues,
    call: Arc<Mutex<Call>>,
    guild_id: GuildId,
    loop_mode: Arc<Mutex<LoopMode>>,
    handler: Arc<Mutex<Call>>,
}

impl EndEventHandler {
    pub fn new(ctx: Context, handler: Arc<Mutex<Call>>, guild_id: GuildId) -> Self {        
        Self {
            channel_id: ctx.channel_id(),
            http: ctx.serenity_context().http.clone(),
            call: handler.clone(),
            guild_id,
            queues: ctx.data().queues.clone(),
            loop_mode: ctx.data().loop_mode.clone(),
            handler,
        }
    }
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

        let fallback_title = "No title found, no seriously this is not the name of the track - for some reason there just isn't one".to_string();
        let title = handle.metadata().title.as_ref().unwrap_or(&fallback_title);

        let _ = self
            .channel_id
            .say(&self.http, format!("Finished playing `{}` uwu", title))
            .await;

        let queues = self.queues.lock().await;

        let queue = queues.get(&self.guild_id).unwrap();

        let loop_mode = self.loop_mode.lock().await;

        let input = handle.metadata().source_url.as_ref().unwrap();

        let mut handler = self.handler.lock().await;

        match *loop_mode {
            LoopMode::Off => {}
            // track-looping is handled on command-execution once
            LoopMode::Track => {
                // TODO: broken, tracks are playing king of the hill and try to silence one another
                let input = songbird::ytdl(&input).await.unwrap();
                let (track, track_handle) = create_player(input);
                let _ = track_handle.add_event(Event::Track(TrackEvent::End), self.clone());

                let _ = queue.pause();
                queue.add(track, &mut handler);
                queue.modify_queue(|q| {
                    let last = q.pop_back().unwrap();
                    q.push_front(last);
                });
                let _ = queue.resume();
            }
            LoopMode::Queue => {
                let input = songbird::ytdl(&input).await.unwrap();
                let (track, track_handle) = create_player(input);
                let _ = track_handle.add_event(Event::Track(TrackEvent::End), self.clone());

                queue.add(track, &mut handler);
            }
        }

        if queue.is_empty() {
            let _ = self
                .channel_id
                .say(&self.http, "No more songs to play OwO, guess I'll go...")
                .await;

            let mut call = self.call.lock().await;
            let _ = call.leave().await;
        }

        None
    }
}
