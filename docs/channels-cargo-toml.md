# omega-channels Cargo.toml -- Developer Guide

This document explains the build manifest for the `omega-channels` crate, which lives at `crates/omega-channels/Cargo.toml`. If you are adding a new messaging platform, modifying dependencies, or just trying to understand how the workspace fits together, this is the right place to start.

## What Does omega-channels Do?

The `omega-channels` crate provides integrations with messaging platforms (Telegram, WhatsApp, and any future platforms). It is responsible for receiving messages from these platforms, forwarding them through the gateway pipeline, and sending responses back.

## How Workspace Inheritance Works

You will notice that every dependency in this crate looks like this:

```toml
tokio = { workspace = true }
```

This means the version, features, and other settings are **not** declared here. Instead, they are declared once in the root `Cargo.toml` under `[workspace.dependencies]`:

```toml
[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
```

When you write `{ workspace = true }` in a crate, Cargo pulls in whatever version and features the workspace root defines. This gives us two important guarantees:

1. **Every crate in the workspace uses the same version** of a given dependency. No version conflicts.
2. **Upgrades happen in one place.** Bump the version in the root `Cargo.toml` and every crate picks it up.

Package metadata fields (`version`, `edition`, `license`, `repository`) are also inherited the same way:

```toml
[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"
repository = "https://github.com/omega-cortex/omega"
```

## Dependencies and What They Do

### Internal

| Dependency   | What It Provides                                                         |
|--------------|--------------------------------------------------------------------------|
| `omega-core` | Shared types (`Message`, `Config`, traits, error types, sanitization)    |

This is the only internal crate that `omega-channels` depends on. It does not depend on `omega-memory` or `omega-providers` directly -- the gateway in `src/gateway.rs` wires those together at a higher level.

### External

| Dependency    | Version | What It Is Used For                                                                                 |
|---------------|---------|-----------------------------------------------------------------------------------------------------|
| `tokio`       | 1       | The async runtime. All network I/O (polling Telegram, sending HTTP requests) runs on Tokio.         |
| `serde`       | 1       | Deriving `Serialize` and `Deserialize` on message structs and API response types.                   |
| `serde_json`  | 1       | Parsing JSON payloads from platform APIs (Telegram Bot API responses, webhook bodies, etc.).        |
| `tracing`     | 0.1     | Structured logging. The project uses `tracing` everywhere instead of `println!` or `log`.           |
| `thiserror`   | 2       | Defining typed error enums (e.g., `ChannelError`) with nice `Display` implementations.             |
| `anyhow`      | 1       | Quick error propagation with `?` in functions that do not need typed errors.                        |
| `async-trait` | 0.1     | Enables `async fn` in trait definitions. Used for the `Channel` trait that each platform implements.|
| `reqwest`     | 0.12    | HTTP client for calling platform APIs. Uses `rustls-tls` for TLS (no OpenSSL dependency).          |
| `uuid`        | 1       | Generating unique v4 UUIDs for messages and conversations.                                         |
| `chrono`      | 0.4     | Working with timestamps on messages (parsing, formatting, comparison).                              |

## How to Add a New Dependency

### Step 1: Add it to the workspace root

Open the root `Cargo.toml` and add the dependency under `[workspace.dependencies]`:

```toml
[workspace.dependencies]
# ... existing entries ...
my-new-crate = { version = "3.0", features = ["some-feature"] }
```

### Step 2: Reference it in the crate

Open `crates/omega-channels/Cargo.toml` and add:

```toml
[dependencies]
my-new-crate = { workspace = true }
```

That is it. The version and features come from the workspace definition.

### Step 3: If only this crate needs a feature

If `omega-channels` needs a feature that no other crate needs, you can add it locally without overriding the workspace version:

```toml
[dependencies]
my-new-crate = { workspace = true, features = ["extra-feature"] }
```

This merges `extra-feature` with whatever features the workspace already declares.

## How to Add a New Messaging Platform

When adding a new channel (e.g., Discord, Slack), you will typically:

1. Create a new module under `crates/omega-channels/src/` (e.g., `discord.rs`).
2. Implement the `Channel` trait from `omega-core` for your new platform.
3. Add any platform-specific dependencies following the two-step process above.
4. Register the new channel in the gateway (`src/gateway.rs`).

The existing Telegram implementation is a good reference for the pattern.

## Common Tasks

**Check that everything compiles:**

```bash
cargo check -p omega-channels
```

**Run clippy on just this crate:**

```bash
cargo clippy -p omega-channels
```

**See the full resolved dependency tree:**

```bash
cargo tree -p omega-channels
```

This shows every transitive dependency, which is useful for debugging version conflicts or understanding binary size.
