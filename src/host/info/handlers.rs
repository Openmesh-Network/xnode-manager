use actix_web::{Responder, get, web};

use crate::{
    common::{
        nix::{EvalQuery, FlakeQuery, eval, flake_metadata},
        response::{ResponseResult, json_response},
    },
    host::handlers::{config_dir, machine},
};

#[get("/flake/metadata")]
async fn flake_metadata_endpoint(query: web::Query<FlakeQuery>) -> ResponseResult<impl Responder> {
    flake_metadata(&query.flake, machine())
        .await
        .map(json_response)
}

#[get("/eval")]
async fn eval_endpoint(query: web::Query<EvalQuery>) -> ResponseResult<impl Responder> {
    let mut statement = query.statement.clone();

    if query.config.unwrap_or(false) {
        statement = format!(
            "{config_dir}#nixosConfigurations.xnode.{statement}",
            config_dir = config_dir().to_string_lossy()
        );
    }

    eval(&statement, machine()).await.map(json_response)
}
