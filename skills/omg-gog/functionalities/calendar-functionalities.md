# Functionalities: Calendar

## Overview
Google Calendar API — event CRUD, calendar listing, freebusy queries, RSVP, cross-calendar search, conflict detection, special event types (focus time, out-of-office, working location), and push notification watches via Channel API.

## CLI Commands

| # | Command | Handler | Location | Description |
|---|---------|---------|----------|-------------|
| 1 | `calendar calendars` | `handle_calendar_calendars_list` | src/cli/mod.rs:1917 | List calendars |
| 2 | `calendar acl <cal_id>` | inline | src/cli/mod.rs | List calendar ACL rules |
| 3 | `calendar events` | `handle_calendar_events_list` | src/cli/mod.rs:1610 | List events (with time range, search, calendar filters) |
| 4 | `calendar event <cal_id> <event_id>` | `handle_calendar_event_get` | src/cli/mod.rs:1690 | Get event details |
| 5 | `calendar create` | `handle_calendar_event_create` | src/cli/mod.rs:1721 | Create event (summary, start, end, attendees, location, description) |
| 6 | `calendar update` | `handle_calendar_event_update` | src/cli/mod.rs:1807 | Update event fields |
| 7 | `calendar delete` | `handle_calendar_event_delete` | src/cli/mod.rs:1870 | Delete event (with confirmation) |
| 8 | `calendar freebusy` | `handle_calendar_freebusy` | src/cli/mod.rs:1970 | Query free/busy for calendars |
| 9 | `calendar respond` / `rsvp` | inline | src/cli/mod.rs | RSVP to event invitation |
| 10 | `calendar search` | inline | src/cli/mod.rs | Cross-calendar event search |
| 11 | `calendar time` | inline | src/cli/mod.rs | Show server time |
| 12 | `calendar users` | inline | src/cli/mod.rs | List workspace users |
| 13 | `calendar team <group_email>` | inline | src/cli/mod.rs | Group calendar events |
| 14 | `calendar colors` | inline | src/cli/mod.rs | Get calendar color definitions |
| 15 | `calendar conflicts` | inline | src/cli/mod.rs | Find scheduling conflicts |
| 16 | `calendar focus-time` | inline | src/cli/mod.rs | Create Focus Time block |
| 17 | `calendar out-of-office` / `ooo` | inline | src/cli/mod.rs | Create out-of-office event |
| 18 | `calendar working-location` / `wl` | inline | src/cli/mod.rs | Set working location |
| 19 | `calendar watch start --callback-url <url>` | `watch_start` | src/services/calendar/watch.rs | Start push notifications; `--callback-url` (required), `--calendar` (default: primary) |
| 20 | `calendar watch stop --channel-id <id> --resource-id <id>` | `watch_stop` | src/services/calendar/watch.rs | Stop push notifications |
| 21 | `calendar watch status` | `watch_status` | src/services/calendar/watch.rs | Show watch status (informational — no API to query active watches) |

## URL/Body Builders

| # | Function | Location | API Endpoint |
|---|----------|----------|-------------|
| 1 | `build_events_list_url` | src/services/calendar/events.rs | `GET /calendar/v3/calendars/{id}/events` |
| 2 | `build_event_get_url` | src/services/calendar/events.rs | `GET /calendar/v3/calendars/{id}/events/{id}` |
| 3 | `build_event_create_body` | src/services/calendar/events.rs | Event creation JSON body |
| 4 | `build_event_create_url` | src/services/calendar/events.rs | `POST /calendar/v3/calendars/{id}/events` |
| 5 | `build_event_update_url` | src/services/calendar/events.rs | `PUT /calendar/v3/calendars/{id}/events/{id}` |
| 6 | `build_event_delete_url` | src/services/calendar/events.rs | `DELETE /calendar/v3/calendars/{id}/events/{id}` |
| 7 | `find_conflicts` | src/services/calendar/events.rs | Detect scheduling conflicts in event list |
| 8 | `build_calendars_list_url` | src/services/calendar/calendars.rs | `GET /calendar/v3/users/me/calendarList` |
| 9 | `build_acl_list_url` | src/services/calendar/calendars.rs | `GET /calendar/v3/calendars/{id}/acl` |
| 10 | `resolve_calendar_id` | src/services/calendar/calendars.rs | Resolve calendar name to ID |
| 11 | `build_freebusy_request` | src/services/calendar/freebusy.rs | FreeBusy request body |
| 12 | `build_freebusy_url` | src/services/calendar/freebusy.rs | `POST /calendar/v3/freeBusy` |
| 13 | `build_rsvp_body` | src/services/calendar/respond.rs | RSVP response body |
| 14 | `build_rsvp_url` | src/services/calendar/respond.rs | RSVP update URL |
| 15 | `validate_rsvp_status` | src/services/calendar/respond.rs | Validate RSVP status string |
| 16 | `build_cross_calendar_search_params` | src/services/calendar/search.rs | Cross-calendar search parameters |
| 17 | `build_focus_time_event` | src/services/calendar/special.rs | Focus Time event body |
| 18 | `build_ooo_event` | src/services/calendar/special.rs | Out-of-office event body |
| 19 | `build_working_location_event` | src/services/calendar/special.rs | Working location event body |
| 20 | `validate_location_type` | src/services/calendar/special.rs | Validate location type string |
| 21 | `build_colors_url` | src/services/calendar/colors.rs | `GET /calendar/v3/colors` |
| 22 | `build_calendar_watch_url` | src/services/calendar/watch.rs | `POST /calendar/v3/calendars/{id}/events/watch` |
| 23 | `build_calendar_stop_url` | src/services/calendar/watch.rs | `POST /calendar/v3/channels/stop` |

## Utility Functions

| # | Function | Location | Description |
|---|----------|----------|-------------|
| 1 | `day_of_week` | src/services/calendar/types.rs | Day of week from date |
| 2 | `resolve_time_range` | src/services/calendar/types.rs | Resolve natural time range strings |
| 3 | `event_url` | src/services/calendar/types.rs | Generate event web URL |
| 4 | `propose_time_url` | src/services/calendar/types.rs | Generate propose-new-time URL |

## Types

| # | Name | Kind | Location | Description |
|---|------|------|----------|-------------|
| 1 | CalendarListResponse | Struct | src/services/calendar/types.rs | Calendar list |
| 2 | CalendarListEntry | Struct | src/services/calendar/types.rs | Calendar entry |
| 3 | EventListResponse | Struct | src/services/calendar/types.rs | Event list |
| 4 | Event | Struct | src/services/calendar/types.rs | Calendar event |
| 5 | EventDateTime | Struct | src/services/calendar/types.rs | Date or datetime with timezone |
| 6 | EventPerson | Struct | src/services/calendar/types.rs | Organizer/creator |
| 7 | Attendee | Struct | src/services/calendar/types.rs | Event attendee |
| 8 | FreeBusyRequest | Struct | src/services/calendar/types.rs | FreeBusy request |
| 9 | FreeBusyResponse | Struct | src/services/calendar/types.rs | FreeBusy response |
| 10 | FreeBusyCalendar | Struct | src/services/calendar/types.rs | Per-calendar free/busy |
| 11 | FreeBusyCalendarId | Struct | src/services/calendar/types.rs | Calendar ID wrapper |
| 12 | FreeBusyPeriod | Struct | src/services/calendar/types.rs | Busy time period |
| 13 | AclListResponse | Struct | src/services/calendar/types.rs | ACL list |
| 14 | AclRule | Struct | src/services/calendar/types.rs | ACL rule |
| 15 | AclScope | Struct | src/services/calendar/types.rs | ACL scope |
| 16 | ColorsResponse | Struct | src/services/calendar/types.rs | Color definitions |
| 17 | ColorDefinition | Struct | src/services/calendar/types.rs | Color entry |
| 18 | CalendarWatchArgs | Struct | src/cli/calendar.rs | CLI args for `calendar watch` |
| 19 | CalendarWatchCommand | Enum | src/cli/calendar.rs | Watch subcommands: Start, Stop, Status |
| 20 | CalendarWatchStartArgs | Struct | src/cli/calendar.rs | `--callback-url`, `--calendar` (default: primary) |
| 21 | CalendarWatchStopArgs | Struct | src/cli/calendar.rs | `--channel-id`, `--resource-id` |
