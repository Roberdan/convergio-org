//! KB seed — platform documentation entries for convergio-io self-help.
//!
//! Seeds the knowledge_base table so that `cvg org ask convergio-io "..."` can
//! answer common onboarding and workflow questions via grounded inference.

use convergio_db::pool::ConnPool;
use rusqlite::params;

/// Platform documentation entries for convergio-io self-help.
pub(crate) fn platform_docs() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "Creating a new project",
            "To create a new project in Convergio:\n\
             1. `cvg newproject` — interactive bootstrap wizard\n\
             2. Or manually: `cvg org onboard /path/to/repo` — scans the repo and creates an org\n\
             3. The system scans your repo, detects language/framework, and designs an org with \
             departments, agents, and knowledge items.\n\
             4. A `.convergio/` directory is generated with org config.\n\
             5. Verify: `cvg org list` to see your new org.",
        ),
        (
            "Gate chain — plan task execution protocol",
            "Every plan task must pass through gates in this order:\n\
             EvidenceGate → TestGate → PrCommitGate → WaveSequenceGate → ValidatorGate\n\n\
             Steps:\n\
             1. Work in a git worktree\n\
             2. Commit, push, create PR\n\
             3. Set notes with PR URL: POST /api/plan-db/task/update\n\
             4. Record evidence (test_result): POST /api/plan-db/task/evidence\n\
             5. Record evidence (test_pass): same endpoint\n\
             6. Update status to submitted\n\
             7. Thor validates after all wave tasks submitted\n\n\
             NEVER skip gates. NEVER update status without evidence.",
        ),
        (
            "Available CLI commands",
            "Core commands:\n\
             - `cvg status` — daemon health and system overview\n\
             - `cvg plan list|show|create` — manage execution plans\n\
             - `cvg task list|update|evidence` — manage plan tasks\n\
             - `cvg wave list|show` — view wave progress\n\
             - `cvg agent list|spawn|context` — manage agents\n\
             - `cvg org list|ask|onboard` — organization management\n\
             - `cvg kb search|write|seed` — knowledge base operations\n\
             - `cvg channel list|send` — inter-agent messaging\n\
             - `cvg memory store|recall` — agent memory\n\
             - `cvg report generate|list|show` — CTT reports\n\
             - `cvg voice start|stop` — voice interface\n\
             - `cvg bus send|subscribe` — message bus\n\
             - `cvg lock acquire|release` — resource locking\n\
             - `cvg setup` — initial configuration wizard",
        ),
        (
            "Agent operating rules",
            "Key rules for all agents:\n\
             - Max 300 lines per file\n\
             - Conventional commits: feat:, fix:, docs:, chore:, refactor:\n\
             - Every task runs in its own git worktree under .worktrees/\n\
             - NEVER work on main branch directly\n\
             - Before done: cargo check + cargo test + cargo fmt\n\
             - Answer 4 questions before any task: Who produces input? Who consumes output? \
             How does user see it? How does system record it's done?\n\
             - Every feature needs: input → processing → output → feedback → state updated → \
             visible to user",
        ),
        (
            "Architecture overview",
            "Convergio is a modular daemon (36 Rust crates):\n\n\
             EXTENSIONS (pluggable): kernel, org, voice, billing, backup, observatory\n\
             PLATFORM (orchestration): orchestrator, agents, inference, prompts, agent-runtime\n\
             INFRASTRUCTURE (core): types, telemetry, db, security, ipc, mesh, server, cli\n\n\
             Every module implements the Extension trait.\n\
             Server runs on port 8420.\n\
             Health: curl http://localhost:8420/api/health",
        ),
        (
            "Task execution flow",
            "To execute a plan task:\n\
             1. Read task: GET /api/plan-db/execution-tree/{plan_id}\n\
             2. Create worktree: git worktree add -b <branch> .worktrees/<name> main\n\
             3. Implement the task\n\
             4. Verify: cargo check && cargo fmt && cargo test\n\
             5. Commit with conventional message + Co-Authored-By trailer\n\
             6. Push and create PR\n\
             7. Record evidence via POST /api/plan-db/task/evidence\n\
             8. Update task status to submitted\n\
             9. Clean up worktree after PR merge",
        ),
        (
            "Delegation protocol",
            "ALL delegated tasks use Opus (tier t1). Sonnet/Haiku are banned for code tasks.\n\
             - Mechanical tasks (effort 1-2): delegate via POST /api/agents/spawn\n\
             - Architecture/security (effort 3): do yourself or escalate to Opus\n\
             - Monitor delegated agents: GET /api/agents/catalog\n\
             - NEVER do everything yourself when tasks can be parallelized",
        ),
        (
            "Merge protocol",
            "Multiple agents may work in parallel. Rules:\n\
             1. NEVER merge with --admin if other PRs are open on same files\n\
             2. Squash merge DISABLED — only merge commits allowed\n\
             3. Check open PRs before merging: gh pr list\n\
             4. Shared files require sequential merge, not parallel\n\
             5. Code files in different crates CAN merge in parallel\n\
             6. If conflict: pull main, rebase, resolve, push --force-with-lease",
        ),
        (
            "Test rules",
            "NON-NEGOTIABLE test rules:\n\
             - NEVER hardcode system counts in tests (use >= not ==)\n\
             - NEVER hardcode versions — use env!(\"CARGO_PKG_VERSION\")\n\
             - NEVER use fixed baselines as CI gates\n\
             - Test helpers shared across binaries need #![allow(dead_code)]\n\
             - Before pushing: cargo fmt --all -- --check && \
             RUSTFLAGS=\"-Dwarnings\" cargo test --workspace",
        ),
        (
            "MCP tools reference",
            "Convergio MCP server provides tools ring-filtered by trust level:\n\
             - cvg_list_plans / cvg_get_plan — plan management\n\
             - cvg_update_task — update task status\n\
             - cvg_list_agents / cvg_agent_start / cvg_agent_complete — agent lifecycle\n\
             - cvg_mesh_status / cvg_node_readiness — mesh topology\n\
             - cvg_cost_summary — spending overview\n\
             - cvg_kernel_ask — ask local LLM with platform context\n\
             - cvg_notify — send notifications\n\
             - cvg_generate_report / cvg_list_reports — CTT reports",
        ),
    ]
}

/// Seed the knowledge base for a given org with platform documentation.
/// Uses INSERT OR REPLACE to be idempotent.
pub(crate) fn seed_platform_docs(pool: &ConnPool, org_id: &str) -> Result<usize, String> {
    let conn = pool.get().map_err(|e| e.to_string())?;
    let docs = platform_docs();
    let mut count = 0usize;
    for (title, content) in &docs {
        conn.execute(
            "INSERT OR REPLACE INTO knowledge_base (domain, title, content, created_at) \
             VALUES (?1, ?2, ?3, datetime('now'))",
            params![org_id, title, content],
        )
        .map_err(|e| format!("seed {title}: {e}"))?;
        count += 1;
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_docs_not_empty() {
        let docs = platform_docs();
        assert!(docs.len() >= 10, "should have at least 10 doc entries");
    }

    #[test]
    fn platform_docs_have_content() {
        for (title, content) in platform_docs() {
            assert!(!title.is_empty(), "title must not be empty");
            assert!(content.len() > 20, "content for '{title}' too short");
        }
    }

    #[test]
    fn gate_chain_doc_exists() {
        let docs = platform_docs();
        let gate = docs.iter().find(|(t, _)| t.contains("Gate chain"));
        assert!(gate.is_some(), "gate chain doc must exist");
        let (_, content) = gate.unwrap();
        assert!(
            content.contains("EvidenceGate"),
            "must mention EvidenceGate"
        );
    }

    #[test]
    fn onboarding_doc_exists() {
        let docs = platform_docs();
        let onboard = docs.iter().find(|(t, _)| t.contains("new project"));
        assert!(onboard.is_some(), "onboarding doc must exist");
        let (_, content) = onboard.unwrap();
        assert!(
            content.contains("cvg newproject") || content.contains("onboard"),
            "must mention newproject or onboard"
        );
    }

    #[test]
    fn commands_doc_exists() {
        let docs = platform_docs();
        let cmds = docs.iter().find(|(t, _)| t.contains("CLI commands"));
        assert!(cmds.is_some(), "commands doc must exist");
        let (_, content) = cmds.unwrap();
        assert!(content.contains("cvg status"), "must mention cvg status");
        assert!(content.contains("cvg kb"), "must mention cvg kb");
    }
}
