# Functionalities: Forms

## Overview
Google Forms API — form metadata retrieval, form creation, and response listing/retrieval.

## CLI Commands

| # | Command | Handler | Location | Description |
|---|---------|---------|----------|-------------|
| 1 | `forms get <form_id>` | `handle_forms_get` | src/cli/mod.rs:5437 | Get form metadata (title, items, settings) |
| 2 | `forms create` | `handle_forms_create` | src/cli/mod.rs:5466 | Create new form |
| 3 | `forms responses list <form_id>` | `handle_forms_responses_list` | src/cli/mod.rs:5506 | List form responses |
| 4 | `forms responses get <form_id> <resp_id>` | `handle_forms_responses_get` | src/cli/mod.rs:5542 | Get specific response |

## URL/Body Builders

| # | Function | Location | Description |
|---|----------|----------|-------------|
| 1 | `build_form_get_url` | src/services/forms/forms.rs | Form metadata URL |
| 2 | `build_form_create_url` | src/services/forms/forms.rs | Form creation URL |
| 3 | `build_form_create_body` | src/services/forms/forms.rs | Form creation body |
| 4 | `build_responses_list_url` | src/services/forms/responses.rs | Responses list URL |
| 5 | `build_response_get_url` | src/services/forms/responses.rs | Single response URL |

## Types

| # | Name | Kind | Location | Description |
|---|------|------|----------|-------------|
| 1 | Form | Struct | src/services/forms/types.rs | Form metadata |
| 2 | FormInfo | Struct | src/services/forms/types.rs | Title, description |
| 3 | FormItem | Struct | src/services/forms/types.rs | Form question/item |
| 4 | FormResponse | Struct | src/services/forms/types.rs | Submitted response |
| 5 | Answer | Struct | src/services/forms/types.rs | Answer to a question |
| 6 | TextAnswers | Struct | src/services/forms/types.rs | Text answer container |
| 7 | TextAnswer | Struct | src/services/forms/types.rs | Text answer value |
| 8 | FormResponseList | Struct | src/services/forms/types.rs | Paginated response list |
