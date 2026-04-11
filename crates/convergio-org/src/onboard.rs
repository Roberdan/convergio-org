//! POST /api/org/projects/onboard — scan a repo and create an org from it.
//!
//! 1. Calls `project_scanner::scan_project()` on the given path.
//! 2. Uses `factory::design_org_from_repo()` to build an `OrgBlueprint`.
//! 3. Persists the org + seeds knowledge items into the DB.
//! 4. Returns the created org details.

use std::path::Path;
use std::sync::Arc;

use axum::extract::State;
use axum::response::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::factory::{design_org_from_repo, OrgBlueprint};
use crate::onboard_catalog::reuse_catalog_agents;
use crate::project_scanner::scan_project;
use crate::repo_scanner::scan_repo;
use crate::routes::OrgState;

#[derive(Deserialize)]
pub struct OnboardBody {
    pub repo_path: String,
}

/// POST /api/org/projects/onboard
pub async fn onboard_project(
    State(s): State<Arc<OrgState>>,
    Json(body): Json<OnboardBody>,
) -> Json<Value> {
    let path = Path::new(&body.repo_path);

    // Reject path traversal attempts
    if let Err(e) = convergio_types::platform_paths::validate_path_components(path) {
        return Json(json!({"ok": false, "error": format!("invalid path: {e}")}));
    }
    if !path.is_absolute() {
        return Json(json!({"ok": false, "error": "repo_path must be absolute"}));
    }
    if !path.exists() {
        return Json(json!({"ok": false, "error": "repo_path does not exist"}));
    }

    // 1. Scan the repository
    let profile = match scan_repo(path) {
        Ok(p) => p,
        Err(e) => return Json(json!({"ok": false, "error": format!("scan failed: {e}")})),
    };

    // 2. Design org from scan results
    let mut blueprint = design_org_from_repo(&profile, None, 100.0);
    let conn = match s.pool.get() {
        Ok(conn) => conn,
        Err(e) => return Json(json!({"ok": false, "error": e.to_string()})),
    };
    if let Err(e) = reuse_catalog_agents(&conn, &mut blueprint) {
        return Json(json!({"ok": false, "error": format!("catalog reuse failed: {e}")}));
    }

    // 3. Persist org to DB
    if let Err(e) = persist_org(&s, &blueprint) {
        return Json(json!({"ok": false, "error": e}));
    }

    // 4. Seed knowledge base
    if let Err(e) = seed_knowledge(&s, &blueprint) {
        return Json(json!({"ok": false, "error": e}));
    }

    // 5. Generate .convergio/ directory
    let convergio_dir =
        crate::onboard_dotfiles::generate_convergio_dir(&blueprint, &profile).is_ok();

    // 6. Log observability events (best-effort)
    log_onboard_events(&s, &blueprint);

    // 7. Build enriched response
    let members: Vec<Value> = blueprint
        .departments
        .iter()
        .flat_map(|d| {
            d.agents.iter().map(move |a| {
                json!({
                    "name": a.name,
                    "role": a.role,
                    "department": d.name,
                    "model": a.model,
                })
            })
        })
        .collect();
    let night_agents: Vec<Value> = blueprint
        .night_agents
        .iter()
        .map(|na| {
            json!({
                "name": na.name,
                "schedule": na.schedule,
                "time": na.time,
                "model": na.model,
            })
        })
        .collect();

    // 8. Return scan + blueprint info
    let scan = match scan_project(path) {
        Ok(s) => serde_json::to_value(s).unwrap_or(json!(null)),
        Err(_) => json!(null),
    };

    Json(json!({
        "ok": true,
        "org_id": blueprint.slug,
        "name": blueprint.name,
        "mission": blueprint.mission,
        "ceo_agent": blueprint.ceo_agent,
        "departments": blueprint.departments.len(),
        "members": members,
        "night_agents": night_agents,
        "knowledge_items": blueprint.knowledge_items.len(),
        "convergio_dir": convergio_dir,
        "scan": scan,
    }))
}

fn persist_org(s: &OrgState, bp: &OrgBlueprint) -> Result<(), String> {
    let conn = s.pool.get().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR REPLACE INTO ipc_orgs (id, mission, objectives, ceo_agent, budget) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![
            bp.slug,
            bp.mission,
            bp.repo_path.as_deref().unwrap_or(""),
            bp.ceo_agent,
            bp.budget_usd.unwrap_or(0.0),
        ],
    )
    .map_err(|e| e.to_string())?;

    // Wire org members from all departments
    for dept in &bp.departments {
        for agent in &dept.agents {
            let _ = conn.execute(
                "INSERT OR REPLACE INTO ipc_org_members (id, org_id, agent, role, department) \
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![agent.name, bp.slug, agent.name, agent.role, dept.name],
            );
            // Wire agent into agent_catalog
            let tier = if agent.model.contains("opus") {
                "t1"
            } else if agent.model.contains("sonnet") {
                "t2"
            } else {
                "t3"
            };
            let caps_json =
                serde_json::to_string(&agent.capabilities).unwrap_or_else(|_| "[]".to_string());
            let _ = conn.execute(
                "INSERT OR REPLACE INTO agent_catalog \
                 (id, name, role, org_id, category, model_tier, capabilities_json) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![
                    agent.name,
                    agent.name,
                    agent.role,
                    bp.slug,
                    dept.name.to_lowercase(),
                    tier,
                    caps_json,
                ],
            );
        }
    }

    // Wire night agents
    for na in &bp.night_agents {
        let _ = conn.execute(
            "INSERT OR REPLACE INTO night_agent_defs \
             (name, org_id, description, schedule, agent_prompt, model) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                na.name,
                bp.slug,
                na.schedule,
                na.time,
                format!("Execute night task: {}", na.schedule),
                na.model,
            ],
        );
    }

    // Create IPC channel for the org
    let _ = conn.execute(
        "INSERT OR IGNORE INTO ipc_channels (name, description, created_by) \
         VALUES (?1, ?2, ?3)",
        rusqlite::params![
            format!("#org-{}", bp.slug),
            format!("Org channel for {}", bp.name),
            "onboard",
        ],
    );

    // Initialize billing budget (best-effort)
    if let Some(budget) = bp.budget_usd {
        let _ = conn.execute(
            "INSERT OR IGNORE INTO billing_budgets \
             (org_id, daily_limit_usd, monthly_limit_usd, auto_pause) \
             VALUES (?1, ?2, ?3, 0)",
            rusqlite::params![bp.slug, budget / 30.0, budget],
        );
    }

    Ok(())
}

fn seed_knowledge(s: &OrgState, bp: &OrgBlueprint) -> Result<(), String> {
    let conn = s.pool.get().map_err(|e| e.to_string())?;
    let domain = bp.slug.as_str();
    for item in &bp.knowledge_items {
        let _ = conn.execute(
            "INSERT OR REPLACE INTO knowledge_base (domain, title, content) \
             VALUES (?1, ?2, ?3)",
            rusqlite::params![domain, item.title, item.content],
        );
    }
    Ok(())
}

fn log_onboard_events(s: &OrgState, bp: &OrgBlueprint) {
    let Ok(conn) = s.pool.get() else { return };
    let member_count: usize = bp.departments.iter().map(|d| d.agents.len()).sum();
    let night_count = bp.night_agents.len();
    let summary = format!(
        "Project {} onboarded with {} members",
        bp.name, member_count
    );
    let details = serde_json::json!({
        "org_id": bp.slug,
        "departments": bp.departments.len(),
        "members": member_count,
        "night_agents": night_count,
    });

    // obs_timeline event
    let _ = conn.execute(
        "INSERT INTO obs_timeline (source, event_type, actor, org_id, summary, details_json) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            "onboard",
            "org_created",
            "system",
            bp.slug,
            summary,
            details.to_string(),
        ],
    );

    // decision_log entry
    let _ = conn.execute(
        "INSERT INTO decision_log (decision, reasoning, agent, plan_id, task_id) \
         VALUES (?1, ?2, ?3, NULL, NULL)",
        rusqlite::params![
            format!("Onboarded project {}", bp.name),
            format!(
                "Auto-onboard from repo scan at {}",
                bp.repo_path.as_deref().unwrap_or("unknown")
            ),
            "onboard-system",
        ],
    );

    // notification_queue entry
    let _ = conn.execute(
        "INSERT INTO notification_queue (severity, title, message, status) \
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![
            "info",
            format!("Project onboarded: {}", bp.name),
            format!("{member_count} members, {night_count} night agents created"),
            "pending",
        ],
    );
}
