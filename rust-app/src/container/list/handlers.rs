use actix_web::{Responder, get, web};

use crate::{
    common::{
        process::list,
        response::{ResponseResult, json_response},
    },
    container::handlers::machine,
};

#[get("/process")]
async fn process_endpoint(path: web::Path<String>) -> ResponseResult<impl Responder> {
    let container = path.into_inner();
    list(machine(container)).await.map(json_response)
}
