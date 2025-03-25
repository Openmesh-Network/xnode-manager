use std::{fs::create_dir_all, path::Path};

use actix_cors::Cors;
use actix_identity::IdentityMiddleware;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, web, App, HttpServer};
use utils::env::{
    authdir, backupdir, buildcores, containerdir, datadir, e2fsprogs, hostname, nix, osdir, owner,
    port, systemd,
};

mod auth;
mod config;
mod os;
mod processes;
mod resource_usage;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    // Clean up installation artifacts
    {
        let dir = std::path::Path::new("/old-root");
        let _ = std::fs::remove_dir_all(dir);
    }

    // Create data directories
    {
        let dir = datadir();
        create_dir_all(&dir).inspect_err(|e| {
            log::error!("Could not create data dir at {}: {}", dir.display(), e)
        })?;
    }
    {
        let osdir = osdir();
        let dir = Path::new(&osdir);
        create_dir_all(dir)
            .inspect_err(|e| log::error!("Could not create OS dir at {}: {}", dir.display(), e))?;
    }
    {
        let dir = authdir();
        create_dir_all(&dir).inspect_err(|e| {
            log::error!("Could not create auth dir at {}: {}", dir.display(), e)
        })?;
    }
    {
        let dir = containerdir();
        create_dir_all(&dir).inspect_err(|e| {
            log::error!("Could not create container dir at {}: {}", dir.display(), e)
        })?;
    }
    {
        let dir = backupdir();
        create_dir_all(&dir).inspect_err(|e| {
            log::error!("Could not create backup dir at {}: {}", dir.display(), e)
        })?;
    }

    // Log env for debugging
    log::info!("Using env:");
    log::info!("HOSTNAME {}", hostname());
    log::info!("PORT {}", port());
    log::info!("OWNER {}", owner());
    log::info!("DATADIR {}", datadir().display());
    log::info!("OSDIR {}", osdir());
    log::info!("AUTHDIR {}", authdir().display());
    log::info!("CONTAINERDIR {}", containerdir().display());
    log::info!("BACKUPDIR {}", backupdir().display());
    log::info!("BUILDCORES {}", buildcores());
    log::info!("NIX {}", nix());
    log::info!("SYSTEMD {}", systemd());
    log::info!("E2FSPROGS {}", e2fsprogs());

    // Start server
    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .wrap(IdentityMiddleware::default())
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&[0; 64]))
                    .cookie_secure(false)
                    .build(),
            )
            .service(web::scope("/auth").configure(auth::configure))
            .service(web::scope("/processes").configure(processes::configure))
            .service(web::scope("/usage").configure(resource_usage::configure))
            .service(web::scope("/os").configure(os::configure))
            .service(web::scope("/config").configure(config::configure))
    })
    .bind(format!("{}:{}", hostname(), port()))?
    .run()
    .await
}
