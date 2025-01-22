use std::{collections::HashSet, env};

use actix_identity::Identity;

use super::models::Scope;

pub fn get_scopes(user: Identity) -> HashSet<Scope> {
    if env::var("OWNER").ok() == user.id().ok() {
        return HashSet::from([Scope::Processes, Scope::ResourceUsage]);
    }

    HashSet::new()
}

pub fn has_permission(user: Identity, scope: Scope) -> bool {
    get_scopes(user).contains(&scope)
}
