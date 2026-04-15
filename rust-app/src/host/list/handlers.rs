use actix_web::{Responder, get};

use crate::{
    common::{
        env::datadir,
        file::{ReadFolderOptions, read_folder},
        process::list,
        response::{ResponseResult, json_response},
    },
    host::handlers::machine,
};

#[get("/process")]
async fn process_endpoint() -> ResponseResult<impl Responder> {
    list(machine()).await.map(json_response)
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
