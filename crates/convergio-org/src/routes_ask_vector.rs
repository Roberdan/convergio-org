//! Vector knowledge enrichment — fetches semantic context from the knowledge store.
//!
//! Used by org-ask to enrich grounded inference with relevant vector matches.

use serde_json::{json, Value};

/// Fetch semantically relevant knowledge from the vector store.
/// Returns formatted text to append to org context. Empty string if unavailable.
pub(crate) async fn fetch_vector_context(
    client: &reqwest::Client,
    daemon_url: &str,
    question: &str,
    org_id: &str,
) -> String {
    let url = format!("{daemon_url}/api/knowledge/search");
    let token = std::env::var("CONVERGIO_AUTH_TOKEN").ok();

    let mut req = client
        .post(&url)
        .timeout(std::time::Duration::from_secs(3))
        .json(&json!({
            "query": question,
            "limit": 3,
            "org_id": org_id,
        }));
    if let Some(t) = &token {
        req = req.bearer_auth(t);
    }

    let resp = match req.send().await {
        Ok(r) => r,
        Err(_) => return String::new(),
    };
    let body: Value = match resp.json().await {
        Ok(v) => v,
        Err(_) => return String::new(),
    };

    let results = match body.get("results").and_then(|v| v.as_array()) {
        Some(arr) if !arr.is_empty() => arr,
        _ => return String::new(),
    };

    let mut context = String::new();
    for r in results {
        let content = r
            .pointer("/entry/content")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let score = r.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
        if score > 0.3 && !content.is_empty() {
            context.push_str(&format!("- {content}\n"));
        }
    }
    context
}
