use std::path::{Path, PathBuf};

use axum::http::StatusCode;

pub(super) fn resolve_audio_path(directory: &Path, requested: &str) -> Result<PathBuf, StatusCode> {
    if requested.contains('\0') || requested.ends_with('/') {
        return Err(StatusCode::NOT_FOUND);
    }
    let names = normalized_names(requested)?;
    if names.iter().any(|name| name.starts_with('.')) {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(names
        .iter()
        .fold(directory.to_path_buf(), |path, name| path.join(name)))
}

fn normalized_names(requested: &str) -> Result<Vec<&str>, StatusCode> {
    requested.split('/').try_fold(Vec::new(), step)
}

fn step<'text>(
    mut names: Vec<&'text str>,
    part: &'text str,
) -> Result<Vec<&'text str>, StatusCode> {
    match part {
        "" | "." => Ok(names),
        ".." => names.pop().map(|_| names).ok_or(StatusCode::NOT_FOUND),
        name => {
            names.push(name);
            Ok(names)
        }
    }
}
