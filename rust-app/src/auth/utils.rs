use actix_identity::Identity;
use std::collections::HashSet;

use crate::utils::env::owner;

use super::models::Scope;

pub fn get_scopes(user: Identity) -> HashSet<Scope> {
    if user.id().is_ok_and(|id| id == owner()) {
        return HashSet::from([
            Scope::Processes,
            Scope::ResourceUsage,
            Scope::OS,
            Scope::Config,
        ]);
    }

    HashSet::new()
}

pub fn has_permission(user: Identity, scope: Scope) -> bool {
    get_scopes(user).contains(&scope)
}
