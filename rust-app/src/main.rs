use actix_identity::IdentityMiddleware;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, web, App, HttpServer};
use std::env;

mod auth;
mod processes;
mod resource_usage;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let hostname = env::var("HOSTNAME").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "34391".to_string());

    HttpServer::new(move || {
        App::new()
            .wrap(IdentityMiddleware::default())
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&[0; 64]))
                    .cookie_secure(false)
                    .build(),
            )
            .service(web::scope("/auth").configure(auth::configure))
            .service(web::scope("/processes").configure(processes::configure))
            .service(web::scope("/usage").configure(resource_usage::configure))
    })
    .bind(format!("{}:{}", hostname, port))?
    .run()
    .await
}
