# Functionalities: Tasks

## Overview
Google Tasks API — task list management, task CRUD, mark done/undo, and clear completed tasks.

## CLI Commands

| # | Command | Handler | Location | Description |
|---|---------|---------|----------|-------------|
| 1 | `tasks lists list` | `handle_tasks_lists_list` | src/cli/mod.rs:7728 | List task lists |
| 2 | `tasks lists create <title>` | `handle_tasks_lists_create` | src/cli/mod.rs:7755 | Create task list |
| 3 | `tasks list <tasklist>` | `handle_tasks_list` | src/cli/mod.rs:7784 | List tasks in a list |
| 4 | `tasks get <tasklist> <task>` | `handle_tasks_get` | src/cli/mod.rs:7812 | Get task details |
| 5 | `tasks add <tasklist>` | `handle_tasks_add` | src/cli/mod.rs:7835 | Add task (title, notes, due date) |
| 6 | `tasks update <tasklist> <task>` | `handle_tasks_update` | src/cli/mod.rs:7870 | Update task fields |
| 7 | `tasks done <tasklist> <task>` | `handle_tasks_done` | src/cli/mod.rs:7904 | Mark task as completed |
| 8 | `tasks undo <tasklist> <task>` | `handle_tasks_undo` | src/cli/mod.rs:7935 | Mark task as not completed |
| 9 | `tasks delete <tasklist> <task>` | `handle_tasks_delete` | src/cli/mod.rs:7966 | Delete task |
| 10 | `tasks clear <tasklist>` | `handle_tasks_clear` | src/cli/mod.rs:7999 | Clear completed tasks from list |

## URL/Body Builders

| # | Function | Location | Description |
|---|----------|----------|-------------|
| 1 | `build_tasklists_list_url` | src/services/tasks/tasklists.rs | Task lists URL |
| 2 | `build_tasklist_create_url` | src/services/tasks/tasklists.rs | Create task list URL |
| 3 | `build_tasklist_create_body` | src/services/tasks/tasklists.rs | Create body |
| 4 | `build_tasks_list_url` | src/services/tasks/task_ops.rs | Tasks list URL |
| 5 | `build_task_get_url` | src/services/tasks/task_ops.rs | Task get URL |
| 6 | `build_task_create_url` | src/services/tasks/task_ops.rs | Task create URL |
| 7 | `build_task_create_body` | src/services/tasks/task_ops.rs | Task create body |
| 8 | `build_task_update_url` | src/services/tasks/task_ops.rs | Task update URL |
| 9 | `build_task_delete_url` | src/services/tasks/task_ops.rs | Task delete URL |
| 10 | `build_tasks_clear_url` | src/services/tasks/task_ops.rs | Clear completed URL |
| 11 | `build_task_status_body` | src/services/tasks/task_ops.rs | Done/undo status body |

## Types

| # | Name | Kind | Location | Description |
|---|------|------|----------|-------------|
| 1 | TaskList | Struct | src/services/tasks/types.rs | Task list metadata |
| 2 | TaskListsResponse | Struct | src/services/tasks/types.rs | Task list collection |
| 3 | Task | Struct | src/services/tasks/types.rs | Task item |
| 4 | TaskLink | Struct | src/services/tasks/types.rs | Task link |
| 5 | TasksResponse | Struct | src/services/tasks/types.rs | Task collection |
