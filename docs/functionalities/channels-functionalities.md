# Functionalities: omega-channels

## Overview

Messaging platform integrations. Telegram uses long-polling with voice transcription via Whisper. WhatsApp uses wacore library with signal protocol for end-to-end encryption.

## Functionalities

| # | Name | Type | Location | Description | Dependencies |
|---|------|------|----------|-------------|--------------|
| 1 | TelegramChannel | Channel | `backend/crates/omega-channels/src/telegram/` | Telegram bot: long-polling, send text/photo, typing indicator, voice transcription via Whisper | whisper |
| 2 | WhatsAppChannel | Channel | `backend/crates/omega-channels/src/whatsapp/` | WhatsApp bot: wacore library, signal protocol, send text/photo, QR pairing, voice transcription | whatsapp_store, whisper |
| 3 | Whisper integration | Service | `backend/crates/omega-channels/src/whisper.rs` | OpenAI Whisper API voice transcription for both channels | reqwest |
| 4 | WhatsApp store | Library | `backend/crates/omega-channels/src/whatsapp_store/` | Signal protocol storage: device identity, sessions, pre-keys, app sync state | -- |
| 5 | Utils module | Library | `backend/crates/omega-channels/src/utils.rs` | Shared channel utilities | -- |

## Internal Dependencies

- Both Telegram and WhatsApp channels use Whisper for voice transcription
- WhatsApp channel uses whatsapp_store for signal protocol persistence
- Gateway accesses WhatsAppChannel via as_any() downcast for pairing operations

## Dead Code / Unused

- `#[allow(dead_code)]` on Telegram type definitions (types.rs:19,31,40,47,56) -- API response structs with fields populated by deserialization but not all read
