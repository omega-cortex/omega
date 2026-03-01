# Functionalities: Scheduler

## Overview

The scheduler polls for due tasks at a configurable interval and delivers them. Reminder tasks send a text message to the user. Action tasks invoke the AI provider with full tool access and process response markers.

## Functionalities

| # | Name | Type | Location | Description | Dependencies |
|---|------|------|----------|-------------|--------------|
| 1 | scheduler_loop() | Background Task | `backend/src/gateway/scheduler.rs:26` | Polls for due tasks at configurable interval; quiet hours gate defers tasks to next active_start; dispatches reminder and action tasks | Store, Channels, scheduler_action |
| 2 | Quiet hours deferral | Service | `backend/src/gateway/scheduler.rs:47` | When outside active hours, defers all due tasks to next active_start time | Store::defer_task, next_active_start_utc |
| 3 | Reminder task delivery | Service | `backend/src/gateway/scheduler.rs:98` | Sends reminder text to user via channel, then completes task (handles repeat) | Channel::send, Store::complete_task |
| 4 | Action task dispatch | Service | `backend/src/gateway/scheduler.rs:72` | Delegates action tasks to execute_action_task() for full provider execution | scheduler_action::execute_action_task |
| 5 | execute_action_task() | Service | `backend/src/gateway/scheduler_action.rs` | Full action task execution: builds system prompt with identity+soul+system, project ROLE.md, user profile, lessons, outcomes; runs provider with MCP servers; parses ACTION_OUTCOME; processes markers; audit logging; retry logic (MAX_ACTION_RETRIES) | Provider, Store, Prompts, skills |
| 6 | process_action_markers() | Service | `backend/src/gateway/scheduler_action.rs` | Processes markers from action task responses: SCHEDULE, SCHEDULE_ACTION, HEARTBEAT_*, CANCEL_TASK, UPDATE_TASK, REWARD, LESSON, PROJECT_*, FORGET_CONVERSATION | shared_markers, marker extractors |

## Internal Dependencies

- scheduler_loop() calls execute_action_task() for action tasks
- execute_action_task() builds a full context and calls provider.complete()
- process_action_markers() reuses shared_markers::process_task_and_learning_markers()

## Dead Code / Unused

- None detected.
