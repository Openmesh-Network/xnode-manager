use std::path::PathBuf;

use crate::common::path::get_scoped_path;

pub fn machine() -> Option<String> {
    None
}

pub fn config_dir() -> PathBuf {
    let scope = ["host"];
    get_scoped_path(&scope, &["config"])
}
