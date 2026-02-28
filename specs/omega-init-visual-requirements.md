# Requirements: OMEGA Init Wizard Visual Identity

## Scope

### Files Modified
- `backend/src/init.rs` — 14 chrome call sites replaced with branded equivalents
- `backend/src/init_wizard.rs` — 19 chrome call sites replaced with branded equivalents
- `backend/src/main.rs` — add `mod init_style;` declaration (1 line)

### Files Created
- `backend/src/init_style.rs` — NEW module: reusable branded output helpers using `console::Style`

### Explicitly NOT in scope
- cliclack interactive widgets (`input()`, `select()`, `confirm()`, `spinner()`) — stay as-is in both files
- `run_noninteractive()` path in init.rs — uses plain `println!`, meant for CI
- Other CLI files (`main.rs` status chrome, `service.rs`, `selfcheck.rs`, `pair.rs`) — accepted as tech debt
- No new crate dependencies — `console` v0.15 already in Cargo.toml
- No changes to wizard functional flow, step order, or logic

## Summary
Replace the generic cliclack "chrome" (intro banners, status lines, info/warning/error logs, boxed notes, outro) in the init wizard with custom OMEGA-branded styled output. The interactive widgets (text inputs, yes/no prompts, spinners, dropdowns) stay exactly as they are — only the surrounding decoration changes. A small helper module (`init_style.rs`) provides the reusable styled functions so both `init.rs` and `init_wizard.rs` share the same visual identity.

## User Stories
- As a new OMEGA user, I want the init wizard to have a distinctive visual identity so that it feels like a premium, purpose-built tool rather than a generic CLI template.
- As the OMEGA maintainer, I want the branded chrome helpers in a single shared module so that visual consistency is maintained without code duplication.
- As a new OMEGA user, I want the wizard to feel fast and confident so that the setup experience conveys authority, not decoration.

## Chrome Call Inventory

### init.rs — 14 call sites to replace

| # | Line(s) | Current Call | Context |
|---|---------|-------------|---------|
| 1 | 33 | `typewrite(LOGO, 2)` | Logo animation (replace with instant print) |
| 2 | 35 | `typewrite("omega init\n", 30)` | Header animation (remove — replaced by branded intro) |
| 3 | 36 | `cliclack::intro("omega init")` | Session banner |
| 4 | 42 | `cliclack::log::success(...)` | Data dir created |
| 5 | 44 | `cliclack::log::success(...)` | Data dir exists |
| 6 | 60-63 | `cliclack::note(...)` | Claude CLI missing instructions |
| 7 | 64 | `cliclack::outro_cancel(...)` | Abort session |
| 8 | 79 | `cliclack::log::info(...)` | Skipping Telegram |
| 9 | 100-103 | `cliclack::note(...)` | Whisper info box |
| 10 | 131-133 | `cliclack::log::warning(...)` | Config already exists |
| 11 | 144 | `cliclack::log::success(...)` | Config generated |
| 12 | 156-157 | `cliclack::log::warning(...)` + `cliclack::log::info(...)` | Service install failed + hint |
| 13 | 179 | `cliclack::note(...)` | Summary box |
| 14 | 181-182 | `cliclack::outro(...)` + typewrite | Session close |

### init_wizard.rs — 19 call sites to replace

| # | Line(s) | Current Call | Context |
|---|---------|-------------|---------|
| 1 | 88-91 | `cliclack::note(...)` | Setup-token instructions |
| 2 | 117 | `cliclack::log::warning(...)` | Auth later hint (stderr path) |
| 3 | 121 | `cliclack::log::warning(...)` | Auth later hint (exec error path) |
| 4 | 125 | `cliclack::log::success(...)` | Auth already configured |
| 5 | 143 | `cliclack::log::success(...)` | WhatsApp already paired |
| 6 | 155 | `cliclack::log::step(...)` | Starting WhatsApp pairing |
| 7 | 156 | `cliclack::log::info(...)` | Phone instructions |
| 8 | 170 | `cliclack::note(...)` | QR code display |
| 9 | 194 | `cliclack::log::warning(...)` | Try WhatsApp again later |
| 10 | 198 | `cliclack::log::error(...)` | WhatsApp pairing error |
| 11 | 229-236 | `cliclack::note(...)` | Google setup instructions |
| 12 | 270 | `cliclack::log::warning(...)` | Skipping Google (cred fail) |
| 13 | 275 | `cliclack::log::warning(...)` | Skipping Google (exec error) |
| 14 | 314 | `cliclack::log::warning(...)` | Incognito browser fail |
| 15 | 328-333 | `cliclack::note(...)` | OAuth troubleshooting |
| 16 | 364-366 | `cliclack::log::warning(...)` | OAuth fail, manual retry |
| 17 | 372 | `cliclack::log::warning(...)` | Google setup incomplete |
| 18 | 385 | `cliclack::log::success(...)` | Google connected |
| 19 | 391 | `cliclack::log::warning(...)` | Could not verify Google |

### init_style.rs — Required helper functions

| Helper | Replaces | Call Count |
|--------|----------|------------|
| `omega_intro(logo, subtitle)` | `cliclack::intro()` + typewrite logo | 1 |
| `omega_outro(message)` | `cliclack::outro()` | 1 |
| `omega_outro_cancel(message)` | `cliclack::outro_cancel()` | 1 |
| `omega_success(message)` | `cliclack::log::success()` | 6 |
| `omega_info(message)` | `cliclack::log::info()` | 3 |
| `omega_warning(message)` | `cliclack::log::warning()` | 10 |
| `omega_error(message)` | `cliclack::log::error()` | 1 |
| `omega_step(message)` | `cliclack::log::step()` | 1 |
| `omega_note(title, body)` | `cliclack::note()` | 7 |
| `typewrite(text, delay)` | (moved from init.rs) | 1 |
| **Total** | | **32** |

## Requirements

| ID | Requirement | Priority | Acceptance Criteria |
|----|------------|----------|-------------------|
| VIS-001 | Create `init_style.rs` helper module with branded output functions | Must | Module exists, uses `console::Style` only, `pub(crate)`, under 500 lines |
| VIS-002 | Branded intro sequence replacing `cliclack::intro` | Must | Logo prints instantly, styled header below, no double-rendering |
| VIS-003 | Branded success status line replacing `cliclack::log::success` | Must | All 6 calls replaced, visually distinguishable from cliclack |
| VIS-004 | Branded info status line replacing `cliclack::log::info` | Must | All 3 calls replaced |
| VIS-005 | Branded warning status line replacing `cliclack::log::warning` | Must | All 10 calls replaced, warning intent obvious |
| VIS-006 | Branded error status line replacing `cliclack::log::error` | Must | 1 call replaced, error intent obvious |
| VIS-007 | Branded step indicator replacing `cliclack::log::step` | Must | 1 call replaced |
| VIS-008 | Branded note/info box replacing `cliclack::note` | Must | All 7 calls replaced, multi-line + QR code renders correctly |
| VIS-009 | Branded outro replacing `cliclack::outro` and `cliclack::outro_cancel` | Must | Custom close, typewrite for "enjoy OMEGA" signature |
| VIS-010 | Logo prints instantly (no character animation) | Must | `typewrite(LOGO, 2)` removed, replaced with instant output |
| VIS-011 | `typewrite()` retained for outro signature only | Should | Only used for "enjoy OMEGA" phrase |
| VIS-012 | Dark/technical visual personality | Should | Bright accent color, no cliclack box-drawing chars |
| VIS-013 | Visual coexistence with cliclack interactive widgets | Should | No jarring transitions (manual testing) |
| VIS-014 | Both files use shared helpers from init_style.rs | Must | Zero remaining cliclack chrome calls in either file |
| VIS-015 | No file exceeds 500-line limit (excluding tests) | Must | All three files under 500 production lines |
| VIS-016 | Helper functions return `Result` | Should | Match existing `?` operator pattern |
| VIS-017 | Unit tests for init_style.rs helpers | Should | At least 5 tests covering helper types |
| VIS-018 | Existing tests continue to pass | Must | All 14 pre-existing tests pass unchanged |
| VIS-019 | Document tech debt for other CLI files | Could | Doc comment lists 4 deferred files |
| VIS-020 | No changes to wizard functional flow | Won't | N/A |
| VIS-021 | Brand other CLI files | Won't | N/A (46 calls deferred) |

## Impact Analysis

### Existing Code Affected
- `backend/src/init.rs`: 14 call sites modified — Risk: **low** (pure display changes)
- `backend/src/init_wizard.rs`: 19 call sites modified — Risk: **low** (pure display changes)
- `backend/src/main.rs`: 1 line added — Risk: **negligible**

### Regression Risk Areas
- QR code rendering inside custom note box (Unicode block characters)
- Spinner stop/error messages stay as cliclack-managed (NOT replaced)
- Terminal width (custom elements should work on 80+ column terminals)
- cliclack import cleanup (remove unused chrome imports)

## Traceability Matrix

| Requirement | Priority | Test IDs | Architecture | Implementation Module |
|-------------|----------|----------|-------------|----------------------|
| VIS-001 | Must | (compile-time: module exists) | Module: `init_style.rs` | `init_style` @ `backend/src/init_style.rs` |
| VIS-002 | Must | `test_omega_intro_returns_ok` | `omega_intro()` | `init_style::omega_intro` @ `backend/src/init_style.rs`, `init::run` @ `backend/src/init.rs` |
| VIS-003 | Must | `test_omega_success_returns_ok` | `omega_success()` | `init_style::omega_success` @ `backend/src/init_style.rs` |
| VIS-004 | Must | `test_omega_info_returns_ok` | `omega_info()` | `init_style::omega_info` @ `backend/src/init_style.rs` |
| VIS-005 | Must | `test_omega_warning_returns_ok` | `omega_warning()` | `init_style::omega_warning` @ `backend/src/init_style.rs` |
| VIS-006 | Must | `test_omega_error_returns_ok` | `omega_error()` | `init_style::omega_error` @ `backend/src/init_style.rs` |
| VIS-007 | Must | `test_omega_step_returns_ok` | `omega_step()` | `init_style::omega_step` @ `backend/src/init_style.rs` |
| VIS-008 | Must | `test_omega_note_returns_ok`, `test_omega_note_empty_body` | `omega_note()` | `init_style::omega_note` @ `backend/src/init_style.rs` |
| VIS-009 | Must | `test_omega_outro_returns_ok`, `test_omega_outro_cancel_returns_ok` | `omega_outro()`, `omega_outro_cancel()` | `init_style::omega_outro` @ `backend/src/init_style.rs`, `init::run` @ `backend/src/init.rs` |
| VIS-010 | Must | (caller-side: verified by VIS-002 test) | `omega_intro()` -- instant print | `init::run` @ `backend/src/init.rs` |
| VIS-011 | Should | `test_typewrite_does_not_panic` | `typewrite()` -- retained for outro | `init_style::typewrite` @ `backend/src/init_style.rs` |
| VIS-012 | Should | (visual: not testable in unit tests) | Color Palette -- cyan/gutter-bar | `init_style` @ `backend/src/init_style.rs` |
| VIS-013 | Should | N/A | Visual Coexistence -- flow mock | manual verification |
| VIS-014 | Must | (grep-verified: no cliclack chrome calls remain) | Integration Pattern | `init` @ `backend/src/init.rs`, `init_wizard` @ `backend/src/init_wizard.rs` |
| VIS-015 | Must | (line count check at review time) | Module Layout -- ~150 lines | `init_style` @ `backend/src/init_style.rs` |
| VIS-016 | Should | (type-checked: all helpers return `io::Result<()>`) | `io::Result<()>` return type | `init_style` @ `backend/src/init_style.rs` |
| VIS-017 | Should | all 11 tests in `init_style.rs` | Test Plan -- 11 tests | `init_style::tests` @ `backend/src/init_style.rs` |
| VIS-018 | Must | existing | N/A | existing suites (verified: 868 tests pass) |
| VIS-019 | Could | (doc comment in module header) | Tech Debt doc comment | `init_style` @ `backend/src/init_style.rs` |
