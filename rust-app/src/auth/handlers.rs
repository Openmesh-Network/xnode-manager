use actix_identity::Identity;
use actix_web::{get, post, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use ethsign::Signature;

use crate::auth::models::LoginMethod;
use crate::utils::error::ResponseError;
use crate::utils::keccak::hash_message;

use super::models::{Login, Scope};
use super::utils::get_scopes;

#[get("/scopes")]
async fn scopes(user: Identity) -> impl Responder {
    let scopes = get_scopes(user);

    let response: Vec<&Scope> = Vec::from_iter(scopes.iter());
    HttpResponse::Ok().json(response)
}

#[post("/login")]
async fn login(login: web::Json<Login>, request: HttpRequest) -> impl Responder {
    let user: String;
    match login.login_method {
        LoginMethod::WalletSignature { v, r, s } => {
            let message = String::from("Create Xnode Manager session");
            let message_bytes = hash_message(&message);
            match (Signature { v, r, s }).recover(&message_bytes) {
                Ok(pubkey) => {
                    user = format!("eth:{}", hex::encode(pubkey.address()));
                }
                Err(e) => {
                    return HttpResponse::BadRequest().json(ResponseError::new(format!(
                        "Signature address recovery failed: {}",
                        e
                    )));
                }
            }
        }
    }

    if let Err(e) = Identity::login(&request.extensions(), user.clone()) {
        return HttpResponse::InternalServerError().json(ResponseError::new(format!(
            "Could not log in user {}: {}",
            user, e
        )));
    }

    HttpResponse::Ok().finish()
}

#[post("/logout")]
async fn logout(user: Identity) -> impl Responder {
    user.logout();
    HttpResponse::Ok()
}
