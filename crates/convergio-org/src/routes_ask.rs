//! POST /api/orgs/:id/ask — org-as-knowledge interface.
//!
//! Follows ADR-032: convergio-org MUST NOT import convergio-kernel.
//! All kernel calls are via HTTP (reqwest).
//!
//! Call sequence:
//!
//! 1. Load org knowledge context from DB.
//! 2. POST /api/kernel/classify-intent  → { intent, confidence }
//! 3. If intent == "factual" && confidence >= 0.7 (and escalate != true):
//!    POST /api/kernel/grounded-infer  → { answer, agent, latency_ms }.
//!    Otherwise: return escalated=true (caller decides whether to use cloud).

use std::sync::Arc;
use std::time::Instant;

use axum::extract::{Path, State};
use axum::response::Json;
use convergio_types::events::{make_event, EventContext, EventKind};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::org_knowledge::load_org_knowledge;
use crate::routes::OrgState;
use crate::routes_ask_vector::fetch_vector_context;

// ── Request / Response types ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct AskBody {
    pub question: String,
    #[serde(default)]
    pub escalate: bool,
}

#[derive(Debug, Serialize)]
pub struct AskResponse {
    pub answer: Option<String>,
    pub intent: String,
    pub confidence: f64,
    pub escalated: bool,
    pub agent: String,
    pub latency_ms: u64,
}

// ── Handler ───────────────────────────────────────────────────────────────────

/// POST /api/orgs/:id/ask
pub async fn ask_org(
    State(s): State<Arc<OrgState>>,
    Path(org_id): Path<String>,
    Json(body): Json<AskBody>,
) -> Json<Value> {
    use crate::validation::validate_long_text;
    if let Err(e) = validate_long_text(&body.question, "question") {
        return Json(json!({"error": e}));
    }
    let t0 = Instant::now();

    // 1. Load org context from DB.
    let conn = match s.pool.get() {
        Ok(conn) => conn,
        Err(e) => return Json(json!({"error": e.to_string()})),
    };
    let mut knowledge = match load_org_knowledge(&conn, &org_id) {
        Ok(k) => k,
        Err(e) if e.contains("not found") => {
            return Json(json!({"error": "org not found", "org_id": org_id}));
        }
        Err(e) => return Json(json!({"error": e})),
    };

    // 1b. Enrich with vector knowledge search
    let vector_context =
        fetch_vector_context(&s.client, &s.daemon_url, &body.question, &org_id).await;
    if !vector_context.is_empty() {
        knowledge
            .summary
            .push_str("\n\n## Semantic Knowledge Matches\n");
        knowledge.summary.push_str(&vector_context);
    }

    // Force escalation path when caller requested it.
    if body.escalate {
        let latency_ms = t0.elapsed().as_millis() as u64;
        emit_org_asked(&s, &org_id, &body.question, "escalation", true, latency_ms);
        return Json(json!(AskResponse {
            answer: None,
            intent: "escalation".to_string(),
            confidence: 1.0,
            escalated: true,
            agent: "claude".to_string(),
            latency_ms,
        }));
    }

    // 2. Classify intent via kernel HTTP endpoint.
    let classify_url = format!("{}/api/kernel/classify-intent", s.daemon_url);
    let classify_payload = json!({
        "question": body.question,
        "context_hint": knowledge.context_hint,
    });

    let classify_resp = match call_kernel(&s.client, &classify_url, classify_payload).await {
        Ok(v) => v,
        Err(_e) => {
            // classify-intent unavailable — escalate gracefully.
            let latency_ms = t0.elapsed().as_millis() as u64;
            emit_org_asked(&s, &org_id, &body.question, "escalation", true, latency_ms);
            return Json(json!(AskResponse {
                answer: None,
                intent: "escalation".to_string(),
                confidence: 0.0,
                escalated: true,
                agent: "claude".to_string(),
                latency_ms,
            }));
        }
    };

    let intent = classify_resp
        .get("intent")
        .and_then(|v| v.as_str())
        .unwrap_or("escalation")
        .to_string();
    let confidence = classify_resp
        .get("confidence")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

    // 3a. Fast path: factual + high confidence → grounded inference.
    if intent == "factual" && confidence >= 0.7 {
        let infer_url = format!("{}/api/kernel/grounded-infer", s.daemon_url);
        let infer_payload = json!({
            "question": body.question,
            "context": knowledge.summary,
        });

        match call_kernel(&s.client, &infer_url, infer_payload).await {
            Ok(infer_resp) => {
                let answer = infer_resp
                    .get("answer")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                let agent = infer_resp
                    .get("agent")
                    .and_then(|v| v.as_str())
                    .unwrap_or("jarvis-mlx")
                    .to_string();
                let latency_ms = t0.elapsed().as_millis() as u64;
                emit_org_asked(&s, &org_id, &body.question, &intent, false, latency_ms);
                return Json(json!(AskResponse {
                    answer,
                    intent,
                    confidence,
                    escalated: false,
                    agent,
                    latency_ms,
                }));
            }
            Err(_) => {
                // grounded-infer unavailable — fall through to escalation.
            }
        }
    }

    // 3b. Escalation path.
    let latency_ms = t0.elapsed().as_millis() as u64;
    emit_org_asked(&s, &org_id, &body.question, &intent, true, latency_ms);
    Json(json!(AskResponse {
        answer: None,
        intent,
        confidence,
        escalated: true,
        agent: "claude".to_string(),
        latency_ms,
    }))
}

// ── Event emission ────────────────────────────────────────────────────────────

fn emit_org_asked(
    s: &OrgState,
    org_id: &str,
    question: &str,
    intent: &str,
    escalated: bool,
    latency_ms: u64,
) {
    // Audit log — write every ask for telemetry and traceability.
    crate::routes_ask_audit::record_ask(&s.pool, org_id, question, intent, escalated, latency_ms);

    if let Some(ref sink) = s.event_sink {
        sink.emit(make_event(
            "convergio-org",
            EventKind::OrgAsked {
                org_id: org_id.to_string(),
                question: question.to_string(),
                intent: intent.to_string(),
                escalated,
                latency_ms,
            },
            EventContext {
                org_id: Some(org_id.to_string()),
                ..Default::default()
            },
        ));
    }
}

// ── HTTP helper ───────────────────────────────────────────────────────────────

async fn call_kernel(client: &reqwest::Client, url: &str, payload: Value) -> Result<Value, String> {
    let token = std::env::var("CONVERGIO_AUTH_TOKEN").ok();
    let mut req = client.post(url).json(&payload);
    if let Some(t) = &token {
        req = req.bearer_auth(t);
    }

    let resp = req.send().await.map_err(|e| format!("POST {url}: {e}"))?;
    let status = resp.status();
    let text = resp.text().await.map_err(|e| format!("read body: {e}"))?;
    if !status.is_success() {
        return Err(format!("POST {url} -> {status}: {text}"));
    }
    serde_json::from_str(&text).map_err(|e| format!("parse response: {e}"))
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ask_response_serialises() {
        let r = AskResponse {
            answer: Some("42".to_string()),
            intent: "factual".to_string(),
            confidence: 0.9,
            escalated: false,
            agent: "jarvis-mlx".to_string(),
            latency_ms: 150,
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["intent"], "factual");
        assert_eq!(v["escalated"], false);
        assert!(v["answer"].as_str().is_some());
    }

    #[test]
    fn escalation_response_serialises() {
        let r = AskResponse {
            answer: None,
            intent: "escalation".to_string(),
            confidence: 0.3,
            escalated: true,
            agent: "claude".to_string(),
            latency_ms: 5,
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["escalated"], true);
        assert!(v["answer"].is_null());
    }
}
