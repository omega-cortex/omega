# Functionalities Index

> Auto-generated inventory of all functionalities in the Omega codebase.
> Generated from code only -- specs and docs were not consulted.

## Summary

- **Total modules analyzed**: 18
- **Total functionalities found**: 154
- **Dead code items flagged**: 12 (mostly `#[allow(dead_code)]` on deserialization struct fields)

## Modules

| Module | File | Functionalities | Dead Code |
|--------|------|-----------------|-----------|
| omega-core | [omega-core-functionalities.md](omega-core-functionalities.md) | 35 | 0 |
| Gateway (Core + Pipeline + Routing) | [gateway-functionalities.md](gateway-functionalities.md) | 25 | 0 |
| Process Markers + Shared Markers | [markers-functionalities.md](markers-functionalities.md) | 23 | 2 |
| Scheduler | [scheduler-functionalities.md](scheduler-functionalities.md) | 6 | 0 |
| Heartbeat | [heartbeat-functionalities.md](heartbeat-functionalities.md) | 10 | 0 |
| Summarizer | [summarizer-functionalities.md](summarizer-functionalities.md) | 4 | 0 |
| Builds / Setup | [builds-setup-functionalities.md](builds-setup-functionalities.md) | 15 | 8 |
| Commands | [commands-functionalities.md](commands-functionalities.md) | 20 | 0 |
| Keywords | [keywords-functionalities.md](keywords-functionalities.md) | 13 | 0 |
| HTTP API | [api-functionalities.md](api-functionalities.md) | 8 | 0 |
| omega-providers | [providers-functionalities.md](providers-functionalities.md) | 12 | 2 |
| omega-channels | [channels-functionalities.md](channels-functionalities.md) | 5 | 0 |
| omega-memory | [memory-functionalities.md](memory-functionalities.md) | 14 | 0 |
| omega-skills | [skills-functionalities.md](skills-functionalities.md) | 11 | 0 |
| omega-sandbox | [sandbox-functionalities.md](sandbox-functionalities.md) | 5 | 0 |
| CLI / Main Binary | [cli-functionalities.md](cli-functionalities.md) | 17 | 0 |
| i18n | [i18n-functionalities.md](i18n-functionalities.md) | 5 | 0 |
| Supporting Modules | [supporting-functionalities.md](supporting-functionalities.md) | 8 | 0 |

## Cross-Module Dependencies

### Primary Message Flow

```
IncomingMessage (omega-channels)
  -> Gateway::dispatch_message() (gateway/mod.rs)
    -> Gateway::handle_message() (gateway/pipeline.rs)
      -> check_auth() (gateway/auth.rs)
      -> sanitize::sanitize() (omega-core)
      -> commands::handle() (commands/)
      -> build_system_prompt() (gateway/prompt_builder.rs)
      -> Store::build_context() (omega-memory)
      -> match_skill_triggers() (omega-skills)
      -> Provider::complete() (omega-providers)
      -> process_markers() (gateway/process_markers.rs)
      -> Store::store_exchange() (omega-memory)
      -> AuditLogger::log() (omega-memory)
      -> Channel::send() (omega-channels)
```

### Background Tasks

```
Gateway::run()
  -> background_summarizer() (gateway/summarizer.rs)
     -> Store::find_idle_conversations() -> Provider::complete() -> Store::close_conversation()
  -> scheduler_loop() (gateway/scheduler.rs)
     -> Store::get_due_tasks() -> Channel::send() (reminder) / execute_action_task() (action)
  -> heartbeat_loop() (gateway/heartbeat.rs)
     -> Provider::complete() (classify) -> Provider::complete() (execute per group)
  -> claudemd_loop() (claudemd.rs)
     -> refresh_claudemd() -> protected_command("claude")
```

### Marker Protocol

```
Provider response text
  -> process_markers() extracts structured markers
    -> SCHEDULE/SCHEDULE_ACTION -> Store::create_task()
    -> PROJECT_ACTIVATE/DEACTIVATE -> Store::store_fact()/delete_fact()
    -> LANG_SWITCH/PERSONALITY -> Store::store_fact()
    -> FORGET_CONVERSATION -> Store::close_current_conversation()
    -> HEARTBEAT_ADD/REMOVE/INTERVAL -> filesystem + config.toml patching
    -> SKILL_IMPROVE -> filesystem (SKILL.md update)
    -> BUG_REPORT -> filesystem (BUG.md append)
    -> REWARD/LESSON -> Store::store_outcome()/store_lesson()
    -> CANCEL_TASK/UPDATE_TASK -> Store::cancel_task()/update_task()
    -> BUILD_PROPOSAL -> Store::store_fact("pending_build_request")
  -> strip_all_remaining_markers() cleans text
  -> send_task_confirmation() sends anti-hallucination confirmation
```

### Build Pipeline

```
User message with build keywords
  -> handle_build_keyword_discovery() (gateway/pipeline_builds.rs)
    -> run_build_phase("build-discovery") -> parse_discovery_output()
    -> Multi-round Q&A or direct confirmation
  -> handle_pending_build_confirmation() (gateway/pipeline_builds.rs)
    -> handle_build_request() (gateway/builds.rs)
      -> load_topology() -> AgentFilesGuard
      -> Phase loop: ParseBrief -> Standard -> CorrectiveLoop -> ParseSummary
      -> Each phase: run_build_phase() with agent mode
```

### Setup Pipeline

```
/setup <description>
  -> start_setup_session() (gateway/setup.rs)
    -> Brain agent: questions/proposal
  -> handle_setup_response() (gateway/setup_response.rs)
    -> Multi-round questioning (max 3)
    -> handle_setup_confirmation()
      -> execute_setup() -> Brain (HEARTBEAT.md) + Role Creator (ROLE.md)
```

## Dead Code Summary

| Location | Item | Reason |
|----------|------|--------|
| `builds_agents.rs:29-57,70,101` | Agent definition struct fields | Deserialized from TOML but not all fields individually read |
| `builds_topology.rs:30,37,131,153` | Topology model struct fields | Deserialized from TOML but not all fields individually read |
| `builds_parse.rs:16` | Parse output struct fields | Constructed but not all fields individually read |
| `builds_loop.rs:132,194,264` | OrchestratorState fields | Accumulated state not always consumed |
| `setup.rs:42` | SetupOutput::Executed variant | Enum variant with field marked dead |
| `markers/schedule.rs:4,50` | Schedule marker struct fields | Parsed but not all fields individually read |
| `telegram/types.rs:19,31,40,47,56` | Telegram API response types | Deserialized from API but not all fields read |
| `mcp_client.rs:64,66,74` | MCP client response fields | Deserialized from JSON but not all fields read |
| `tools.rs:76` | Tool result fields | Deserialized but not all fields read |

All dead code items are struct fields populated by deserialization (serde/TOML/JSON) where the struct is used but individual fields are not accessed in code. None represent truly orphaned or unreachable functionality.
