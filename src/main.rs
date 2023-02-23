use std::{path::Path, sync::Arc};

use dotenvy::dotenv;
use poise::{
    async_trait,
    serenity_prelude::{
        self as serenity, CacheHttp, ChannelId, EmbedImage, Http, Message, Mutex, Timestamp,
    },
    FrameworkError,
};
use songbird::{tracks::Track, Call, Event, EventContext, EventHandler, SerenityInit, TrackEvent};
mod error;
use error::{error_embed, Error};
use tracing::{info, debug, warn, error, trace};

struct Data {} // User data, which is stored and accessible in all command invocations
type Context<'a> = poise::Context<'a, Data, Error>;

struct LeaveAfterEndEventHandler {
    handler: Arc<Mutex<Call>>,
    http: Arc<Http>,
    channel_id: ChannelId,
}

#[async_trait]
impl EventHandler for LeaveAfterEndEventHandler {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        debug!(channel_id = self.channel_id.to_string(), "Triggered leave-after-end-event-handler");
    
        let mut handler = self.handler.lock().await;
        let res = handler.leave().await;

        if let Err(err) = res {
            self.channel_id
                .send_message(&self.http, |create| {
                    create.embed(|e| error_embed(e, &err.into()))
                })
                .await;
        }

        None
    }
}

/// Jamming
#[poise::command(slash_command, prefix_command)]
async fn play(
    ctx: Context<'_>,
    #[description = "URL"] url: String,
) -> Result<(), Error> {
    let guild = ctx.guild().unwrap();
    let channel_id = guild
        .voice_states
        .get(&ctx.author().id)
        .and_then(|vs| vs.channel_id)
        .unwrap();

    let serenity_ctx = ctx.serenity_context();

    let manager = songbird::get(serenity_ctx).await.unwrap().clone();

    let (handler, _) = manager.join(guild.id, channel_id).await;

    let mut handler_lock = handler.lock().await;
    let source = songbird::ytdl(&url).await?;

    let _ = handler_lock.play_source(source);

    handler_lock.add_global_event(
        Event::Track(TrackEvent::End),
        LeaveAfterEndEventHandler {
            handler: handler.clone(),
            channel_id: ctx.channel_id(),
            http: serenity_ctx.http.clone(),
        },
    );

    // tracker.add_event(Event::Track(TrackEvent::End), LeaveAfterEndEventHandler {
    //     handler: handler.clone(),
    //     poise_ctx: ctx.clone()
    // });

    Ok(())
}

async fn on_error(error: FrameworkError<'_, Data, Error>) {
    let res = match error {
        FrameworkError::Setup {
            error,
            framework,
            data_about_bot,
            ctx,
        } => todo!(),
        FrameworkError::EventHandler {
            error,
            ctx,
            event,
            framework,
        } => todo!(),
        FrameworkError::Command { error, ctx } => {
            error!(error = error, "Unexpected error occured");

            ctx.send(|create| create.embed(|e| error_embed(e, &error)))
                .await
        }
        FrameworkError::ArgumentParse { error, input, ctx } => todo!(),
        FrameworkError::CommandStructureMismatch { description, ctx } => todo!(),
        FrameworkError::CooldownHit {
            remaining_cooldown,
            ctx,
        } => todo!(),
        FrameworkError::MissingBotPermissions {
            missing_permissions,
            ctx,
        } => todo!(),
        FrameworkError::MissingUserPermissions {
            missing_permissions,
            ctx,
        } => todo!(),
        FrameworkError::NotAnOwner { ctx } => todo!(),
        FrameworkError::GuildOnly { ctx } => todo!(),
        FrameworkError::DmOnly { ctx } => todo!(),
        FrameworkError::NsfwOnly { ctx } => todo!(),
        FrameworkError::CommandCheckFailed { error, ctx } => todo!(),
        FrameworkError::DynamicPrefix { error, ctx, msg } => todo!(),
        FrameworkError::UnknownCommand {
            ctx,
            msg,
            prefix,
            msg_content,
            framework,
            invocation_data,
            trigger,
        } => todo!(),
        FrameworkError::UnknownInteraction {
            ctx,
            framework,
            interaction,
        } => todo!(),
        _ => todo!(),
    };

    if let Err(err_err) = res {
        tracing::error!(error = err_err.to_string(), "Unexpected error occured");
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![play()],
            on_error: |err| Box::pin(on_error(err)),
            ..Default::default()
        })
        .token(token)
        .intents(serenity::GatewayIntents::non_privileged())
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .client_settings(|builder| builder.register_songbird());

    framework.run().await.unwrap();
}
