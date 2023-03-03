use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use poise::serenity_prelude::Mutex;
use serde::Serialize;

use crate::bot::State;

type DataState = web::Data<Arc<Mutex<State>>>;

#[derive(Serialize)]
struct PingStatus<'a> {
    status: &'a str,
}

#[get("/ping")]
async fn ping() -> impl Responder {
    HttpResponse::Ok().json(PingStatus { status: "owo" })
}

#[derive(Serialize)]
struct Track {
    title: String,
}

#[get("/queue")]
async fn queue(state: DataState) -> impl Responder {
    let state = state.lock().await;
    let queues = state.queues.lock().await;
    let queues = queues.values().collect::<Vec<_>>();
    let queue = queues.first().unwrap();
    let queue = queue.current_queue();
    let track = queue.first().unwrap();
    let res = track
        .metadata()
        .title
        .as_ref()
        .unwrap_or(&"sussy".to_string())
        .clone();

    HttpResponse::Ok().json(Track { title: res })
}

pub async fn api_server(state: Arc<Mutex<State>>) {
    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .wrap(cors)
            .app_data(DataState::new(state.clone()))
            .service(ping)
            .service(queue)
    })
    .bind(("0.0.0.0", 8080))
    .expect("Could not bind port")
    .run()
    .await
    .expect("Could not start server");
}
