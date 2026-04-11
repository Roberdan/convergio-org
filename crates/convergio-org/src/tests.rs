//! Tests for convergio-org.

mod ext_tests {
    use convergio_types::extension::Extension;
    use convergio_types::manifest::ModuleKind;

    use crate::ext::OrgExtension;

    #[test]
    fn manifest_is_extension_kind() {
        let pool = convergio_db::pool::create_memory_pool().unwrap();
        let ext = OrgExtension::new(pool);
        let m = ext.manifest();
        assert_eq!(m.id, "convergio-org");
        assert!(matches!(m.kind, ModuleKind::Extension));
        assert!(!m.provides.is_empty());
    }

    #[test]
    fn has_one_migration() {
        let pool = convergio_db::pool::create_memory_pool().unwrap();
        let ext = OrgExtension::new(pool);
        let migs = ext.migrations();
        assert!(
            !migs.is_empty(),
            "org extension must have at least one migration"
        );
    }

    #[test]
    fn migrations_sql_is_valid() {
        let pool = convergio_db::pool::create_memory_pool().unwrap();
        let conn = pool.get().unwrap();
        let ext = OrgExtension::new(pool.clone());
        for mig in ext.migrations() {
            conn.execute_batch(mig.up).unwrap_or_else(|e| {
                panic!("migration {} failed: {e}", mig.description);
            });
        }
    }
}

mod factory_tests {
    use crate::factory::*;

    #[test]
    fn slugify_basic() {
        assert_eq!(slugify("My Cool Org"), "my-cool-org");
        assert_eq!(slugify("test"), "test");
        assert_eq!(slugify("  spaces  "), "spaces");
    }

    #[test]
    fn design_from_mission_fitness() {
        let bp = design_org_from_mission("FitCorp", "fitness training platform", 500.0);
        assert_eq!(bp.slug, "fitcorp");
        assert_eq!(bp.ceo_agent, "fitcorp-ceo");
        assert!(bp.departments.iter().any(|d| d.name == "Training"));
        assert!(bp.departments.iter().any(|d| d.name == "Nutrition"));
        assert_eq!(bp.budget_usd, Some(500.0));
    }

    #[test]
    fn design_from_mission_software() {
        let bp = design_org_from_mission("DevCorp", "software development platform", 1000.0);
        assert!(bp.departments.iter().any(|d| d.name == "Development"));
        assert!(bp.departments.iter().any(|d| d.name == "QA"));
        assert!(bp
            .night_agents
            .iter()
            .any(|na| na.schedule.contains("monitor_prs")));
    }

    #[test]
    fn design_from_mission_marketing() {
        let bp = design_org_from_mission("AdCorp", "marketing growth", 200.0);
        assert!(bp.departments.iter().any(|d| d.name == "Marketing"));
        assert!(bp
            .night_agents
            .iter()
            .any(|na| na.schedule.contains("daily_report")));
    }

    #[test]
    fn design_from_mission_generic() {
        let bp = design_org_from_mission("GenCorp", "do things", 100.0);
        assert!(bp.departments.iter().any(|d| d.name == "Strategy"));
        assert!(bp.departments.iter().any(|d| d.name == "Execution"));
    }

    #[test]
    fn design_from_repo() {
        use crate::repo_scanner::{RepoProfile, RepoStructure};
        let profile = RepoProfile {
            path: "/tmp/myapp".to_string(),
            languages: vec![("Rust".to_string(), 50)],
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
            dependencies: vec!["axum".to_string()],
        };
        let bp = design_org_from_repo(&profile, None, 500.0);
        assert_eq!(bp.slug, "myapp");
        assert!(bp.departments.iter().any(|d| d.name == "Backend"));
    }

    #[test]
    fn agents_have_roles_and_skills() {
        use crate::repo_scanner::{RepoProfile, RepoStructure};
        let profile = RepoProfile {
            path: "/tmp/rustapp".to_string(),
            languages: vec![("Rust".to_string(), 80), ("TypeScript".to_string(), 20)],
            frameworks: vec!["Axum".to_string(), "Next.js".to_string()],
            structure: RepoStructure {
                has_src: true,
                has_tests: true,
                has_docs: false,
                has_ci: true,
                manifest_files: vec!["Cargo.toml".to_string(), "package.json".to_string()],
            },
            ci: None,
            readme_summary: String::new(),
            total_files: 200,
            total_lines: 10000,
            dependencies: vec!["axum".to_string(), "serde".to_string()],
        };
        let bp = design_org_from_repo(&profile, None, 1000.0);
        for dept in &bp.departments {
            for agent in &dept.agents {
                assert!(!agent.role.is_empty(), "agent {} has no role", agent.name);
                assert!(
                    !agent.skills.is_empty(),
                    "agent {} has no skills",
                    agent.name
                );
            }
        }
        let backend = bp.departments.iter().find(|d| d.name == "Backend").unwrap();
        let rust_dev = backend
            .agents
            .iter()
            .find(|a| a.role == "Rust Developer")
            .unwrap();
        assert!(rust_dev.skills.contains(&"Rust".to_string()));
        assert!(rust_dev.skills.contains(&"Axum".to_string()));
    }

    #[test]
    fn org_template_has_knowledge_items() {
        use crate::repo_scanner::{CiInfo, RepoProfile, RepoStructure};
        let profile = RepoProfile {
            path: "/tmp/myproject".to_string(),
            languages: vec![("Rust".to_string(), 60)],
            frameworks: vec!["Axum".to_string()],
            structure: RepoStructure {
                has_src: true,
                has_tests: true,
                has_docs: false,
                has_ci: true,
                manifest_files: vec!["Cargo.toml".to_string()],
            },
            ci: Some(CiInfo {
                provider: "github-actions".to_string(),
                workflows: vec!["ci.yml".to_string()],
            }),
            readme_summary: "A test project".to_string(),
            total_files: 50,
            total_lines: 2000,
            dependencies: vec!["axum".to_string(), "tokio".to_string()],
        };
        let bp = design_org_from_repo(&profile, None, 500.0);
        assert!(
            !bp.knowledge_items.is_empty(),
            "org template must have knowledge items"
        );
        assert!(
            bp.knowledge_items
                .iter()
                .any(|k| k.category == "architecture"),
            "must have architecture knowledge"
        );
        assert!(
            bp.knowledge_items.iter().any(|k| k.category == "run_guide"),
            "must have run guide"
        );
        assert!(
            bp.knowledge_items.iter().any(|k| k.category == "infra"),
            "must have infra knowledge"
        );
        assert!(
            bp.knowledge_items
                .iter()
                .any(|k| k.category == "requirements"),
            "must have requirements (README)"
        );
        let tech = bp
            .knowledge_items
            .iter()
            .find(|k| k.title == "Tech Stack")
            .unwrap();
        assert!(tech.content.contains("Rust"));
        let run = bp
            .knowledge_items
            .iter()
            .find(|k| k.title == "How to Run")
            .unwrap();
        assert!(run.content.contains("cargo"));
    }

    #[test]
    fn mission_org_has_empty_knowledge_items() {
        let bp = design_org_from_mission("TestCorp", "software platform", 300.0);
        // mission-based orgs don't have repo knowledge — that's fine
        assert_eq!(bp.knowledge_items.len(), 0);
    }
}

mod orgchart_tests {
    use crate::factory::design_org_from_mission;
    use crate::orgchart::*;

    #[test]
    fn render_orgchart_contains_org_name() {
        let bp = design_org_from_mission("TestOrg", "software development", 500.0);
        let chart = render_orgchart(&bp);
        assert!(chart.contains("TestOrg"));
        assert!(chart.contains("CEO:"));
    }

    #[test]
    fn render_compact_contains_departments() {
        let bp = design_org_from_mission("TestOrg", "software development", 500.0);
        let chart = render_orgchart_compact(&bp);
        assert!(chart.contains("Development"));
        assert!(chart.contains("CEO: testorg-ceo"));
    }
}

mod repo_scanner_tests {
    use std::fs;
    use std::path::PathBuf;

    use crate::repo_scanner::scan_repo;

    fn create_test_repo() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("cvg_test_repo_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(dir.join("src/main.rs"), "fn main() {}").unwrap();
        fs::write(dir.join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();
        dir
    }

    #[test]
    fn scan_detects_rust() {
        let dir = create_test_repo();
        let profile = scan_repo(&dir).unwrap();
        assert!(profile.languages.iter().any(|(l, _)| l == "Rust"));
        assert!(profile.structure.has_src);
        assert!(profile
            .structure
            .manifest_files
            .contains(&"Cargo.toml".to_string()));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn scan_rejects_non_directory() {
        let result = scan_repo(std::path::Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }
}
