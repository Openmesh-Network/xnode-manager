use actix_cors::Cors;
use actix_web::{App, HttpServer};
use common::env::socket;

mod common;
mod container;
mod host;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    container::prepare_module()
        .await
        .unwrap_or_else(|e| panic!("Could not prepare container module: {e}"));

    // Start server
    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .service(host::service())
            .service(container::service())
    })
    .bind_uds(socket())?
    .run()
    .await
}
