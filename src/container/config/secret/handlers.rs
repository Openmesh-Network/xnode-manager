use actix_web::{Responder, get, post, web};

use crate::{
    common::{
        file::{create_folder, shift},
        path::get_scoped_path,
        response::{ResponseResult, json_response, raw_response},
        secret::{self, SecretOptions, list},
    },
    container::handlers::{config_dir, ensure_initialized, machine},
};

#[get("")]
async fn secret_endpoint(
    path: web::Path<String>,
    options: web::Query<SecretOptions>,
) -> ResponseResult<impl Responder> {
    let container = path.into_inner();
    let options = options.into_inner();

    let scope = ["container", &container];
    let path = get_scoped_path(&scope, &["data", "config", "xnode-config", "secret"]);
    list(path, machine(&container), options)
        .await
        .map(json_response)
}

#[get("/get")]
async fn get_endpoint(path: web::Path<(String, String)>) -> ResponseResult<impl Responder> {
    let (container, secret) = path.into_inner();
    get(&container, &secret).await.map(raw_response)
}

#[post("/set")]
async fn set_endpoint(
    path: web::Path<(String, String)>,
    data: web::Bytes,
) -> ResponseResult<impl Responder> {
    let (container, secret) = path.into_inner();
    ensure_initialized(&container).await?;

    let path = config_dir().join("xnode-config").join("secret");
    {
        create_folder(&path).await?;
        shift(&path, "foreign").await?;
    }
    secret::set(path.join(&secret), &data, machine(container))
        .await
        .map(raw_response)
}

async fn get(container: impl AsRef<str>, secret: impl AsRef<str>) -> ResponseResult<Vec<u8>> {
    let path = config_dir()
        .join("xnode-config")
        .join("secret")
        .join(secret.as_ref());
    secret::get(&path, machine(container)).await
}
