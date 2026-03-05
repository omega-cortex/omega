# Functionalities: Gmail

## Overview
Full Gmail API coverage — thread/message search, read, send, labels, drafts, settings (filters, forwarding, send-as, delegates, vacation, auto-forward), watch (push notifications), history, batch operations, MIME message building, and attachment handling.

## CLI Commands

| # | Command | Handler | Location | Description |
|---|---------|---------|----------|-------------|
| 1 | `gmail search <query>` | `handle_gmail_search` | src/cli/mod.rs:1049 | Search threads |
| 2 | `gmail messages search <query>` | `handle_gmail_messages` | src/cli/mod.rs:1109 | Search messages |
| 3 | `gmail thread get <id>` | `handle_gmail_thread` | src/cli/mod.rs:1177 | Get thread by ID |
| 4 | `gmail thread modify <id>` | `handle_gmail_thread` | src/cli/mod.rs:1177 | Modify thread labels |
| 5 | `gmail thread attachments <id>` | `handle_gmail_thread` | src/cli/mod.rs:1177 | Download thread attachments |
| 6 | `gmail get <message_id>` | `handle_gmail_message_get` | src/cli/mod.rs:1249 | Get message by ID |
| 7 | `gmail attachment <msg_id> <att_id>` | `handle_gmail_attachment` | src/cli/mod.rs:3043 | Download attachment |
| 8 | `gmail url <thread_ids>` | inline | src/cli/mod.rs | Generate Gmail thread URLs (offline) |
| 9 | `gmail history --since <id>` | inline | src/cli/mod.rs | List mailbox history |
| 10 | `gmail send` | `handle_gmail_send` | src/cli/mod.rs:1286 | Send email (to, cc, bcc, subject, body, attachments) |
| 11 | `gmail labels list` | `handle_gmail_labels` | src/cli/mod.rs:1366 | List labels |
| 12 | `gmail labels get <id>` | `handle_gmail_labels` | src/cli/mod.rs:1366 | Get label details |
| 13 | `gmail labels create <name>` | `handle_gmail_labels` | src/cli/mod.rs:1366 | Create label |
| 14 | `gmail labels modify <id>` | `handle_gmail_labels` | src/cli/mod.rs:1366 | Modify label |
| 15 | `gmail labels delete <id>` | `handle_gmail_labels` | src/cli/mod.rs:1366 | Delete label |
| 16 | `gmail batch delete` | inline | src/cli/mod.rs | Batch delete messages |
| 17 | `gmail batch modify` | inline | src/cli/mod.rs | Batch modify message labels |
| 18 | `gmail drafts list` | inline | src/cli/mod.rs | List drafts |
| 19 | `gmail drafts get <id>` | inline | src/cli/mod.rs | Get draft |
| 20 | `gmail drafts create` | inline | src/cli/mod.rs | Create draft |
| 21 | `gmail drafts update <id>` | inline | src/cli/mod.rs | Update draft |
| 22 | `gmail drafts send <id>` | inline | src/cli/mod.rs | Send draft |
| 23 | `gmail drafts delete <id>` | inline | src/cli/mod.rs | Delete draft |
| 24 | `gmail settings filters list` | inline | src/cli/mod.rs | List email filters |
| 25 | `gmail settings filters get <id>` | inline | src/cli/mod.rs | Get filter |
| 26 | `gmail settings filters create` | inline | src/cli/mod.rs | Create filter |
| 27 | `gmail settings filters delete <id>` | inline | src/cli/mod.rs | Delete filter |
| 28 | `gmail settings forwarding list/get/create/delete` | inline | src/cli/mod.rs | Forwarding addresses |
| 29 | `gmail settings sendas list/get/create/verify/delete/update` | inline | src/cli/mod.rs | Send-As aliases |
| 30 | `gmail settings delegates list/get/add/remove` | inline | src/cli/mod.rs | Delegates |
| 31 | `gmail settings vacation get/update` | inline | src/cli/mod.rs | Vacation responder |
| 32 | `gmail settings autoforward get/update` | inline | src/cli/mod.rs | Auto-forwarding |
| 33 | `gmail watch start --topic <topic>` | inline | src/cli/mod.rs | Start Pub/Sub push notifications; `--topic` (required): Pub/Sub topic name, `--label` (optional, repeatable): filter by label IDs |
| 34 | `gmail watch status` | inline | src/cli/mod.rs | Check watch status (returns historyId, expiration) |
| 35 | `gmail watch renew` | inline | src/cli/mod.rs | Renew watch (re-calls watch start) |
| 36 | `gmail watch stop` | inline | src/cli/mod.rs | Stop push notifications |

## URL Builders

| # | Function | Location | API Endpoint |
|---|----------|----------|-------------|
| 1 | `build_thread_search_url` | src/services/gmail/search.rs | `GET /gmail/v1/users/me/threads` |
| 2 | `build_message_search_url` | src/services/gmail/search.rs | `GET /gmail/v1/users/me/messages` |
| 3 | `build_message_get_url` | src/services/gmail/message.rs | `GET /gmail/v1/users/me/messages/{id}` |
| 4 | `build_attachment_url` | src/services/gmail/message.rs | `GET /gmail/v1/users/me/messages/{id}/attachments/{id}` |
| 5 | `build_thread_get_url` | src/services/gmail/thread.rs | `GET /gmail/v1/users/me/threads/{id}` |
| 6 | `build_thread_modify_request` | src/services/gmail/thread.rs | `POST /gmail/v1/users/me/threads/{id}/modify` |
| 7 | `build_labels_list_url` | src/services/gmail/labels.rs | `GET /gmail/v1/users/me/labels` |
| 8 | `build_label_get_url` | src/services/gmail/labels.rs | `GET /gmail/v1/users/me/labels/{id}` |
| 9 | `build_label_create_request` | src/services/gmail/labels.rs | `POST /gmail/v1/users/me/labels` |
| 10 | `build_label_delete_url` | src/services/gmail/labels.rs | `DELETE /gmail/v1/users/me/labels/{id}` |
| 11 | `resolve_label_id` | src/services/gmail/labels.rs | Resolve label name to ID |
| 12 | `build_send_url` | src/services/gmail/send.rs | `POST /gmail/v1/users/me/messages/send` |
| 13 | `build_send_body` | src/services/gmail/send.rs | Build send request body |
| 14 | `build_send_draft_url` | src/services/gmail/send.rs | `POST /gmail/v1/users/me/drafts/send` |
| 15 | `build_drafts_list_url` | src/services/gmail/drafts.rs | `GET /gmail/v1/users/me/drafts` |
| 16 | `build_draft_get_url` | src/services/gmail/drafts.rs | `GET /gmail/v1/users/me/drafts/{id}` |
| 17 | `build_draft_create_url` | src/services/gmail/drafts.rs | `POST /gmail/v1/users/me/drafts` |
| 18 | `build_draft_update_url` | src/services/gmail/drafts.rs | `PUT /gmail/v1/users/me/drafts/{id}` |
| 19 | `build_draft_delete_url` | src/services/gmail/drafts.rs | `DELETE /gmail/v1/users/me/drafts/{id}` |
| 20 | `build_draft_send_url` | src/services/gmail/drafts.rs | `POST /gmail/v1/users/me/drafts/send` |
| 21 | `build_watch_start_url` | src/services/gmail/watch.rs | `POST /gmail/v1/users/me/watch` |
| 22 | `build_watch_stop_url` | src/services/gmail/watch.rs | `POST /gmail/v1/users/me/stop` |
| 23 | `build_history_list_url` | src/services/gmail/history.rs | `GET /gmail/v1/users/me/history` |
| 24 | `build_batch_modify_url` | src/services/gmail/batch.rs | `POST /gmail/v1/users/me/messages/batchModify` |
| 25 | `build_batch_delete_url` | src/services/gmail/batch.rs | `POST /gmail/v1/users/me/messages/batchDelete` |
| 26 | `build_filters_list_url` | src/services/gmail/settings.rs | `GET /gmail/v1/users/me/settings/filters` |
| 27 | `build_filter_get_url` | src/services/gmail/settings.rs | `GET /gmail/v1/users/me/settings/filters/{id}` |
| 28 | `build_filter_create_url` | src/services/gmail/settings.rs | `POST /gmail/v1/users/me/settings/filters` |
| 29 | `build_filter_delete_url` | src/services/gmail/settings.rs | `DELETE /gmail/v1/users/me/settings/filters/{id}` |
| 30 | `build_forwarding_list_url` | src/services/gmail/settings.rs | Forwarding addresses |
| 31 | `build_vacation_get_url` | src/services/gmail/settings.rs | Vacation responder |
| 32 | `build_vacation_update_url` | src/services/gmail/settings.rs | Update vacation responder |
| 33 | `build_autoforward_get_url` | src/services/gmail/settings.rs | Auto-forwarding settings |
| 34 | `build_sendas_list_url` | src/services/gmail/settings.rs | Send-As aliases |
| 35 | `build_delegates_list_url` | src/services/gmail/settings.rs | Delegates |

## MIME Utilities

| # | Function | Location | Description |
|---|----------|----------|-------------|
| 1 | `build_mime_message` | src/services/gmail/mime.rs | Build full MIME message (headers + body + attachments) |
| 2 | `generate_boundary` | src/services/gmail/mime.rs | Generate MIME boundary string |
| 3 | `base64url_encode` | src/services/gmail/mime.rs | URL-safe base64 encoding |
| 4 | `guess_content_type` | src/services/gmail/mime.rs | Guess MIME type from filename |

## Types

| # | Name | Kind | Location | Description |
|---|------|------|----------|-------------|
| 1 | ThreadListResponse | Struct | src/services/gmail/types.rs | Paginated thread list |
| 2 | Thread | Struct | src/services/gmail/types.rs | Gmail thread |
| 3 | Message | Struct | src/services/gmail/types.rs | Gmail message |
| 4 | MessagePart | Struct | src/services/gmail/types.rs | MIME message part |
| 5 | Header | Struct | src/services/gmail/types.rs | Email header |
| 6 | MessagePartBody | Struct | src/services/gmail/types.rs | Message body data |
| 7 | LabelListResponse | Struct | src/services/gmail/types.rs | Label list |
| 8 | Label | Struct | src/services/gmail/types.rs | Gmail label |
| 9 | LabelColor | Struct | src/services/gmail/types.rs | Label color |
| 10 | DraftListResponse | Struct | src/services/gmail/types.rs | Draft list |
| 11 | Draft | Struct | src/services/gmail/types.rs | Gmail draft |
| 12 | HistoryListResponse | Struct | src/services/gmail/types.rs | History list |
| 13 | History | Struct | src/services/gmail/types.rs | History record |
| 14 | HistoryMessageAdded | Struct | src/services/gmail/types.rs | Message added event |
| 15 | HistoryMessageDeleted | Struct | src/services/gmail/types.rs | Message deleted event |
| 16 | HistoryLabelAdded | Struct | src/services/gmail/types.rs | Label added event |
| 17 | HistoryLabelRemoved | Struct | src/services/gmail/types.rs | Label removed event |
| 18 | WatchResponse | Struct | src/services/gmail/types.rs | Watch response |
| 19 | WatchRequest | Struct | src/services/gmail/types.rs | Watch request body |
| 20 | BatchModifyRequest | Struct | src/services/gmail/types.rs | Batch modify request |
| 21 | BatchDeleteRequest | Struct | src/services/gmail/types.rs | Batch delete request |
| 22 | Filter | Struct | src/services/gmail/types.rs | Email filter |
| 23 | FilterCriteria | Struct | src/services/gmail/types.rs | Filter match criteria |
| 24 | FilterAction | Struct | src/services/gmail/types.rs | Filter action |
| 25 | ForwardingAddress | Struct | src/services/gmail/types.rs | Forwarding address |
| 26 | SendAs | Struct | src/services/gmail/types.rs | Send-As alias |
| 27 | Delegate | Struct | src/services/gmail/types.rs | Delegate |
| 28 | VacationSettings | Struct | src/services/gmail/types.rs | Vacation responder settings |
| 29 | AutoForwarding | Struct | src/services/gmail/types.rs | Auto-forwarding settings |

## Service Modules

| Module | File | Description |
|--------|------|-------------|
| search | src/services/gmail/search.rs | Thread and message search URL builders |
| message | src/services/gmail/message.rs | Message get, attachment URL builders |
| thread | src/services/gmail/thread.rs | Thread get, modify builders |
| labels | src/services/gmail/labels.rs | Label CRUD + resolve |
| send | src/services/gmail/send.rs | Send message/draft builders |
| drafts | src/services/gmail/drafts.rs | Draft CRUD builders |
| settings | src/services/gmail/settings.rs | Filters, forwarding, send-as, delegates, vacation, autoforward |
| watch | src/services/gmail/watch.rs | Push notification start/stop |
| history | src/services/gmail/history.rs | Mailbox history |
| batch | src/services/gmail/batch.rs | Batch modify/delete |
| mime | src/services/gmail/mime.rs | MIME message builder, base64url, content type |
| types | src/services/gmail/types.rs | All serde data types |
