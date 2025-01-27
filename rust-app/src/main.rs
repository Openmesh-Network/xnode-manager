use std::fs::create_dir_all;

use actix_identity::IdentityMiddleware;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, web, App, HttpServer};
use utils::env::{backupdir, containerdir, datadir, hostname, port};

mod auth;
mod config;
mod processes;
mod resource_usage;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Clean up installation artifacts
    {
        let dir = std::path::Path::new("/old-root");
        let _ = std::fs::remove_dir_all(dir);
    }

    // Create data directories
    {
        let dir = datadir();
        create_dir_all(&dir)
            .inspect_err(|e| println!("Could not create data dir at {}: {}", dir.display(), e))?;
    }
    {
        let dir = containerdir();
        create_dir_all(&dir).inspect_err(|e| {
            println!("Could not create container dir at {}: {}", dir.display(), e)
        })?;
    }
    {
        let dir = backupdir();
        create_dir_all(&dir)
            .inspect_err(|e| println!("Could not create backup dir at {}: {}", dir.display(), e))?;
    }

    // Start server
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
            .service(web::scope("/config").configure(config::configure))
    })
    .bind(format!("{}:{}", hostname(), port()))?
    .run()
    .await
}
