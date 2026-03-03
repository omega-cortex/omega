# Functionalities: Chat

## Overview
Google Chat API — spaces (list, find, create), messages (list, send), threads (list), and direct messages (find/create DM space, send DM).

## CLI Commands

| # | Command | Handler | Location | Description |
|---|---------|---------|----------|-------------|
| 1 | `chat spaces list` | `handle_chat_spaces_list` | src/cli/mod.rs:5605 | List spaces |
| 2 | `chat spaces find <name>` | `handle_chat_spaces_find` | src/cli/mod.rs:5632 | Find space by name |
| 3 | `chat spaces create <name>` | `handle_chat_spaces_create` | src/cli/mod.rs:5658 | Create space |
| 4 | `chat messages list <space>` | `handle_chat_messages_list` | src/cli/mod.rs:5688 | List messages in space |
| 5 | `chat messages send <space>` | `handle_chat_messages_send` | src/cli/mod.rs:5718 | Send message to space |
| 6 | `chat threads list <space>` | `handle_chat_threads_list` | src/cli/mod.rs:5750 | List threads in space |
| 7 | `chat dm space <user>` | `handle_chat_dm_space` | src/cli/mod.rs:5778 | Find or create DM space |
| 8 | `chat dm send <email>` | `handle_chat_dm_send` | src/cli/mod.rs:5807 | Send direct message |

## URL/Body Builders

| # | Function | Location | Description |
|---|----------|----------|-------------|
| 1 | `build_spaces_list_url` | src/services/chat/spaces.rs | Spaces list URL |
| 2 | `build_space_setup_url` | src/services/chat/spaces.rs | Space creation URL |
| 3 | `build_space_setup_body` | src/services/chat/spaces.rs | Space creation body |
| 4 | `build_messages_list_url` | src/services/chat/messages.rs | Messages list URL |
| 5 | `build_message_create_url` | src/services/chat/messages.rs | Message send URL |
| 6 | `build_message_create_body` | src/services/chat/messages.rs | Message body |
| 7 | `build_threads_list_url` | src/services/chat/threads.rs | Threads list URL |
| 8 | `build_dm_find_url` | src/services/chat/dm.rs | Find DM space URL |
| 9 | `build_dm_find_body` | src/services/chat/dm.rs | Find DM request body |

## Types

| # | Name | Kind | Location | Description |
|---|------|------|----------|-------------|
| 1 | Space | Struct | src/services/chat/types.rs | Chat space |
| 2 | SpaceListResponse | Struct | src/services/chat/types.rs | Space list |
| 3 | Message | Struct | src/services/chat/types.rs | Chat message |
| 4 | MessageSender | Struct | src/services/chat/types.rs | Message sender info |
| 5 | MessageListResponse | Struct | src/services/chat/types.rs | Message list |
| 6 | Thread | Struct | src/services/chat/types.rs | Chat thread |
| 7 | ThreadListResponse | Struct | src/services/chat/types.rs | Thread list |
| 8 | CreateSpaceRequest | Struct | src/services/chat/types.rs | Space creation request |
| 9 | CreateMessageRequest | Struct | src/services/chat/types.rs | Message creation request |
