# SPECS.md — Omega Documentation Tracker

> Every file in the repository listed as a milestone.
> Check the box once the file has been fully documented.

---

## Milestone 1: Root / Workspace

- [x] `Cargo.toml`
- [x] `Cargo.lock`
- [x] `CLAUDE.md`
- [x] `README.md`
- [x] `LICENSE`
- [x] `config.example.toml`
- [x] `.gitignore`
- [x] `.claude/settings.local.json`

---

## Milestone 2: Binary (`src/`)

- [x] `src/main.rs`
- [x] `src/gateway.rs`
- [x] `src/commands.rs`
- [x] `src/init.rs`
- [x] `src/selfcheck.rs`

---

## Milestone 3: omega-core (`crates/omega-core/`)

- [x] `crates/omega-core/Cargo.toml`
- [x] `crates/omega-core/src/lib.rs`
- [x] `crates/omega-core/src/config.rs`
- [x] `crates/omega-core/src/context.rs`
- [x] `crates/omega-core/src/error.rs`
- [x] `crates/omega-core/src/message.rs`
- [x] `crates/omega-core/src/sanitize.rs`
- [x] `crates/omega-core/src/traits.rs`

---

## Milestone 4: omega-providers (`crates/omega-providers/`)

- [x] `crates/omega-providers/Cargo.toml`
- [x] `crates/omega-providers/src/lib.rs`
- [x] `crates/omega-providers/src/claude_code.rs`
- [x] `crates/omega-providers/src/anthropic.rs`
- [x] `crates/omega-providers/src/openai.rs`
- [x] `crates/omega-providers/src/ollama.rs`
- [x] `crates/omega-providers/src/openrouter.rs`

---

## Milestone 5: omega-channels (`crates/omega-channels/`)

- [x] `crates/omega-channels/Cargo.toml`
- [x] `crates/omega-channels/src/lib.rs`
- [x] `crates/omega-channels/src/telegram.rs`
- [x] `crates/omega-channels/src/whatsapp.rs`

---

## Milestone 6: omega-memory (`crates/omega-memory/`)

- [x] `crates/omega-memory/Cargo.toml`
- [x] `crates/omega-memory/src/lib.rs`
- [x] `crates/omega-memory/src/store.rs`
- [x] `crates/omega-memory/src/audit.rs`
- [x] `crates/omega-memory/migrations/001_init.sql`
- [x] `crates/omega-memory/migrations/002_audit_log.sql`
- [x] `crates/omega-memory/migrations/003_memory_enhancement.sql`

---

## Milestone 7: omega-skills (`crates/omega-skills/`)

- [x] `crates/omega-skills/Cargo.toml`
- [x] `crates/omega-skills/src/lib.rs`
- [x] `crates/omega-skills/src/builtin/mod.rs`

---

## Milestone 8: omega-sandbox (`crates/omega-sandbox/`)

- [x] `crates/omega-sandbox/Cargo.toml`
- [x] `crates/omega-sandbox/src/lib.rs`

---

## Progress

| Milestone | Files | Documented |
|-----------|-------|------------|
| 1. Root / Workspace | 8 | 8 |
| 2. Binary (`src/`) | 5 | 5 |
| 3. omega-core | 8 | 8 |
| 4. omega-providers | 7 | 7 |
| 5. omega-channels | 4 | 4 |
| 6. omega-memory | 7 | 7 |
| 7. omega-skills | 3 | 3 |
| 8. omega-sandbox | 2 | 2 |
| **Total** | **44** | **44** |
