use poise::{
    serenity_prelude::{CreateEmbed, Timestamp},
};
use tracing::{error, Value};

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub fn error_embed<'a>(create: &'a mut CreateEmbed, error: &Error) -> &'a mut CreateEmbed {
    create.title("Woopsie doodle, something happened owo")
        .description("An error occured because of (most likely) your incompetence :)")
        .thumbnail("https://raw.githubusercontent.com/Giftzwerg02/oxo/33856f5c3ad1549de092f7f58a83b05e1b060398/resources/unsafe-ferris-transparent.png")
        .field("Error", format!("```{:?}```", error), false)
        .footer(|f| f.text("XOXO"))
        .timestamp(Timestamp::now())
}


pub fn log_unexpected_error(error: &dyn Value) {
    error!(error = error, "Unexpected error occured");
}