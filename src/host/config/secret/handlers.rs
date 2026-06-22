use actix_web::{Responder, get, post, web};

use crate::{
    common::{
        file::create_folder,
        response::{ResponseResult, json_response, raw_response},
        secret::{self, SecretOptions, list},
    },
    host::handlers::{config_dir, machine},
};

#[get("")]
async fn secret_endpoint(options: web::Query<SecretOptions>) -> ResponseResult<impl Responder> {
    let options = options.into_inner();

    let path = config_dir().join("xnode-config").join("secret");
    list(&path, &path, machine(), options)
        .await
        .map(json_response)
}

#[get("/get")]
async fn get_endpoint(path: web::Path<String>) -> ResponseResult<impl Responder> {
    let secret = path.into_inner();
    get(&secret).await.map(raw_response)
}

#[post("/set")]
async fn set_endpoint(path: web::Path<String>, data: web::Bytes) -> ResponseResult<impl Responder> {
    let secret = path.into_inner();
    let path = config_dir().join("xnode-config").join("secret");
    {
        create_folder(&path).await?;
    }
    secret::set(path.join(&secret), &data, machine())
        .await
        .map(raw_response)
}

async fn get(secret: impl AsRef<str>) -> ResponseResult<Vec<u8>> {
    let path = config_dir()
        .join("xnode-config")
        .join("secret")
        .join(secret.as_ref());
    secret::get(&path, machine()).await
}
