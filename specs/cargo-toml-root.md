# Cargo.toml (Root Workspace)

## Path
`Cargo.toml`

## Purpose
Root workspace manifest file for the Omega project. Defines a Cargo workspace containing 6 crates, establishes shared dependencies with explicit versions, and configures workspace-level settings that apply to all member crates. The root workspace also provides the main binary target (`omega`).

## Workspace Configuration
- **Resolver Version:** `2` (modern dependency resolution)
- **Members:** All crates under `crates/*` (glob pattern)

## Workspace-Level Package Settings
These settings are inherited by all member crates unless explicitly overridden:

| Setting | Value |
|---------|-------|
| Version | `0.1.0` |
| Edition | `2021` (Rust 2021 edition) |
| License | `MIT OR Apache-2.0` (dual-licensed) |
| Repository | `https://github.com/omega-cortex/omega` |

## Workspace Members
The following 6 crates are part of this workspace:

1. **`omega-core`** (`crates/omega-core`)
   - Core types, traits, configuration structures, error handling, and prompt sanitization

2. **`omega-providers`** (`crates/omega-providers`)
   - AI backend integrations (Claude Code CLI, Anthropic, OpenAI, Ollama, OpenRouter)

3. **`omega-channels`** (`crates/omega-channels`)
   - Messaging platform implementations (Telegram, WhatsApp)

4. **`omega-memory`** (`crates/omega-memory`)
   - SQLite-based storage layer, conversation history, and audit logging

5. **`omega-skills`** (`crates/omega-skills`)
   - Plugin/skill system (planned phase)

6. **`omega-sandbox`** (`crates/omega-sandbox`)
   - Secure command execution environment (planned phase)

## Workspace Dependencies
All dependencies are declared at workspace level for consistency and easier version management:

### Async Runtime & Concurrency
- **`tokio`** `1.x` - Async runtime with full feature set
  - Features: `full` (includes all tokio features)

- **`async-trait`** `0.1.x` - Async trait support for trait objects

### Serialization & Configuration
- **`serde`** `1.x` - Serialization/deserialization framework
  - Features: `derive` (macros for custom types)

- **`serde_json`** `1.x` - JSON support (no special features)

- **`toml`** `0.8.x` - TOML parsing and serialization (for config files)

### HTTP Client
- **`reqwest`** `0.12.x` - HTTP client library
  - Features: `json` (JSON support), `rustls-tls` (TLS via rustls, not openssl)

### Database
- **`sqlx`** `0.8.x` - Async SQL toolkit
  - Features: `runtime-tokio` (tokio integration), `sqlite` (SQLite driver)

### Logging & Tracing
- **`tracing`** `0.1.x` - Structured logging framework
- **`tracing-subscriber`** `0.3.x` - Tracing utilities and subscribers
  - Features: `env-filter` (environment-based filtering)

### Error Handling
- **`thiserror`** `2.x` - Derive macros for `std::error::Error`
- **`anyhow`** `1.x` - Flexible error handling with context

### CLI
- **`clap`** `4.x` - Command-line argument parsing
  - Features: `derive` (procedural macros for CLI definitions)

### Utilities
- **`uuid`** `1.x` - UUID generation
  - Features: `v4` (v4 UUIDs), `serde` (serialization support)

- **`chrono`** `0.4.x` - Date and time handling
  - Features: `serde` (serialization support)

### Platform-Specific
- **`libc`** `0.2.x` - Direct bindings to C library (used for `geteuid()` in root detection)

## Root Package Configuration

### Package Metadata
- **Name:** `omega`
- **Description:** "Personal AI agent infrastructure, forged in Rust"
- **Version, Edition, License, Repository:** Inherited from workspace settings

### Binary Target
- **Binary Name:** `omega`
- **Entry Point:** `src/main.rs`

## Root Package Dependencies
The root `omega` binary depends on all 6 internal crates and a curated selection of workspace dependencies:

**Internal Crates:**
- `omega-core`
- `omega-providers`
- `omega-channels`
- `omega-memory`
- `omega-skills`
- `omega-sandbox`

**External Dependencies:**
- `tokio` (async runtime for main event loop)
- `clap` (CLI argument parsing)
- `tracing` and `tracing-subscriber` (logging)
- `anyhow` (error handling)
- `serde_json` (JSON processing)
- `sqlx` (database access)
- `chrono` (timestamp handling)
- `reqwest` (HTTP requests)

## Notable Design Decisions

1. **Workspace Dependency Management:** All external dependencies are declared at the workspace level (`[workspace.dependencies]`), allowing member crates to reference them with `workspace = true`. This ensures version consistency across the project.

2. **TLS Configuration:** `reqwest` uses `rustls-tls` instead of the default OpenSSL, reducing dependencies and improving security posture.

3. **SQLite as Primary Storage:** `sqlx` is configured specifically for SQLite async runtime, reflecting the project's decision to use SQLite for all persistence (memory, audit logs, state).

4. **Full Tokio Features:** The workspace enables all tokio features (`features = ["full"]`) to avoid feature resolution issues during development.

5. **Async-First Design:** The presence of `tokio`, `async-trait`, and `sqlx` with async runtime reflects the architecture's commitment to fully async I/O operations.

6. **Dual Licensing:** MIT OR Apache-2.0 dual license allows flexibility for diverse use cases.

## Version Lock
The workspace uses specific major versions without pre-release specifiers (e.g., `1`, `2`, `0.1`, `0.2`), allowing patch and minor updates within those versions. This balances stability with access to bug fixes.
