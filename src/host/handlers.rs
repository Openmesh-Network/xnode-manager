use std::path::Path;

use crate::common::path::get_scoped_path;

pub fn machine() -> Option<impl AsRef<str>> {
    None::<String>
}

pub fn flake() -> impl AsRef<Path> {
    let scope = ["host"];
    get_scoped_path(&scope, &["config"])
}
