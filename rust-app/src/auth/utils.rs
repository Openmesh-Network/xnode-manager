use std::collections::HashSet;

use actix_identity::Identity;

use super::models::Scope;

pub fn get_scopes(user: Identity) -> HashSet<Scope> {
    match user.id() {
        Ok(_id) => HashSet::from([Scope::Read, Scope::Write]),
        Err(_) => HashSet::new(),
    }
}

pub fn has_permission(user: Identity, scope: Scope) -> bool {
    get_scopes(user).contains(&scope)
}
