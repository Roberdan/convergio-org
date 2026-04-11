//! Project scanner tests.
#![allow(dead_code)]

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::project_scanner::{scan_project, RepoType};

static COUNTER: AtomicU32 = AtomicU32::new(0);

fn create_rust_service_repo(label: &str) -> PathBuf {
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "cvg_proj_test_{}_{}_{label}",
        std::process::id(),
        seq
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/main.rs"), "fn main() {}").unwrap();
    fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"my-service\"\n\n[dependencies]\naxum = \"0.7\"\nsqlx = \"0.7\"\n",
    )
    .unwrap();
    fs::write(dir.join("Dockerfile"), "FROM rust:latest\nCOPY . .").unwrap();
    dir
}

#[test]
fn scan_detects_service_type() {
    let dir = create_rust_service_repo("svc");
    let scan = scan_project(&dir).unwrap();
    assert!(scan.languages.iter().any(|(l, _)| l == "Rust"));
    assert!(!matches!(scan.repo_type, RepoType::Unknown));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn scan_detects_docker_infra() {
    let dir = create_rust_service_repo("docker");
    let scan = scan_project(&dir).unwrap();
    assert!(scan.infra.iter().any(|i| i.provider == "docker"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn scan_detects_postgres_from_deps() {
    let dir = create_rust_service_repo("pg");
    let scan = scan_project(&dir).unwrap();
    assert!(scan.services.iter().any(|s| s.name == "postgres"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn scan_monorepo_detection() {
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("cvg_mono_test_{}_{}", std::process::id(), seq));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/*\"]\n",
    )
    .unwrap();
    let scan = scan_project(&dir).unwrap();
    assert_eq!(scan.repo_type, RepoType::Monorepo);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn scan_rejects_nonexistent_path() {
    let result = scan_project(std::path::Path::new("/nonexistent/proj"));
    assert!(result.is_err());
}
