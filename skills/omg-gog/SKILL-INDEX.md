# SKILL-INDEX — Detailed Functionality Files

Index of all per-service functionality files for `omega-google`. Each file contains: serde types, URL/body builders, CLI handler functions, and inline test inventories.

All files are relative to `functionalities/` in this skill directory.

---

## Infrastructure & Utilities

| # | Domain | File | Contents |
|---|--------|------|----------|
| 1 | Auth | [auth-functionalities.md](functionalities/auth-functionalities.md) | OAuth flows, token storage, credential backends, multi-account, aliasing |
| 2 | Config | [config-functionalities.md](functionalities/config-functionalities.md) | JSON5 config CRUD, paths, keys |
| 3 | Infrastructure | [infrastructure-functionalities.md](functionalities/infrastructure-functionalities.md) | HTTP client, retry, circuit breaker, output formatting, UI, errors, time |
| 4 | Utilities | [utilities-functionalities.md](functionalities/utilities-functionalities.md) | version, open, completion, exit-codes, schema, agent, time |

## Core Services

| # | Domain | File | Contents |
|---|--------|------|----------|
| 5 | Gmail | [gmail-functionalities.md](functionalities/gmail-functionalities.md) | Threads, messages, labels, drafts, settings, watch, history, batch, attachments |
| 6 | Calendar | [calendar-functionalities.md](functionalities/calendar-functionalities.md) | Events, calendars, freebusy, RSVP, search, conflicts, special events |
| 7 | Drive | [drive-functionalities.md](functionalities/drive-functionalities.md) | Files, upload (simple + resumable), download/export, permissions, comments, shared drives |

## Productivity Services

| # | Domain | File | Contents |
|---|--------|------|----------|
| 8 | Docs | [docs-functionalities.md](functionalities/docs-functionalities.md) | Content, export, editing, sed-like regex, markdown, comments |
| 9 | Sheets | [sheets-functionalities.md](functionalities/sheets-functionalities.md) | Cell read/write, A1 notation, append, insert, format, notes, metadata |
| 10 | Slides | [slides-functionalities.md](functionalities/slides-functionalities.md) | Presentations, slides, notes, export, markdown-to-slides |
| 11 | Forms | [forms-functionalities.md](functionalities/forms-functionalities.md) | Form metadata, creation, responses |

## Communication Services

| # | Domain | File | Contents |
|---|--------|------|----------|
| 12 | Chat | [chat-functionalities.md](functionalities/chat-functionalities.md) | Spaces, messages, threads, DMs |
| 13 | Classroom | [classroom-functionalities.md](functionalities/classroom-functionalities.md) | Courses, roster, coursework, materials, submissions, announcements, topics, invitations, guardians (~60+ commands) |

## Task & Contact Services

| # | Domain | File | Contents |
|---|--------|------|----------|
| 14 | Tasks | [tasks-functionalities.md](functionalities/tasks-functionalities.md) | Task lists, task CRUD, done/undo, clear |
| 15 | Contacts | [contacts-functionalities.md](functionalities/contacts-functionalities.md) | Contact CRUD, search, directory, other contacts |
| 16 | People | [people-functionalities.md](functionalities/people-functionalities.md) | Profile retrieval, search, relationships |
| 17 | Groups | [groups-functionalities.md](functionalities/groups-functionalities.md) | Group listing, membership |

## Other Services

| # | Domain | File | Contents |
|---|--------|------|----------|
| 18 | Keep | [keep-functionalities.md](functionalities/keep-functionalities.md) | Notes listing, search, attachments |
| 19 | Apps Script | [appscript-functionalities.md](functionalities/appscript-functionalities.md) | Project metadata, source files, function execution, creation |
