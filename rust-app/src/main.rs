use std::{fs::create_dir_all, os::unix::net::UnixListener};

use actix_cors::Cors;
use actix_web::{App, HttpServer};
use common::{
    env::{datadir, socket},
    error::ResponseError,
    info::get_groups,
};
use posix_acl::{ACL_READ, ACL_WRITE, PosixACL, Qualifier};

mod common;
mod host;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    // Create data directory
    {
        let dir = datadir();
        create_dir_all(&dir).unwrap_or_else(|e| {
            panic!(
                "Could not create data dir at {dir}: {e}",
                dir = dir.display()
            )
        });
    }

    // Create unix socket
    let path: std::path::PathBuf = socket();
    let unix_socket = UnixListener::bind(&path).unwrap_or_else(|e| {
        panic!(
            "Could not bind to unix socket {path}: {e}",
            path = path.display()
        )
    });
    let mut socket_acl = PosixACL::new(0o660);

    let reverse_proxy_group = "xnode-reverse-proxy";
    match get_groups(None).await.and_then(|groups| {
        groups
            .into_iter()
            .find(|group| group.name == reverse_proxy_group)
            .map(|group| group.id)
            .ok_or(ResponseError::new(format!(
                "Could not find group {reverse_proxy_group}"
            )))
    }) {
        Ok(group) => {
            socket_acl.set(Qualifier::Group(group), ACL_READ | ACL_WRITE);
        }
        Err(e) => {
            log::warn!("Error assigning unix socket permission to reverse proxy group: {e}")
        }
    };

    socket_acl.write_acl(&path).unwrap_or_else(|e| {
        panic!(
            "Could not set permissions on unix socket {path}: {e}",
            path = path.display()
        )
    });

    // Start server
    HttpServer::new(move || App::new().wrap(Cors::permissive()).service(host::service()))
        .listen_uds(unix_socket)?
        .run()
        .await
}
