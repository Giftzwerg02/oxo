use std::{fmt, sync::Arc};

use poise::{
    serenity_prelude::{CreateEmbed, Mutex},
    FrameworkError,
};

use tracing::{error, Value};

use crate::bot::{bot::State, embed_ext::CreateEmbedExt};

pub type Error = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug)]
pub enum AppErrorType {
    NotFound,
}

#[derive(Debug)]
pub struct AppError {
    pub cause: Option<String>,
    pub message: Option<String>,
    pub error_type: AppErrorType,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl AppError {
    pub fn message(&self) -> String {
        match self {
            AppError {
                cause: _,
                message: Some(message),
                error_type: _,
            } => message.clone(),
            AppError {
                cause: _,
                message: None,
                error_type: AppErrorType::NotFound,
            } => "The requested item was not found".to_string(),
            _ => "An unexpected error has occured".to_string(),
        }
    }

    pub fn not_found() -> Self {
        AppError {
            cause: None,
            message: None,
            error_type: AppErrorType::NotFound,
        }
    }
}

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
    create
        .error_styling()
        .title("Woopsie doodle, something happened owo")
        .description("An error occured because of (most likely) your incompetence :)")
        .field("Error", format!("```{:?}```", error), false)
}
