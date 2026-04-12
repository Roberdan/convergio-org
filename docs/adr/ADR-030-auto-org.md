# ADR-030: Auto-Org Creation per Project

**Status:** Accepted  
**Date:** 2026-04-06  
**Author:** Roberto D'Angelo

## Context

When onboarding a new repository, users had to manually create an org,
configure departments, assign agents, and seed knowledge. This was
tedious and error-prone.

## Decision

Introduce `POST /api/org/projects/onboard` that:

1. **Scans** the repository using `project_scanner::scan_project()` to
   detect languages, frameworks, services, and infra.
2. **Designs** an org via `factory::design_org_from_repo()` which maps
   scan results to departments, agents, night agents, and knowledge items.
3. **Persists** the org to the database and seeds the knowledge base
   with project info (tech stack, CI config, run guide, dependencies).
4. **Auto-configures night agents** via `auto_config::auto_assign_agents()`
   which selects agents based on project type (e.g. Rust gets
   security-scanner, Next.js gets lighthouse-auditor).

## CLI Integration

- `cvg project add <path>` -- onboards a repo via the API
- `cvg project switch <id>` -- sets the active project locally
- `cvg project status` -- lists all orgs and health

## Consequences

- One-command onboarding reduces setup from ~10 manual steps to one.
- Knowledge base is pre-seeded, so agents have context from day one.
- Night agent selection is deterministic and extensible.
- The scan is read-only and safe to run on any repository.

## Update (2026-04-07) — Member Wiring

ADR-030 originally only persisted the org row in `ipc_orgs`. As of
[ADR-031](ADR-031-onboard-wiring.md), `persist_org()` now also wires:
- All agents into `ipc_org_members` and `agent_catalog`
- Night agents into `night_agent_defs`
- IPC channel `#org-{slug}` into `ipc_channels`
- Billing budget into `billing_budgets`
- `.convergio/` directory with config, agents, and knowledge files

The mission is now derived from README/package.json/Cargo.toml instead of
a hardcoded format string. A mandatory Leadership department (CEO, PM,
Tech Lead, Release Manager, 2 Developers) is always added first.

`DELETE /api/orgs/:id` enables safe re-onboarding by cascading all
related records. See ADR-031 for full details.
