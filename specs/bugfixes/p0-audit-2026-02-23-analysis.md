# Bugfix Analysis: P0 Audit Findings (2026-02-23)

## Bug 1: UTF-8 Byte-Slicing Panics

**Root cause:** All 5 locations compute string slice boundaries using `s.len()` (byte count) and index with `&s[..n]`. When `n` falls inside a multi-byte UTF-8 character, Rust panics.

**Affected files:**
- `crates/omega-providers/src/tools.rs:352` -- `truncate_output()` uses `&s[..max_chars]`
- `crates/omega-memory/src/audit.rs:91` -- `truncate()` uses `&s[..max]`
- `crates/omega-memory/src/store/context.rs:332` -- inline `&content[..200]`
- `crates/omega-channels/src/telegram/send.rs:161-170` -- `split_message()` byte arithmetic
- `crates/omega-channels/src/whatsapp/send.rs:156` -- `split_message()` byte arithmetic (found during review)

**Fix:** Use `str::floor_char_boundary()` (stable Rust 1.82+) at all byte-computed slice points.

**Status:** All 5 locations fixed. 10 multi-byte tests added (2 per truncation site, 2 per split_message).

## Bug 2: No HTTP Timeout on Provider Clients

**Root cause:** All 5 HTTP providers use `reqwest::Client::new()` which has no timeout. A hung API call blocks the tokio task indefinitely.

**Affected files:**
- `crates/omega-providers/src/openai.rs:41`
- `crates/omega-providers/src/ollama.rs:30`
- `crates/omega-providers/src/anthropic.rs:33`
- `crates/omega-providers/src/openrouter.rs:34`
- `crates/omega-providers/src/gemini.rs:32`

**Fix:** Replace `Client::new()` with `Client::builder().timeout(Duration::from_secs(120)).build()` -- using `expect()` since builder only fails if TLS backend is missing (compile-time dependency).

**Existing tests:** Provider tests exist in `claude_code/tests.rs` but HTTP providers have no timeout-specific tests.

## Bug 3: FTS5 Query Syntax Injection

**Root cause:** User-derived text is bound directly to `MATCH ?` in FTS5 query. FTS5 operators (`AND`, `OR`, `NOT`, `NEAR`, `*`, `"`, parentheses) in user messages cause parse errors or unexpected query behavior.

**Affected file:** `crates/omega-memory/src/store/messages.rs:77`

**Fix:** Wrap query in double quotes and escape internal double quotes: `format!("\"{}\"", query.replace('"', "\"\""))`. This forces FTS5 to treat the input as a phrase literal.

**Existing tests:** `store/tests.rs` has no FTS5-specific tests.
