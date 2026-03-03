# Functionalities: Slides

## Overview
Google Slides API — presentation info, slide listing, add/delete/read/replace slides, speaker notes, export (PPTX/PDF/PNG/SVG), create from template, and markdown-to-slides conversion.

## CLI Commands

| # | Command | Handler | Location | Description |
|---|---------|---------|----------|-------------|
| 1 | `slides export <id>` | `handle_slides_export` | src/cli/mod.rs:4769 | Export as PPTX/PDF/PNG/SVG |
| 2 | `slides info <id>` | `handle_slides_info` | src/cli/mod.rs:4819 | Get presentation metadata |
| 3 | `slides create <title>` | `handle_slides_create` | src/cli/mod.rs:4849 | Create presentation (optionally from template) |
| 4 | `slides create-from-markdown` | `handle_slides_create_from_markdown` | src/cli/mod.rs:4925 | Parse markdown into slide deck |
| 5 | `slides copy <id> <title>` | `handle_slides_copy` | src/cli/mod.rs:5027 | Copy presentation |
| 6 | `slides list-slides <id>` | `handle_slides_list_slides` | src/cli/mod.rs:5067 | List all slides with IDs |
| 7 | `slides add-slide <id>` | `handle_slides_add_slide` | src/cli/mod.rs:5116 | Add slide (layout, image, notes) |
| 8 | `slides delete-slide <id> <slide_id>` | `handle_slides_delete_slide` | src/cli/mod.rs:5158 | Delete slide |
| 9 | `slides read-slide <id> <slide_id>` | `handle_slides_read_slide` | src/cli/mod.rs:5202 | Read slide text content |
| 10 | `slides update-notes <id> <slide_id>` | `handle_slides_update_notes` | src/cli/mod.rs:5254 | Update speaker notes |
| 11 | `slides replace-slide <id> <slide_id>` | `handle_slides_replace_slide` | src/cli/mod.rs:5350 | Replace slide image |

## URL/Body Builders

| # | Function | Location | Description |
|---|----------|----------|-------------|
| 1 | `build_presentation_get_url` | src/services/slides/presentations.rs | Presentation metadata URL |
| 2 | `build_presentation_create_body` | src/services/slides/presentations.rs | Create body |
| 3 | `build_presentation_batch_update_url` | src/services/slides/presentations.rs | Batch update URL |
| 4 | `build_add_slide_request` | src/services/slides/slides_ops.rs | Add slide request |
| 5 | `build_delete_slide_request` | src/services/slides/slides_ops.rs | Delete slide request |
| 6 | `build_replace_image_request` | src/services/slides/slides_ops.rs | Replace slide image |
| 7 | `extract_slide_text` | src/services/slides/slides_ops.rs | Extract text from slide |
| 8 | `build_update_notes_request` | src/services/slides/notes.rs | Update speaker notes |
| 9 | `extract_speaker_notes` | src/services/slides/notes.rs | Extract speaker notes text |
| 10 | `build_export_url` | src/services/slides/export.rs | Export URL with format |

## Markdown Conversion

| # | Function | Location | Description |
|---|----------|----------|-------------|
| 1 | `parse_markdown_slides` | src/services/slides/markdown.rs | Parse markdown into slide structures |
| 2 | `build_slides_from_markdown` | src/services/slides/markdown.rs | Convert parsed slides to API requests |

## Types

| # | Name | Kind | Location | Description |
|---|------|------|----------|-------------|
| 1 | Presentation | Struct | src/services/slides/types.rs | Slide deck |
| 2 | Page | Struct | src/services/slides/types.rs | Slide page |
| 3 | PageElement | Struct | src/services/slides/types.rs | Element on slide |
| 4 | Shape | Struct | src/services/slides/types.rs | Shape element |
| 5 | TextContent | Struct | src/services/slides/types.rs | Text in shape |
| 6 | TextElement | Struct | src/services/slides/types.rs | Text element |
| 7 | TextRun | Struct | src/services/slides/types.rs | Text with style |
| 8 | TextStyle | Struct | src/services/slides/types.rs | Text formatting |
| 9 | SlideProperties | Struct | src/services/slides/types.rs | Slide metadata |
| 10 | NotesPage | Struct | src/services/slides/types.rs | Speaker notes page |
| 11 | NotesProperties | Struct | src/services/slides/types.rs | Notes metadata |
| 12 | PageSize | Struct | src/services/slides/types.rs | Slide dimensions |
| 13 | Dimension | Struct | src/services/slides/types.rs | Size value |
| 14 | Transform | Struct | src/services/slides/types.rs | Element transform |
| 15 | Size | Struct | src/services/slides/types.rs | Width/height |
| 16 | Image | Struct | src/services/slides/types.rs | Image element |
| 17 | SpeakerNotesText | Struct | src/services/slides/types.rs | Notes text |
| 18 | ParagraphMarker | Struct | src/services/slides/types.rs | Paragraph marker |
| 19 | Link | Struct | src/services/slides/types.rs | Hyperlink |
| 20 | Placeholder | Struct | src/services/slides/types.rs | Placeholder type |
| 21 | BatchUpdateResponse | Struct | src/services/slides/types.rs | Batch update response |
