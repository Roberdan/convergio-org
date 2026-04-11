//! Org factory builders — entry point; delegates to mission/knowledge sub-modules.

#[path = "factory_builders_domain.rs"]
mod domain;
#[path = "factory_builders_keywords.rs"]
mod keywords;
#[path = "factory_builders_knowledge.rs"]
mod knowledge;
#[path = "factory_builders_mission.rs"]
mod mission;
#[path = "factory_builders_names.rs"]
mod names;

pub(super) use domain::domain_departments;
pub(super) use knowledge::repo_knowledge_items;
pub(super) use mission::{mission_departments, mission_night_agents};

use super::{AgentSpec, Department, KnowledgeItem, NightAgentSpec, MODEL_OPUS};
use crate::repo_scanner::RepoProfile;

// ---------- shared helpers (used by mission + repo builders) ----------

pub(super) fn agent(
    slug: &str,
    suffix: &str,
    model: &str,
    capability: &str,
    role: &str,
    skills: &[&str],
) -> AgentSpec {
    AgentSpec {
        name: names::human_name(slug, suffix),
        model: model.to_string(),
        capabilities: vec![capability.to_string()],
        role: role.to_string(),
        skills: skills.iter().map(|s| s.to_string()).collect(),
    }
}

pub(super) fn dept_with_agents(name: &str, agents: Vec<AgentSpec>) -> Department {
    Department {
        name: name.to_string(),
        agents,
    }
}

pub(super) fn night(slug: &str, suffix: &str, task: &str, schedule: &str) -> NightAgentSpec {
    NightAgentSpec {
        name: names::human_name(slug, &format!("night-{suffix}")),
        schedule: task.to_string(),
        time: schedule.to_string(),
        model: MODEL_OPUS.to_string(),
    }
}

// ---------- leadership ----------

fn leadership_department(slug: &str) -> Department {
    dept_with_agents(
        "Leadership",
        vec![
            agent(
                slug,
                "ceo",
                MODEL_OPUS,
                "architecture decisions, code review",
                "CEO",
                &["architecture", "code review", "strategy"],
            ),
            agent(
                slug,
                "pm",
                MODEL_OPUS,
                "planning, tracking, priorities",
                "PM",
                &["planning", "tracking", "priorities"],
            ),
            agent(
                slug,
                "tech-lead",
                MODEL_OPUS,
                "code review, standards, mentoring",
                "Tech Lead",
                &["code review", "standards", "mentoring"],
            ),
            agent(
                slug,
                "release-mgr",
                MODEL_OPUS,
                "CI/CD, deploy, versioning",
                "Release Manager",
                &["CI/CD", "deploy", "versioning"],
            ),
            agent(
                slug,
                "dev-1",
                MODEL_OPUS,
                "implementation",
                "Developer",
                &["software development"],
            ),
            agent(
                slug,
                "dev-2",
                MODEL_OPUS,
                "implementation",
                "Developer",
                &["software development"],
            ),
        ],
    )
}

// ---------- repo-based ----------

pub(super) fn repo_departments(profile: &RepoProfile, slug: &str) -> Vec<Department> {
    // Leadership dept is ALWAYS first
    let mut depts = vec![leadership_department(slug)];
    let langs: Vec<String> = profile
        .languages
        .iter()
        .map(|(l, _)| l.to_lowercase())
        .collect();

    if langs.iter().any(|l| l == "rust") {
        let fw: Vec<&str> = profile
            .frameworks
            .iter()
            .filter(|f| matches!(f.as_str(), "Axum" | "Actix"))
            .map(|f| f.as_str())
            .collect();
        let mut skills = vec!["Rust", "cargo", "clippy"];
        skills.extend_from_slice(&fw);
        depts.push(dept_with_agents(
            "Backend",
            vec![
                agent(
                    slug,
                    "rust-dev",
                    MODEL_OPUS,
                    "Rust development",
                    "Rust Developer",
                    &skills,
                ),
                agent(
                    slug,
                    "clippy-reviewer",
                    MODEL_OPUS,
                    "Rust lint review",
                    "Code Reviewer",
                    &["clippy", "rustfmt", "code quality"],
                ),
            ],
        ));
    }

    if langs.iter().any(|l| l == "typescript" || l == "javascript") {
        let has_next = profile.frameworks.iter().any(|f| f == "Next.js");
        let has_react = profile.frameworks.iter().any(|f| f == "React");
        let has_vue = profile.frameworks.iter().any(|f| f == "Vue");
        let mut skills = vec!["TypeScript", "JavaScript"];
        if has_next {
            skills.push("Next.js");
        }
        if has_react {
            skills.push("React");
        }
        if has_vue {
            skills.push("Vue");
        }
        depts.push(dept_with_agents(
            "Frontend",
            vec![
                agent(
                    slug,
                    "component-dev",
                    MODEL_OPUS,
                    "UI components",
                    "Frontend Developer",
                    &skills,
                ),
                agent(
                    slug,
                    "design-reviewer",
                    MODEL_OPUS,
                    "design review",
                    "Design Reviewer",
                    &["UI/UX", "accessibility", "CSS"],
                ),
            ],
        ));
    }

    if langs.iter().any(|l| l == "python") {
        let mut skills = vec!["Python"];
        for fw in &profile.frameworks {
            if matches!(fw.as_str(), "Django" | "Flask" | "FastAPI") {
                skills.push(fw.as_str());
            }
        }
        depts.push(dept_with_agents(
            "Backend",
            vec![
                agent(
                    slug,
                    "python-dev",
                    MODEL_OPUS,
                    "Python development",
                    "Python Developer",
                    &skills,
                ),
                agent(
                    slug,
                    "test-runner",
                    MODEL_OPUS,
                    "test execution",
                    "QA Engineer",
                    &["pytest", "test automation"],
                ),
            ],
        ));
    }

    if profile.structure.has_ci {
        depts.push(dept_with_agents(
            "DevOps",
            vec![agent(
                slug,
                "ci-monitor",
                MODEL_OPUS,
                "CI monitoring",
                "DevOps Engineer",
                &["CI/CD", "GitHub Actions", "deployment"],
            )],
        ));
    }

    if profile.structure.has_tests {
        depts.push(dept_with_agents(
            "QA",
            vec![agent(
                slug,
                "test-runner",
                MODEL_OPUS,
                "test management",
                "QA Engineer",
                &["testing", "test coverage", "regression"],
            )],
        ));
    }

    if depts.len() <= 1 {
        depts.push(dept_with_agents(
            "General",
            vec![agent(
                slug,
                "dev",
                MODEL_OPUS,
                "general development",
                "Developer",
                &["software development"],
            )],
        ));
    }

    depts
}

pub(super) fn repo_night_agents(profile: &RepoProfile, slug: &str) -> Vec<NightAgentSpec> {
    let mut agents = vec![
        night(slug, "daily-report", "daily_report", "0 2 * * *"),
        night(
            slug,
            "stale-branch-cleanup",
            "stale_branch_cleanup",
            "0 3 * * *",
        ),
    ];
    if profile.structure.has_ci {
        agents.push(night(slug, "pr-monitor", "monitor_prs", "*/30 0-6 * * *"));
        agents.push(night(slug, "coverage-check", "test_coverage", "0 4 * * *"));
    }
    if !profile.dependencies.is_empty() {
        agents.push(night(slug, "dep-updater", "dependency_update", "0 5 * * 1"));
    }
    agents
}
