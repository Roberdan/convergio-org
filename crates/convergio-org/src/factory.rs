//! Org factory — designs an org structure from a mission statement or repo profile.

#[path = "factory_builders.rs"]
mod factory_builders;
#[path = "factory_types.rs"]
mod factory_types;

pub use factory_types::{AgentSpec, Department, KnowledgeItem, NightAgentSpec, OrgBlueprint};

use std::path::Path;

use super::repo_scanner::RepoProfile;

pub(crate) const MODEL_OPUS: &str = "claude-opus-4-6";

/// Convert a name to a URL-safe slug (lowercase, alphanumeric + hyphens).
pub fn slugify(name: &str) -> String {
    let mut slug = String::with_capacity(name.len());
    let mut prev_dash = true;
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash {
            slug.push('-');
            prev_dash = true;
        }
    }
    if slug.ends_with('-') {
        slug.pop();
    }
    slug
}

/// Design an org from a mission statement (rule-based, no LLM call).
pub fn design_org_from_mission(name: &str, mission: &str, budget: f64) -> OrgBlueprint {
    let slug = slugify(name);
    let lower = mission.to_lowercase();
    let departments = factory_builders::mission_departments(&lower, &slug);
    let night_agents = factory_builders::mission_night_agents(&lower, &slug);
    OrgBlueprint {
        name: name.to_string(),
        slug: slug.clone(),
        mission: mission.to_string(),
        ceo_agent: format!("{slug}-ceo"),
        departments,
        night_agents,
        repo_path: None,
        budget_usd: Some(budget),
        knowledge_items: Vec::new(),
    }
}

/// Design an org by mapping a scanned repo profile to departments, roles, skills, and knowledge.
pub fn design_org_from_repo(
    profile: &RepoProfile,
    name: Option<&str>,
    budget: f64,
) -> OrgBlueprint {
    let repo_name = name.unwrap_or_else(|| {
        profile
            .path
            .rsplit('/')
            .find(|s| !s.is_empty())
            .unwrap_or("org")
    });
    let slug = slugify(repo_name);
    let mission = read_repo_mission(&profile.path)
        .unwrap_or_else(|| format!("Maintain and evolve {repo_name}"));
    let readme = read_full_readme(&profile.path);
    let mut departments = factory_builders::repo_departments(profile, &slug);
    departments.extend(factory_builders::domain_departments(
        &readme, &mission, &slug,
    ));
    let night_agents = factory_builders::repo_night_agents(profile, &slug);
    let knowledge_items = factory_builders::repo_knowledge_items(profile);
    let ceo = departments
        .first()
        .and_then(|d| d.agents.first())
        .map(|a| a.name.clone())
        .unwrap_or_else(|| format!("{slug}-ceo"));
    OrgBlueprint {
        name: repo_name.to_string(),
        slug: slug.clone(),
        mission,
        ceo_agent: ceo,
        departments,
        night_agents,
        repo_path: Some(profile.path.clone()),
        budget_usd: Some(budget),
        knowledge_items,
    }
}

/// Try to extract a project mission/description from repo files.
/// Priority: README.md first line after H1 > package.json description > Cargo.toml description.
pub fn read_repo_mission(repo_path: &str) -> Option<String> {
    let base = Path::new(repo_path);

    // 1. Try README.md
    if let Ok(content) = std::fs::read_to_string(base.join("README.md")) {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if trimmed.starts_with("# ") || trimmed.starts_with("<h1") {
                continue;
            }
            // Strip HTML first, then check if the remaining text is useful
            let clean = strip_html(trimmed);
            let clean = clean.trim();
            if clean.is_empty() || clean == "---" {
                continue;
            }
            if is_skippable_readme_line(clean) {
                continue;
            }
            return Some(truncate_to(clean, 200));
        }
    }

    // 2. Try package.json
    if let Ok(content) = std::fs::read_to_string(base.join("package.json")) {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(desc) = val["description"].as_str() {
                if !desc.is_empty() {
                    return Some(truncate_to(desc, 200));
                }
            }
        }
    }

    // 3. Try Cargo.toml (simple parse, no toml crate)
    if let Ok(content) = std::fs::read_to_string(base.join("Cargo.toml")) {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("description") {
                if let Some(val) = trimmed.split('=').nth(1) {
                    let val = val.trim().trim_matches('"').trim();
                    if !val.is_empty() {
                        return Some(truncate_to(val, 200));
                    }
                }
            }
        }
    }

    None
}

fn truncate_to(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max - 1])
    }
}

/// Lines to skip when parsing README for mission.
fn is_skippable_readme_line(line: &str) -> bool {
    if line.starts_with("![")
        || line.starts_with("[!")
        || line.starts_with("[![")
        || line.starts_with("http")
        || line.starts_with("git clone")
    {
        return true;
    }
    // HTML tags
    if line.starts_with('<') {
        return true;
    }
    // Markdown links that look like instructions, not descriptions
    if line.starts_with("Open [http") || line.starts_with("You can start") {
        return true;
    }
    // Shell commands
    let cmd_prefixes = [
        "npm ",
        "yarn ",
        "pnpm ",
        "bun ",
        "cargo ",
        "pip ",
        "python ",
        "docker ",
        "git ",
        "cd ",
        "mkdir ",
        "curl ",
        "cvg ",
        "```",
        "This is a [",
    ];
    if cmd_prefixes.iter().any(|p| line.starts_with(p)) {
        return true;
    }
    if line.starts_with("## ") || line.starts_with("### ") {
        return true;
    }
    false
}

/// Strip basic HTML tags from a mission string.
fn strip_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for ch in s.chars() {
        if ch == '<' {
            in_tag = true;
        } else if ch == '>' {
            in_tag = false;
        } else if !in_tag {
            out.push(ch);
        }
    }
    out.trim().to_string()
}

/// Read entire README.md for domain analysis.
fn read_full_readme(repo_path: &str) -> String {
    let p = Path::new(repo_path).join("README.md");
    std::fs::read_to_string(p).unwrap_or_default()
}
