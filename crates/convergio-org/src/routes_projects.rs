//! HTTP handlers for project scanning.
//!
//! - GET /api/projects/scan?path=<absolute_path>

use axum::extract::Query;
use axum::response::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::Path;

use crate::project_scanner::scan_project;

#[derive(Deserialize)]
pub struct ScanQuery {
    path: Option<String>,
}

/// Scan a repository at the given path and return a [`ProjectScan`] as JSON.
///
/// GET /api/projects/scan?path=/absolute/path/to/repo
/// If path is omitted, scans $HOME/GitHub.
pub async fn scan_project_handler(Query(q): Query<ScanQuery>) -> Json<Value> {
    let default_path = std::env::var("HOME")
        .map(|h| std::path::PathBuf::from(h).join("GitHub"))
        .unwrap_or_default();
    let scan_path = q
        .path
        .as_deref()
        .map(Path::new)
        .unwrap_or(default_path.as_path());
    match scan_project(scan_path) {
        Ok(scan) => Json(json!({"ok": true, "scan": scan})),
        Err(e) => Json(json!({"ok": false, "error": e})),
    }
}
