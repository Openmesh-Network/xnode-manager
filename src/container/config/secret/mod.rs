use actix_web::{dev::HttpServiceFactory, web};

pub mod handlers;
pub mod models;

pub fn service() -> impl HttpServiceFactory {
    web::scope("/secret").configure(|cfg| {
        cfg.service(handlers::secret_endpoint);
        cfg.service(web::scope("/{secret}").configure(|cfg| {
            cfg.service(handlers::get_endpoint);
            cfg.service(handlers::set_endpoint);
        }));
    })
}
