# README.md Specification

## File Location and Purpose

**File Path:** `/Users/isudoajl/ownCloud/Projects/omega/README.md`

**Purpose:** Primary project documentation and entry point for the Omega repository. Serves as the first point of contact for users and contributors, providing a high-level overview of the project's capabilities, architecture, setup instructions, and development guidelines.

**Target Audience:** New users, contributors, and anyone exploring the Omega project on GitHub.

---

## Sections Breakdown

### 1. Title and Tagline (Lines 1-5)
- **Content:**
  - Main title: "Omega"
  - Tagline: "Your AI, your server, your rules."
  - Description: One-sentence summary of what Omega does
- **Purpose:** Immediately communicates the project's core value proposition
- **Key Message:** Omega is a locally-running personal AI agent with zero cloud dependency

### 2. What Makes Omega Different (Lines 7-13)
- **Content:** Five bullet points highlighting differentiators
  - Local execution with privacy preservation
  - Persistent memory and learning across sessions
  - Zero-config AI using local claude CLI authentication
  - Action-oriented capabilities (not just conversational)
  - Rapid 2-minute setup via `omega init`
- **Purpose:** Establishes Omega's competitive advantages and reasons to use it
- **Audience:** Users evaluating whether Omega fits their needs

### 3. Quick Start (Lines 15-33)
- **Content:** Two setup paths
  - Automated: `cargo build --release` → `./backend/target/release/omega init` → `./backend/target/release/omega start`
  - Manual: Copy and edit `config.example.toml` → run binary
- **Purpose:** Enables users to get started in minimal time
- **Structure:** Bash code blocks with step-by-step instructions
- **Assumption:** Users have Rust toolchain installed

### 4. How It Works (Lines 35-55)
- **Content:**
  - ASCII diagram showing message flow through gateway, memory, and audit log
  - 7-step process pipeline (Auth → Sanitize → Memory → Provider → Store → Audit → Respond)
  - Conversation lifecycle: 30+ minute idle timeout triggers automatic summarization
- **Purpose:** Explains the internal architecture and request handling flow
- **Key Concept:** Every interaction is authenticated, sanitized, contextualized, processed, logged, and audited

### 5. Commands (Lines 57-68)
- **Content:** Command reference table with 6 built-in commands
  - `/status`: System information and uptime
  - `/memory`: Conversation and fact statistics
  - `/history`: Recent conversation summaries
  - `/facts`: Learned facts about the user
  - `/forget`: Clear current conversation
  - `/help`: Command listing
- **Purpose:** Documents user-facing bot commands available via Telegram
- **Note:** Commands are instant (no AI processing); everything else delegates to provider

### 6. Requirements (Lines 70-74)
- **Content:** Minimal dependencies
  - Rust 1.70+
  - `claude` CLI (installed and authenticated)
  - Telegram bot token from @BotFather
- **Purpose:** Clear prerequisite checklist before installation
- **Link:** References Telegram's @BotFather bot for token generation

### 7. Configuration (Lines 76-102)
- **Content:** Sample `config.toml` with all major sections
  - `[omega]`: Basic naming
  - `[auth]`: Authentication enablement flag
  - `[provider]`: Default provider selection and claude-code specific settings (max_turns, allowed_tools)
  - `[channel.telegram]`: Bot token and allowed user IDs
  - `[memory]`: Database path and context window settings
- **Purpose:** Demonstrates configuration file structure and key options
- **Note:** Indicates `config.toml` is gitignored; `config.example.toml` is the template
- **Security:** Shows allowed_tools whitelist and per-user auth

### 8. Architecture (Lines 104-115)
- **Content:** Cargo workspace composition with 6 crates
  - `omega-core`: Core types, traits, config, error handling, sanitization
  - `omega-providers`: AI backend integrations (Claude Code CLI implemented; others planned)
  - `omega-channels`: Messaging platform adapters (Telegram implemented; WhatsApp planned)
  - `omega-memory`: SQLite-backed storage for conversations, facts, audit log
  - `omega-skills`: Plugin/skill system (planned)
  - `omega-sandbox`: Secure command execution (planned)
- **Purpose:** High-level view of codebase organization and responsibility boundaries
- **Status Indicator:** Distinguishes implemented vs. planned components

### 9. macOS Service (Lines 117-124)
- **Content:** LaunchAgent setup instructions for persistent background execution
  - Copy plist file to `~/Library/LaunchAgents/`
  - Load with `launchctl`
- **Purpose:** Enables Omega to run as a system service on macOS
- **Platform:** macOS-specific feature
- **File Referenced:** `com.omega-cortex.omega.plist`

### 10. Development (Lines 126-133)
- **Content:** Quality assurance and build workflow
  - `cargo clippy --workspace`: Linting (zero warnings required)
  - `cargo test --workspace`: Full test suite
  - `cargo fmt --check`: Code formatting validation
  - `cargo build --release`: Optimized production binary
- **Purpose:** Documents development best practices and pre-commit checks
- **Audience:** Contributors and developers
- **Standards:** Enforces code quality and consistency

### 11. License (Lines 135-138)
- **Content:** MIT license declaration
- **Purpose:** Legal framework for open-source distribution

---

## Key Information Conveyed

### Value Proposition
1. **Privacy-First:** Messages stay local (except provider API calls)
2. **Stateful AI:** Conversation history and learned facts create continuity
3. **Low Friction:** No API key management; leverages existing `claude` CLI
4. **Practical:** Can execute actions, not just provide information
5. **Fast Setup:** Automated initialization wizard

### Technical Architecture
- **Single Binary:** Compiled Rust executable with no external dependencies beyond `claude` CLI
- **Message Pipeline:** Standardized flow with authentication, sanitization, memory augmentation, provider delegation, and audit logging
- **Multi-Channel:** Messaging platform adapter pattern allows Telegram + future platforms
- **Persistent Storage:** SQLite for all data (conversations, facts, audit trail)
- **Modular Design:** Six-crate workspace for separation of concerns

### User Experience
- **Interactive Setup:** `omega init` command streamlines configuration
- **Instant Commands:** Built-in bot commands for system introspection and memory management
- **Conversation Continuity:** Automatic summarization preserves context across sessions
- **Service Integration:** LaunchAgent support for macOS persistent execution

### Security and Reliability
- **Per-User Auth:** Telegram user ID whitelist prevents unauthorized access
- **Prompt Injection Prevention:** Sanitization layer neutralizes malicious patterns
- **Audit Trail:** Complete logging of all interactions
- **Tool Allowlisting:** Claude Code provider limited to approved tools

### Development Standards
- **Zero Warnings Policy:** All clippy lints must pass
- **Comprehensive Testing:** Full test suite required
- **Code Formatting:** Standardized format via `cargo fmt`
- **Optimization:** Release builds for production deployment

---

## Installation/Setup Instructions Referenced

### Build from Source
```bash
cargo build --release
```
- Compiles optimized binary to `backend/target/release/omega`

### Automated Setup
```bash
./backend/target/release/omega init
```
- Interactive wizard for:
  - Generating/configuring bot token
  - Setting Telegram user ID allowlist
  - Configuring provider and memory settings
  - Creating LaunchAgent plist (macOS)

### Manual Setup
```bash
cp config.example.toml config.toml
# Edit config.toml with your values
```
- Direct configuration file approach
- Users must manually set Telegram user ID and bot token

### Run Omega
```bash
./backend/target/release/omega start
```
- Starts the gateway event loop
- Connects to Telegram, begins processing messages

### macOS Service Setup
```bash
cp com.omega-cortex.omega.plist ~/Library/LaunchAgents/
launchctl load ~/Library/LaunchAgents/com.omega-cortex.omega.plist
```
- Registers Omega as persistent LaunchAgent
- Auto-starts on login

### Development Workflow
```bash
cargo clippy --workspace      # Lint check
cargo test --workspace        # Run tests
cargo fmt --check             # Format validation
cargo build --release         # Production build
```

---

## Badges, Links, and External References

### External Services Referenced
1. **@BotFather** (`https://t.me/BotFather`)
   - Telegram bot for generating bot tokens
   - User must interact with this to obtain Telegram bot credentials

### Repository and Tools
- **Cargo:** Rust package manager and build system
- **Rust:** Language version requirement 1.70+
- **Claude CLI:** Local authentication mechanism (must be pre-installed)
- **Telegram Bot API:** Messaging platform

### Files Referenced
- `config.example.toml`: Template configuration file
- `config.toml`: User configuration (gitignored)
- `com.omega-cortex.omega.plist`: macOS LaunchAgent manifest
- `backend/target/release/omega`: Compiled binary

### License
- **MIT License:** Permissive open-source license (MIT)

---

## Design Philosophy Communicated

The README emphasizes several core design principles:

1. **Simplicity First:** One binary, minimal setup, standard configuration format
2. **Privacy by Default:** No cloud lock-in or external dependencies beyond AI provider
3. **Developer-Friendly:** Rust language, cargo ecosystem, clear architecture documentation
4. **Extensible:** Modular six-crate design supports future platforms and features
5. **Production-Ready:** Service integration, audit logging, error handling
6. **Transparency:** Complete documentation of message flow and security measures

---

## Target User Workflows

### New User
1. Read title and "What Makes Omega Different"
2. Follow Quick Start (automated setup via `omega init`)
3. Reference Commands for available operations
4. Explore Configuration section for customization

### Developer
1. Review Architecture for codebase understanding
2. Follow Development workflow for contribution process
3. Reference Requirements and specific crate purposes

### System Administrator
1. Review Requirements and Configuration
2. Follow macOS Service section for deployment
3. Consult Architecture for system resource requirements

### Telegram User
1. Reference Commands table for available operations
2. Understand How It Works for privacy/security assurance
3. Check Requirements if troubleshooting authentication
