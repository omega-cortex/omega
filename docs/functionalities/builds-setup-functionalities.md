# Functionalities: Builds / Setup

## Overview

Topology-driven build orchestrator with multi-phase execution (ParseBrief, Standard, CorrectiveLoop, ParseSummary) and project setup sessions with a Brain agent.

## Functionalities

| # | Name | Type | Location | Description | Dependencies |
|---|------|------|----------|-------------|--------------|
| 1 | handle_build_request() | Service | `backend/src/gateway/builds.rs` | Loads topology, writes agent files, iterates phases with state accumulation | builds_topology, builds_agents, builds_loop, builds_parse |
| 2 | run_build_phase() | Service | `backend/src/gateway/builds_loop.rs` | Runs a single build phase with 3-attempt retry using Claude Code CLI agent mode | Provider |
| 3 | Phase: ParseBrief | Build Phase | `backend/src/gateway/builds_loop.rs` | Analyst agent parses and enriches the brief, creates project directory | -- |
| 4 | Phase: Standard | Build Phase | `backend/src/gateway/builds_loop.rs` | Architect, test-writer, developer agents execute in sequence | -- |
| 5 | Phase: CorrectiveLoop | Build Phase | `backend/src/gateway/builds_loop.rs` | QA agent validates with retries, developer fixes issues | -- |
| 6 | Phase: ParseSummary | Build Phase | `backend/src/gateway/builds_loop.rs` | Delivery agent generates build summary | -- |
| 7 | OrchestratorState | Model | `backend/src/gateway/builds_loop.rs` | Accumulates state across phases (brief, architecture, tests, implementation, qa_results, summary) | -- |
| 8 | AgentFilesGuard | Utility | `backend/src/gateway/builds_agents.rs` | RAII guard for writing/cleaning up .claude/agents/ files from topology | -- |
| 9 | load_topology() | Service | `backend/src/gateway/builds_topology.rs` | Loads TOML topology definition for build phase sequencing | -- |
| 10 | start_setup_session() | Service | `backend/src/gateway/setup.rs` | Starts a setup session: concurrent guard, collision detection, Brain agent invocation, questions/proposal parsing | Provider, Store |
| 11 | execute_setup() | Service | `backend/src/gateway/setup.rs` | Two-phase setup execution: Brain creates HEARTBEAT.md + markers, Role Creator writes ROLE.md | Provider |
| 12 | handle_setup_response() | Service | `backend/src/gateway/setup_response.rs` | Handles user response to setup session: TTL check, cancellation, phase detection | -- |
| 13 | handle_setup_confirmation() | Service | `backend/src/gateway/setup_response.rs` | Handles setup confirmation: execute on confirm, modification request on other input | execute_setup |
| 14 | handle_setup_questioning() | Service | `backend/src/gateway/setup_response.rs` | Multi-round questioning (max 3 rounds), context accumulation | Provider |
| 15 | SetupOutput | Model | `backend/src/gateway/setup.rs` | Enum: Questions, Proposal, Executed | -- |

## Internal Dependencies

- handle_build_request() calls load_topology() -> AgentFilesGuard -> run_build_phase() per phase
- start_setup_session() -> handle_setup_response() -> handle_setup_confirmation()/handle_setup_questioning()

## Dead Code / Unused

- `#[allow(dead_code)]` on multiple agent definition structs in builds_agents.rs (lines 29-57, 70, 101)
- `#[allow(dead_code)]` on topology model fields in builds_topology.rs (lines 30, 37, 131, 153)
- `#[allow(dead_code)]` on build parse structs in builds_parse.rs (line 16)
- `#[allow(dead_code)]` on build loop state fields in builds_loop.rs (lines 132, 194, 264)
- SetupOutput::Executed variant (setup.rs:42) has #[allow(dead_code)]
