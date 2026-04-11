use super::*;

fn setup_db() -> rusqlite::Connection {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    conn.execute_batch(
        "CREATE TABLE ipc_orgs (
            id TEXT PRIMARY KEY,
            mission TEXT NOT NULL DEFAULT '',
            objectives TEXT NOT NULL DEFAULT '',
            ceo_agent TEXT NOT NULL DEFAULT '',
            budget REAL NOT NULL DEFAULT 0,
            daily_budget_tokens INTEGER NOT NULL DEFAULT 0,
            status TEXT NOT NULL DEFAULT 'active',
            created_at TEXT NOT NULL DEFAULT '',
            updated_at TEXT NOT NULL DEFAULT ''
        );
        CREATE TABLE agent_catalog (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            role TEXT NOT NULL,
            org_id TEXT NOT NULL DEFAULT 'convergio',
            category TEXT NOT NULL,
            model_tier TEXT NOT NULL DEFAULT 't2',
            max_tokens INTEGER NOT NULL DEFAULT 200000,
            hourly_budget REAL NOT NULL DEFAULT 0.0,
            capabilities_json TEXT NOT NULL DEFAULT '[]',
            prompt_ref TEXT,
            escalation_target TEXT,
            status TEXT NOT NULL DEFAULT 'active',
            created_at TEXT,
            updated_at TEXT
        );
        CREATE TABLE tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id TEXT,
            plan_id INTEGER NOT NULL DEFAULT 0,
            wave_id INTEGER,
            title TEXT NOT NULL DEFAULT '',
            description TEXT,
            status TEXT NOT NULL DEFAULT 'pending',
            executor_agent TEXT,
            started_at TEXT,
            completed_at TEXT,
            notes TEXT,
            tokens INTEGER,
            output_data TEXT,
            executor_host TEXT
        );",
    )
    .unwrap();
    conn
}

fn insert_org(conn: &rusqlite::Connection, id: &str) {
    conn.execute(
        "INSERT INTO ipc_orgs (id, mission, ceo_agent) VALUES (?1, 'test', 'ceo')",
        [id],
    )
    .unwrap();
}

fn insert_agent(conn: &rusqlite::Connection, name: &str, org: &str, caps: &[&str]) {
    let caps_json = serde_json::to_string(&caps).unwrap();
    conn.execute(
        "INSERT INTO agent_catalog (id, name, role, org_id, category, capabilities_json) \
         VALUES (?1, ?2, 'worker', ?3, 'core_utility', ?4)",
        rusqlite::params![format!("ag-{name}"), name, org, caps_json],
    )
    .unwrap();
}

fn insert_task(conn: &rusqlite::Connection, agent: &str, status: &str) {
    conn.execute(
        "INSERT INTO tasks (plan_id, executor_agent, status) VALUES (1, ?1, ?2)",
        [agent, status],
    )
    .unwrap();
}

#[test]
fn capability_matching_is_case_insensitive_and_partial() {
    assert!(capability_matches(
        "planning, tracking, priorities",
        "planning"
    ));
    assert!(capability_matches("Rust development", "rust"));
    assert!(!capability_matches("design tokens", "backend"));
}

#[test]
fn finds_matching_agent() {
    let conn = setup_db();
    insert_org(&conn, "org1");
    insert_agent(&conn, "alice", "org1", &["rust", "review"]);
    insert_agent(&conn, "bob", "org1", &["python"]);

    let caps = vec!["rust".to_string()];
    let candidates = find_matching_agents(&conn, "org1", &caps).unwrap();
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].name, "alice");
}

#[test]
fn picks_lowest_load() {
    let conn = setup_db();
    insert_org(&conn, "org1");
    insert_agent(&conn, "alice", "org1", &["rust"]);
    insert_agent(&conn, "bob", "org1", &["rust"]);

    for _ in 0..3 {
        insert_task(&conn, "alice", "in_progress");
    }
    insert_task(&conn, "bob", "in_progress");

    let caps = vec!["rust".to_string()];
    let candidates = find_matching_agents(&conn, "org1", &caps).unwrap();
    assert_eq!(candidates.len(), 2);

    let best = candidates
        .iter()
        .max_by(|a, b| {
            a.matched_caps
                .cmp(&b.matched_caps)
                .then_with(|| b.in_progress.cmp(&a.in_progress))
        })
        .unwrap();
    assert_eq!(best.name, "bob");
}

#[test]
fn no_match_returns_empty() {
    let conn = setup_db();
    insert_org(&conn, "org1");
    insert_agent(&conn, "alice", "org1", &["python"]);

    let caps = vec!["rust".to_string()];
    let candidates = find_matching_agents(&conn, "org1", &caps).unwrap();
    assert!(candidates.is_empty());
}

#[test]
fn prefers_more_capabilities_matched() {
    let conn = setup_db();
    insert_org(&conn, "org1");
    insert_agent(&conn, "alice", "org1", &["rust", "review", "testing"]);
    insert_agent(&conn, "bob", "org1", &["rust"]);

    let caps = vec!["rust".to_string(), "review".to_string()];
    let candidates = find_matching_agents(&conn, "org1", &caps).unwrap();

    let best = candidates
        .iter()
        .max_by(|a, b| {
            a.matched_caps
                .cmp(&b.matched_caps)
                .then_with(|| b.in_progress.cmp(&a.in_progress))
        })
        .unwrap();
    assert_eq!(best.name, "alice");
    assert_eq!(best.matched_caps, 2);
}
