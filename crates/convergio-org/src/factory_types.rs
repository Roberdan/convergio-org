//! Org factory types — data structures for org blueprints.

use serde::{Deserialize, Serialize};

/// Complete blueprint for an organisation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgBlueprint {
    pub name: String,
    pub slug: String,
    pub mission: String,
    pub repo_path: Option<String>,
    pub budget_usd: Option<f64>,
    pub ceo_agent: String,
    pub departments: Vec<Department>,
    pub night_agents: Vec<NightAgentSpec>,
    /// Knowledge items to seed into the org (infra, requirements, how to run/stop/clean).
    pub knowledge_items: Vec<KnowledgeItem>,
}

/// A department within the org containing one or more agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Department {
    pub name: String,
    pub agents: Vec<AgentSpec>,
}

/// Specification for a single agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSpec {
    pub name: String,
    pub model: String,
    pub capabilities: Vec<String>,
    /// Human-readable role title (e.g. "Tech Lead", "Security Auditor").
    pub role: String,
    /// Technical skills this agent possesses (e.g. ["Rust", "axum", "postgres"]).
    pub skills: Vec<String>,
}

/// Specification for a night (off-peak) agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NightAgentSpec {
    pub name: String,
    pub schedule: String,
    pub time: String,
    pub model: String,
}

/// A piece of knowledge to seed into the org knowledge base.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeItem {
    /// Short title (used as the knowledge entry key).
    pub title: String,
    /// Body text — markdown or plain text.
    pub content: String,
    /// Category: "infra", "requirements", "run_guide", "architecture", "security".
    pub category: String,
}
