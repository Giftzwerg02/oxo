use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{
    get,
    http::StatusCode,
    web::{self, Json},
    App, HttpResponse, HttpServer, Responder, ResponseError, Result,
};
use poise::serenity_prelude::{GuildId, Mutex};
use serde::Serialize;

use crate::{
    bot::bot::State,
    error::{self, AppError},
};

use super::types::Track;

type DataState = web::Data<Arc<Mutex<State>>>;

#[derive(Debug, Serialize)]
struct AppErrorResponse {
    error: String,
}

impl ResponseError for AppError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self.error_type {
            error::AppErrorType::NotFound => StatusCode::NOT_FOUND,
        }
    }

    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status_code()).json(AppErrorResponse {
            error: self.message(),
        })
    }
}

#[derive(Serialize)]
struct PingStatus<'a> {
    status: &'a str,
}

#[get("/ping")]
async fn ping() -> impl Responder {
    HttpResponse::Ok().json(PingStatus { status: "owo" })
}

#[get("/guilds/with_queues")]
async fn guilds_with_queues(state: DataState) -> Result<Json<Vec<GuildId>>> {
    let state = state.lock().await;
    let queues = state.queues.lock().await;
    let guild_ids = queues.keys().cloned().collect();

    Ok(Json(guild_ids))
}

#[get("/queues/queue/{guild_id}")]
async fn queue(state: DataState, guild_id: web::Path<u64>) -> Result<Json<Vec<Track>>> {
    let guild_id = GuildId(*guild_id);
    let state = state.lock().await;
    let queues = state.queues.lock().await;
    let queue = queues.get(&guild_id).ok_or(AppError::not_found())?;

    let tracks = queue
        .current_queue()
        .into_iter()
        .map(Track::from)
        .collect::<Vec<_>>();

    Ok(Json(tracks))
}

pub async fn api_server(state: Arc<Mutex<State>>) {
    let host = std::env::var("API_HOST").unwrap_or("0.0.0.0".to_owned());
    let port = std::env::var("API_PORT")
            .unwrap_or("8080".to_owned())
            .parse()
            .expect("Incorrect value for env variable API_PORT");
    
    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .wrap(cors)
            .app_data(DataState::new(state.clone()))
            .service(
                web::scope("/api")
                    .service(ping)
                    .service(guilds_with_queues)
                    .service(queue),
            )
    })
    .bind((host, port))
    .expect("Could not bind port")
    .run()
    .await
    .expect("Could not start server");
}
