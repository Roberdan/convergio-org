//! Extension trait implementation for convergio-org.

use std::sync::Arc;
use std::time::Duration;

use convergio_db::pool::ConnPool;
use convergio_types::extension::{AppContext, Extension, Health, McpToolDef, Metric, Migration};
use convergio_types::manifest::{Capability, Dependency, Manifest, ModuleKind};

use crate::routes::{org_routes, OrgState};

/// Org extension — organization design, provisioning, notifications, decisions.
pub struct OrgExtension {
    pool: ConnPool,
}

impl OrgExtension {
    pub fn new(pool: ConnPool) -> Self {
        Self { pool }
    }

    fn state_with_sink(
        &self,
        event_sink: Option<std::sync::Arc<dyn convergio_types::events::DomainEventSink>>,
    ) -> Arc<OrgState> {
        let daemon_url = std::env::var("CONVERGIO_PORT")
            .map(|p| format!("http://localhost:{p}"))
            .unwrap_or_else(|_| "http://localhost:8420".into());
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap();
        Arc::new(OrgState {
            pool: self.pool.clone(),
            daemon_url,
            client,
            event_sink,
        })
    }
}

impl Extension for OrgExtension {
    fn manifest(&self) -> Manifest {
        Manifest {
            id: "convergio-org".to_string(),
            description: "Organization chart, provisioning, notifications, decision log"
                .to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            kind: ModuleKind::Extension,
            provides: vec![
                Capability {
                    name: "org-design".to_string(),
                    version: "1.0.0".to_string(),
                    description: "Design orgs from mission statements or repo profiles".to_string(),
                },
                Capability {
                    name: "org-provisioning".to_string(),
                    version: "1.0.0".to_string(),
                    description: "Provision orgs via daemon HTTP API".to_string(),
                },
                Capability {
                    name: "notification-queue".to_string(),
                    version: "1.0.0".to_string(),
                    description: "Queue and deliver notifications with audit trail".to_string(),
                },
                Capability {
                    name: "decision-log".to_string(),
                    version: "1.0.0".to_string(),
                    description: "Audit trail of agent decisions with reasoning".to_string(),
                },
            ],
            requires: vec![Dependency {
                capability: "db-pool".to_string(),
                version_req: ">=1.0.0".to_string(),
                required: true,
            }],
            agent_tools: vec![],
            required_roles: vec![],
        }
    }

    fn routes(&self, ctx: &AppContext) -> Option<axum::Router> {
        let sink = ctx
            .get_arc::<std::sync::Arc<dyn convergio_types::events::DomainEventSink>>()
            .map(|s| (*s).clone());
        Some(org_routes(self.state_with_sink(sink)))
    }

    fn migrations(&self) -> Vec<Migration> {
        vec![
            Migration {
                version: 1,
                description: "org tables",
                up: "CREATE TABLE IF NOT EXISTS notifications (\
                    id INTEGER PRIMARY KEY AUTOINCREMENT,\
                    type TEXT NOT NULL DEFAULT '',\
                    title TEXT NOT NULL DEFAULT '',\
                    message TEXT NOT NULL DEFAULT '',\
                    is_read INTEGER DEFAULT 0,\
                    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,\
                    updated_at DATETIME\
                );\
                CREATE TABLE IF NOT EXISTS notification_queue (\
                    id INTEGER PRIMARY KEY,\
                    severity TEXT DEFAULT 'info',\
                    title TEXT NOT NULL DEFAULT '',\
                    message TEXT,\
                    plan_id INTEGER,\
                    link TEXT,\
                    status TEXT DEFAULT 'pending',\
                    created_at TEXT DEFAULT (datetime('now')),\
                    delivered_at TEXT\
                );\
                CREATE INDEX IF NOT EXISTS idx_nq_status \
                    ON notification_queue(status);\
                CREATE TABLE IF NOT EXISTS notification_deliveries (\
                    id INTEGER PRIMARY KEY AUTOINCREMENT,\
                    notification_id INTEGER NOT NULL,\
                    trace_id TEXT NOT NULL,\
                    channel TEXT NOT NULL,\
                    success INTEGER NOT NULL DEFAULT 0,\
                    error_message TEXT,\
                    duration_ms INTEGER NOT NULL DEFAULT 0,\
                    created_at TEXT NOT NULL DEFAULT (datetime('now'))\
                );\
                CREATE INDEX IF NOT EXISTS idx_nd_notification \
                    ON notification_deliveries(notification_id);\
                CREATE INDEX IF NOT EXISTS idx_nd_trace \
                    ON notification_deliveries(trace_id);\
                CREATE TABLE IF NOT EXISTS decision_log (\
                    id INTEGER PRIMARY KEY,\
                    plan_id INTEGER,\
                    task_id INTEGER,\
                    decision TEXT NOT NULL,\
                    reasoning TEXT NOT NULL,\
                    first_principles TEXT,\
                    alternatives_considered TEXT,\
                    outcome TEXT,\
                    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,\
                    agent TEXT\
                );",
            },
            Migration {
                version: 2,
                description: "org ask audit log",
                up: "CREATE TABLE IF NOT EXISTS org_ask_log (\
                    id INTEGER PRIMARY KEY AUTOINCREMENT,\
                    org_id TEXT NOT NULL,\
                    question TEXT NOT NULL,\
                    intent TEXT NOT NULL DEFAULT '',\
                    escalated INTEGER NOT NULL DEFAULT 0,\
                    latency_ms INTEGER NOT NULL DEFAULT 0,\
                    created_at TEXT NOT NULL DEFAULT (datetime('now'))\
                );\
                CREATE INDEX IF NOT EXISTS idx_oal_org ON org_ask_log(org_id);\
                CREATE INDEX IF NOT EXISTS idx_oal_created ON org_ask_log(created_at);",
            },
            Migration {
                version: 3,
                description: "org skills for skill-based routing and delegation",
                up: "CREATE TABLE IF NOT EXISTS org_skills (\
                    id INTEGER PRIMARY KEY AUTOINCREMENT,\
                    org_id TEXT NOT NULL,\
                    skill TEXT NOT NULL,\
                    description TEXT DEFAULT '',\
                    confidence REAL DEFAULT 0.5,\
                    created_at TEXT DEFAULT (datetime('now')),\
                    UNIQUE(org_id, skill)\
                );\
                CREATE INDEX IF NOT EXISTS idx_os_org ON org_skills(org_id);",
            },
        ]
    }

    fn health(&self) -> Health {
        match self.pool.get() {
            Ok(conn) => {
                let count: i64 = conn
                    .query_row("SELECT count(*) FROM ipc_orgs", [], |r| r.get(0))
                    .unwrap_or(0);
                if count >= 0 {
                    Health::Ok
                } else {
                    Health::Degraded {
                        reason: "negative org count".into(),
                    }
                }
            }
            Err(e) => Health::Degraded {
                reason: format!("db: {e}"),
            },
        }
    }

    fn metrics(&self) -> Vec<Metric> {
        let conn = match self.pool.get() {
            Ok(c) => c,
            Err(_) => return vec![],
        };
        let orgs: f64 = conn
            .query_row("SELECT count(*) FROM ipc_orgs", [], |r| r.get(0))
            .unwrap_or(0.0);
        let notifs: f64 = conn
            .query_row(
                "SELECT count(*) FROM notification_queue WHERE status = 'pending'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0.0);
        let ask_total = crate::routes_ask_audit::ask_total(&self.pool);
        let ask_escalated = crate::routes_ask_audit::ask_escalated_total(&self.pool);
        vec![
            Metric {
                name: "org_count".into(),
                value: orgs,
                labels: vec![],
            },
            Metric {
                name: "org_pending_notifications".into(),
                value: notifs,
                labels: vec![],
            },
            Metric {
                name: "org_ask_total".into(),
                value: ask_total,
                labels: vec![],
            },
            Metric {
                name: "org_ask_escalated_total".into(),
                value: ask_escalated,
                labels: vec![],
            },
        ]
    }

    fn mcp_tools(&self) -> Vec<McpToolDef> {
        crate::mcp_defs::org_tools()
    }
}
