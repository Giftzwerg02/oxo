use std::sync::Arc;

use poise::{
    serenity_prelude::{CreateEmbed, Timestamp, Mutex},
    FrameworkError,
};
use tokio::sync::OwnedMutexGuard;
use tracing::{error, Value};

use crate::bot::State;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub async fn on_error(error: FrameworkError<'_, Arc<Mutex<State>>, Error>) {
    let res = match error {
        FrameworkError::Setup {
            error: _,
            framework: _,
            data_about_bot: _,
            ctx: _,
        } => todo!(),
        FrameworkError::EventHandler {
            error: _,
            ctx: _,
            event: _,
            framework: _,
        } => todo!(),
        FrameworkError::Command { error, ctx } => {
            log_unexpected_error(&error);

            ctx.send(|create| create.embed(|e| error_embed(e, &error)))
                .await
        }
        FrameworkError::ArgumentParse {
            error: _,
            input: _,
            ctx: _,
        } => todo!(),
        FrameworkError::CommandStructureMismatch {
            description: _,
            ctx: _,
        } => todo!(),
        FrameworkError::CooldownHit {
            remaining_cooldown: _,
            ctx: _,
        } => todo!(),
        FrameworkError::MissingBotPermissions {
            missing_permissions: _,
            ctx: _,
        } => todo!(),
        FrameworkError::MissingUserPermissions {
            missing_permissions: _,
            ctx: _,
        } => todo!(),
        FrameworkError::NotAnOwner { ctx: _ } => todo!(),
        FrameworkError::GuildOnly { ctx: _ } => todo!(),
        FrameworkError::DmOnly { ctx: _ } => todo!(),
        FrameworkError::NsfwOnly { ctx: _ } => todo!(),
        FrameworkError::CommandCheckFailed { error: _, ctx: _ } => todo!(),
        FrameworkError::DynamicPrefix {
            error: _,
            ctx: _,
            msg: _,
        } => todo!(),
        FrameworkError::UnknownCommand {
            ctx: _,
            msg: _,
            prefix: _,
            msg_content: _,
            framework: _,
            invocation_data: _,
            trigger: _,
        } => todo!(),
        FrameworkError::UnknownInteraction {
            ctx: _,
            framework: _,
            interaction: _,
        } => todo!(),
        _ => todo!(),
    };

    if let Err(err_err) = res {
        log_unexpected_error(&err_err.to_string());
    }
}

fn log_unexpected_error(error: &dyn Value) {
    error!(error = error, "Unexpected error occured");
}

fn error_embed<'a>(create: &'a mut CreateEmbed, error: &Error) -> &'a mut CreateEmbed {
    create.title("Woopsie doodle, something happened owo")
        .description("An error occured because of (most likely) your incompetence :)")
        .thumbnail("https://raw.githubusercontent.com/Giftzwerg02/oxo/33856f5c3ad1549de092f7f58a83b05e1b060398/resources/unsafe-ferris-transparent.png")
        .field("Error", format!("```{:?}```", error), false)
        .footer(|f| f.text("XOXO"))
        .timestamp(Timestamp::now())
}
