//! Org telemetry, digest and plans routes.
//!
//! These handlers are called by `cvg org list`, `cvg org show`,
//! and `cvg org plans` to enrich org data with cost/activity info.

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::response::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::routes::OrgState;

#[derive(Deserialize)]
pub struct TelemetryQuery {
    #[serde(default = "default_period")]
    pub period: String,
}

fn default_period() -> String {
    "day".to_string()
}

/// GET /api/orgs/:id/telemetry?period=day|month
pub async fn get_org_telemetry(
    State(s): State<Arc<OrgState>>,
    Path(id): Path<String>,
    Query(q): Query<TelemetryQuery>,
) -> Json<Value> {
    let conn = match s.pool.get() {
        Ok(c) => c,
        Err(e) => return Json(json!({"error": e.to_string()})),
    };
    let exists: bool = conn
        .query_row("SELECT COUNT(*) FROM ipc_orgs WHERE id = ?1", [&id], |r| {
            r.get::<_, i64>(0)
        })
        .map(|c| c > 0)
        .unwrap_or(false);
    if !exists {
        return Json(json!({"error": "org not found"}));
    }
    let (cost, tokens, requests) = match q.period.as_str() {
        "month" => aggregate_billing(&conn, &id, "start of month"),
        _ => aggregate_billing(&conn, &id, "-1 day"),
    };
    Json(json!({
        "aggregate": {
            "cost": cost,
            "tokens": tokens,
            "requests": requests,
            "period": q.period,
            "org_id": id,
        }
    }))
}

fn aggregate_billing(conn: &rusqlite::Connection, org_id: &str, offset: &str) -> (f64, i64, i64) {
    conn.query_row(
        "SELECT COALESCE(SUM(cost_usd), 0.0), \
                CAST(COALESCE(SUM(quantity), 0) AS INTEGER), \
                COUNT(*) \
         FROM billing_usage \
         WHERE org_id = ?1 AND created_at >= datetime('now', ?2)",
        rusqlite::params![org_id, offset],
        |r| {
            Ok((
                r.get::<_, f64>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
            ))
        },
    )
    .unwrap_or((0.0, 0, 0))
}

/// GET /api/orgs/:id/digest
pub async fn get_org_digest(State(s): State<Arc<OrgState>>, Path(id): Path<String>) -> Json<Value> {
    let conn = match s.pool.get() {
        Ok(c) => c,
        Err(e) => return Json(json!({"error": e.to_string()})),
    };
    let org = conn.query_row(
        "SELECT mission, status, ceo_agent FROM ipc_orgs WHERE id = ?1",
        [&id],
        |r| {
            Ok(json!({
                "mission": r.get::<_, String>(0)?,
                "status": r.get::<_, String>(1)?,
                "ceo_agent": r.get::<_, String>(2)?,
            }))
        },
    );
    let org = match org {
        Ok(o) => o,
        Err(_) => return Json(json!({"error": "org not found"})),
    };
    let members = crate::routes_members::load_members(&conn, &id);
    let member_count = members.len();
    let night_agent_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM night_agent_defs WHERE org_id = ?1",
            [&id],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let recent_decisions = load_recent_decisions(&conn, &id, 5);
    Json(json!({
        "digest": {
            "org_id": id,
            "org": org,
            "member_count": member_count,
            "night_agent_count": night_agent_count,
            "recent_decisions": recent_decisions,
        }
    }))
}

fn load_recent_decisions(conn: &rusqlite::Connection, org_id: &str, limit: u32) -> Vec<Value> {
    // Filter decisions by agent membership in this org for org isolation.
    let mut stmt = match conn.prepare(
        "SELECT d.decision, d.reasoning, d.agent, d.created_at \
         FROM decision_log d \
         WHERE d.agent IN (SELECT agent FROM ipc_org_members WHERE org_id = ?1) \
            OR d.agent = 'onboard-system' \
         ORDER BY d.id DESC LIMIT ?2",
    ) {
        Ok(s) => s,
        Err(_) => return vec![],
    };
    stmt.query_map(rusqlite::params![org_id, limit], |r| {
        Ok(json!({
            "decision": r.get::<_, String>(0)?,
            "reasoning": r.get::<_, String>(1)?,
            "agent": r.get::<_, Option<String>>(2)?,
            "created_at": r.get::<_, Option<String>>(3)?,
        }))
    })
    .map(|rows| rows.filter_map(|r| r.ok()).collect())
    .unwrap_or_default()
}

/// GET /api/orgs/:slug/plans
pub async fn get_org_plans(
    State(s): State<Arc<OrgState>>,
    Path(slug): Path<String>,
) -> Json<Value> {
    let conn = match s.pool.get() {
        Ok(c) => c,
        Err(e) => return Json(json!({"error": e.to_string()})),
    };
    let mut stmt = match conn.prepare(
        "SELECT id, name, status, \
                (SELECT COUNT(*) FROM tasks \
                 WHERE tasks.plan_id = plans.id AND status = 'done') \
                 as tasks_done, \
                (SELECT COUNT(*) FROM tasks \
                 WHERE tasks.plan_id = plans.id) as tasks_total \
         FROM plans WHERE project_id = ?1 ORDER BY id DESC",
    ) {
        Ok(s) => s,
        Err(_) => return Json(json!({"plans": []})),
    };
    let plans: Vec<Value> = stmt
        .query_map([&slug], |r| {
            Ok(json!({
                "id": r.get::<_, i64>(0)?,
                "name": r.get::<_, String>(1)?,
                "status": r.get::<_, String>(2)?,
                "tasks_done": r.get::<_, i64>(3)?,
                "tasks_total": r.get::<_, i64>(4)?,
            }))
        })
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default();
    Json(json!({"plans": plans}))
}
