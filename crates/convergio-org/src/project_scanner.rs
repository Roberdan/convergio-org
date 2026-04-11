//! Project scanner — detects repo type, framework, services, and infra from config files.
//!
//! Produces a [`ProjectScan`] that downstream tasks (org template generator, onboarding) use.

use std::fs;
use std::path::Path;

use serde::Serialize;

use crate::repo_scanner::{scan_repo, RepoProfile};

// ── Public types ─────────────────────────────────────────────────────────────

/// High-level classification of the repository.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RepoType {
    Monorepo,
    FullStack,
    Service,
    Frontend,
    Library,
    Cli,
    Mobile,
    Unknown,
}

/// A running service (database, cache, queue, …) detected from config files.
#[derive(Debug, Clone, Serialize)]
pub struct ServiceInfo {
    /// Canonical name, e.g. `"postgres"`, `"redis"`.
    pub name: String,
    /// Category: `"database"`, `"cache"`, `"queue"`, `"search"`, `"storage"`.
    pub service_type: String,
    /// Config file where this was found, e.g. `"docker-compose.yml"`.
    pub detected_from: String,
}

/// Infrastructure / deployment platform detected from config files.
#[derive(Debug, Clone, Serialize)]
pub struct InfraInfo {
    /// Provider name, e.g. `"vercel"`, `"fly.io"`, `"docker"`, `"kubernetes"`.
    pub provider: String,
    /// Config file that confirmed the provider, e.g. `"fly.toml"`.
    pub config_file: String,
}

/// Full project scan result.
#[derive(Debug, Serialize)]
pub struct ProjectScan {
    pub path: String,
    pub repo_type: RepoType,
    /// Primary languages sorted by file count.
    pub languages: Vec<(String, usize)>,
    /// Detected frameworks, e.g. `["Next.js", "Axum"]`.
    pub frameworks: Vec<String>,
    /// External services the project depends on.
    pub services: Vec<ServiceInfo>,
    /// Deployment / infra platforms detected.
    pub infra: Vec<InfraInfo>,
    /// First 500 chars of README.
    pub readme_summary: String,
    pub total_files: usize,
    pub total_lines: usize,
}

// ── Entry point ──────────────────────────────────────────────────────────────

/// Scan `path` and produce a [`ProjectScan`].
pub fn scan_project(path: &Path) -> Result<ProjectScan, String> {
    convergio_types::platform_paths::validate_path_components(path)
        .map_err(|e| format!("path validation failed: {e}"))?;
    let profile: RepoProfile = scan_repo(path)?;
    let repo_type = detect_repo_type(path, &profile);
    let services = detect_services(path);
    let infra = detect_infra(path);
    Ok(ProjectScan {
        path: profile.path,
        repo_type,
        languages: profile.languages,
        frameworks: profile.frameworks,
        services,
        infra,
        readme_summary: profile.readme_summary,
        total_files: profile.total_files,
        total_lines: profile.total_lines,
    })
}

// ── RepoType detection ───────────────────────────────────────────────────────

fn detect_repo_type(root: &Path, profile: &RepoProfile) -> RepoType {
    // Monorepo: multiple top-level workspace members or multiple manifests
    if is_monorepo(root) {
        return RepoType::Monorepo;
    }

    let has_frontend = profile
        .frameworks
        .iter()
        .any(|f| matches!(f.as_str(), "Next.js" | "React" | "Vue"));
    let has_backend = profile.frameworks.iter().any(|f| {
        matches!(
            f.as_str(),
            "Axum" | "Actix" | "Django" | "Flask" | "FastAPI"
        )
    });
    let has_rust = profile.languages.iter().any(|(l, _)| l == "Rust");
    let has_js = profile
        .languages
        .iter()
        .any(|(l, _)| l == "TypeScript" || l == "JavaScript");

    // Mobile
    if root.join("Package.swift").exists()
        || root.join("android").is_dir()
        || root.join("ios").is_dir()
    {
        return RepoType::Mobile;
    }

    // FullStack = frontend + backend both present
    if has_frontend && (has_backend || has_rust) {
        return RepoType::FullStack;
    }

    // Pure frontend
    if has_frontend && has_js && !has_backend {
        return RepoType::Frontend;
    }

    // CLI: has Cargo.toml with [[bin]] only and no axum
    if has_rust && !has_backend && is_cli_crate(root) {
        return RepoType::Cli;
    }

    // Library: Cargo.toml with [lib] and no binary
    if has_rust && is_library_crate(root) {
        return RepoType::Library;
    }

    // Default backend/service
    if has_backend || has_rust || has_js {
        return RepoType::Service;
    }

    RepoType::Unknown
}

fn is_monorepo(root: &Path) -> bool {
    // Cargo workspace
    if let Ok(content) = fs::read_to_string(root.join("Cargo.toml")) {
        if content.contains("[workspace]") {
            return true;
        }
    }
    // npm workspaces
    if let Ok(content) = fs::read_to_string(root.join("package.json")) {
        if content.contains("\"workspaces\"") {
            return true;
        }
    }
    false
}

fn is_cli_crate(root: &Path) -> bool {
    if let Ok(content) = fs::read_to_string(root.join("Cargo.toml")) {
        return content.contains("[[bin]]") && !content.contains("axum");
    }
    false
}

fn is_library_crate(root: &Path) -> bool {
    if let Ok(content) = fs::read_to_string(root.join("Cargo.toml")) {
        return content.contains("[lib]") && !content.contains("[[bin]]");
    }
    false
}

// ── Services detection ───────────────────────────────────────────────────────

fn detect_services(root: &Path) -> Vec<ServiceInfo> {
    let mut services: Vec<ServiceInfo> = Vec::new();
    detect_compose_services(root, &mut services);
    detect_dep_services(root, &mut services);
    services
}

const DB_IMAGES: &[(&str, &str)] = &[
    ("postgres", "database"),
    ("mysql", "database"),
    ("mariadb", "database"),
    ("mongodb", "database"),
    ("redis", "cache"),
    ("valkey", "cache"),
    ("rabbitmq", "queue"),
    ("kafka", "queue"),
    ("elasticsearch", "search"),
    ("meilisearch", "search"),
    ("minio", "storage"),
];

fn detect_compose_services(root: &Path, services: &mut Vec<ServiceInfo>) {
    for name in &[
        "docker-compose.yml",
        "docker-compose.yaml",
        "compose.yml",
        "compose.yaml",
    ] {
        let path = root.join(name);
        if let Ok(content) = fs::read_to_string(&path) {
            for (image, svc_type) in DB_IMAGES {
                if content.contains(image) && !services.iter().any(|s| s.name == *image) {
                    services.push(ServiceInfo {
                        name: image.to_string(),
                        service_type: svc_type.to_string(),
                        detected_from: name.to_string(),
                    });
                }
            }
        }
    }
}

fn detect_dep_services(root: &Path, services: &mut Vec<ServiceInfo>) {
    // Cargo.toml deps: sqlx/postgres, redis, etc.
    for cargo_path in &[root.join("Cargo.toml"), root.join("daemon/Cargo.toml")] {
        if let Ok(content) = fs::read_to_string(cargo_path) {
            let src = "Cargo.toml";
            if (content.contains("sqlx") || content.contains("postgres"))
                && !services.iter().any(|s| s.name == "postgres")
            {
                services.push(ServiceInfo {
                    name: "postgres".to_string(),
                    service_type: "database".to_string(),
                    detected_from: src.to_string(),
                });
            }
            if content.contains("redis") && !services.iter().any(|s| s.name == "redis") {
                services.push(ServiceInfo {
                    name: "redis".to_string(),
                    service_type: "cache".to_string(),
                    detected_from: src.to_string(),
                });
            }
        }
    }
}

// ── Infra detection ──────────────────────────────────────────────────────────

fn detect_infra(root: &Path) -> Vec<InfraInfo> {
    let mut infra: Vec<InfraInfo> = Vec::new();

    let checks: &[(&str, &str, &str)] = &[
        ("vercel.json", "vercel", "vercel.json"),
        (".vercel", "vercel", ".vercel/"),
        ("fly.toml", "fly.io", "fly.toml"),
        ("netlify.toml", "netlify", "netlify.toml"),
        ("render.yaml", "render", "render.yaml"),
        ("railway.toml", "railway", "railway.toml"),
        ("Dockerfile", "docker", "Dockerfile"),
        ("docker-compose.yml", "docker", "docker-compose.yml"),
        ("compose.yml", "docker", "compose.yml"),
    ];

    for (file, provider, config) in checks {
        let p = root.join(file);
        if p.exists() && !infra.iter().any(|i| i.provider == *provider) {
            infra.push(InfraInfo {
                provider: provider.to_string(),
                config_file: config.to_string(),
            });
        }
    }

    // Kubernetes: k8s/ or kubernetes/ directory, or *.k8s.yaml
    if root.join("k8s").is_dir() || root.join("kubernetes").is_dir() {
        infra.push(InfraInfo {
            provider: "kubernetes".to_string(),
            config_file: "k8s/".to_string(),
        });
    }

    // Terraform: any .tf file at root level
    if let Ok(entries) = fs::read_dir(root) {
        let has_tf = entries
            .flatten()
            .any(|e| e.path().extension().and_then(|x| x.to_str()) == Some("tf"));
        if has_tf {
            infra.push(InfraInfo {
                provider: "terraform".to_string(),
                config_file: "*.tf".to_string(),
            });
        }
    }

    infra
}
