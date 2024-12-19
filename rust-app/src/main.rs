use actix_identity::IdentityMiddleware;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, web, App, HttpServer};
use serde::{Deserialize, Serialize};
use std::{env, sync::RwLock};

mod auth;
mod resource_usage;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct AppData {
    name: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let hostname = env::var("HOSTNAME").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "34391".to_string());

    let appdata = web::Data::new(RwLock::new(AppData {
        name: "Hi".to_string(),
    }));

    HttpServer::new(move || {
        App::new()
            .app_data(appdata.clone())
            .wrap(IdentityMiddleware::default())
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&[0; 64]))
                    .cookie_secure(false)
                    .build(),
            )
            .service(web::scope("/auth").configure(auth::configure))
            .service(web::scope("/usage").configure(resource_usage::configure))
    })
    .bind(format!("{}:{}", hostname, port))?
    .run()
    .await
}
