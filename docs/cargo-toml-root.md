# Understanding the Root Cargo.toml

## What Is This File?

The `Cargo.toml` file in the root of the Omega project is the **workspace manifest**. It's like the master configuration that tells Rust's package manager (`cargo`) about your entire project structure and how all the pieces fit together.

Think of it as a construction blueprint—it defines what you're building, what materials (dependencies) you need, and how all the sub-projects (crates) are organized.

## Why Does It Matter?

### Single Source of Truth for Dependencies
Instead of each of the 6 crates specifying its own version of `tokio`, `serde`, or `sqlx`, they all reference the same version declared here. This means:
- **Consistency:** Everyone uses the same library versions
- **Easier Updates:** Change a version once, and it updates everywhere
- **Reduced Conflicts:** No accidental version mismatches between crates

### Workspace Organization
The file declares that your project is a **workspace** containing 6 independent crates:
- `omega-core`
- `omega-providers`
- `omega-channels`
- `omega-memory`
- `omega-skills`
- `omega-sandbox`

Plus the root `omega` binary that ties everything together.

### Shared Settings
Version, license, and repository information are defined once here and inherited by all member crates. This keeps metadata consistent without duplication.

## Key Sections Explained

### Workspace Members
```toml
[workspace]
members = ["crates/*"]
```
This glob pattern means "all directories under `crates/` are part of this workspace." Cargo automatically discovers them.

### Shared Package Metadata
```toml
[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"
repository = "https://github.com/omega-cortex/omega"
```
Every crate in the workspace inherits these values, ensuring they stay synchronized.

### Workspace Dependencies
```toml
[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
# ... more dependencies
```
These are the third-party libraries (from crates.io) that the project needs. They're declared once here and referenced by individual crates.

### Root Binary Dependencies
```toml
[dependencies]
omega-core = { workspace = true }
omega-providers = { workspace = true }
# ... internal crates
tokio = { workspace = true }
# ... external dependencies
```
The root `omega` binary depends on all internal crates and selected external libraries. The `workspace = true` notation means "use the version declared in `[workspace.dependencies]`."

## How to Modify It

### Adding a New Dependency

1. **Add it to `[workspace.dependencies]`:**
   ```toml
   [workspace.dependencies]
   my-library = "1.2"
   ```

2. **Use it in a crate's `Cargo.toml`:**
   ```toml
   [dependencies]
   my-library = { workspace = true }
   ```

3. **Or, if only the root binary needs it:**
   ```toml
   [dependencies]
   my-library = "1.2"
   ```

### Updating a Dependency Version

Change the version in `[workspace.dependencies]`:
```toml
[workspace.dependencies]
tokio = { version = "1.40", features = ["full"] }  # was "1"
```

All crates automatically use the new version on next `cargo build`.

### Adding a New Crate

1. Create a new directory under `crates/`: `mkdir crates/omega-newcrate`
2. The workspace automatically discovers it (because of `members = ["crates/*"]`)
3. No need to edit `Cargo.toml` unless you want custom settings

### Feature Toggles

Some dependencies have **features** that can be enabled or disabled:
```toml
tokio = { version = "1", features = ["full"] }  # Enable all tokio features
```

The word inside the `features` list activates optional functionality. For `tokio`, `"full"` means "enable everything." For others, you might see:
```toml
serde = { version = "1", features = ["derive"] }  # Only enable derive macros
```

## Notable Design Choices

### Why `rustls-tls` Instead of OpenSSL?
```toml
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
```
`rustls` is a pure-Rust TLS library. It's more secure and has fewer external dependencies than OpenSSL. This is a security-conscious choice.

### Why SQLite Specifically?
The workspace pulls in `sqlx` with SQLite support:
```toml
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite"] }
```
SQLite is embedded, serverless, and supports async operations. Perfect for a personal agent that stores data locally without external database infrastructure.

### Why "Full" Tokio Features?
```toml
tokio = { version = "1", features = ["full"] }
```
During development, it's easier to enable everything and avoid feature resolution issues. Production builds can be optimized later to disable unused features.

## Common Tasks

### Check for Outdated Dependencies
```bash
cargo outdated
```
(requires `cargo-outdated` plugin)

### Update All Dependencies to Latest Minor Versions
```bash
cargo update
```

### Check for Security Vulnerabilities
```bash
cargo audit
```

### Verify All Dependencies Compile
```bash
cargo check --workspace
```

## Related Files

- **Individual Crate Manifests:** Each crate under `crates/*/Cargo.toml` has its own manifest that references workspace dependencies and defines crate-specific settings.
- **Cargo.lock:** Auto-generated file that locks exact versions for reproducible builds. Commit this if building a binary, ignore it for libraries.
- **`config.example.toml`:** Application configuration (different from Cargo.toml). This is where Omega's runtime settings live.

## Troubleshooting

### "cannot find ... in this workspace"
The workspace might not have discovered a new crate. Try:
```bash
cargo check --workspace
```
If it still fails, ensure the new crate is under `crates/` and has its own `Cargo.toml`.

### "version conflict for package..."
Two crates are trying to use different versions of the same dependency. Ensure they both reference `{ workspace = true }`.

### Build is slow
You might have too many features enabled. Check `[workspace.dependencies]` and disable unused ones.

## Summary

The root `Cargo.toml` is the organizational backbone of Omega:
- It declares a **6-crate workspace**
- It defines **shared dependencies and versions**
- It specifies **shared metadata** (license, repository, version)
- It defines the main **`omega` binary** that orchestrates everything

Keep it clean, document major changes, and remember: consistency across all crates starts here.
