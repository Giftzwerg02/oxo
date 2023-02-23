use std::path::Path;

use dotenvy::dotenv;
use poise::{serenity_prelude::{self as serenity, Timestamp, EmbedImage}, FrameworkError};
use songbird::SerenityInit;

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Jamming
#[poise::command(slash_command, prefix_command)]
async fn play(ctx: Context<'_>, #[description = "URL"] url: String) -> Result<(), Error> {
    let guild = ctx.guild().unwrap();
    let channel_id = guild
        .voice_states
        .get(&ctx.author().id)
        .and_then(|vs| vs.channel_id)
        .unwrap();

    let ctx = ctx.serenity_context();

    let manager = songbird::get(ctx).await.unwrap().clone();

    let (handler, _) = manager.join(guild.id, channel_id).await;

    let mut handler = handler.lock().await;
    let source = songbird::ytdl(&url).await?;

    handler.play_source(source);

    Ok(())
}

async fn on_error(error: FrameworkError<'_, Data, Error>) {
    let res = match error {
        FrameworkError::Setup { error, framework, data_about_bot, ctx } => todo!(),
        FrameworkError::EventHandler { error, ctx, event, framework } => todo!(),
        FrameworkError::Command { error, ctx } => {
            tracing::error!(error = error, "Unexpected error occured");

            ctx.send(|msg| {
                msg.embed(|e| {
                    e.title("Woopsie doodle, something happened owo")
                        .description("An error occured because of (most likely) your incompetence :)")
                        .thumbnail("https://raw.githubusercontent.com/Giftzwerg02/oxo/33856f5c3ad1549de092f7f58a83b05e1b060398/resources/unsafe-ferris-transparent.png")
                        .field("Error", format!("```{:?}```", error), false)
                        .footer(|f| f.text("XOXO"))
                        .timestamp(Timestamp::now())
                })
            }).await
        },
        FrameworkError::ArgumentParse { error, input, ctx } => todo!(),
        FrameworkError::CommandStructureMismatch { description, ctx } => todo!(),
        FrameworkError::CooldownHit { remaining_cooldown, ctx } => todo!(),
        FrameworkError::MissingBotPermissions { missing_permissions, ctx } => todo!(),
        FrameworkError::MissingUserPermissions { missing_permissions, ctx } => todo!(),
        FrameworkError::NotAnOwner { ctx } => todo!(),
        FrameworkError::GuildOnly { ctx } => todo!(),
        FrameworkError::DmOnly { ctx } => todo!(),
        FrameworkError::NsfwOnly { ctx } => todo!(),
        FrameworkError::CommandCheckFailed { error, ctx } => todo!(),
        FrameworkError::DynamicPrefix { error, ctx, msg } => todo!(),
        FrameworkError::UnknownCommand { ctx, msg, prefix, msg_content, framework, invocation_data, trigger } => todo!(),
        FrameworkError::UnknownInteraction { ctx, framework, interaction } => todo!(),
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
