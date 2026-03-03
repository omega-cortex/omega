# Functionalities: Config

## Overview
Configuration file management — read/write JSON5 config, OAuth client credential management, platform-specific paths, multi-client support.

## CLI Commands

| # | Command | Handler | Location | Description |
|---|---------|---------|----------|-------------|
| 1 | `config get <key>` | `handle_config_get` | src/cli/mod.rs:193 | Get a config value |
| 2 | `config set <key> <value>` | `handle_config_set` | src/cli/mod.rs:227 | Set a config value |
| 3 | `config unset <key>` | `handle_config_unset` | src/cli/mod.rs:252 | Remove a config value |
| 4 | `config list` | `handle_config_list` | src/cli/mod.rs:280 | List all config values |
| 5 | `config keys` | `handle_config_keys` | src/cli/mod.rs:314 | List known config keys |
| 6 | `config path` | `handle_config_path` | src/cli/mod.rs:330 | Print config file path |

## Types

| # | Name | Kind | Location | Description |
|---|------|------|----------|-------------|
| 1 | ConfigFile | Struct | src/config/mod.rs | Parsed config file contents |
| 2 | ConfigArgs | Struct | src/cli/root.rs:323 | CLI args for `config` command |

## Functions

| # | Name | Location | Description |
|---|------|----------|-------------|
| 1 | read_config | src/config/mod.rs | Read config file (JSON5 in, tolerates comments/trailing commas) |
| 2 | write_config | src/config/mod.rs | Write config file (standard JSON, atomic via tmp+rename) |
| 3 | config_dir | src/config/paths.rs | Platform-specific config directory (overridable via `OMEGA_GOOGLE_CONFIG_DIR`) |
| 4 | config_file_path | src/config/paths.rs | Full path to config file |
| 5 | credentials_path | src/config/paths.rs | Path to default `credentials.json` |
| 6 | named_credentials_path | src/config/paths.rs | Path to named `credentials-{name}.json` |
| 7 | parse_credentials | src/config/credentials.rs | Parse Google OAuth client JSON (installed + web formats) |
| 8 | read_client_credentials | src/config/mod.rs | Read and parse credential file for a client name |

## Config Keys

| Key | Description |
|-----|-------------|
| `keyring_backend` | Credential store backend: file, keyring, encrypted |
| `default_timezone` | Default timezone for time operations |
| `account_aliases` | Account alias mappings |
| `account_clients` | Per-account OAuth client mappings |
| `client_domains` | Per-domain OAuth client mappings |

## Dependencies

- **Depends on**: filesystem
- **Depended on by**: `auth` (credential store factory, credentials), `services::bootstrap_service_context()`
