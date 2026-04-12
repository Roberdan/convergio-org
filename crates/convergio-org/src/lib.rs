//! convergio-org — Org chart, inter-org comms, provisioner.
//!
//! Extension: owns notifications, notification_queue, notification_deliveries,
//! decision_log. Provides org design from mission/repo, provisioning, orgchart rendering.

pub mod ext;
pub mod factory;
pub mod kb_seed;
pub mod onboard;
mod onboard_catalog;
pub mod onboard_dotfiles;
mod org_knowledge;
pub mod org_skills;
pub mod orgchart;
pub mod project_scanner;
pub mod provisioner;
pub mod repo_scanner;
mod repo_scanner_helpers;
pub mod role_dispatcher;
pub mod routes;
pub mod routes_ask;
pub mod routes_ask_audit;
pub mod routes_ask_vector;
pub mod routes_decisions;
pub mod routes_members;
pub mod routes_notify;
pub mod routes_projects;
pub mod routes_telemetry;
pub mod service_requests;
pub mod telegram;
pub mod validation;

pub use ext::OrgExtension;
pub use factory::{
    design_org_from_mission, design_org_from_repo, slugify, AgentSpec, Department, KnowledgeItem,
    NightAgentSpec, OrgBlueprint,
};
pub use orgchart::{render_orgchart, render_orgchart_compact};
pub use project_scanner::{scan_project, InfraInfo, ProjectScan, RepoType, ServiceInfo};
pub use provisioner::{provision_org, ProvisionResult};
pub use repo_scanner::{scan_repo, CiInfo, RepoProfile, RepoStructure};

pub mod mcp_defs;
#[cfg(test)]
mod tests;
#[cfg(test)]
#[path = "tests_onboard.rs"]
mod tests_onboard;
#[cfg(test)]
#[path = "tests_project_scanner.rs"]
mod tests_project_scanner;
