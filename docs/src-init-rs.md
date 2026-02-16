# src/init.rs — Init Wizard Documentation

## Overview

The init wizard is Omega's **first impression and first-time user experience**. It's a 2-minute interactive setup that transforms Omega from an uninitialized Rust project into a working personal AI agent connected to Telegram.

The init wizard solves a critical problem: **new users need a frictionless path from "I cloned this repo" to "my bot works."** Without it, new users would face:
- Manual directory creation
- Manual config file writing (or copying example)
- Uncertainty about required credentials
- Risk of misconfiguration

The wizard eliminates these friction points through guided, interactive setup.

---

## What is `omega init`?

`omega init` is the **onboarding command** for Omega. It's the first thing a new user runs after cloning the repository.

### Command Usage
```bash
omega init
```

### Execution Context
- **Requires:** Rust environment (user has already built/run Omega)
- **Runs:** Synchronously in the terminal where user types
- **Duration:** 1–2 minutes including user input time
- **Output:** Visual prompts, confirmations, and next-step instructions
- **Side Effects:** Creates `~/.omega/`, generates `config.toml`, validates Claude CLI

### Success Criteria
User can run `omega start` immediately after and have a working bot.

---

## The Onboarding Experience: Step-by-Step

### Step 1: Welcome (Instant)

**What the User Sees:**
```
  Omega — Setup Wizard
  ====================
```

**What's Happening:**
The wizard prints a visual banner to signal that the interactive setup phase has begun. This is important because Omega is command-line based; users need explicit visual cues that something is happening.

**User Action:** None. Just read the banner.

**Time:** < 1 second

---

### Step 2: Create Data Directory (< 1 second)

**What the User Sees (Success):**
```
  Created ~/.omega
```

**What the User Sees (If Already Exists):**
```
  ~/.omega already exists
```

**What's Happening:**
The wizard creates `~/.omega`, a hidden directory in the user's home directory where Omega will store:
- SQLite database (`memory.db`) — conversation history and memory
- Log files (`omega.log`) — runtime logs
- Future state files (planned)

**Why It Matters:**
Without this directory, Omega can't persist data between sessions. Creating it upfront ensures the user won't see mysterious "directory not found" errors later.

**User Action:** None. The wizard creates this automatically.

**Time:** < 1 second

---

### Step 3: Validate Claude CLI (1–3 seconds)

**What the User Sees (Success):**
```
  Checking claude CLI... found
```

**What the User Sees (Failure):**
```
  Checking claude CLI... NOT FOUND

  Install claude CLI first:
    npm install -g @anthropic-ai/claude-code

  Then run 'omega init' again.
```

**What's Happening:**
The wizard runs `claude --version` to verify that the Claude Code CLI is installed and accessible. Claude Code is Omega's default AI backend, so it's **mandatory** for Omega to work.

**Why It Matters:**
If Claude CLI is missing, Omega cannot function. Rather than letting the user discover this later during `omega start`, the wizard fails fast with a helpful installation command.

**User Action:**
- If found: Proceed to next step
- If not found: User must install Claude CLI via npm, then re-run `omega init`

**Time:** 1–3 seconds (includes subprocess execution)

**Implementation Detail:**
The wizard uses `.unwrap_or(false)` to gracefully handle execution failures. If the `claude` command can't be found, the check fails safely without panicking, showing the user-friendly error message.

---

### Step 4: Telegram Bot Setup — Token Collection (< 1 minute)

**What the User Sees:**
```
  Telegram Bot Setup
  ------------------
  Create a bot with @BotFather on Telegram, then paste the token.

  Bot token: _
```

The user sees a prompt where `_` represents the blinking cursor waiting for input.

**What's Happening:**
The wizard prompts the user to provide a Telegram bot token. This token allows Omega to receive messages from Telegram users and send responses back.

**Where Does the Token Come From?**
New Telegram users who don't have a bot:
1. Open Telegram
2. Search for `@BotFather`
3. Send `/newbot`
4. Follow BotFather's prompts to name the bot
5. BotFather responds with a bot token
6. Copy/paste that token into this prompt

**User Action Options:**

**Option A: User Has Token Ready**
```
  Bot token: 123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11
```
User pastes token, presses Enter → Wizard stores token, proceeds

**Option B: User Doesn't Have Token Yet**
```
  Bot token:
```
User just presses Enter (leaves blank) → Wizard skips Telegram setup

```
  Skipping Telegram setup.
  You can add it later in config.toml.
```

**Why Skipping is OK:**
Telegram integration is powerful but not required. Users might want to:
- Test Omega locally first without connecting to Telegram
- Integrate with WhatsApp or other platforms instead
- Set up Telegram token manually later

The wizard's philosophy: **Don't block the user on optional features.**

**Time:** 30 seconds to 1 minute (includes user time to find/copy token)

---

### Step 5: Telegram User ID — Optional Allowlist (Optional, < 30 seconds)

**What the User Sees (Only if Token Was Provided):**
```
  Your Telegram user ID (send /start to @userinfobot to find it).
  Leave blank to allow all users.

  User ID: _
```

**What's Happening:**
If the user provided a Telegram token, the wizard optionally asks for their Telegram user ID. This enables **auth filtering**: only specified users can send messages to the bot.

**Two Scenarios:**

**Scenario 1: User Provides Their ID**
```
  User ID: 123456789
```
The wizard records this ID. Later, the bot will only respond to messages from this specific Telegram user. This is secure; the bot ignores everyone else.

**Scenario 2: User Leaves Blank**
```
  User ID:
```
The wizard records `None` (no ID). The bot accepts messages from any Telegram user who knows the bot token. This is useful for:
- Testing the bot locally without auth restrictions
- Shared bots or group deployments
- Later adding auth via manual config editing

**How to Find User ID:**
1. In Telegram, search for `@userinfobot`
2. Send `/start`
3. Bot responds with your user ID number
4. Copy/paste into this prompt

**Important:** This step is skipped entirely if the user didn't provide a bot token in the previous step. If Telegram is disabled, there's no reason to collect user IDs.

**Time:** Optional; 20–30 seconds if performed

---

### Step 6: Generate Configuration File (< 1 second)

**What the User Sees (Success):**
```
  Generated config.toml
```

**What the User Sees (If Config Already Exists):**
```
  config.toml already exists — skipping generation.
  Delete it and run 'omega init' again to regenerate.
```

**What's Happening:**
The wizard creates `config.toml`, the main configuration file that Omega reads on startup. The config file is **generated based on the user's inputs** (token, user ID, etc.).

**The Generated Config (Example)**
```toml
[omega]
name = "Omega"
data_dir = "~/.omega"
log_level = "info"

[auth]
enabled = true

[provider]
default = "claude-code"

[provider.claude-code]
enabled = true
max_turns = 10
allowed_tools = ["Bash", "Read", "Write", "Edit"]

[channel.telegram]
enabled = true
bot_token = "123456:ABC-DEF1234..."
allowed_users = [123456789]

[memory]
backend = "sqlite"
db_path = "~/.omega/memory.db"
max_context_messages = 50
```

**What Each Section Means:**

| Section | Purpose |
|---------|---------|
| `[omega]` | Global Omega settings (name, storage path, log level) |
| `[auth]` | Authentication enforcement (always enabled) |
| `[provider]` | Which AI backend to use (claude-code is default) |
| `[provider.claude-code]` | Claude Code specific settings (max turns, allowed tools) |
| `[channel.telegram]` | Telegram integration (token, allowed users) |
| `[memory]` | Conversation storage (SQLite database settings) |

**Why Config is Generated:**
Rather than making users manually edit a config template, the wizard generates a working config based on their choices. This eliminates errors like:
- Forgetting to change a placeholder value
- Invalid TOML syntax
- Mismatched credentials and allowed_users

**What Happens if Config Already Exists?**
The wizard skips generation to prevent overwriting a user's customized config. If the user wants a fresh config, they delete the old one and re-run `omega init`.

**Where is config.toml Located?**
Current working directory (typically the project root). The user should run `omega init` from the directory where they cloned the Omega repository.

**Time:** < 1 second (write operation)

---

### Step 7: Success Message and Next Steps (Instant)

**What the User Sees:**
```
  Setup Complete
  ==============

  Next steps:
    1. Review config.toml
    2. Run: omega start
    3. Send a message to your bot on Telegram
```

**What's Happening:**
The wizard has completed all setup steps successfully. It now provides explicit next actions to guide the user toward a working bot.

**What Should the User Do?**

**Step 1: Review config.toml**
The user should open `config.toml` in a text editor and:
- Verify the bot token and user ID are correct
- Adjust settings like `log_level` (change to `debug` for troubleshooting)
- Review allowed tools (Bash, Read, Write, Edit are safe defaults)

This step ensures the user understands what they just configured.

**Step 2: Run `omega start`**
```bash
omega start
```

This starts the Omega daemon. It will:
1. Load `config.toml`
2. Initialize the SQLite database
3. Connect to the Telegram bot API
4. Start listening for incoming Telegram messages
5. Log all activity to `~/.omega/omega.log`

**Step 3: Send a Message to the Bot**
In Telegram, find the bot you created (via @BotFather) and send it a message, e.g.:
```
Hello Omega, what time is it?
```

The bot will:
1. Receive your message
2. Check auth (verify your user ID matches)
3. Delegate to Claude Code CLI
4. Get Claude's reasoning response
5. Send response back to Telegram
6. Store the conversation in memory

**Time:** < 1 second (display)

---

## Complete First-Time User Journey

Here's what a new user experiences from start to finish:

```
User clones repo
       ↓
User reads README
       ↓
User runs: cargo build --release
       ↓
User runs: omega init
       ↓
[WIZARD BEGINS]
       ↓
1. Welcome banner displayed
2. ~/.omega directory created
3. claude CLI validated ✓
4. Telegram token collected (or skipped)
5. User ID collected (if token provided)
6. config.toml generated
7. Success message + next steps
       ↓
[WIZARD ENDS]
       ↓
User reviews config.toml
       ↓
User runs: omega start
       ↓
Bot is running and listening on Telegram
       ↓
User sends first message to bot
       ↓
Bot responds with Claude Code output
       ↓
✓ Success: Omega is working
```

**Total time:** 2–3 minutes (mostly user input time, not waiting)

---

## Why The Wizard Matters

### Without the Wizard
New users would face:
- Manual creation of `~/.omega` directory (confusion: "where should I put files?")
- Manual copy/edit of config file (risk of breaking TOML syntax)
- Manual lookup of how to create Telegram bot (external documentation required)
- Uncertainty: "Did I configure this right?"

**Result:** ~15–30 minutes to get a working bot, high risk of misconfiguration

### With the Wizard
New users get:
- Guided, interactive setup (clear prompts and instructions)
- Automatic directory and config generation (no manual file editing)
- Integrated help (links to @BotFather, @userinfobot)
- Fast validation (Claude CLI check, clear error messages)

**Result:** 2 minutes to get a working bot, low risk of misconfiguration

---

## Error Handling During Onboarding

### Error: Claude CLI Not Found
**User sees:**
```
  Checking claude CLI... NOT FOUND

  Install claude CLI first:
    npm install -g @anthropic-ai/claude-code

  Then run 'omega init' again.
```

**Why:** Claude Code is mandatory. Without it, Omega can't function.

**User action:** Install npm package, re-run `omega init`

---

### Error: I/O Failure (Rare)
**User sees:** Rust error message from anyhow (e.g., "Permission denied" or "Disk full")

**Why:** Filesystem error when creating directory or writing config

**User action:** Fix filesystem issue (permissions, disk space), re-run `omega init`

---

### Error: Invalid TOML Written (Shouldn't Happen)
If there's a bug in the template, `config.toml` will be invalid. User would discover this when running `omega start`.

**Prevention:** The TOML template is hard-coded in `init.rs` and tested. Template syntax is validated before shipping.

---

## Customizing Omega After Init

After the wizard, users can customize Omega by editing `config.toml`:

**Change log level for debugging:**
```toml
log_level = "debug"  # More verbose logging
```

**Restrict tools Claude Code can use:**
```toml
allowed_tools = ["Read", "Write"]  # Remove Bash and Edit
```

**Add more Telegram users:**
```toml
allowed_users = [123456789, 987654321]  # Multiple user IDs
```

**Switch to different provider (when available):**
```toml
[provider]
default = "anthropic"  # or "openai", "ollama", etc.
```

**Increase context window:**
```toml
max_context_messages = 100  # Remember more history
```

After editing, restart Omega:
```bash
omega stop   # If running
omega start  # Restart with new config
```

---

## Resetting Omega to Fresh State

If the user wants to start over:

```bash
# Stop Omega if running
omega stop

# Delete config and data
rm config.toml
rm -rf ~/.omega

# Re-run setup wizard
omega init

# Generate new config
omega start
```

---

## Related Commands

### `omega start`
Starts the Omega daemon after init is complete. Loads config, initializes database, connects to Telegram.

### `omega stop`
Stops the running Omega daemon. Gracefully shuts down all connections.

### `omega service`
(Separate from init) Registers Omega as a macOS LaunchAgent so it starts automatically on login. Not part of the init wizard; optional separate step.

### `omega ask`
(After Omega is running) Sends a message directly to Omega via CLI. Useful for testing without Telegram.

---

## Implementation Insights

### Why Not Auto-Detect Telegram Token?
The wizard could theoretically:
- Check environment variables for `TELEGRAM_BOT_TOKEN`
- Read from a `.env` file
- Use OS keychain

**Decision:** Explicit prompt instead because:
- Force user to verify they have correct token
- Prevent accidental use of wrong token
- Keep wizard self-contained (no external file dependencies)
- Clear audit trail of what user configured

### Why Allow Skipping Telegram?
Some use cases don't need Telegram:
- Local CLI-only usage: `omega ask "your question"`
- Integration with other platforms (planned)
- Testing without live bot

**Decision:** Make token optional because:
- Users can test Omega locally without Telegram complexity
- Add Telegram to config.toml manually later
- Reduces setup friction for non-Telegram use cases

### Why Store Token in Config File?
Concern: Bot token in plaintext is a security risk.

**Current approach:** Token stored in `config.toml` (plaintext)

**Future improvement:** Support environment variables:
```bash
export TELEGRAM_BOT_TOKEN="123456:ABC..."
omega start
```

Then config.toml would reference the env var instead of token directly.

---

## Troubleshooting Common Issues

### "Command 'omega' not found"
**Problem:** Binary not built or not in PATH

**Solution:**
```bash
cargo build --release
# Now omega binary is at ./target/release/omega
# Either add to PATH or use full path: ./target/release/omega init
```

### "claude CLI not found"
**Problem:** Claude Code CLI not installed

**Solution:**
```bash
npm install -g @anthropic-ai/claude-code
```

### "config.toml already exists"
**Problem:** User wants to re-run setup wizard but config exists

**Solution:**
```bash
rm config.toml
omega init  # Generates new config
```

### "Failed to create ~/.omega directory"
**Problem:** Permission denied or disk full

**Solution:**
```bash
# Check permissions on home directory
ls -la ~

# Check disk space
df -h

# Try creating manually
mkdir -p ~/.omega
omega init  # Retry
```

### "Invalid bot token" (error during `omega start`)
**Problem:** Token was mistyped or copied incorrectly

**Solution:**
1. Get correct token from @BotFather again
2. Edit `config.toml` and update `bot_token = "..."`
3. Restart: `omega stop && omega start`

---

## Design Philosophy

The init wizard embodies these principles:

### 1. **Guided, Not Opinionated**
The wizard guides users through necessary steps without forcing opinions on advanced customization. Users can edit `config.toml` afterward.

### 2. **Fail Fast, Fail Gracefully**
Critical dependencies (Claude CLI) are checked immediately with helpful error messages. Optional features (Telegram token) can be skipped.

### 3. **Minimize User Errors**
By generating config instead of asking users to edit templates, we eliminate syntax errors and misconfiguration.

### 4. **Transparency**
Every step is visible to the user. No hidden operations. Users know what the wizard created and where.

### 5. **Completeness**
After the wizard, the system is fully functional. No additional setup required; user can immediately use Omega.

---

## Metrics of Success

The init wizard is successful if:

1. **New User Can Get Working Bot in 2 Minutes** ✓ (Time target met)
2. **No Surprises or Errors** ✓ (Fast validation catches issues early)
3. **User Understands What Was Configured** ✓ (Explicit messages and next steps)
4. **User Can Customize Later** ✓ (Config.toml is documented and editableworking)
5. **Bad State is Recoverable** ✓ (User can delete and re-run)

---

## Conclusion

The init wizard is the **entry point to Omega**. It transforms a raw codebase into a working personal AI agent in 2 minutes. By combining interactive guidance, automatic generation, and fast validation, the wizard removes friction while maintaining clarity and user control.

For new users, `omega init` is the bridge between "I found an interesting project" and "I have a working bot."
