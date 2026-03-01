# Functionalities: Summarizer

## Overview

Background conversation summarization with fact extraction. Automatically summarizes idle conversations (2h timeout) and extracts personal facts about the user.

## Functionalities

| # | Name | Type | Location | Description | Dependencies |
|---|------|------|----------|-------------|--------------|
| 1 | background_summarizer() | Background Task | `backend/src/gateway/summarizer.rs:105` | Polls every 60s for idle conversations (2h timeout), summarizes and clears sessions | Store, Provider |
| 2 | summarize_conversation() | Service | `backend/src/gateway/summarizer.rs:140` | Summarizes a conversation and extracts facts using two provider calls, then closes it | Provider, Store |
| 3 | summarize_and_extract() | Service | `backend/src/gateway/summarizer.rs:13` | Combined summary + fact extraction in a single provider call (used by /forget background path) | Provider, Store |
| 4 | handle_forget() | Service | `backend/src/gateway/summarizer.rs:213` | Handles /forget command: closes conversation immediately, summarizes in background | summarize_and_extract, Store, i18n |

## Internal Dependencies

- background_summarizer() calls summarize_conversation()
- handle_forget() calls summarize_and_extract() in a background spawn
- Both use is_valid_fact() from keywords module for fact validation

## Dead Code / Unused

- None detected.
