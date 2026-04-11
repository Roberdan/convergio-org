//! Mission-based org department + night-agent generators.

use super::{agent, dept_with_agents, night, Department, NightAgentSpec, MODEL_OPUS};

const MODEL_DEFAULT: &str = MODEL_OPUS;

pub fn mission_departments(lower: &str, slug: &str) -> Vec<Department> {
    let kw_fit = ["fitness", "health", "training", "workout", "gym"];
    let kw_sw = ["software", "code", "app", "platform", "saas"];
    let kw_mkt = ["marketing", "sales", "growth", "ads", "campaign"];

    if kw_fit.iter().any(|k| lower.contains(k)) {
        vec![
            dept_with_agents(
                "Nutrition",
                vec![agent(
                    slug,
                    "nutritionist",
                    MODEL_DEFAULT,
                    "meal planning",
                    "Nutritionist",
                    &["nutrition", "meal planning", "dietary guidelines"],
                )],
            ),
            dept_with_agents(
                "Training",
                vec![
                    agent(
                        slug,
                        "trainer",
                        MODEL_DEFAULT,
                        "workout programming",
                        "Personal Trainer",
                        &["exercise science", "workout design", "progressive overload"],
                    ),
                    agent(
                        slug,
                        "form-checker",
                        MODEL_DEFAULT,
                        "exercise form review",
                        "Form Analyst",
                        &["biomechanics", "injury prevention"],
                    ),
                ],
            ),
            dept_with_agents(
                "Analytics",
                vec![agent(
                    slug,
                    "analyst",
                    MODEL_DEFAULT,
                    "progress tracking",
                    "Data Analyst",
                    &["statistics", "data visualization", "progress metrics"],
                )],
            ),
        ]
    } else if kw_sw.iter().any(|k| lower.contains(k)) {
        vec![
            dept_with_agents(
                "Development",
                vec![agent(
                    slug,
                    "lead-dev",
                    MODEL_DEFAULT,
                    "architecture",
                    "Tech Lead",
                    &["software architecture", "code review", "API design"],
                )],
            ),
            dept_with_agents(
                "QA",
                vec![agent(
                    slug,
                    "tester",
                    MODEL_DEFAULT,
                    "testing",
                    "QA Engineer",
                    &["test automation", "integration testing", "regression"],
                )],
            ),
            dept_with_agents(
                "DevOps",
                vec![agent(
                    slug,
                    "ci-ops",
                    MODEL_DEFAULT,
                    "CI/CD",
                    "DevOps Engineer",
                    &["CI/CD", "Docker", "deployment automation"],
                )],
            ),
        ]
    } else if kw_mkt.iter().any(|k| lower.contains(k)) {
        vec![
            dept_with_agents(
                "Marketing",
                vec![agent(
                    slug,
                    "strategist",
                    MODEL_DEFAULT,
                    "campaign strategy",
                    "Marketing Strategist",
                    &["campaign planning", "audience targeting", "brand strategy"],
                )],
            ),
            dept_with_agents(
                "Analytics",
                vec![agent(
                    slug,
                    "data-analyst",
                    MODEL_DEFAULT,
                    "metrics",
                    "Data Analyst",
                    &["metrics analysis", "attribution", "A/B testing"],
                )],
            ),
            dept_with_agents(
                "Content",
                vec![agent(
                    slug,
                    "writer",
                    MODEL_DEFAULT,
                    "copywriting",
                    "Content Writer",
                    &["copywriting", "SEO", "content strategy"],
                )],
            ),
        ]
    } else {
        vec![
            dept_with_agents(
                "Strategy",
                vec![agent(
                    slug,
                    "strategist",
                    MODEL_DEFAULT,
                    "planning",
                    "Strategist",
                    &["planning", "OKR setting", "roadmapping"],
                )],
            ),
            dept_with_agents(
                "Execution",
                vec![agent(
                    slug,
                    "executor",
                    MODEL_DEFAULT,
                    "task execution",
                    "Executor",
                    &["task management", "delivery", "coordination"],
                )],
            ),
            dept_with_agents(
                "Analytics",
                vec![agent(
                    slug,
                    "analyst",
                    MODEL_DEFAULT,
                    "data analysis",
                    "Data Analyst",
                    &["data analysis", "reporting", "KPIs"],
                )],
            ),
        ]
    }
}

pub fn mission_night_agents(lower: &str, slug: &str) -> Vec<NightAgentSpec> {
    let mut agents = vec![night(slug, "daily-report", "daily_report", "0 2 * * *")];
    if lower.contains("software") || lower.contains("code") || lower.contains("app") {
        agents.push(night(slug, "pr-monitor", "monitor_prs", "*/30 0-6 * * *"));
        agents.push(night(slug, "dep-updater", "dep_update", "0 5 * * 1"));
    }
    if lower.contains("marketing") || lower.contains("sales") {
        agents.push(night(slug, "metrics-digest", "daily_report", "0 3 * * *"));
    }
    agents
}
