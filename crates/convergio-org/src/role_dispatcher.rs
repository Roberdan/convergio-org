//! Role dispatcher — assigns the best-fit agent for a task based on
//! required capabilities and current workload.
//!
//! POST /api/org/:id/dispatch  {task_description, required_capabilities}
//! Returns: {assigned_agent, reason}

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::response::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::routes::OrgState;

#[derive(Debug, Deserialize)]
pub struct DispatchRequest {
    pub task_description: String,
    pub required_capabilities: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct DispatchResult {
    pub assigned_agent: String,
    pub reason: String,
}

/// Candidate agent with its current in-progress task count.
struct Candidate {
    name: String,
    matched_caps: usize,
    in_progress: i64,
}

/// POST /api/org/:id/dispatch
pub async fn dispatch_task(
    State(s): State<Arc<OrgState>>,
    Path(org_id): Path<String>,
    Json(body): Json<DispatchRequest>,
) -> Json<Value> {
    let conn = match s.pool.get() {
        Ok(c) => c,
        Err(e) => return Json(json!({"error": e.to_string()})),
    };

    // 1. Verify org exists.
    let org_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM ipc_orgs WHERE id = ?1",
            [&org_id],
            |r| r.get::<_, i64>(0),
        )
        .map(|c| c > 0)
        .unwrap_or(false);

    if !org_exists {
        return Json(json!({"error": "org not found"}));
    }

    if body.required_capabilities.is_empty() {
        return Json(json!({"error": "required_capabilities must not be empty"}));
    }

    // 2. Find active agents in this org that have at least one matching capability.
    let agents = match find_matching_agents(&conn, &org_id, &body.required_capabilities) {
        Ok(a) => a,
        Err(e) => return Json(json!({"error": format!("query failed: {e}")})),
    };

    if agents.is_empty() {
        return Json(json!({
            "error": "no agents with matching capabilities found in this org"
        }));
    }

    // 3. Pick the agent with best match (most matched caps), then lowest load.
    let best = agents
        .iter()
        .max_by(|a, b| {
            a.matched_caps
                .cmp(&b.matched_caps)
                .then_with(|| b.in_progress.cmp(&a.in_progress))
        })
        .expect("agents is non-empty");

    let reason = format!(
        "matched {}/{} capabilities, {} in-progress tasks (lowest load among candidates)",
        best.matched_caps,
        body.required_capabilities.len(),
        best.in_progress,
    );

    Json(json!(DispatchResult {
        assigned_agent: best.name.clone(),
        reason,
    }))
}

/// Query agent_catalog for agents in the org with matching capabilities,
/// then count their in-progress tasks.
fn find_matching_agents(
    conn: &rusqlite::Connection,
    org_id: &str,
    required_caps: &[String],
) -> rusqlite::Result<Vec<Candidate>> {
    let mut stmt = conn.prepare(
        "SELECT name, capabilities_json FROM agent_catalog \
         WHERE org_id = ?1 AND status = 'active'",
    )?;

    let rows: Vec<(String, String)> = stmt
        .query_map([org_id], |r| Ok((r.get(0)?, r.get(1)?)))?
        .filter_map(|r| r.ok())
        .collect();

    let mut candidates = Vec::new();
    for (name, caps_json) in &rows {
        let caps: Vec<String> = serde_json::from_str(caps_json).unwrap_or_default();
        let matched = required_caps
            .iter()
            .filter(|required| caps.iter().any(|cap| capability_matches(cap, required)))
            .count();
        if matched == 0 {
            continue;
        }

        let in_progress: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM tasks \
                 WHERE executor_agent = ?1 AND status = 'in_progress'",
                [&name],
                |r| r.get(0),
            )
            .unwrap_or(0);

        candidates.push(Candidate {
            name: name.clone(),
            matched_caps: matched,
            in_progress,
        });
    }

    Ok(candidates)
}

fn capability_matches(agent_cap: &str, required: &str) -> bool {
    let agent_cap = agent_cap.to_lowercase();
    let required = required.to_lowercase();
    agent_cap == required
        || agent_cap.contains(&required)
        || agent_cap
            .split(',')
            .map(str::trim)
            .any(|part| part == required || part.contains(&required))
}

#[cfg(test)]
#[path = "role_dispatcher_tests.rs"]
mod tests;
