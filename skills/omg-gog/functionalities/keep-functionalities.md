# Functionalities: Keep

## Overview
Google Keep API — note listing, retrieval, client-side search, and attachment download. Supports `--service-account` and `--impersonate` for domain-wide access.

## CLI Commands

| # | Command | Handler | Location | Description |
|---|---------|---------|----------|-------------|
| 1 | `keep list` | `handle_keep_list` | src/cli/mod.rs:8673 | List notes |
| 2 | `keep get <note_id>` | `handle_keep_get` | src/cli/mod.rs:8731 | Get note by ID |
| 3 | `keep search <query>` | `handle_keep_search` | src/cli/mod.rs:8760 | Search notes (client-side filtering) |
| 4 | `keep attachment <name>` | `handle_keep_attachment` | src/cli/mod.rs:8829 | Download attachment |

## URL/Body Builders

| # | Function | Location | Description |
|---|----------|----------|-------------|
| 1 | `build_notes_list_url` | src/services/keep/notes.rs | Notes list URL |
| 2 | `build_note_get_url` | src/services/keep/notes.rs | Note get URL |
| 3 | `build_attachment_download_url` | src/services/keep/attachments.rs | Attachment download URL |

## Types

| # | Name | Kind | Location | Description |
|---|------|------|----------|-------------|
| 1 | Note | Struct | src/services/keep/types.rs | Keep note |
| 2 | NoteBody | Struct | src/services/keep/types.rs | Note body (text or list) |
| 3 | TextContent | Struct | src/services/keep/types.rs | Text content |
| 4 | ListContent | Struct | src/services/keep/types.rs | Checklist content |
| 5 | ListItem | Struct | src/services/keep/types.rs | Checklist item |
| 6 | NoteListResponse | Struct | src/services/keep/types.rs | Note list |
| 7 | Attachment | Struct | src/services/keep/types.rs | Note attachment |
| 8 | Permission | Struct | src/services/keep/types.rs | Note permission |
