//! Org skills CRUD — each org declares its competencies for skill-based routing.
use axum::extract::{Path, State};
use axum::response::Json;
use axum::routing::{delete, get, post};
use axum::Router;
use rusqlite::params;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::routes::OrgState;

pub fn skill_routes(state: Arc<OrgState>) -> Router {
    Router::new()
        .route("/api/orgs/:org_id/skills", get(list_skills))
        .route("/api/orgs/:org_id/skills", post(add_skill))
        .route("/api/orgs/:org_id/skills/:skill", delete(remove_skill))
        .with_state(state)
}

#[derive(Debug, Deserialize)]
struct AddSkill {
    skill: String,
    #[serde(default)]
    description: String,
    #[serde(default = "default_confidence")]
    confidence: f64,
}
fn default_confidence() -> f64 {
    0.5
}

async fn list_skills(
    State(state): State<Arc<OrgState>>,
    Path(org_id): Path<String>,
) -> Json<Value> {
    let conn = match state.pool.get() {
        Ok(c) => c,
        Err(e) => return Json(json!({"error": e.to_string()})),
    };
    let mut stmt = match conn.prepare(
        "SELECT skill, description, confidence FROM org_skills WHERE org_id = ?1 ORDER BY skill",
    ) {
        Ok(s) => s,
        Err(e) => return Json(json!({"error": e.to_string()})),
    };
    let skills: Vec<Value> = stmt
        .query_map(params![org_id], |r| {
            Ok(json!({
                "skill": r.get::<_,String>(0)?,
                "description": r.get::<_,String>(1)?,
                "confidence": r.get::<_,f64>(2)?
            }))
        })
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default();
    Json(json!({"org_id": org_id, "skills": skills}))
}

async fn add_skill(
    State(state): State<Arc<OrgState>>,
    Path(org_id): Path<String>,
    Json(req): Json<AddSkill>,
) -> Json<Value> {
    let conn = match state.pool.get() {
        Ok(c) => c,
        Err(e) => return Json(json!({"error": e.to_string()})),
    };
    match conn.execute(
        "INSERT OR REPLACE INTO org_skills (org_id, skill, description, confidence) \
         VALUES (?1, ?2, ?3, ?4)",
        params![org_id, req.skill, req.description, req.confidence],
    ) {
        Ok(_) => Json(json!({"ok": true, "org_id": org_id, "skill": req.skill})),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn remove_skill(
    State(state): State<Arc<OrgState>>,
    Path((org_id, skill)): Path<(String, String)>,
) -> Json<Value> {
    let conn = match state.pool.get() {
        Ok(c) => c,
        Err(e) => return Json(json!({"error": e.to_string()})),
    };
    match conn.execute(
        "DELETE FROM org_skills WHERE org_id = ?1 AND skill = ?2",
        params![org_id, skill],
    ) {
        Ok(0) => Json(json!({"error": "skill not found"})),
        Ok(_) => Json(json!({"ok": true, "deleted": skill})),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}
