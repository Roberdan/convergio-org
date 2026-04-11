//! MCP tool definitions for the org extension.

use convergio_types::extension::McpToolDef;
use serde_json::json;

pub fn org_tools() -> Vec<McpToolDef> {
    vec![
        McpToolDef {
            name: "cvg_list_orgs".into(),
            description: "List all organizations.".into(),
            method: "GET".into(),
            path: "/api/orgs".into(),
            input_schema: json!({"type": "object", "properties": {}}),
            min_ring: "sandboxed".into(),
            path_params: vec![],
        },
        McpToolDef {
            name: "cvg_create_org".into(),
            description: "Create a new organization.".into(),
            method: "POST".into(),
            path: "/api/orgs".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "name": {"type": "string"},
                    "description": {"type": "string"}
                },
                "required": ["name"]
            }),
            min_ring: "trusted".into(),
            path_params: vec![],
        },
        McpToolDef {
            name: "cvg_get_org".into(),
            description: "Get organization details by ID.".into(),
            method: "GET".into(),
            path: "/api/orgs/:id".into(),
            input_schema: json!({
                "type": "object",
                "properties": {"id": {"type": "string"}},
                "required": ["id"]
            }),
            min_ring: "community".into(),
            path_params: vec!["id".into()],
        },
        McpToolDef {
            name: "cvg_update_org".into(),
            description: "Update organization details.".into(),
            method: "PUT".into(),
            path: "/api/orgs/:id".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "id": {"type": "string"},
                    "name": {"type": "string"},
                    "description": {"type": "string"}
                },
                "required": ["id"]
            }),
            min_ring: "trusted".into(),
            path_params: vec!["id".into()],
        },
        McpToolDef {
            name: "cvg_org_telemetry".into(),
            description: "Get telemetry data for an organization.".into(),
            method: "GET".into(),
            path: "/api/orgs/:id/telemetry".into(),
            input_schema: json!({
                "type": "object",
                "properties": {"id": {"type": "string"}},
                "required": ["id"]
            }),
            min_ring: "community".into(),
            path_params: vec!["id".into()],
        },
        McpToolDef {
            name: "cvg_org_digest".into(),
            description: "Get digest summary for an organization.".into(),
            method: "GET".into(),
            path: "/api/orgs/:id/digest".into(),
            input_schema: json!({
                "type": "object",
                "properties": {"id": {"type": "string"}},
                "required": ["id"]
            }),
            min_ring: "community".into(),
            path_params: vec!["id".into()],
        },
        McpToolDef {
            name: "cvg_org_plans".into(),
            description: "List plans for an organization.".into(),
            method: "GET".into(),
            path: "/api/orgs/:id/plans".into(),
            input_schema: json!({
                "type": "object",
                "properties": {"id": {"type": "string"}},
                "required": ["id"]
            }),
            min_ring: "community".into(),
            path_params: vec!["id".into()],
        },
        McpToolDef {
            name: "cvg_org_add_member".into(),
            description: "Add a member to an organization.".into(),
            method: "POST".into(),
            path: "/api/orgs/:id/members".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "id": {"type": "string"},
                    "agent_name": {"type": "string", "description": "Agent name to add"}
                },
                "required": ["id", "agent_name"]
            }),
            min_ring: "trusted".into(),
            path_params: vec!["id".into()],
        },
        McpToolDef {
            name: "cvg_org_remove_member".into(),
            description: "Remove a member from an organization.".into(),
            method: "DELETE".into(),
            path: "/api/orgs/:id/members/:agent".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "id": {"type": "string"},
                    "agent": {"type": "string"}
                },
                "required": ["id", "agent"]
            }),
            min_ring: "trusted".into(),
            path_params: vec!["id".into(), "agent".into()],
        },
        McpToolDef {
            name: "cvg_org_chart".into(),
            description: "Get the org chart for an organization.".into(),
            method: "GET".into(),
            path: "/api/orgs/:id/orgchart".into(),
            input_schema: json!({
                "type": "object",
                "properties": {"id": {"type": "string"}},
                "required": ["id"]
            }),
            min_ring: "community".into(),
            path_params: vec!["id".into()],
        },
        McpToolDef {
            name: "cvg_org_dispatch".into(),
            description: "Dispatch a task within an organization.".into(),
            method: "POST".into(),
            path: "/api/org/:id/dispatch".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "id": {"type": "string"},
                    "task": {"type": "string", "description": "Task description"}
                },
                "required": ["id", "task"]
            }),
            min_ring: "trusted".into(),
            path_params: vec!["id".into()],
        },
        McpToolDef {
            name: "cvg_list_decisions".into(),
            description: "List organization decisions.".into(),
            method: "GET".into(),
            path: "/api/decisions".into(),
            input_schema: json!({"type": "object", "properties": {}}),
            min_ring: "community".into(),
            path_params: vec![],
        },
        McpToolDef {
            name: "cvg_create_decision".into(),
            description: "Record an organization decision.".into(),
            method: "POST".into(),
            path: "/api/decisions".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "title": {"type": "string"},
                    "description": {"type": "string"},
                    "rationale": {"type": "string"}
                },
                "required": ["title"]
            }),
            min_ring: "trusted".into(),
            path_params: vec![],
        },
        McpToolDef {
            name: "cvg_org_ask".into(),
            description: "Ask a question to the organization AI.".into(),
            method: "POST".into(),
            path: "/api/orgs/:id/ask".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "id": {"type": "string"},
                    "question": {"type": "string"}
                },
                "required": ["id", "question"]
            }),
            min_ring: "trusted".into(),
            path_params: vec!["id".into()],
        },
        McpToolDef {
            name: "cvg_org_ask_log".into(),
            description: "Get the ask log for an organization.".into(),
            method: "GET".into(),
            path: "/api/orgs/:id/ask-log".into(),
            input_schema: json!({
                "type": "object",
                "properties": {"id": {"type": "string"}},
                "required": ["id"]
            }),
            min_ring: "community".into(),
            path_params: vec!["id".into()],
        },
        McpToolDef {
            name: "cvg_org_kb_seed".into(),
            description: "Seed the knowledge base for an organization.".into(),
            method: "POST".into(),
            path: "/api/orgs/:id/kb/seed".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "id": {"type": "string"},
                    "content": {"type": "string"}
                },
                "required": ["id"]
            }),
            min_ring: "trusted".into(),
            path_params: vec!["id".into()],
        },
        McpToolDef {
            name: "cvg_scan_projects".into(),
            description: "Scan filesystem for projects to onboard.".into(),
            method: "GET".into(),
            path: "/api/projects/scan".into(),
            input_schema: json!({"type": "object", "properties": {}}),
            min_ring: "community".into(),
            path_params: vec![],
        },
        McpToolDef {
            name: "cvg_onboard_project".into(),
            description: "Onboard a project into the organization.".into(),
            method: "POST".into(),
            path: "/api/org/projects/onboard".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Project path"},
                    "name": {"type": "string", "description": "Project name"}
                },
                "required": ["path"]
            }),
            min_ring: "trusted".into(),
            path_params: vec![],
        },
    ]
}
