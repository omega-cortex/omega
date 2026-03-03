# Functionalities: Apps Script

## Overview
Google Apps Script API — project metadata, content (source files), function execution, and project creation.

## CLI Commands

| # | Command | Handler | Location | Description |
|---|---------|---------|----------|-------------|
| 1 | `appscript get <script_id>` | `handle_appscript_get` | src/cli/mod.rs:8905 | Get project metadata |
| 2 | `appscript content <script_id>` | `handle_appscript_content` | src/cli/mod.rs:8936 | Get project source files |
| 3 | `appscript run <script_id> <function>` | `handle_appscript_run` | src/cli/mod.rs:8967 | Execute function (with optional parameters) |
| 4 | `appscript create` | `handle_appscript_create` | src/cli/mod.rs:9019 | Create new project |

## URL/Body Builders

| # | Function | Location | Description |
|---|----------|----------|-------------|
| 1 | `build_project_get_url` | src/services/appscript/scripts.rs | Project metadata URL |
| 2 | `build_project_content_url` | src/services/appscript/scripts.rs | Project content URL |
| 3 | `build_script_run_url` | src/services/appscript/scripts.rs | Script execution URL |
| 4 | `build_script_run_body` | src/services/appscript/scripts.rs | Execution request body |
| 5 | `build_project_create_url` | src/services/appscript/scripts.rs | Project creation URL |
| 6 | `build_project_create_body` | src/services/appscript/scripts.rs | Project creation body |

## Types

| # | Name | Kind | Location | Description |
|---|------|------|----------|-------------|
| 1 | Project | Struct | src/services/appscript/types.rs | Script project |
| 2 | Content | Struct | src/services/appscript/types.rs | Project content |
| 3 | ScriptFile | Struct | src/services/appscript/types.rs | Source file |
| 4 | FunctionSet | Struct | src/services/appscript/types.rs | Set of functions |
| 5 | Function | Struct | src/services/appscript/types.rs | Individual function |
| 6 | ExecutionResponse | Struct | src/services/appscript/types.rs | Execution result |
| 7 | ExecutionError | Struct | src/services/appscript/types.rs | Execution error |
| 8 | Operation | Struct | src/services/appscript/types.rs | Async operation |
