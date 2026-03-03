# Functionalities: Sheets

## Overview
Google Sheets API — cell read/write with A1 notation, append rows, insert rows/columns, clear ranges, cell formatting (bold, color, alignment, borders), cell notes, spreadsheet metadata, create, copy, and export.

## CLI Commands

| # | Command | Handler | Location | Description |
|---|---------|---------|----------|-------------|
| 1 | `sheets get <id> <range>` | `handle_sheets_get` | src/cli/mod.rs:4217 | Read cell values from range |
| 2 | `sheets update <id> <range> <values>` | `handle_sheets_update` | src/cli/mod.rs:4277 | Write cell values to range |
| 3 | `sheets append <id> <range> <values>` | `handle_sheets_append` | src/cli/mod.rs:4327 | Append rows after range |
| 4 | `sheets insert <id> <sheet> <dim> <start>` | `handle_sheets_insert` | src/cli/mod.rs:4380 | Insert rows or columns |
| 5 | `sheets clear <id> <range>` | `handle_sheets_clear` | src/cli/mod.rs:4446 | Clear cell values in range |
| 6 | `sheets format <id> <range>` | `handle_sheets_format` | src/cli/mod.rs:4487 | Format cells (bold, italic, color, alignment, borders, number format) |
| 7 | `sheets notes <id> <range>` | `handle_sheets_notes` | src/cli/mod.rs:4553 | Read cell notes |
| 8 | `sheets metadata <id>` | `handle_sheets_metadata` | src/cli/mod.rs:4591 | Get spreadsheet metadata (sheets, properties) |
| 9 | `sheets create <title>` | `handle_sheets_create` | src/cli/mod.rs:4623 | Create new spreadsheet |
| 10 | `sheets copy <id> <title>` | `handle_sheets_copy` | src/cli/mod.rs:4668 | Copy spreadsheet |
| 11 | `sheets export <id>` | `handle_sheets_export` | src/cli/mod.rs:4707 | Export as XLSX/CSV/PDF/ODS/TSV |

## URL/Body Builders

| # | Function | Location | Description |
|---|----------|----------|-------------|
| 1 | `build_values_get_url` | src/services/sheets/read.rs | Read values URL |
| 2 | `build_values_update_url` | src/services/sheets/write.rs | Write values URL |
| 3 | `build_values_append_url` | src/services/sheets/write.rs | Append values URL |
| 4 | `build_values_clear_url` | src/services/sheets/write.rs | Clear values URL |
| 5 | `build_update_body` | src/services/sheets/write.rs | ValueRange body for update |
| 6 | `build_append_body` | src/services/sheets/write.rs | ValueRange body for append |
| 7 | `parse_values_input` | src/services/sheets/write.rs | Parse comma/pipe-delimited values |
| 8 | `build_insert_dimension_request` | src/services/sheets/structure.rs | Insert rows/columns request |
| 9 | `build_batch_update_url` | src/services/sheets/structure.rs | Batch update URL |
| 10 | `build_spreadsheet_create_url` | src/services/sheets/structure.rs | Create spreadsheet URL |
| 11 | `build_spreadsheet_create_body` | src/services/sheets/structure.rs | Create spreadsheet body |
| 12 | `build_metadata_url` | src/services/sheets/structure.rs | Spreadsheet metadata URL |
| 13 | `build_format_request` | src/services/sheets/format.rs | Cell formatting request |
| 14 | `build_repeat_cell_request` | src/services/sheets/format.rs | RepeatCell formatting |
| 15 | `parse_color` | src/services/sheets/format.rs | Parse hex color to RGB |
| 16 | `build_notes_url` | src/services/sheets/read.rs | Notes read URL |

## A1 Notation Parser

| # | Function | Location | Description |
|---|----------|----------|-------------|
| 1 | `parse_a1` | src/services/sheets/a1.rs | Parse A1 notation (e.g., "Sheet1!A1:B5") |
| 2 | `column_to_index` | src/services/sheets/a1.rs | Convert column letter to 0-based index |
| 3 | `index_to_column` | src/services/sheets/a1.rs | Convert 0-based index to column letter |
| 4 | `validate_range` | src/services/sheets/a1.rs | Validate A1 range string |

## Types

| # | Name | Kind | Location | Description |
|---|------|------|----------|-------------|
| 1 | Spreadsheet | Struct | src/services/sheets/types.rs | Spreadsheet metadata |
| 2 | SpreadsheetProperties | Struct | src/services/sheets/types.rs | Title, locale, etc |
| 3 | Sheet | Struct | src/services/sheets/types.rs | Sheet tab |
| 4 | SheetProperties | Struct | src/services/sheets/types.rs | Sheet name, index, type |
| 5 | GridProperties | Struct | src/services/sheets/types.rs | Row/column counts |
| 6 | GridData | Struct | src/services/sheets/types.rs | Grid data for notes |
| 7 | RowData | Struct | src/services/sheets/types.rs | Row of cells |
| 8 | CellData | Struct | src/services/sheets/types.rs | Cell with value + format |
| 9 | ExtendedValue | Struct | src/services/sheets/types.rs | Typed cell value |
| 10 | CellFormat | Struct | src/services/sheets/types.rs | Cell formatting |
| 11 | TextFormat | Struct | src/services/sheets/types.rs | Text style in cell |
| 12 | Color | Struct | src/services/sheets/types.rs | RGBA color |
| 13 | ValueRange | Struct | src/services/sheets/types.rs | Range of values |
| 14 | UpdateValuesResponse | Struct | src/services/sheets/types.rs | Update response |
| 15 | AppendValuesResponse | Struct | src/services/sheets/types.rs | Append response |
| 16 | BatchUpdateResponse | Struct | src/services/sheets/types.rs | Batch update response |
| 17 | ClearValuesResponse | Struct | src/services/sheets/types.rs | Clear response |
