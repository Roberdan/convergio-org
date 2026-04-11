//! Knowledge item generation from a repo profile.

use crate::repo_scanner::RepoProfile;

use super::KnowledgeItem;

/// Generate knowledge items from a repo profile.
pub fn repo_knowledge_items(profile: &RepoProfile) -> Vec<KnowledgeItem> {
    let mut items = Vec::new();

    // Architecture: languages + frameworks
    let langs: Vec<String> = profile
        .languages
        .iter()
        .map(|(l, c)| format!("{l} ({c} files)"))
        .collect();
    let fw_list = if profile.frameworks.is_empty() {
        "none detected".to_string()
    } else {
        profile.frameworks.join(", ")
    };
    items.push(KnowledgeItem {
        title: "Tech Stack".to_string(),
        content: format!(
            "Languages: {}\nFrameworks: {}\nTotal files: {}, lines: {}",
            langs.join(", "),
            fw_list,
            profile.total_files,
            profile.total_lines
        ),
        category: "architecture".to_string(),
    });

    // Infra: manifest files
    if !profile.structure.manifest_files.is_empty() {
        items.push(KnowledgeItem {
            title: "Project Manifests".to_string(),
            content: format!(
                "Manifest files found:\n{}",
                profile
                    .structure
                    .manifest_files
                    .iter()
                    .map(|f| format!("- {f}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            ),
            category: "infra".to_string(),
        });
    }

    // CI info
    if let Some(ci) = &profile.ci {
        items.push(KnowledgeItem {
            title: "CI Configuration".to_string(),
            content: format!(
                "Provider: {}\nWorkflows: {}",
                ci.provider,
                ci.workflows.join(", ")
            ),
            category: "infra".to_string(),
        });
    }

    // Run guide
    items.push(KnowledgeItem {
        title: "How to Run".to_string(),
        content: run_command_for_profile(profile),
        category: "run_guide".to_string(),
    });

    // README summary
    if !profile.readme_summary.is_empty() {
        items.push(KnowledgeItem {
            title: "README Summary".to_string(),
            content: profile.readme_summary.clone(),
            category: "requirements".to_string(),
        });
    }

    // Key dependencies
    if !profile.dependencies.is_empty() {
        items.push(KnowledgeItem {
            title: "Key Dependencies".to_string(),
            content: profile
                .dependencies
                .iter()
                .map(|d| format!("- {d}"))
                .collect::<Vec<_>>()
                .join("\n"),
            category: "architecture".to_string(),
        });
    }

    items
}

fn run_command_for_profile(profile: &RepoProfile) -> String {
    let langs: Vec<String> = profile
        .languages
        .iter()
        .map(|(l, _)| l.to_lowercase())
        .collect();

    let has_rust = langs.iter().any(|l| l == "rust");
    let has_js = langs.iter().any(|l| l == "typescript" || l == "javascript");
    let has_python = langs.iter().any(|l| l == "python");

    let mut lines = vec!["# Run guide (auto-generated from repo scan)".to_string()];

    if has_rust {
        lines.push("## Rust".to_string());
        lines.push("cargo build --release".to_string());
        lines.push("cargo test --workspace".to_string());
        if profile
            .frameworks
            .iter()
            .any(|f| f == "Axum" || f == "Actix")
        {
            lines.push("cargo run -- --config config.toml".to_string());
        }
    }
    if has_js {
        lines.push("## Node.js / TypeScript".to_string());
        lines.push("npm install".to_string());
        if profile.frameworks.iter().any(|f| f == "Next.js") {
            lines.push("npm run dev   # development".to_string());
            lines.push("npm run build && npm start   # production".to_string());
        } else {
            lines.push("npm start".to_string());
        }
    }
    if has_python {
        lines.push("## Python".to_string());
        lines.push("pip install -r requirements.txt".to_string());
        if profile.frameworks.iter().any(|f| f == "Django") {
            lines.push("python manage.py runserver".to_string());
        } else if profile.frameworks.iter().any(|f| f == "FastAPI") {
            lines.push("uvicorn main:app --reload".to_string());
        } else {
            lines.push("python main.py".to_string());
        }
    }
    if lines.len() == 1 {
        lines.push("No standard run command detected. Check project README.".to_string());
    }

    lines.join("\n")
}
