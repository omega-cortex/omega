# Functionalities: Auth

## Overview
Authentication and account management — OAuth 2.0 flows (desktop/manual/remote), token storage with 3 credential backends (file, OS keyring, encrypted file), service account JWT, multi-account management, and account aliasing.

## CLI Commands

| # | Command | Handler | Location | Description |
|---|---------|---------|----------|-------------|
| 1 | `auth credentials <path>` | `handle_auth_credentials` | src/cli/mod.rs:362 | Store OAuth client credentials JSON |
| 2 | `auth add` | `handle_auth_add` | src/cli/mod.rs:406 | OAuth flow (desktop/manual/remote); `--services`, `--readonly`, `--drive-scope`, `--force-consent` |
| 3 | `auth remove <email>` | `handle_auth_remove` | src/cli/mod.rs:587 | Delete stored account (with confirmation) |
| 4 | `auth list` | `handle_auth_list` | src/cli/mod.rs:638 | List authenticated accounts |
| 5 | `auth status` | `handle_auth_status` | src/cli/mod.rs:706 | Show config path, keyring, current account, token status |
| 6 | `auth services` | `handle_auth_services` | src/cli/mod.rs:786 | List available services and their scopes |
| 7 | `auth tokens list` | `handle_auth_tokens_list` | src/cli/mod.rs:811 | List stored tokens |
| 8 | `auth tokens delete <email>` | `handle_auth_tokens_delete` | src/cli/mod.rs:863 | Delete a specific token |
| 9 | `auth alias set <alias> <email>` | `handle_auth_alias` | src/cli/mod.rs:892 | Set account alias |
| 10 | `auth alias unset <alias>` | `handle_auth_alias` | src/cli/mod.rs:892 | Remove alias |
| 11 | `auth alias list` | `handle_auth_alias` | src/cli/mod.rs:892 | List aliases |

## Types

| # | Name | Kind | Location | Description |
|---|------|------|----------|-------------|
| 1 | Service | Enum | src/auth/mod.rs | 15 variants: Gmail, Calendar, Drive, Docs, Sheets, Slides, Forms, Chat, Tasks, Classroom, Contacts, People, Groups, Keep, AppScript |
| 2 | TokenData | Struct | src/auth/mod.rs | Stored token (access_token, refresh_token, expiry, scopes) |
| 3 | CredentialStore | Trait | src/auth/mod.rs | get_token, set_token, delete_token, list_tokens, get/set_default_account, delete_token_by_raw_key |
| 4 | FileCredentialStore | Struct | src/auth/keyring.rs | File-based token storage |
| 5 | KeyringCredentialStore | Struct | src/auth/keyring.rs | OS keyring token storage (macOS Keychain, Linux secret-service) |
| 6 | EncryptedFileCredentialStore | Struct | src/auth/keyring.rs | AES-GCM encrypted file storage |
| 7 | FlowMode | Enum | src/auth/oauth.rs | Desktop, Manual, Remote |
| 8 | TokenResponse | Struct | src/auth/oauth.rs | OAuth token exchange response |
| 9 | OAuthFlowResult | Struct | src/auth/oauth_flow.rs | Result of OAuth flow (email, token_data) |
| 10 | ServiceAccountKey | Struct | src/auth/service_account.rs | Parsed service account JSON key file |
| 11 | JwtClaims | Struct | src/auth/service_account.rs | JWT assertion claims |
| 12 | AuthArgs | Struct | src/cli/root.rs:189 | CLI args for `auth` command |
| 13 | AuthCommand | Enum | src/cli/root.rs | Auth subcommand variants |
| 14 | TokensCommand | Enum | src/cli/root.rs | Tokens subcommand variants |

## Functions

| # | Name | Location | Description |
|---|------|----------|-------------|
| 1 | resolve_account | src/auth/mod.rs | Resolve account from --account flag, alias, or default |
| 2 | scopes_for_service | src/auth/scopes.rs | Return OAuth scopes for a service |
| 3 | readonly_scopes_for_service | src/auth/scopes.rs | Return read-only OAuth scopes |
| 4 | drive_scope_for_service | src/auth/scopes.rs | Return drive-scope OAuth scopes |
| 5 | build_auth_url | src/auth/oauth.rs | Build Google OAuth authorization URL |
| 6 | exchange_code | src/auth/oauth.rs | Exchange auth code for tokens |
| 7 | run_oauth_flow | src/auth/oauth_flow.rs | Execute full OAuth flow (desktop with local HTTP server, manual OOB, remote) |
| 8 | needs_refresh | src/auth/token.rs | Check if token needs refresh |
| 9 | refresh_access_token | src/auth/token.rs | Refresh expired access token |
| 10 | serialize_token | src/auth/token.rs | Serialize token to JSON |
| 11 | deserialize_token | src/auth/token.rs | Deserialize token from JSON |
| 12 | credential_store_factory | src/auth/keyring.rs | Create credential store based on config (file/keyring/encrypted) |
| 13 | load_service_account_key | src/auth/service_account.rs | Load service account key from JSON file |
| 14 | build_jwt_assertion | src/auth/service_account.rs | Build signed JWT for service account auth |
| 15 | exchange_jwt | src/auth/service_account.rs | Exchange JWT assertion for access token |
| 16 | parse_token_key | src/auth/mod.rs | Parse token storage key |
| 17 | token_key | src/auth/mod.rs | Generate token storage key |
| 18 | legacy_token_key | src/auth/mod.rs | Generate legacy token storage key |

## Dependencies

- **Depends on**: `config` (read config for keyring backend, credentials), `http` (token exchange HTTP calls)
- **Depended on by**: `services::bootstrap_service_context()` (all service handlers use auth)
