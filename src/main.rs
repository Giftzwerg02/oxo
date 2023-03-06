mod bot;
mod commands;
mod embed_ext;
mod endpoints;
mod error;
mod events;

use bot::start_bot;
use dotenvy::dotenv;
use endpoints::api_server;

use crate::bot::State;

#[macro_export]
macro_rules! mugly {
    ($e:expr) => {
        std::sync::Arc::new(poise::serenity_prelude::Mutex::new($e))
    };
}

#[actix_web::main]
async fn main() {
    // Load dotenv
    dotenv().ok();

    // Init Logger
    tracing_subscriber::fmt::init();

    let state = mugly!(State::default());

    tokio::join!(
        // Start API Server
        api_server(state.clone()),
        // Start Discord Bot
        start_bot(state)
    );
}
