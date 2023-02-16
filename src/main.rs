use dotenvy::dotenv;
use poise::serenity_prelude::{self as serenity, CacheHttp, Message};
use songbird::SerenityInit;

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Jam
#[poise::command(slash_command, prefix_command)]
async fn play(
    ctx: Context<'_>,
    #[description = "URL"] url: String,
) -> Result<(), Error> {
    let guild = ctx.guild().unwrap();
    let channel_id = guild
        .voice_states.get(&ctx.author().id)
        .and_then(|vs| vs.channel_id)
        .unwrap();

    let ctx = ctx.serenity_context();

    let manager = songbird::get(ctx).await
            .unwrap()
            .clone();

    let (handler, _) = manager.join(guild.id, channel_id).await;

    let mut handler = handler.lock().await;
    let source = match songbird::ytdl(&url).await {
        Ok(source) => source,
        Err(reason) => {
            panic!("{}", reason)
        }
    };

    handler.play_source(source);

    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = std::env::var("DISCORD_TOKEN")
        .expect("missing DISCORD_TOKEN");

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![play()],
            ..Default::default()
        })
        .token(token)
        .intents(serenity::GatewayIntents::non_privileged())
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        }).client_settings(|builder| builder.register_songbird());

    framework.run().await.unwrap();
}