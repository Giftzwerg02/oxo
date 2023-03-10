mod api;
mod client;
mod error;
mod mappers;

use api::endpoints::api_server;
use client::bot::start_bot;
use dotenvy::dotenv;

use client::bot::State;

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
