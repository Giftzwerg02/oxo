use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{
    get,
    http::StatusCode,
    post,
    web::{self, Json, Path},
    App, HttpResponse, HttpServer, Responder, ResponseError, Result,
};
use poise::serenity_prelude::{GuildId, Mutex};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
    client::bot::State,
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
            error::AppErrorType::SongbirdError(_) => StatusCode::INTERNAL_SERVER_ERROR,
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
async fn queue(state: DataState, guild_id: Path<u64>) -> Result<Json<Vec<Track>>> {
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

#[derive(Deserialize)]
struct TrackUrl {
    track_url: String,
}

#[post("/queues/queue/{guild_id}/add-song")]
async fn add_song_to_queue(
    state: DataState,
    guild_id: Path<u64>,
    track_url: Json<TrackUrl>,
) -> Result<Json<Track>> {
    let guild_id = GuildId(*guild_id);
    let state = state.lock().await;
    let queues = state.queues.lock().await;

    // REF Suggestion: Make clippy shut up, somehow
    let this_queue = queues.get(&guild_id).ok_or(AppError::not_found())?;

    let voice = state.songbird_instance.clone();
    let handler = voice.get(guild_id).ok_or(AppError::not_found())?;

    let source = songbird::ytdl(track_url.0.track_url)
        .await
        .map_err(AppError::from)?;

    let mut handler = handler.lock().await;

    let track = this_queue.add_source(source, &mut handler);

    Ok(Json(track.into()))
}

pub async fn api_server(state: Arc<Mutex<State>>) {
    if std::env::var("DISABLE_WEB_API").is_ok() {
        info!("Not starting api-server because env variable DISABLE_WEB_API is set");
        return;
    }

    // REF Suggestion: Make clippy shut up, somehow
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
                    .service(queue)
                    .service(add_song_to_queue),
            )
    })
    .bind((host, port))
    .expect("Could not bind port")
    .run()
    .await
    .expect("Could not start server");
}
