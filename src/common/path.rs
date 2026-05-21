use std::path::PathBuf;

use super::env::datadir;

pub fn get_scope_root<SCOPE: AsRef<str>>(scope: &[SCOPE]) -> PathBuf {
    get_scoped_path(scope, &[] as &[&str])
}

pub fn get_scoped_path<SCOPE: AsRef<str>, PATH: AsRef<str>>(
    scope: &[SCOPE],
    path: &[PATH],
) -> PathBuf {
    let mut scoped_path = datadir();
    for part in scope {
        scoped_path.push(part.as_ref());
    }
    for part in path {
        scoped_path.push(part.as_ref());
    }
    scoped_path
}
