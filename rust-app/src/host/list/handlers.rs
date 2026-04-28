use actix_web::{Responder, get, web};

use crate::{
    common::{
        env::datadir,
        file::{ReadFolderOptions, read_folder},
        process::{ProcessListOptions, list},
        response::{ResponseResult, json_response},
    },
    host::handlers::machine,
};

#[get("/process")]
async fn process_endpoint(
    options: web::Query<ProcessListOptions>,
) -> ResponseResult<impl Responder> {
    let options = options.into_inner();

    list(machine(), options).await.map(json_response)
}

#[get("/container")]
async fn container_endpoint() -> ResponseResult<impl Responder> {
    let path = datadir().join("container");
    read_folder(path, &ReadFolderOptions { metadata: None })
        .await
        .map(|items| {
            items
                .into_iter()
                .map(|item| item.name)
                .collect::<Vec<String>>()
        })
        .map(json_response)
}
