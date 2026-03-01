# Audit Fix Progress

## Summary
- **Audit report:** docs/audits/audit-full-2026-03-01.md
- **Total findings:** 109
- **P0 (Critical):** 4
- **P1 (Major):** 15
- **P2 (Minor):** 30
- **P3 (Suggestions):** 12 SKIPPED

## Priority Pass Status

| Priority | Status | Findings | Fixed | Escalated | Commit |
|----------|--------|----------|-------|-----------|--------|
| P0 | PENDING | 4 | 0 | 0 | — |
| P1 | PENDING | 15 | 0 | 0 | — |
| P2 | PENDING | 30 | 0 | 0 | — |
| P3 | SKIPPED | 12 | 0 | 0 | — |

## Findings Detail

### P0: Critical
| ID | Title | Status | Test File | Fix Commit |
|----|-------|--------|-----------|------------|
| P0-001 | Auth bypass — empty allowed_users permits all users | PENDING | — | — |
| P0-002 | Sandbox bypass via relative paths | PENDING | — | — |
| P0-003 | ToolExecutor.config_path never set | PENDING | — | — |
| P0-004 | Seatbelt doesn't block config.toml writes | PENDING | — | — |

### P1: Major
| ID | Title | Status | Test File | Fix Commit |
|----|-------|--------|-----------|------------|
| P1-001 | API server no auth by default | PENDING | — | — |
| P1-002 | constant_time_eq leaks token length | PENDING | — | — |
| P1-003 | Case-sensitive role tag matching | PENDING | — | — |
| P1-004 | Agent name path traversal unvalidated | PENDING | — | — |
| P1-005 | String-level prefix matching false positives | PENDING | — | — |
| P1-006 | Per-message filesystem I/O for project loading | PENDING | — | — |
| P1-007 | O(n²) message cloning in agentic loops | PENDING | — | — |
| P1-008 | Missing tests for auth module | PENDING | — | — |
| P1-009 | Landlock restrictions skip non-existent paths | PENDING | — | — |
| P1-010 | selfcheck.rs no timeout on HTTP request | PENDING | — | — |
| P1-011 | Repository URL inconsistency | PENDING | — | — |
| P1-012 | Blocking I/O in async runtime | PENDING | — | — |
| P1-013 | Telegram send_text swallows errors | PENDING | — | — |
| P1-014 | WhatsApp Mutex held during retry_send | PENDING | — | — |
| P1-015 | WhatsApp SQLite missing WAL mode | PENDING | — | — |

### P2: Minor
| ID | Title | Status | Test File | Fix Commit |
|----|-------|--------|-----------|------------|
| P2-SEC-001 | Override phrase detection bypassable | PENDING | — | — |
| P2-SEC-002 | No path traversal validation in skill loading | PENDING | — | — |
| P2-SEC-003 | MCP command field trusted without validation | PENDING | — | — |
| P2-PERF-001 | WhatsApp store INSERTs without transaction | PENDING | — | — |
| P2-PERF-002 | WhatsApp store INSERTs without transaction (skdm) | PENDING | — | — |
| P2-PERF-003 | New reqwest::Client per voice message | PENDING | — | — |
| P2-PERF-004 | DESC + reverse instead of ASC subquery | PENDING | — | — |
| P2-PERF-005 | 7 sequential DB queries could be parallelized | PENDING | — | — |
| P2-DEBT-001 | migrate_layout blocking I/O | PENDING | — | — |
| P2-DEBT-002 | install_bundled_prompts blocking I/O | PENDING | — | — |
| P2-DEBT-003 | config::load blocking I/O | PENDING | — | — |
| P2-DEBT-004 | send_photo_bytes swallows errors | PENDING | — | — |
| P2-DEBT-005 | split_message duplicated | PENDING | — | — |
| P2-DEBT-006 | download_telegram_file no size limit | PENDING | — | — |
| P2-DEBT-007 | format! for SQL offset | PENDING | — | — |
| P2-DEBT-008 | get_due_tasks returns 8-element tuple | PENDING | — | — |
| P2-DEBT-009 | build_system_prompt has 9 parameters | PENDING | — | — |
| P2-DEBT-010 | Gateway::new takes 14 parameters | PENDING | — | — |
| P2-DEBT-011 | println in non-interactive init | PENDING | — | — |
| P2-DEBT-012 | TODO comments in builds modules | PENDING | — | — |
| P2-DEAD-001 | Dead code in routing.rs | PENDING | — | — |
| P2-DEAD-002 | #[allow(dead_code)] on unused fields | PENDING | — | — |
| P2-TEST-001 | No tests for OmegaError | PENDING | — | — |
| P2-TEST-002 | No tests for message types | PENDING | — | — |
| P2-TEST-003 | Provider factory has no tests | PENDING | — | — |
| P2-TEST-004 | Pipeline has no direct tests | PENDING | — | — |
| P2-TEST-005 | Marker processing orchestration untested | PENDING | — | — |
| P2-TEST-006 | AuditLogger::log() has no test | PENDING | — | — |
| P2-COMP-001 | Stale .gitignore entry | PENDING | — | — |
| P2-COMP-002 | unsafe not in CLAUDE.md exemptions | PENDING | — | — |

### P3: Suggestions
| ID | Title | Status |
|----|-------|--------|
| P3-001 | Wrap data in Arc | SKIPPED |
| P3-002 | Add spec files for undocumented modules | SKIPPED |
| P3-003 | Update gateway spec file count | SKIPPED |
| P3-004 | Update config spec for WhatsApp fields | SKIPPED |
| P3-005 | Replace which subprocess with pure-Rust | SKIPPED |
| P3-006 | match_skill_triggers clones | SKIPPED |
| P3-007 | expand_tilde bare ~ handling | SKIPPED |
| P3-008 | README license mismatch | SKIPPED |
| P3-009 | Hardcoded 120s timeout | SKIPPED |
| P3-010 | Double JSON parsing in claude_code | SKIPPED |
| P3-011 | WhatsApp store zero test coverage | SKIPPED |
| P3-012 | MCP client no integration test | SKIPPED |
