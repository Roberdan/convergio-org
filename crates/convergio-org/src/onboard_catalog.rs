use rusqlite::Connection;

use crate::factory::OrgBlueprint;

const OPUS_MODEL: &str = "claude-opus-4-6";

struct CatalogAgent {
    name: String,
    role: String,
    capabilities: Vec<String>,
}

pub(crate) fn reuse_catalog_agents(
    conn: &Connection,
    blueprint: &mut OrgBlueprint,
) -> Result<(), String> {
    let catalog = load_global_catalog(conn).map_err(|e| e.to_string())?;
    if catalog.is_empty() {
        force_opus(blueprint);
        return Ok(());
    }

    for dept in &mut blueprint.departments {
        for agent in &mut dept.agents {
            agent.model = OPUS_MODEL.to_string();
            if let Some(best) = find_best_match(&catalog, &agent.role, &agent.capabilities) {
                agent.name = best.name.clone();
            }
        }
    }

    for night in &mut blueprint.night_agents {
        night.model = OPUS_MODEL.to_string();
    }

    if let Some(ceo) = blueprint
        .departments
        .iter()
        .flat_map(|d| &d.agents)
        .find(|a| a.role == "CEO")
    {
        blueprint.ceo_agent = ceo.name.clone();
    }

    Ok(())
}

fn force_opus(blueprint: &mut OrgBlueprint) {
    for dept in &mut blueprint.departments {
        for agent in &mut dept.agents {
            agent.model = OPUS_MODEL.to_string();
        }
    }
    for night in &mut blueprint.night_agents {
        night.model = OPUS_MODEL.to_string();
    }
}

fn load_global_catalog(conn: &Connection) -> rusqlite::Result<Vec<CatalogAgent>> {
    conn.prepare(
        "SELECT name, role, capabilities_json FROM agent_catalog \
         WHERE org_id = 'convergio' AND status = 'active' \
         AND name NOT LIKE '_doctor_test_%' ORDER BY name",
    )?
    .query_map([], |r| {
        Ok(CatalogAgent {
            name: r.get(0)?,
            role: r.get(1)?,
            capabilities: serde_json::from_str(&r.get::<_, String>(2)?).unwrap_or_default(),
        })
    })?
    .collect()
}

fn find_best_match<'a>(
    catalog: &'a [CatalogAgent],
    role: &str,
    capabilities: &[String],
) -> Option<&'a CatalogAgent> {
    let role_tokens = role_keywords(role);
    catalog
        .iter()
        .filter_map(|agent| {
            let role_score = role_tokens
                .iter()
                .filter(|token| agent.role.to_lowercase().contains(**token))
                .count();
            let cap_score = capabilities
                .iter()
                .filter(|cap| {
                    let needle = cap.to_lowercase();
                    agent
                        .capabilities
                        .iter()
                        .any(|c| c.to_lowercase().contains(&needle))
                })
                .count();
            let total = (role_score * 10) + cap_score;
            (total > 0).then_some((total, agent))
        })
        .max_by_key(|(score, _)| *score)
        .map(|(_, agent)| agent)
}

fn role_keywords(role: &str) -> Vec<&'static str> {
    match role {
        "CEO" => vec!["ceo", "chief of staff"],
        "PM" => vec!["pm", "project manager", "product owner"],
        "Tech Lead" => vec!["tech lead", "architect"],
        "Release Manager" => vec!["release manager"],
        "Developer" | "Rust Developer" | "Python Developer" | "Frontend Developer" => {
            vec!["developer", "executor", "frontend", "rust", "python"]
        }
        "Code Reviewer" => vec!["code reviewer", "review"],
        "QA Engineer" => vec!["qa", "testing", "validator"],
        "DevOps Engineer" => vec!["devops", "deployment", "infra"],
        other => vec![Box::leak(other.to_lowercase().into_boxed_str())],
    }
}
