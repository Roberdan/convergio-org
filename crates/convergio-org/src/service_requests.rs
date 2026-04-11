//! Service request routes — cross-org delegation with skill-based routing.
use axum::extract::{Path, State};
use axum::response::Json;
use axum::routing::{get, post};
use axum::Router;
use rusqlite::params;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::routes::OrgState;

pub fn service_request_routes(state: Arc<OrgState>) -> Router {
    Router::new()
        .route("/api/orgs/service-request", post(create_request))
        .route("/api/orgs/service-requests", get(list_requests))
        .route("/api/orgs/service-request/:id/accept", post(accept_request))
        .route(
            "/api/orgs/service-request/:id/complete",
            post(complete_request),
        )
        .with_state(state)
}

#[derive(Debug, Deserialize)]
struct CreateRequest {
    requester_org: String,
    service_name: String,
    #[serde(default)]
    request_payload: Option<String>,
}

async fn create_request(
    State(state): State<Arc<OrgState>>,
    Json(req): Json<CreateRequest>,
) -> Json<Value> {
    let conn = match state.pool.get() {
        Ok(c) => c,
        Err(e) => return Json(json!({"error": e.to_string()})),
    };
    // Skill-based routing: find org with matching skill
    let provider: Option<(String,)> = conn
        .query_row(
            "SELECT org_id FROM org_skills WHERE skill = ?1 ORDER BY confidence DESC LIMIT 1",
            params![req.service_name],
            |r| Ok((r.get::<_, String>(0)?,)),
        )
        .ok();
    let provider_org = match provider {
        Some((org,)) => org,
        None => {
            return Json(json!({"error": format!("no org has skill '{}'", req.service_name)}));
        }
    };
    let id = format!("sr-{}", uuid::Uuid::new_v4());
    match conn.execute(
        "INSERT INTO ipc_service_requests (id, requester_org, provider_org, service_name, request_payload) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, req.requester_org, provider_org, req.service_name, req.request_payload],
    ) {
        Ok(_) => Json(json!({"id": id, "provider_org": provider_org, "status": "pending"})),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn list_requests(State(state): State<Arc<OrgState>>) -> Json<Value> {
    let conn = match state.pool.get() {
        Ok(c) => c,
        Err(e) => return Json(json!({"error": e.to_string()})),
    };
    let mut stmt = match conn.prepare(
        "SELECT id, requester_org, provider_org, service_name, status, created_at \
         FROM ipc_service_requests ORDER BY created_at DESC LIMIT 50",
    ) {
        Ok(s) => s,
        Err(e) => return Json(json!({"error": e.to_string()})),
    };
    let reqs: Vec<Value> = stmt
        .query_map([], |r| {
            Ok(json!({
                "id": r.get::<_,String>(0)?,
                "requester_org": r.get::<_,String>(1)?,
                "provider_org": r.get::<_,String>(2)?,
                "service_name": r.get::<_,String>(3)?,
                "status": r.get::<_,String>(4)?,
                "created_at": r.get::<_,String>(5)?
            }))
        })
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default();
    Json(json!({"requests": reqs}))
}

async fn accept_request(State(state): State<Arc<OrgState>>, Path(id): Path<String>) -> Json<Value> {
    update_status(&state, &id, "accepted").await
}

async fn complete_request(
    State(state): State<Arc<OrgState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    update_status(&state, &id, "completed").await
}

async fn update_status(state: &OrgState, id: &str, status: &str) -> Json<Value> {
    let conn = match state.pool.get() {
        Ok(c) => c,
        Err(e) => return Json(json!({"error": e.to_string()})),
    };
    let completed = if status == "completed" {
        ", completed_at = datetime('now')"
    } else {
        ""
    };
    let sql = format!("UPDATE ipc_service_requests SET status = ?1{completed} WHERE id = ?2");
    match conn.execute(&sql, params![status, id]) {
        Ok(0) => Json(json!({"error": "request not found"})),
        Ok(_) => Json(json!({"ok": true, "id": id, "status": status})),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}
