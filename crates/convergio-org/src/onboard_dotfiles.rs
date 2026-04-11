//! Generate `.convergio/` directory structure in the target repo.
//!
//! Creates config.toml, agents.toml, knowledge/stack.md, knowledge/runbook.md.

use std::fs;
use std::path::Path;

use crate::factory::OrgBlueprint;
use crate::repo_scanner::RepoProfile;

/// Generate the `.convergio/` directory in the target repo.
pub fn generate_convergio_dir(bp: &OrgBlueprint, profile: &RepoProfile) -> Result<(), String> {
    let repo_path = bp
        .repo_path
        .as_deref()
        .ok_or_else(|| "no repo_path in blueprint".to_string())?;
    let base = Path::new(repo_path).join(".convergio");
    let knowledge_dir = base.join("knowledge");

    fs::create_dir_all(&knowledge_dir).map_err(|e| format!("mkdir .convergio: {e}"))?;

    // 1. config.toml
    let config = build_config_toml(bp, profile);
    fs::write(base.join("config.toml"), config).map_err(|e| format!("write config.toml: {e}"))?;

    // 2. agents.toml
    let agents = build_agents_toml(bp);
    fs::write(base.join("agents.toml"), agents).map_err(|e| format!("write agents.toml: {e}"))?;

    // 3. knowledge/stack.md
    let stack = build_stack_md(profile);
    fs::write(knowledge_dir.join("stack.md"), stack).map_err(|e| format!("write stack.md: {e}"))?;

    // 4. knowledge/runbook.md
    let runbook = build_runbook_md(bp, profile);
    fs::write(knowledge_dir.join("runbook.md"), runbook)
        .map_err(|e| format!("write runbook.md: {e}"))?;

    Ok(())
}

fn build_config_toml(bp: &OrgBlueprint, profile: &RepoProfile) -> String {
    let langs: Vec<&str> = profile.languages.iter().map(|(l, _)| l.as_str()).collect();
    let (build, test, dev) = commands_for_langs(&langs, &profile.frameworks);
    let deploy = if profile.structure.has_ci {
        "see .github/workflows/"
    } else {
        "# no deploy configured"
    };
    format!(
        "[project]\nname = \"{name}\"\nmission = \"{mission}\"\n\n\
         [commands]\nbuild = \"{build}\"\ntest = \"{test}\"\n\
         dev = \"{dev}\"\ndeploy = \"{deploy}\"\n",
        name = bp.name,
        mission = bp.mission,
    )
}

fn build_agents_toml(bp: &OrgBlueprint) -> String {
    let mut out = String::new();
    for dept in &bp.departments {
        for agent in &dept.agents {
            out.push_str("[[agents]]\n");
            out.push_str(&format!("name = \"{}\"\n", agent.name));
            out.push_str(&format!("role = \"{}\"\n", agent.role));
            out.push_str(&format!("model = \"{}\"\n", agent.model));
            let caps: Vec<String> = agent
                .capabilities
                .iter()
                .map(|c| format!("\"{c}\""))
                .collect();
            out.push_str(&format!("capabilities = [{}]\n", caps.join(", ")));
            out.push_str(&format!("department = \"{}\"\n\n", dept.name));
        }
    }
    out
}

fn build_stack_md(profile: &RepoProfile) -> String {
    let mut out = String::from("# Tech Stack\n\n## Languages\n");
    for (lang, lines) in &profile.languages {
        out.push_str(&format!("- {lang} ({lines} lines)\n"));
    }
    if !profile.frameworks.is_empty() {
        out.push_str("\n## Frameworks\n");
        for fw in &profile.frameworks {
            out.push_str(&format!("- {fw}\n"));
        }
    }
    if !profile.dependencies.is_empty() {
        out.push_str("\n## Dependencies\n");
        for dep in profile.dependencies.iter().take(10) {
            out.push_str(&format!("- {dep}\n"));
        }
    }
    out
}

fn build_runbook_md(bp: &OrgBlueprint, profile: &RepoProfile) -> String {
    let langs: Vec<&str> = profile.languages.iter().map(|(l, _)| l.as_str()).collect();
    let (build, test, dev) = commands_for_langs(&langs, &profile.frameworks);
    format!(
        "# Runbook — {name}\n\n\
         ## Build\n```bash\n{build}\n```\n\n\
         ## Test\n```bash\n{test}\n```\n\n\
         ## Run Dev\n```bash\n{dev}\n```\n\n\
         ## Deploy\n{deploy}\n\n\
         ## Clean\n```bash\n{clean}\n```\n",
        name = bp.name,
        deploy = if profile.structure.has_ci {
            "See `.github/workflows/`"
        } else {
            "No deploy configured"
        },
        clean = clean_cmd(&langs),
    )
}

fn commands_for_langs(
    langs: &[&str],
    _frameworks: &[String],
) -> (&'static str, &'static str, &'static str) {
    let has_rust = langs.iter().any(|l| l.eq_ignore_ascii_case("rust"));
    let has_ts = langs
        .iter()
        .any(|l| l.eq_ignore_ascii_case("typescript") || l.eq_ignore_ascii_case("javascript"));
    let has_python = langs.iter().any(|l| l.eq_ignore_ascii_case("python"));

    if has_rust {
        ("cargo build", "cargo test", "cargo run")
    } else if has_ts {
        ("npm run build", "npm test", "npm run dev")
    } else if has_python {
        ("python -m build", "pytest", "python main.py")
    } else {
        ("# build command", "# test command", "# run command")
    }
}

fn clean_cmd(langs: &[&str]) -> &'static str {
    let has_rust = langs.iter().any(|l| l.eq_ignore_ascii_case("rust"));
    if has_rust {
        "cargo clean"
    } else {
        "rm -rf dist/ build/ node_modules/"
    }
}
