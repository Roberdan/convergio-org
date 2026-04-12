//! Audit log for POST /api/orgs/:id/ask.
//!
//! Every org-ask query is recorded in org_ask_log for audit + telemetry.
//! Table is created by migration version 2 in OrgExtension.

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::response::Json;
use convergio_db::pool::ConnPool;
use rusqlite::Connection;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::routes::OrgState;

/// Write one ask record to the audit log.
/// Errors are silently ignored — audit must never block the response path.
pub fn record_ask(
    pool: &ConnPool,
    org_id: &str,
    question: &str,
    intent: &str,
    escalated: bool,
    latency_ms: u64,
) {
    let Ok(conn) = pool.get() else { return };
    ensure_ask_log_table(&conn);
    let _ = conn.execute(
        "INSERT INTO org_ask_log \
         (org_id, question, intent, escalated, latency_ms, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'))",
        rusqlite::params![
            org_id,
            question,
            intent,
            escalated as i64,
            latency_ms as i64
        ],
    );
}

// ── Query parameters ──────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct AskLogQuery {
    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_limit() -> u32 {
    50
}

/// GET /api/orgs/:id/ask-log?limit=N
///
/// Returns the last N ask records for the given org, newest first.
pub async fn get_ask_log(
    State(s): State<Arc<OrgState>>,
    Path(org_id): Path<String>,
    Query(q): Query<AskLogQuery>,
) -> Json<Value> {
    let conn = match s.pool.get() {
        Ok(c) => c,
        Err(e) => return Json(json!({"error": e.to_string()})),
    };
    ensure_ask_log_table(&conn);

    let limit = crate::validation::validate_limit(q.limit, 200);

    let mut stmt = match conn.prepare(
        "SELECT id, question, intent, escalated, latency_ms, created_at \
         FROM org_ask_log WHERE org_id = ?1 \
         ORDER BY id DESC LIMIT ?2",
    ) {
        Ok(s) => s,
        Err(e) => return Json(json!({"error": e.to_string()})),
    };

    let rows: Vec<Value> = stmt
        .query_map(rusqlite::params![org_id, limit], |r| {
            Ok(json!({
                "id":         r.get::<_, i64>(0)?,
                "question":   r.get::<_, String>(1)?,
                "intent":     r.get::<_, String>(2)?,
                "escalated":  r.get::<_, i64>(3)? != 0,
                "latency_ms": r.get::<_, i64>(4)?,
                "created_at": r.get::<_, String>(5)?,
            }))
        })
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default();

    Json(json!({ "org_id": org_id, "ask_log": rows }))
}

// ── Count helpers (used by ext.rs metrics) ────────────────────────────────────

pub fn ask_total(pool: &ConnPool) -> f64 {
    pool.get()
        .ok()
        .and_then(|c| {
            ensure_ask_log_table(&c);
            c.query_row("SELECT COUNT(*) FROM org_ask_log", [], |r| {
                r.get::<_, f64>(0)
            })
            .ok()
        })
        .unwrap_or(0.0)
}

pub fn ask_escalated_total(pool: &ConnPool) -> f64 {
    pool.get()
        .ok()
        .and_then(|c| {
            ensure_ask_log_table(&c);
            c.query_row(
                "SELECT COUNT(*) FROM org_ask_log WHERE escalated = 1",
                [],
                |r| r.get::<_, f64>(0),
            )
            .ok()
        })
        .unwrap_or(0.0)
}

fn ensure_ask_log_table(conn: &Connection) {
    let _ = conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS org_ask_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            org_id TEXT NOT NULL,
            question TEXT NOT NULL,
            intent TEXT NOT NULL DEFAULT '',
            escalated INTEGER NOT NULL DEFAULT 0,
            latency_ms INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_oal_org ON org_ask_log(org_id);
        CREATE INDEX IF NOT EXISTS idx_oal_created ON org_ask_log(created_at);",
    );
}
