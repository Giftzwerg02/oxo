use actix_cors::Cors;
use actix_web::{HttpResponse, get, Responder, HttpServer, App};
use serde::Serialize;

#[derive(Serialize)]
struct PingStatus<'a> {
    status: &'a str
}

#[get("/ping")]
async fn ping() -> impl Responder {
    HttpResponse::Ok().json(PingStatus { status: "owo" })
}

pub async fn api_server() {
    HttpServer::new(|| {
        let cors = Cors::permissive();

        App::new()
            .wrap(cors)
            .service(ping)
    })
    .bind(("0.0.0.0", 8080))
    .expect("Could not bind port")
    .run()
    .await
    .expect("Could not start server")
}