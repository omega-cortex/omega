# Functionalities: CLI / Main Binary

## Overview

The main binary provides a CLI interface with 5 commands (Start, Status, Ask, Init, Pair) plus service management. Uses clap for argument parsing and cliclack for styled terminal output.

## Functionalities

| # | Name | Type | Location | Description | Dependencies |
|---|------|------|----------|-------------|--------------|
| 1 | omega start | CLI Command | `backend/src/main.rs:47` | Start the OMEGA agent: config load, layout migration, logging setup, prompt deploy, skill load, provider build, channel build, memory init, self-check, gateway run | All subsystems |
| 2 | omega status | CLI Command | `backend/src/main.rs:49` | Check system health: config, provider availability, channel status | ClaudeCodeProvider::check_cli |
| 3 | omega ask \<message\> | CLI Command | `backend/src/main.rs:52` | Send a one-shot message to the AI provider | Provider::complete |
| 4 | omega init | CLI Command | `backend/src/main.rs:57` | Interactive setup wizard (or non-interactive with --telegram-token): Claude CLI check, Telegram token, user ID, Whisper key, WhatsApp, Google, config generation, service install | init module |
| 5 | omega pair | CLI Command | `backend/src/main.rs:84` | Standalone WhatsApp pairing via QR code in terminal | pair module |
| 6 | omega service install/uninstall/status | CLI Command | `backend/src/main.rs:86` | System service management: macOS LaunchAgent or Linux systemd unit | service module |
| 7 | Root guard | Security | `backend/src/main.rs:107` | Refuses to run as root (libc::geteuid) | -- |
| 8 | selfcheck::run() | Service | `backend/src/selfcheck.rs:14` | Startup checks: database accessible, provider available, Telegram getMe API call | Store, Provider, Telegram API |
| 9 | init::run() | Service | `backend/src/init.rs` | Interactive wizard: Claude CLI check, Anthropic auth, Telegram token, user ID, Whisper, WhatsApp, Google, config generation, service install | cliclack |
| 10 | init::run_noninteractive() | Service | `backend/src/init.rs` | Non-interactive init with CLI args/env vars | -- |
| 11 | pair::pair_whatsapp() | Service | `backend/src/pair.rs` | Standalone WhatsApp pairing flow with QR terminal display | -- |
| 12 | service::install() | Service | `backend/src/service.rs:133` | Interactive service installation: generates plist/systemd unit, writes file, activates service | -- |
| 13 | service::uninstall() | Service | `backend/src/service.rs:203` | Removes system service: stops, deletes file | -- |
| 14 | service::status() | Service | `backend/src/service.rs:228` | Checks service status: installed, running | -- |
| 15 | service::install_quiet() | Service | `backend/src/service.rs:319` | Non-interactive service installation (used by init) | -- |
| 16 | generate_plist() | Utility | `backend/src/service.rs:20` | Generates macOS LaunchAgent plist with XML escaping | -- |
| 17 | generate_systemd_unit() | Utility | `backend/src/service.rs:73` | Generates Linux systemd user unit file | -- |

## Internal Dependencies

- cmd_start() is the main entry point that wires everything together
- selfcheck::run() validates database, provider, and channels before starting
- init::run() generates config and optionally installs the service
- service::install() and install_quiet() generate platform-specific service files

## Dead Code / Unused

- None detected.
