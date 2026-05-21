use actix_web::{dev::HttpServiceFactory, web};

pub mod cpu;
pub mod disk;
pub mod gpu;
pub mod memory;
pub mod network;

pub fn service() -> impl HttpServiceFactory {
    web::scope("/hardware")
        .service(cpu::service())
        .service(disk::service())
        .service(gpu::service())
        .service(memory::service())
        .service(network::service())
}
