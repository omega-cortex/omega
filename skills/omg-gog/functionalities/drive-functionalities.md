# Functionalities: Drive

## Overview
Google Drive API — file listing/search, metadata, download/export, upload (simple + resumable), copy, move, rename, trash/delete, sharing/permissions, comments, shared drives, and MIME type handling.

## CLI Commands

| # | Command | Handler | Location | Description |
|---|---------|---------|----------|-------------|
| 1 | `drive ls` | `handle_drive_list` | src/cli/mod.rs:2082 | List files (with folder, type, owner filters) |
| 2 | `drive search <query>` | `handle_drive_search` | src/cli/mod.rs:2138 | Full-text search or Drive query language |
| 3 | `drive get <file_id>` | `handle_drive_get` | src/cli/mod.rs:2194 | Get file metadata |
| 4 | `drive download <file_id>` | `handle_drive_download` | src/cli/mod.rs:2575 | Download file or export Google Workspace doc |
| 5 | `drive copy <file_id>` | `handle_drive_copy` | src/cli/mod.rs:2527 | Copy file |
| 6 | `drive upload <path>` | `handle_drive_upload` / `handle_drive_resumable_upload` | src/cli/mod.rs:2698/2842 | Upload file (simple or resumable for large files) |
| 7 | `drive mkdir <name>` | `handle_drive_mkdir` | src/cli/mod.rs:2223 | Create folder |
| 8 | `drive delete` / `rm` | `handle_drive_delete` | src/cli/mod.rs:2261 | Trash or permanently delete |
| 9 | `drive move <file_id>` | `handle_drive_move` | src/cli/mod.rs:2333 | Move file to different folder |
| 10 | `drive rename <file_id> <name>` | `handle_drive_rename` | src/cli/mod.rs:2398 | Rename file |
| 11 | `drive share <file_id>` | `handle_drive_share` | src/cli/mod.rs:2442 | Share file (user/group/domain/anyone) |
| 12 | `drive unshare` | inline | src/cli/mod.rs | Remove permission |
| 13 | `drive permissions <file_id>` | `handle_drive_permissions_list` | src/cli/mod.rs:2495 | List permissions |
| 14 | `drive url <file_ids>` | inline | src/cli/mod.rs | Generate Drive file URLs (offline) |
| 15 | `drive comments list/create/reply` | inline | src/cli/mod.rs | Comments CRUD |
| 16 | `drive drives` | inline | src/cli/mod.rs | List shared drives |

## URL/Body Builders

| # | Function | Location | Description |
|---|----------|----------|-------------|
| 1 | `build_file_get_url` | src/services/drive/files.rs | File metadata URL |
| 2 | `build_file_download_url` | src/services/drive/files.rs | File download URL (alt=media) |
| 3 | `build_file_export_url` | src/services/drive/files.rs | Google Workspace export URL |
| 4 | `build_file_upload_url` | src/services/drive/files.rs | Simple upload URL |
| 5 | `build_resumable_upload_url` | src/services/drive/files.rs | Resumable upload initiation URL |
| 6 | `build_file_copy_url` | src/services/drive/files.rs | File copy URL |
| 7 | `resolve_download_path` | src/services/drive/files.rs | Resolve local download path from file metadata |
| 8 | `build_list_query` | src/services/drive/list.rs | Build file list query with filters |
| 9 | `build_search_query` | src/services/drive/list.rs | Build fullTextContains search query |
| 10 | `build_filter_query` | src/services/drive/list.rs | Build filter query from flags |
| 11 | `looks_like_drive_query_language` | src/services/drive/list.rs | Detect raw Drive query language |
| 12 | `has_trashed_predicate` | src/services/drive/list.rs | Check if query includes trashed filter |
| 13 | `escape_query_string` | src/services/drive/list.rs | Escape query string for Drive API |
| 14 | `build_mkdir_body` | src/services/drive/folders.rs | Folder creation body |
| 15 | `build_move_params` | src/services/drive/folders.rs | File move parameters |
| 16 | `build_rename_body` | src/services/drive/folders.rs | File rename body |
| 17 | `build_trash_url` | src/services/drive/folders.rs | Trash file URL |
| 18 | `build_permanent_delete_url` | src/services/drive/folders.rs | Permanent delete URL |
| 19 | `build_share_permission` | src/services/drive/permissions.rs | Permission creation body |
| 20 | `build_create_permission_url` | src/services/drive/permissions.rs | Create permission URL |
| 21 | `build_list_permissions_url` | src/services/drive/permissions.rs | List permissions URL |
| 22 | `build_delete_permission_url` | src/services/drive/permissions.rs | Delete permission URL |
| 23 | `validate_role` | src/services/drive/permissions.rs | Validate permission role |
| 24 | `validate_share_target` | src/services/drive/permissions.rs | Validate share target type |
| 25 | `build_comments_list_url` | src/services/drive/comments.rs | Comments list URL |
| 26 | `build_comment_create_url` | src/services/drive/comments.rs | Comment create URL |
| 27 | `build_comment_reply_url` | src/services/drive/comments.rs | Comment reply URL |
| 28 | `build_drives_list_url` | src/services/drive/drives.rs | Shared drives list URL |

## Utility Functions

| # | Function | Location | Description |
|---|----------|----------|-------------|
| 1 | `drive_type` | src/services/drive/types.rs | Determine file type from MIME |
| 2 | `guess_mime_type` | src/services/drive/types.rs | Guess MIME from filename |
| 3 | `default_export_mime` | src/services/drive/types.rs | Default export MIME for workspace type |
| 4 | `export_mime_for_format` | src/services/drive/types.rs | Map format name to export MIME |
| 5 | `file_url` | src/services/drive/types.rs | Generate file web URL |
| 6 | `extension_for_mime` | src/services/drive/types.rs | File extension for MIME type |
| 7 | `convert_to_mime` | src/services/drive/types.rs | Convert format to upload MIME |
| 8 | `is_google_workspace_type` | src/services/drive/types.rs | Check if MIME is Google Workspace |
| 9 | `strip_office_extension` | src/services/drive/types.rs | Strip Office extension for conversion |

## Types

| # | Name | Kind | Location | Description |
|---|------|------|----------|-------------|
| 1 | FileListResponse | Struct | src/services/drive/types.rs | Paginated file list |
| 2 | DriveFile | Struct | src/services/drive/types.rs | Drive file metadata |
| 3 | PermissionListResponse | Struct | src/services/drive/types.rs | Permission list |
| 4 | Permission | Struct | src/services/drive/types.rs | File permission |
| 5 | CommentListResponse | Struct | src/services/drive/types.rs | Comment list |
| 6 | Comment | Struct | src/services/drive/types.rs | File comment |
| 7 | CommentAuthor | Struct | src/services/drive/types.rs | Comment author |
| 8 | QuotedContent | Struct | src/services/drive/types.rs | Quoted content in comment |
| 9 | CommentReply | Struct | src/services/drive/types.rs | Comment reply |
| 10 | SharedDriveListResponse | Struct | src/services/drive/types.rs | Shared drive list |
| 11 | SharedDrive | Struct | src/services/drive/types.rs | Shared drive |
