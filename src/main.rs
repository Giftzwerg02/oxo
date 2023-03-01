mod commands;
mod events;
mod endpoints;
mod error;
mod bot;

use bot::start_bot;
use dotenvy::dotenv;
use endpoints::api_server;

#[actix_web::main]
async fn main() {
    // Load dotenv
    dotenv().ok();

    // Init Logger
    tracing_subscriber::fmt::init();

    tokio::join!(
        // Start API Server
        api_server(), 

        // Start Discord Bot
        start_bot()
    );
}
