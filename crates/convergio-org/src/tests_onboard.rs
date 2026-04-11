//! Onboard wiring tests — leadership, mission derivation, dotfiles.
#![allow(dead_code)]

use crate::factory::*;
use crate::onboard_dotfiles::generate_convergio_dir;
use crate::repo_scanner::{RepoProfile, RepoStructure};
use std::fs;

fn test_profile(path: &str) -> RepoProfile {
    RepoProfile {
        path: path.to_string(),
        languages: vec![("Rust".to_string(), 5000)],
        frameworks: vec!["Axum".to_string()],
        structure: RepoStructure {
            has_src: true,
            has_tests: true,
            has_docs: false,
            has_ci: true,
            manifest_files: vec!["Cargo.toml".to_string()],
        },
        ci: None,
        readme_summary: String::new(),
        total_files: 100,
        total_lines: 5000,
        dependencies: vec!["axum".to_string(), "tokio".to_string()],
    }
}

#[test]
fn test_leadership_dept_always_present() {
    let profile = test_profile("/tmp/test-leadership");
    let bp = design_org_from_repo(&profile, None, 100.0);
    assert!(
        !bp.departments.is_empty(),
        "must have at least one department"
    );
    assert_eq!(
        bp.departments[0].name, "Leadership",
        "first department must be Leadership"
    );
    let leadership = &bp.departments[0];
    let roles: Vec<&str> = leadership.agents.iter().map(|a| a.role.as_str()).collect();
    assert!(roles.contains(&"CEO"), "Leadership must have CEO");
    assert!(roles.contains(&"PM"), "Leadership must have PM");
    assert!(
        roles.contains(&"Tech Lead"),
        "Leadership must have Tech Lead"
    );
    assert!(
        roles.contains(&"Release Manager"),
        "Leadership must have Release Manager"
    );
    assert!(
        leadership.agents.len() >= 5,
        "Leadership must have at least 5 agents"
    );
}

#[test]
fn test_mission_from_readme() {
    let dir = std::env::temp_dir().join(format!("cvg_mission_readme_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("README.md"),
        "# MyApp\nA great app for great people\n",
    )
    .unwrap();
    let mission = read_repo_mission(dir.to_str().unwrap());
    assert_eq!(mission, Some("A great app for great people".to_string()));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_mission_fallback_package_json() {
    let dir = std::env::temp_dir().join(format!("cvg_mission_pkg_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("package.json"),
        r#"{"name":"cool","description":"Cool tool for cool people"}"#,
    )
    .unwrap();
    let mission = read_repo_mission(dir.to_str().unwrap());
    assert_eq!(mission, Some("Cool tool for cool people".to_string()));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_mission_fallback_cargo_toml() {
    let dir = std::env::temp_dir().join(format!("cvg_mission_cargo_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"mylib\"\ndescription = \"A useful library\"\n",
    )
    .unwrap();
    let mission = read_repo_mission(dir.to_str().unwrap());
    assert_eq!(mission, Some("A useful library".to_string()));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_generate_convergio_dir() {
    let dir = std::env::temp_dir().join(format!("cvg_dotfiles_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    let profile = test_profile(dir.to_str().unwrap());
    let bp = design_org_from_repo(&profile, Some("test-proj"), 100.0);
    let result = generate_convergio_dir(&bp, &profile);
    assert!(
        result.is_ok(),
        "generate_convergio_dir failed: {:?}",
        result
    );

    let cvg_dir = dir.join(".convergio");
    assert!(cvg_dir.join("config.toml").exists(), "config.toml missing");
    assert!(cvg_dir.join("agents.toml").exists(), "agents.toml missing");
    assert!(
        cvg_dir.join("knowledge/stack.md").exists(),
        "stack.md missing"
    );
    assert!(
        cvg_dir.join("knowledge/runbook.md").exists(),
        "runbook.md missing"
    );

    let config = fs::read_to_string(cvg_dir.join("config.toml")).unwrap();
    assert!(
        config.contains("test-proj"),
        "config must contain project name"
    );

    let agents = fs::read_to_string(cvg_dir.join("agents.toml")).unwrap();
    assert!(agents.contains("CEO"), "agents must contain CEO");

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_mission_skips_readme_badges() {
    let dir = std::env::temp_dir().join(format!("cvg_mission_badge_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("README.md"),
        "# BadgeApp\n![Build](https://img.shields.io/badge)\n[![CI](https://ci.com)]\nThe real description\n",
    )
    .unwrap();
    let mission = read_repo_mission(dir.to_str().unwrap());
    assert_eq!(mission, Some("The real description".to_string()));
    let _ = fs::remove_dir_all(&dir);
}
