use actix_web::{Responder, get, web};

use crate::{
    common::{
        nix::{EvalQuery, FlakeQuery, eval, flake_metadata},
        response::{ResponseResult, json_response},
    },
    container::handlers::{config_dir, ensure_initialized, machine},
};

#[get("/flake/metadata")]
async fn flake_metadata_endpoint(
    path: web::Path<String>,
    query: web::Query<FlakeQuery>,
) -> ResponseResult<impl Responder> {
    let container = path.into_inner();
    ensure_initialized(&container).await?;

    flake_metadata(&query.flake, machine(&container))
        .await
        .map(json_response)
}

#[get("/eval")]
async fn eval_endpoint(
    path: web::Path<String>,
    query: web::Query<EvalQuery>,
) -> ResponseResult<impl Responder> {
    let container = path.into_inner();
    ensure_initialized(&container).await?;

    let mut statement = query.statement.clone();

    if query.config.unwrap_or(false) {
        statement = format!(
            "{config_dir}#nixosConfigurations.xnode.{statement}",
            config_dir = config_dir().to_string_lossy()
        );
    }

    eval(&statement, machine(&container))
        .await
        .map(json_response)
}
