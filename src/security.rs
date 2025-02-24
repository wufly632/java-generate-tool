// src/security.rs
use std::path::{Path, PathBuf};
use std::collections::HashSet;

pub fn validate_path(
    target: &Path,
    allowed: &HashSet<PathBuf>
) -> Result<(), String> {
    let canonical = target.canonicalize()
        .map_err(|e| format!("Invalid path: {}", e))?;

    // if allowed.iter().any(|p| canonical.starts_with(p)) {
    //     Ok(())
    // } else {
    //     Err("Path not allowed".into())
    // }
}
