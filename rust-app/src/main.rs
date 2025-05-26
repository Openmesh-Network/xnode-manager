use std::{fs::create_dir_all, path::Path};

use actix_cors::Cors;
use actix_identity::IdentityMiddleware;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, web, App, HttpServer};
use usage::models::AppData as ResourceUsageAppData;
use utils::env::{
    authdir, backupdir, buildcores, commandstream, containerconfig, containerprofile,
    containersettings, containerstate, datadir, e2fsprogs, hostname, nix, nixosrebuild, osdir,
    owner, port, systemd,
};

mod auth;
mod config;
mod file;
mod os;
mod process;
mod request;
mod usage;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

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
        let dir = containersettings();
        create_dir_all(&dir).inspect_err(|e| {
            log::error!(
                "Could not create container settings dir at {}: {}",
                dir.display(),
                e
            )
        })?;
    }
    {
        let dir = containerstate();
        create_dir_all(&dir).inspect_err(|e| {
            log::error!(
                "Could not create container state dir at {}: {}",
                dir.display(),
                e
            )
        })?;
    }
    {
        let dir = containerprofile();
        create_dir_all(&dir).inspect_err(|e| {
            log::error!(
                "Could not create container profile dir at {}: {}",
                dir.display(),
                e
            )
        })?;
    }
    {
        let dir = containerconfig();
        create_dir_all(&dir).inspect_err(|e| {
            log::error!(
                "Could not create container config dir at {}: {}",
                dir.display(),
                e
            )
        })?;
    }
    {
        let dir = backupdir();
        create_dir_all(&dir).inspect_err(|e| {
            log::error!("Could not create backup dir at {}: {}", dir.display(), e)
        })?;
    }
    {
        let dir = commandstream();
        create_dir_all(&dir).inspect_err(|e| {
            log::error!(
                "Could not create command stream dir at {}: {}",
                dir.display(),
                e
            )
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
    log::info!("CONTAINERSETTINGS {}", containersettings().display());
    log::info!("CONTAINERSTATE {}", containerstate().display());
    log::info!("CONTAINERPROFILE {}", containerprofile().display());
    log::info!("CONTAINERCONFIG {}", containerconfig().display());
    log::info!("BACKUPDIR {}", backupdir().display());
    log::info!("COMMANDSTREAM {}", commandstream().display());
    log::info!("BUILDCORES {}", buildcores());
    log::info!("NIX {}", nix());
    log::info!("NIXOSREBUILD {}", nixosrebuild());
    log::info!("SYSTEMD {}", systemd());
    log::info!("E2FSPROGS {}", e2fsprogs());

    // Start server
    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .wrap(IdentityMiddleware::default())
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&[0; 64]))
                    .build(),
            )
            .app_data(web::Data::new(ResourceUsageAppData::default()))
            .service(web::scope(&auth::scope()).configure(auth::configure))
            .service(web::scope(&config::scope()).configure(config::configure))
            .service(web::scope(&file::scope()).configure(file::configure))
            .service(web::scope(&os::scope()).configure(os::configure))
            .service(web::scope(&process::scope()).configure(process::configure))
            .service(web::scope(&usage::scope()).configure(usage::configure))
            .service(web::scope(&request::scope()).configure(request::configure))
    })
    .bind(format!("{}:{}", hostname(), port()))?
    .run()
    .await
}
