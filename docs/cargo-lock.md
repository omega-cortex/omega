# Understanding Cargo.lock

## What is Cargo.lock?

Cargo.lock is a file that records the exact versions of all dependencies (direct and transitive) used in a Rust project. It's automatically generated and maintained by Cargo, Rust's package manager and build system.

Think of it as a "snapshot" of your entire dependency tree at a specific point in time. When you build the Omega project, Cargo consults both `Cargo.toml` (which specifies version constraints) and `Cargo.lock` (which specifies exact versions) to determine what to build.

### Example
- **Cargo.toml** says: "I want tokio version 1.x.x"
- **Cargo.lock** says: "The exact version we tested with is tokio 1.49.0"

When you run `cargo build`, Cargo will use **1.49.0** specifically, not the latest 1.50.0 that might be available.

---

## Why Does Cargo.lock Matter?

### 1. **Reproducible Builds**

Reproducible builds are critical for software reliability. Without Cargo.lock, two developers running `cargo build` at different times might end up with different dependency versions:

- **Developer A** (Jan 2025): Builds with tokio 1.48.0, reqwest 0.12.25
- **Developer B** (Feb 2026): Builds with tokio 1.49.0, reqwest 0.12.28

These different versions might have subtle behavior differences, leading to:
- Different bugs in different environments
- "Works on my machine" syndrome
- Difficult-to-diagnose production issues

**Cargo.lock prevents this** by pinning exact versions.

### 2. **Consistent CI/CD Pipelines**

When Cargo.lock is committed to version control:
- GitHub Actions runners use the same dependency versions
- CI builds are byte-for-byte identical
- Release binaries are reproducible and auditable

### 3. **Debugging & Support**

When a user reports a bug, you know exactly what versions they're running with. With Cargo.lock:
- You can reproduce the exact environment
- You can bisect which dependency version caused the issue
- You can provide targeted patches

### 4. **Security**

Cargo.lock includes cryptographic checksums for each package:
```
checksum = "ddd31a130427c27518df266943a5308ed92d4b226cc639f5a8f1002816174301"
```

These checksums:
- Verify package integrity from crates.io
- Detect tampered or corrupted downloads
- Ensure you get exactly the code you reviewed

### 5. **Audit Trail**

Looking at the git history of Cargo.lock:
- See when each dependency was updated
- Understand why (via commit messages)
- Identify which versions were used in production at any time

---

## When is Cargo.lock Updated?

### Automatic Updates
Cargo.lock is automatically updated when:

1. **You run `cargo update`**
   ```bash
   cargo update              # Updates all within Cargo.toml constraints
   cargo update -p tokio     # Updates only tokio
   ```

2. **A new dependency is added**
   ```bash
   cargo add serde_yaml      # Adds package and updates Cargo.lock
   ```

3. **Dependency versions in Cargo.toml change**
   ```toml
   # Changed from "1" to "1.50"
   tokio = { version = "1.50", features = ["full"] }
   ```
   Running `cargo build` will update Cargo.lock.

### Manual Updates (Not Recommended)
- **Do not edit Cargo.lock directly.** It's auto-generated and manual edits will be overwritten.
- Always use `cargo` commands to manage dependencies.

---

## For Binary Crates vs. Libraries

### Binary Crates (like Omega)
**Should commit Cargo.lock to version control.** Binary crates are end products (executables) where you want reproducible builds. Omega is a binary crate, so Cargo.lock is committed in the repository.

### Library Crates
**Cargo.lock is often in .gitignore.** Library crates (packages consumed by other projects) use Cargo.toml version constraints. Downstream users generate their own Cargo.lock based on their dependency constraints.

**Omega's approach:** Commits Cargo.lock because it's a binary/application, not a library for others to depend on.

---

## Common Cargo.lock Workflows

### Scenario 1: A New Developer Clones the Project

```bash
git clone https://github.com/omega-cortex/omega.git
cd omega
cargo build
```

**What happens:**
1. Cargo reads `Cargo.lock`
2. Downloads exact versions specified in the lock file
3. Builds with identical dependencies to the rest of the team

### Scenario 2: Updating All Dependencies (Carefully)

```bash
cargo update
git diff backend/Cargo.lock
# Review what changed
git add backend/Cargo.lock backend/Cargo.toml
git commit -m "chore: update dependencies"
```

**Best practice:** Test after updating!
```bash
cargo test --workspace
cargo clippy --workspace
```

### Scenario 3: Updating a Specific Dependency

```bash
cargo update -p tokio
# Now tokio is updated within its version constraint (1.x.x)
# Other packages stay the same
```

### Scenario 4: Pinning to a Specific Version

```toml
# In Cargo.toml
tokio = "=1.49.0"  # Exact version only
```

Running `cargo build` will use 1.49.0 and update Cargo.lock accordingly.

---

## Reading Cargo.lock

The lock file is a TOML-based format. Here's a typical entry:

```toml
[[package]]
name = "tokio"
version = "1.49.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ddd31a130427c27518df266943a5308ed92d4b226cc639f5a8f1002816174301"
dependencies = [
  "bytes",
  "libc",
  "mio",
  "num_cpus",
  "parking_lot",
  "pin-project-lite",
  "signal-hook-registry",
  "socket2",
  "windows-sys 0.61.0",
]
```

Breaking this down:
- **name:** Package identifier
- **version:** Exact locked version
- **source:** Where the package comes from (crates.io)
- **checksum:** SHA-256 hash for integrity verification
- **dependencies:** Packages this depends on (their versions in lock file)

---

## Troubleshooting Cargo.lock Issues

### Issue 1: "Cargo.lock is out of sync with Cargo.toml"

**Cause:** Cargo.toml was modified but Cargo.lock wasn't updated.

**Fix:**
```bash
cargo update
```

### Issue 2: Merge Conflicts in Cargo.lock

**Cause:** Two branches updated dependencies differently.

**Fix:**
```bash
# Option 1: Resolve using theirs or ours, then regenerate
git checkout --theirs Cargo.lock
cargo update
git add Cargo.lock

# Option 2: Manual resolution (not recommended)
# Edit conflicts in Cargo.lock, then run:
cargo build  # Cargo will validate and regenerate if needed
```

### Issue 3: Different Behavior Between Local and CI

**Cause:** CI is using different dependency versions (Cargo.lock not committed or outdated).

**Fix:**
- Ensure Cargo.lock is committed to git
- Run `cargo update` before committing
- Check CI uses `cargo build` without `--offline` initially

---

## Best Practices for Omega

1. **Always commit Cargo.lock changes**
   ```bash
   git add Cargo.lock
   git commit -m "deps: update dependencies"
   ```

2. **Test after updating dependencies**
   ```bash
   cargo update
   cargo clippy --workspace
   cargo test --workspace
   cargo fmt --check
   ```

3. **Review dependency updates carefully**
   - Use `cargo update --dry-run` to preview changes
   - Check CHANGELOG.md of major dependencies
   - Look for security advisories: `cargo audit`

4. **Keep Cargo.lock updated**
   - Run `cargo update` periodically (monthly recommended)
   - Use `cargo outdated` to see what's outdated
   - Don't wait too long—too many updates at once is harder to debug

5. **Use exact version pins for critical dependencies**
   ```toml
   # In workspace.dependencies
   tokio = "1.49.0"  # Exact if this is critical
   ```

---

## Key Omega Dependencies (Locked)

Based on the current Cargo.lock:

| Dependency | Version | Purpose |
|-----------|---------|---------|
| tokio | 1.49.0 | Async runtime |
| serde | 1.0.228 | Serialization |
| sqlx | 0.8.6 | Database (SQLite) |
| tracing | 0.1.44 | Observability |
| reqwest | 0.12.28 | HTTP requests |
| clap | 4.5.58 | CLI argument parsing |
| chrono | 0.4.43 | Date/time |
| uuid | 1.21.0 | UUID generation |

These versions are carefully chosen to work together and have been tested as a group. Updating one might require updating others for compatibility.

---

## Further Reading

- [Cargo Book - Lock Files](https://doc.rust-lang.org/cargo/guide/cargo-lock.html)
- [Cargo Book - Dependency Resolution](https://doc.rust-lang.org/cargo/guide/dependency-resolution.html)
- [Reproducible Builds](https://reproducible-builds.org/)
- [Rust Security Advisory Database](https://github.com/rustsec/advisory-db)
