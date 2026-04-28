use actix_web::{Responder, get, web};

use crate::{
    common::{
        process::{ProcessListOptions, list},
        response::{ResponseResult, json_response},
    },
    container::handlers::machine,
};

#[get("/process")]
async fn process_endpoint(
    path: web::Path<String>,
    options: web::Query<ProcessListOptions>,
) -> ResponseResult<impl Responder> {
    let container = path.into_inner();
    let options = options.into_inner();

    list(machine(container), options).await.map(json_response)
}
