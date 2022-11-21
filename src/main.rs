use actix_web::{web, App, HttpResponse, HttpServer};
use std::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let server =
        HttpServer::new(move || App::new().route("/links", web::get().to(links_get_handler)))
            .listen(TcpListener::bind("0.0.0.0:4000").unwrap())?
            .run();

    server.await
}

async fn links_get_handler() -> HttpResponse {
    HttpResponse::Ok().finish()
}
