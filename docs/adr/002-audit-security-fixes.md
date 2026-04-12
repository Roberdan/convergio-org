# ADR-002 — Security Audit and Input Validation Hardening

| Field       | Value                           |
|------------|----------------------------------|
| Status     | Accepted                         |
| Date       | 2025-07-17                       |
| Author     | Security Audit (Copilot)         |
| Scope      | convergio-org v0.1.1 → v0.1.2   |

## Context

A comprehensive security audit of convergio-org (6081 LOC, 38 source files) identified
several hardening opportunities. While no critical exploitable vulnerabilities were found
(all SQL is parameterised, no unsafe blocks, no command injection vectors), the crate
lacked systematic input validation and had minor org-isolation gaps.

## Findings Summary

### ✅ Already Secure (no changes needed)

| Category          | Status | Notes |
|-------------------|--------|-------|
| SQL injection     | ✅ Safe | All queries use `?N` params via `rusqlite::params![]` |
| Command injection | ✅ Safe | No `Command::new` / `process::Command` usage |
| SSRF              | ✅ Safe | All outbound URLs derive from server config (`daemon_url`) |
| Unsafe blocks     | ✅ Safe | Zero `unsafe` in codebase |
| Path traversal    | ✅ Safe | `validate_path_components` + `is_absolute()` on all path inputs |
| Secret exposure   | ✅ Safe | Tokens from env vars, not logged in responses |
| Auth/AuthZ        | ✅ N/A  | Handled by daemon server layer (documented) |

### 🔧 Fixed in This ADR

| # | Finding | Severity | Fix |
|---|---------|----------|-----|
| 1 | No input validation on org/member/skill creation endpoints | Medium | New `validation.rs` module with length limits, format checks, enum validation |
| 2 | Memory leak via `Box::leak` in `onboard_catalog::role_keywords` | Medium | Return empty vec for unknown roles instead of leaking |
| 3 | UTF-8 truncation panic in `factory::truncate_to` | Medium | Find valid char boundary before slicing |
| 4 | Org digest leaks cross-org decisions | Low | Filter `decision_log` by org membership |
| 5 | `cascade_delete_org` used dynamic SQL via closure | Low | Inline all DELETE statements as static SQL |
| 6 | Ask-log limit not clamped | Low | Apply `validate_limit(min=1, max=200)` |
| 7 | Notification severity not validated | Low | Validate against allowlist: info/warning/error/success |
| 8 | Skill confidence not range-checked | Low | Validate `0.0 ≤ confidence ≤ 1.0` |

### ⚠️ Known Limitations (documented, not fixed)

| Item | Reason |
|------|--------|
| `decision_log` has no `org_id` column | Schema change requires SDK migration; mitigated by agent-membership filter |
| `list_notifications` returns all orgs | Notification queue is global by design (daemon dispatches) |
| No rate limiting on endpoints | Handled by daemon reverse proxy layer |

## Decision

1. Add `validation.rs` module centralising all input validation rules.
2. Apply validation at handler entry points (fail-fast pattern).
3. Fix memory leak, UTF-8 safety, and org-isolation query.
4. Eliminate dynamic SQL pattern in cascade delete.
5. Add 6 new tests for the validation module.

## Consequences

- All endpoints now reject malformed input with clear error messages.
- No memory leaks from unknown role strings.
- No panic possible from multi-byte UTF-8 truncation.
- Org digest no longer leaks cross-org decision data.
- Test count increased from 46 → 52.
