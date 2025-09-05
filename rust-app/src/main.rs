use std::{
    fs::{create_dir_all, remove_file},
    os::unix::net::UnixListener,
    path::Path,
};

use actix_cors::Cors;
use actix_web::{App, HttpServer, web};
use posix_acl::{ACL_READ, ACL_WRITE, PosixACL, Qualifier};
use usage::models::AppData as ResourceUsageAppData;
use utils::env::{
    backupdir, buildcores, commandstream, containerconfig, containerprofile, containersettings,
    containerstate, datadir, e2fsprogs, nix, nixosrebuild, osdir, socket, systemd,
};

use crate::{info::handlers::get_groups, utils::error::ResponseError};

mod config;
mod file;
mod info;
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
        create_dir_all(&dir)
            .unwrap_or_else(|e| panic!("Could not create data dir at {}: {}", dir.display(), e));
    }
    {
        let osdir = osdir();
        let dir = Path::new(&osdir);
        create_dir_all(dir)
            .unwrap_or_else(|e| panic!("Could not create OS dir at {}: {}", dir.display(), e));
    }
    {
        let dir = containersettings();
        create_dir_all(&dir).unwrap_or_else(|e| {
            panic!(
                "Could not create container settings dir at {}: {}",
                dir.display(),
                e
            )
        });
    }
    {
        let dir = containerstate();
        create_dir_all(&dir).unwrap_or_else(|e| {
            panic!(
                "Could not create container state dir at {}: {}",
                dir.display(),
                e
            )
        });
    }
    {
        let dir = containerprofile();
        create_dir_all(&dir).unwrap_or_else(|e| {
            panic!(
                "Could not create container profile dir at {}: {}",
                dir.display(),
                e
            )
        });
    }
    {
        let dir = containerconfig();
        create_dir_all(&dir).unwrap_or_else(|e| {
            panic!(
                "Could not create container config dir at {}: {}",
                dir.display(),
                e
            )
        });
    }
    {
        let dir = backupdir();
        create_dir_all(&dir)
            .unwrap_or_else(|e| panic!("Could not create backup dir at {}: {}", dir.display(), e));
    }
    {
        let dir = commandstream();
        create_dir_all(&dir).unwrap_or_else(|e| {
            panic!(
                "Could not create command stream dir at {}: {}",
                dir.display(),
                e
            )
        });
    }

    // Log env for debugging
    log::info!("Using env:");
    log::info!("SOCKET {}", socket().display());
    log::info!("DATADIR {}", datadir().display());
    log::info!("OSDIR {}", osdir());
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

    // Set socket permissions
    let path: std::path::PathBuf = socket();
    remove_file(&path)
        .unwrap_or_else(|e| log::warn!("Could not remove unix socket {}: {}", path.display(), e));
    let unix_socket = UnixListener::bind(&path)
        .unwrap_or_else(|e| panic!("Could not bind to unix socket {}: {}", path.display(), e));
    let mut socket_acl = PosixACL::new(0o660);

    let reverse_proxy_group = "xnode-reverse-proxy";
    match get_groups(None).and_then(|groups| {
        groups
            .into_iter()
            .find_map(|group| (group.name == reverse_proxy_group).then_some(Ok(group.id)))
            .unwrap_or_else(|| {
                Err(ResponseError::new(format!(
                    "Could not find group {}",
                    reverse_proxy_group
                )))
            })
    }) {
        Ok(group) => {
            socket_acl.set(Qualifier::Group(group), ACL_READ | ACL_WRITE);
        }
        Err(e) => {
            log::warn!(
                "Error assigning unix socket permission to reverse proxy group: {}",
                e.error
            )
        }
    };

    socket_acl.write_acl(&path).unwrap_or_else(|e| {
        panic!(
            "Could not set permissions on unix socket {}: {}",
            path.display(),
            e
        )
    });

    // Start server
    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .app_data(web::Data::new(ResourceUsageAppData::default()))
            .service(web::scope(&config::scope()).configure(config::configure))
            .service(web::scope(&file::scope()).configure(file::configure))
            .service(web::scope(&info::scope()).configure(info::configure))
            .service(web::scope(&os::scope()).configure(os::configure))
            .service(web::scope(&process::scope()).configure(process::configure))
            .service(web::scope(&usage::scope()).configure(usage::configure))
            .service(web::scope(&request::scope()).configure(request::configure))
    })
    .listen_uds(unix_socket)?
    .run()
    .await
}
