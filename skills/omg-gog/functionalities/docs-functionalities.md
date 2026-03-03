# Functionalities: Docs

## Overview
Google Docs API — document content extraction, export, creation, copy, text editing (write/insert/delete/find-replace/update/clear), sed-like regex operations, comments, and markdown conversion.

## CLI Commands

| # | Command | Handler | Location | Description |
|---|---------|---------|----------|-------------|
| 1 | `docs export <doc_id>` | `handle_docs_export` | src/cli/mod.rs:3178 | Export as PDF/DOCX/TXT/HTML/EPUB/ODT |
| 2 | `docs info <doc_id>` | `handle_docs_info` | src/cli/mod.rs:3228 | Get document metadata |
| 3 | `docs create <title>` | `handle_docs_create` | src/cli/mod.rs:3258 | Create new document |
| 4 | `docs copy <doc_id> <title>` | `handle_docs_copy` | src/cli/mod.rs:3298 | Copy document |
| 5 | `docs cat <doc_id>` | `handle_docs_cat` | src/cli/mod.rs:3337 | Extract plain text content |
| 6 | `docs list-tabs <doc_id>` | `handle_docs_list_tabs` | src/cli/mod.rs:3410 | List document tabs |
| 7 | `docs comments list` | inline | src/cli/mod.rs:3457 | List comments |
| 8 | `docs comments get <comment_id>` | inline | src/cli/mod.rs | Get comment |
| 9 | `docs comments add` | inline | src/cli/mod.rs | Add comment |
| 10 | `docs comments reply` | inline | src/cli/mod.rs | Reply to comment |
| 11 | `docs comments resolve` | inline | src/cli/mod.rs | Resolve comment |
| 12 | `docs comments delete` | inline | src/cli/mod.rs | Delete comment |
| 13 | `docs write <doc_id> <content>` | `handle_docs_write` | src/cli/mod.rs:3663 | Replace document body with content |
| 14 | `docs insert <doc_id> <content>` | `handle_docs_insert` | src/cli/mod.rs:3753 | Insert text at position |
| 15 | `docs delete <doc_id>` | `handle_docs_delete` | src/cli/mod.rs:3818 | Delete text range |
| 16 | `docs find-replace` | `handle_docs_find_replace` | src/cli/mod.rs:3862 | ReplaceAllText request |
| 17 | `docs update <doc_id>` | `handle_docs_update` | src/cli/mod.rs:3901 | Replace or append content |
| 18 | `docs edit <doc_id>` | `handle_docs_edit` | src/cli/mod.rs:3964 | Find/replace with flags |
| 19 | `docs sed <doc_id> <expr>` | `handle_docs_sed` | src/cli/mod.rs:4003 | Sed-like regex find/replace |
| 20 | `docs clear <doc_id>` | `handle_docs_clear` | src/cli/mod.rs:4107 | Clear all document content |

## URL/Body Builders

| # | Function | Location | Description |
|---|----------|----------|-------------|
| 1 | `build_doc_get_url` | src/services/docs/mod.rs | Document metadata URL |
| 2 | `build_doc_export_url` | src/services/docs/export.rs | Export URL with format |
| 3 | `build_doc_create_body` | src/services/docs/export.rs | Create document body |
| 4 | `build_doc_copy_url` | src/services/docs/export.rs | Copy URL |
| 5 | `build_batch_update_url` | src/services/docs/edit.rs | Batch update URL |
| 6 | `build_insert_text_request` | src/services/docs/edit.rs | Insert text at index |
| 7 | `build_delete_content_range_request` | src/services/docs/edit.rs | Delete text range |
| 8 | `build_replace_all_text_request` | src/services/docs/edit.rs | Find/replace all |
| 9 | `build_write_body_request` | src/services/docs/edit.rs | Replace full body content |
| 10 | `build_clear_body_requests` | src/services/docs/edit.rs | Clear all content |
| 11 | `build_comments_list_url` | src/services/docs/comments.rs | List comments URL |
| 12 | `build_comment_get_url` | src/services/docs/comments.rs | Get comment URL |
| 13 | `build_comment_create_url` | src/services/docs/comments.rs | Create comment URL |
| 14 | `build_comment_reply_url` | src/services/docs/comments.rs | Reply URL |
| 15 | `build_comment_resolve_url` | src/services/docs/comments.rs | Resolve comment URL |
| 16 | `build_comment_delete_url` | src/services/docs/comments.rs | Delete comment URL |

## Content Extraction

| # | Function | Location | Description |
|---|----------|----------|-------------|
| 1 | `extract_text` | src/services/docs/content.rs | Extract plain text from document body |
| 2 | `extract_tab_text` | src/services/docs/content.rs | Extract text from specific tab |
| 3 | `list_tabs` | src/services/docs/content.rs | List document tabs |

## Sed/Regex

| # | Function | Location | Description |
|---|----------|----------|-------------|
| 1 | `parse_sed_expression` | src/services/docs/sedmat.rs | Parse sed-like expression (s/find/replace/flags) |
| 2 | `apply_sed_flags` | src/services/docs/sedmat.rs | Apply sed flags (g, i) |

## Markdown

| # | Function | Location | Description |
|---|----------|----------|-------------|
| 1 | `doc_to_markdown` | src/services/docs/markdown.rs | Convert document to markdown |

## Types

| # | Name | Kind | Location | Description |
|---|------|------|----------|-------------|
| 1 | Document | Struct | src/services/docs/types.rs | Google Doc |
| 2 | Body | Struct | src/services/docs/types.rs | Document body |
| 3 | StructuralElement | Struct | src/services/docs/types.rs | Paragraph/table/etc |
| 4 | Paragraph | Struct | src/services/docs/types.rs | Text paragraph |
| 5 | ParagraphElement | Struct | src/services/docs/types.rs | Text run/inline |
| 6 | TextRun | Struct | src/services/docs/types.rs | Text with style |
| 7 | TextStyle | Struct | src/services/docs/types.rs | Text formatting |
| 8 | Dimension | Struct | src/services/docs/types.rs | Size value |
| 9 | Link | Struct | src/services/docs/types.rs | Hyperlink |
| 10 | Tab | Struct | src/services/docs/types.rs | Document tab |
| 11 | TabProperties | Struct | src/services/docs/types.rs | Tab metadata |
| 12 | DocumentTab | Struct | src/services/docs/types.rs | Tab with body |
| 13 | Comment | Struct | src/services/docs/types.rs | Document comment |
| 14 | Author | Struct | src/services/docs/types.rs | Comment author |
| 15 | Reply | Struct | src/services/docs/types.rs | Comment reply |
| 16 | ReplaceAllTextResponse | Struct | src/services/docs/types.rs | Replace result |
| 17 | Table | Struct | src/services/docs/types.rs | Table element |
| 18 | TableRow | Struct | src/services/docs/types.rs | Table row |
| 19 | TableCell | Struct | src/services/docs/types.rs | Table cell |
| 20 | ParagraphStyle | Struct | src/services/docs/types.rs | Paragraph formatting |
