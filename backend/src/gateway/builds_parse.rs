//! Pure parsing functions, data structures, and prompt templates for the build pipeline.

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

/// Parsed output from Phase 1 (Clarification).
///
/// Fields beyond `name` and `scope` are parsed for completeness and passed to Phase 2
/// via the raw `brief_text` string. They are available for future orchestrator logic
/// (e.g., conditional frontend phase, language-specific verification commands).
#[allow(dead_code)]
pub(super) struct ProjectBrief {
    pub(super) name: String,
    pub(super) language: String,
    pub(super) database: String,
    pub(super) frontend: bool,
    pub(super) scope: String,
    pub(super) components: Vec<String>,
}

/// Result of Phase 4 (Verification).
pub(super) enum VerificationResult {
    Pass,
    Fail(String),
}

/// Parsed output from Phase 5 (Delivery).
pub(super) struct BuildSummary {
    pub(super) project: String,
    pub(super) location: String,
    pub(super) language: String,
    pub(super) summary: String,
    pub(super) usage: String,
    pub(super) skill: Option<String>,
}

// ---------------------------------------------------------------------------
// Phase prompt templates
// ---------------------------------------------------------------------------

pub(super) const PHASE_1_PROMPT: &str = "\
You are analyzing a build request. Extract and decide:
- Project name (kebab-case, max 3 words, descriptive)
- Programming language (default: Rust unless user specifies otherwise)
- Database (default: SQLite unless user specifies otherwise)
- Scope summary (1-3 sentences of what the project does)
- Key components (list of modules/features)
- Whether a frontend is needed (yes/no)

The project will follow a CLI-first architecture: every feature is a CLI subcommand \
and ALL output is structured JSON. Keep this in mind when defining components and scope.

Do NOT ask questions. Make reasonable defaults for anything ambiguous.
Output ONLY a structured brief in this exact format:

PROJECT_NAME: <name>
LANGUAGE: <language>
DATABASE: <database>
FRONTEND: <yes/no>
SCOPE: <1-3 sentence description>
COMPONENTS:
- <component 1>
- <component 2>";

pub(super) const PHASE_2_TEMPLATE: &str = "\
You are designing the architecture for a software project. You have full tool access.
Your working directory is: {project_dir}

Project brief:
{brief_text}

## Directory Convention (mandatory)

Create exactly this directory tree:
```
{project_dir}/
  specs/            # Technical specifications
  docs/             # User-facing documentation
  backend/          # All server/CLI source code
  backend/data/db/  # SQLite or other DB files (if applicable)
  backend/scripts/  # Test and verification scripts
  frontend/         # Only if FRONTEND: yes in the brief
```

## Architecture Convention — CLI-first, JSON output

- The project is a CLI application. Every feature = a CLI subcommand.
- ALL output from every command MUST be structured JSON:
  - Success: {{\"status\":\"ok\",\"data\":...}}
  - Error:   {{\"status\":\"error\",\"message\":\"...\"}}
- No plain-text or human-readable output. JSON only.

## Your Tasks

1. Create the directory structure above.
2. Initialize the project for {language}:
   - Rust   → `cargo init` inside `backend/`, set up Cargo.toml with dependencies.
   - Python → create `backend/pyproject.toml` with project metadata.
   - TypeScript → `npm init -y` inside `backend/`, set up package.json.
   - Other  → appropriate init for the language.
3. Write `specs/architecture.md` with module descriptions, data flow, CLI subcommands.
4. Write `specs/requirements.md` with functional requirements.
5. Write `specs/commands.md` documenting EVERY CLI subcommand and its JSON output structure. \
Example entry:
   ```
   ## {project_name} list
   Lists all items.
   Output: {{\"status\":\"ok\",\"data\":[{{\"id\":1,\"name\":\"...\"}},...]}}
   ```
6. Create stub files for each module (empty files with doc comments only).

Do NOT implement any logic. Only create structure and specifications.
When done, output: ARCHITECTURE_COMPLETE";

pub(super) const PHASE_3_TEMPLATE: &str = "\
You are implementing a software project. You have full tool access.
Your working directory is: {project_dir}

## Mandatory Conventions

- **CLI-first**: every feature = a CLI subcommand.
- **JSON output**: ALL command output follows this convention:
  - Success: {{\"status\":\"ok\",\"data\":...}}
  - Error:   {{\"status\":\"error\",\"message\":\"...\"}}
- **No file may exceed 500 lines** (excluding tests). Split into modules proactively.
- **Test scripts**: for every major feature, create a test script in `backend/scripts/` \
that invokes the CLI and validates JSON output. Scripts must be executable (`chmod +x`).

## Your Tasks

1. Read `specs/architecture.md`, `specs/requirements.md`, and `specs/commands.md`.
2. Implement each module described in the architecture, one at a time.
3. Every CLI subcommand MUST produce JSON matching the structure in `specs/commands.md`.
4. Write unit tests alongside the code.
5. Create test scripts in `backend/scripts/` (e.g. `backend/scripts/test-{project_name}.sh`) \
that exercise every CLI subcommand and verify JSON output via `jq`.
6. Ensure all code compiles.

Do NOT write documentation. Do NOT create skills. Focus only on working code.
When done, output: IMPLEMENTATION_COMPLETE";

pub(super) const PHASE_3_RETRY_TEMPLATE: &str = "\
You are fixing a software project that failed verification. You have full tool access.
Your working directory is: {project_dir}

The previous verification found these issues:
{failure_reason}

Read the code, fix the issues, and ensure:
1. The code compiles without errors
2. All lint warnings are fixed
3. All tests pass
4. All CLI commands output valid JSON ({{\"status\":\"ok\",...}} or {{\"status\":\"error\",...}})

When done, output: IMPLEMENTATION_COMPLETE";

pub(super) const PHASE_4_TEMPLATE: &str = "\
You are verifying a software project. You have full tool access.
Your working directory is: {project_dir}

## Step 1 — Build and Lint

Run the language-appropriate build and lint commands:
- **Rust**:       `cargo build` then `cargo clippy --workspace` — fix ALL warnings.
- **Python**:     `python -m py_compile` on all .py files, then `ruff check .` or `flake8`.
- **TypeScript**: `npx tsc --noEmit` then `npx eslint .`.
- **Other**:      use the standard build + lint for {language}.

All must pass with zero errors and zero warnings.

## Step 2 — Unit Tests

Run the language-appropriate test runner:
- **Rust**:       `cargo test --workspace`
- **Python**:     `pytest`
- **TypeScript**: `npx jest` or `npx vitest run`
- **Other**:      standard test runner for {language}.

## Step 3 — Integration / Script Tests

Run every executable script in `backend/scripts/`:
```
for f in backend/scripts/*.sh; do bash \"$f\"; done
```
Each script should exit 0. If any script fails, fix the underlying code.

## Step 4 — JSON Output Compliance

For every CLI subcommand listed in `specs/commands.md`:
1. Run the command.
2. Pipe output through `echo '<output>' | jq .` to validate it is valid JSON.
3. Verify the top-level key `\"status\"` is either `\"ok\"` or `\"error\"`.

If any command produces non-JSON output or missing status key, fix it.

## Reporting

After all checks pass, output exactly: VERIFICATION: PASS
If you cannot fix the issues, output: VERIFICATION: FAIL followed by REASON: <brief description>";

pub(super) const PHASE_5_TEMPLATE: &str = "\
You are delivering a completed software project. You have full tool access.
Your working directory is: {project_dir}

## Task 1 — User Documentation

Create `docs/README.md` with:
- Project name and one-line description
- Installation / build instructions for {language}
- Quick-start example

Create `docs/commands.md` with a user-friendly version of every CLI subcommand, \
including example invocations and example JSON output.

## Task 2 — Skill File

Create a skill file at `{skills_dir}/{project_name}/SKILL.md` using this exact format:

```
---
name: {project_name}
description: <one-line description of what the project does>
trigger: <keyword1>|<keyword2>|<keyword3>
---

## Binary

Path: `{project_dir}/backend/target/release/{project_name}` (for Rust) \
or equivalent entry point for {language}.

## Database

Path: `{project_dir}/backend/data/db/{project_name}.db` (if applicable, otherwise omit section).

## Commands

### {project_name} <subcommand1>
<description>
```bash
{project_name} <subcommand1> [args]
```
Output:
```json
{{\"status\":\"ok\",\"data\":...}}
```

(Repeat for every subcommand documented in specs/commands.md)

## Extending This Skill

To add a new command:
1. Implement the subcommand in the source code.
2. Add its JSON schema to `specs/commands.md`.
3. Add a section above with usage and output example.
4. Create a test script in `backend/scripts/`.
```

## Task 3 — Final Summary

Output the summary in this format:
BUILD_COMPLETE
PROJECT: <name>
LOCATION: <full path>
LANGUAGE: <language>
SUMMARY: <2-3 sentences>
USAGE: <primary CLI command or entry point>
SKILL: <skill name>";

// ---------------------------------------------------------------------------
// Pure parsing functions (testable without mocking)
// ---------------------------------------------------------------------------

/// Parse structured output from Phase 1 into a `ProjectBrief`.
pub(super) fn parse_project_brief(text: &str) -> Option<ProjectBrief> {
    let get_field = |key: &str| -> Option<String> {
        text.lines()
            .find(|line| line.starts_with(&format!("{key}:")))
            .map(|line| line[key.len() + 1..].trim().to_string())
    };

    let name = get_field("PROJECT_NAME")?;
    if name.is_empty()
        || name.contains('/')
        || name.contains('\\')
        || name.contains("..")
        || name.starts_with('.')
    {
        return None;
    }

    let language = get_field("LANGUAGE").unwrap_or_else(|| "Rust".to_string());
    let database = get_field("DATABASE").unwrap_or_else(|| "SQLite".to_string());
    let frontend = get_field("FRONTEND")
        .map(|v| v.to_lowercase().starts_with('y'))
        .unwrap_or(false);
    let scope = get_field("SCOPE").unwrap_or_else(|| "A software project.".to_string());

    let components: Vec<String> = text
        .lines()
        .skip_while(|line| !line.starts_with("COMPONENTS:"))
        .skip(1)
        .take_while(|line| line.starts_with("- "))
        .map(|line| line[2..].trim().to_string())
        .collect();

    Some(ProjectBrief {
        name,
        language,
        database,
        frontend,
        scope,
        components,
    })
}

/// Parse Phase 4 verification output into a pass/fail result.
pub(super) fn parse_verification_result(text: &str) -> VerificationResult {
    if text.contains("VERIFICATION: PASS") {
        VerificationResult::Pass
    } else if let Some(reason_line) = text.lines().find(|l| l.starts_with("REASON:")) {
        VerificationResult::Fail(reason_line["REASON:".len()..].trim().to_string())
    } else if text.contains("VERIFICATION: FAIL") {
        VerificationResult::Fail("Verification failed (no reason provided)".to_string())
    } else {
        // No marker found — treat as failure to avoid silently passing a broken build.
        VerificationResult::Fail("No verification marker found in response".to_string())
    }
}

/// Parse Phase 5 delivery output into a `BuildSummary`.
pub(super) fn parse_build_summary(text: &str) -> Option<BuildSummary> {
    if !text.contains("BUILD_COMPLETE") {
        return None;
    }

    let get_field = |key: &str| -> Option<String> {
        text.lines()
            .find(|line| line.starts_with(&format!("{key}:")))
            .map(|line| line[key.len() + 1..].trim().to_string())
    };

    Some(BuildSummary {
        project: get_field("PROJECT").unwrap_or_default(),
        location: get_field("LOCATION").unwrap_or_default(),
        language: get_field("LANGUAGE").unwrap_or_default(),
        summary: get_field("SUMMARY").unwrap_or_default(),
        usage: get_field("USAGE").unwrap_or_default(),
        skill: get_field("SKILL").filter(|s| !s.is_empty()),
    })
}

/// Localized phase progress message for the 7-phase build pipeline.
///
/// | Phase | Agent          | English                 |
/// |-------|----------------|-------------------------|
/// | 1     | Analyst        | Analyzing requirements  |
/// | 2     | Architect      | Designing architecture  |
/// | 3     | Test Writer    | Writing tests           |
/// | 4     | Developer      | Implementing code       |
/// | 5     | QA             | Validating quality      |
/// | 6     | Reviewer       | Reviewing code          |
/// | 7     | Delivery       | Preparing delivery      |
pub(super) fn phase_message(lang: &str, phase: u8, action: &str) -> String {
    match lang {
        "Spanish" => match phase {
            1 => "Analizando requisitos...".to_string(),
            2 => "Dise\u{f1}ando arquitectura...".to_string(),
            3 => "Escribiendo pruebas...".to_string(),
            4 => "Implementando c\u{f3}digo...".to_string(),
            5 => "Validando calidad...".to_string(),
            6 => "Revisando c\u{f3}digo...".to_string(),
            7 => "Preparando entrega...".to_string(),
            _ => format!("Fase {phase}: {action}..."),
        },
        "Portuguese" => match phase {
            1 => "Analisando requisitos...".to_string(),
            2 => "Projetando arquitetura...".to_string(),
            3 => "Escrevendo testes...".to_string(),
            4 => "Implementando c\u{f3}digo...".to_string(),
            5 => "Validando qualidade...".to_string(),
            6 => "Revisando c\u{f3}digo...".to_string(),
            7 => "Preparando entrega...".to_string(),
            _ => format!("Fase {phase}: {action}..."),
        },
        "French" => match phase {
            1 => "Analyse des exigences...".to_string(),
            2 => "Conception de l'architecture...".to_string(),
            3 => "R\u{e9}daction des tests...".to_string(),
            4 => "Impl\u{e9}mentation du code...".to_string(),
            5 => "Validation de la qualit\u{e9}...".to_string(),
            6 => "R\u{e9}vision du code...".to_string(),
            7 => "Pr\u{e9}paration de la livraison...".to_string(),
            _ => format!("Phase {phase}\u{a0}: {action}..."),
        },
        "German" => match phase {
            1 => "Analysiere Anforderungen...".to_string(),
            2 => "Architektur entwerfen...".to_string(),
            3 => "Tests schreiben...".to_string(),
            4 => "Code implementieren...".to_string(),
            5 => "Qualit\u{e4}t validieren...".to_string(),
            6 => "Code \u{fc}berpr\u{fc}fen...".to_string(),
            7 => "Lieferung vorbereiten...".to_string(),
            _ => format!("Phase {phase}: {action}..."),
        },
        "Italian" => match phase {
            1 => "Analisi dei requisiti...".to_string(),
            2 => "Progettazione dell'architettura...".to_string(),
            3 => "Scrittura dei test...".to_string(),
            4 => "Implementazione del codice...".to_string(),
            5 => "Validazione della qualit\u{e0}...".to_string(),
            6 => "Revisione del codice...".to_string(),
            7 => "Preparazione della consegna...".to_string(),
            _ => format!("Fase {phase}: {action}..."),
        },
        "Dutch" => match phase {
            1 => "Vereisten analyseren...".to_string(),
            2 => "Architectuur ontwerpen...".to_string(),
            3 => "Tests schrijven...".to_string(),
            4 => "Code implementeren...".to_string(),
            5 => "Kwaliteit valideren...".to_string(),
            6 => "Code reviewen...".to_string(),
            7 => "Levering voorbereiden...".to_string(),
            _ => format!("Fase {phase}: {action}..."),
        },
        "Russian" => match phase {
            1 => "\u{410}\u{43d}\u{430}\u{43b}\u{438}\u{437} \u{442}\u{440}\u{435}\u{431}\u{43e}\u{432}\u{430}\u{43d}\u{438}\u{439}...".to_string(),
            2 => "\u{41f}\u{440}\u{43e}\u{435}\u{43a}\u{442}\u{438}\u{440}\u{43e}\u{432}\u{430}\u{43d}\u{438}\u{435} \u{430}\u{440}\u{445}\u{438}\u{442}\u{435}\u{43a}\u{442}\u{443}\u{440}\u{44b}...".to_string(),
            3 => "\u{41d}\u{430}\u{43f}\u{438}\u{441}\u{430}\u{43d}\u{438}\u{435} \u{442}\u{435}\u{441}\u{442}\u{43e}\u{432}...".to_string(),
            4 => "\u{420}\u{435}\u{430}\u{43b}\u{438}\u{437}\u{430}\u{446}\u{438}\u{44f} \u{43a}\u{43e}\u{434}\u{430}...".to_string(),
            5 => "\u{41f}\u{440}\u{43e}\u{432}\u{435}\u{440}\u{43a}\u{430} \u{43a}\u{430}\u{447}\u{435}\u{441}\u{442}\u{432}\u{430}...".to_string(),
            6 => "\u{41e}\u{431}\u{437}\u{43e}\u{440} \u{43a}\u{43e}\u{434}\u{430}...".to_string(),
            7 => "\u{41f}\u{43e}\u{434}\u{433}\u{43e}\u{442}\u{43e}\u{432}\u{43a}\u{430} \u{43a} \u{434}\u{43e}\u{441}\u{442}\u{430}\u{432}\u{43a}\u{435}...".to_string(),
            _ => format!("\u{424}\u{430}\u{437}\u{430} {phase}: {action}..."),
        },
        // English and any unknown language
        _ => match phase {
            1 => "Analyzing requirements...".to_string(),
            2 => "Designing architecture...".to_string(),
            3 => "Writing tests...".to_string(),
            4 => "Implementing code...".to_string(),
            5 => "Validating quality...".to_string(),
            6 => "Reviewing code...".to_string(),
            7 => "Preparing delivery...".to_string(),
            _ => format!("Phase {phase}: {action}..."),
        },
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_project_brief_valid() {
        let text = "PROJECT_NAME: price-tracker\nLANGUAGE: Rust\nDATABASE: SQLite\nFRONTEND: no\nSCOPE: A CLI tool that tracks cryptocurrency prices.\nCOMPONENTS:\n- price fetcher\n- storage engine\n- alert system";
        let brief = parse_project_brief(text).unwrap();
        assert_eq!(brief.name, "price-tracker");
        assert_eq!(brief.language, "Rust");
        assert_eq!(brief.database, "SQLite");
        assert!(!brief.frontend);
        assert!(brief.scope.contains("cryptocurrency"));
        assert_eq!(brief.components.len(), 3);
    }

    #[test]
    fn test_parse_project_brief_minimal() {
        let text = "PROJECT_NAME: my-tool\nSCOPE: Does stuff";
        let brief = parse_project_brief(text).unwrap();
        assert_eq!(brief.name, "my-tool");
        assert_eq!(brief.language, "Rust"); // default
        assert_eq!(brief.database, "SQLite"); // default
        assert!(!brief.frontend); // default
    }

    #[test]
    fn test_parse_project_brief_missing_name() {
        let text = "LANGUAGE: Python\nSCOPE: A web scraper";
        assert!(parse_project_brief(text).is_none());
    }

    #[test]
    fn test_parse_project_brief_empty_name() {
        let text = "PROJECT_NAME: \nLANGUAGE: Rust";
        assert!(parse_project_brief(text).is_none());
    }

    #[test]
    fn test_parse_project_brief_path_traversal_rejected() {
        assert!(parse_project_brief("PROJECT_NAME: ../../../etc\nSCOPE: evil").is_none());
        assert!(parse_project_brief("PROJECT_NAME: foo/bar\nSCOPE: evil").is_none());
        assert!(parse_project_brief("PROJECT_NAME: .hidden\nSCOPE: evil").is_none());
        assert!(parse_project_brief("PROJECT_NAME: foo\\bar\nSCOPE: evil").is_none());
    }

    #[test]
    fn test_parse_project_brief_with_frontend() {
        let text = "PROJECT_NAME: dashboard\nFRONTEND: yes\nSCOPE: A web dashboard";
        let brief = parse_project_brief(text).unwrap();
        assert!(brief.frontend);
    }

    #[test]
    fn test_parse_project_brief_components_parsing() {
        let text =
            "PROJECT_NAME: my-app\nCOMPONENTS:\n- auth module\n- api layer\n- database\nSome other text";
        let brief = parse_project_brief(text).unwrap();
        assert_eq!(
            brief.components,
            vec!["auth module", "api layer", "database"]
        );
    }

    #[test]
    fn test_parse_verification_pass() {
        let text = "All tests passed.\n\nVERIFICATION: PASS";
        assert!(matches!(
            parse_verification_result(text),
            VerificationResult::Pass
        ));
    }

    #[test]
    fn test_parse_verification_fail_with_reason() {
        let text = "VERIFICATION: FAIL\nREASON: cargo test failed with 3 errors";
        match parse_verification_result(text) {
            VerificationResult::Fail(reason) => assert!(reason.contains("3 errors")),
            _ => panic!("expected Fail"),
        }
    }

    #[test]
    fn test_parse_verification_fail_no_reason() {
        let text = "VERIFICATION: FAIL";
        match parse_verification_result(text) {
            VerificationResult::Fail(reason) => assert!(reason.contains("no reason")),
            _ => panic!("expected Fail"),
        }
    }

    #[test]
    fn test_parse_verification_no_marker_implicit_fail() {
        let text = "Fixed all issues. Everything compiles now.";
        match parse_verification_result(text) {
            VerificationResult::Fail(reason) => {
                assert!(reason.contains("No verification marker"))
            }
            _ => panic!("expected Fail when no marker present"),
        }
    }

    #[test]
    fn test_parse_build_summary_valid() {
        let text = "BUILD_COMPLETE\nPROJECT: price-tracker\nLOCATION: /home/user/.omega/workspace/builds/price-tracker\nLANGUAGE: Rust\nSUMMARY: A CLI tool for tracking crypto prices with alerts.\nUSAGE: price-tracker watch BTC\nSKILL: price-tracker";
        let summary = parse_build_summary(text).unwrap();
        assert_eq!(summary.project, "price-tracker");
        assert!(summary.location.contains("price-tracker"));
        assert_eq!(summary.language, "Rust");
        assert!(summary.summary.contains("crypto"));
        assert_eq!(summary.usage, "price-tracker watch BTC");
        assert_eq!(summary.skill, Some("price-tracker".to_string()));
    }

    #[test]
    fn test_parse_build_summary_no_marker() {
        let text = "Here's what I built: a price tracker tool.";
        assert!(parse_build_summary(text).is_none());
    }

    #[test]
    fn test_parse_build_summary_no_skill() {
        let text = "BUILD_COMPLETE\nPROJECT: one-off\nLOCATION: /tmp/one-off\nLANGUAGE: Python\nSUMMARY: A quick script\nUSAGE: python main.py\nSKILL: ";
        let summary = parse_build_summary(text).unwrap();
        assert_eq!(summary.skill, None); // empty string filtered out
    }

    #[test]
    fn test_phase_message_english() {
        let msg = phase_message("English", 1, "analyzing");
        assert!(msg.contains("Analyzing"));
    }

    #[test]
    fn test_phase_message_spanish() {
        let msg = phase_message("Spanish", 1, "analyzing");
        assert!(msg.contains("Analizando"));
    }

    #[test]
    fn test_phase_message_all_languages() {
        // Phase 1 (Analyst) — each language has a custom string
        assert!(phase_message("Portuguese", 1, "").contains("Analisando"));
        assert!(phase_message("French", 1, "").contains("Analyse"));
        assert!(phase_message("German", 1, "").contains("Analysiere"));
        assert!(phase_message("Italian", 1, "").contains("Analisi"));
        assert!(phase_message("Dutch", 1, "").contains("analyseren"));
        assert!(phase_message("Russian", 1, "").contains("Анализ"));

        // Phase 7 (Delivery) — moved from phase 5 in the 7-phase pipeline
        assert!(phase_message("French", 7, "").contains("livraison"));
        assert!(phase_message("German", 7, "").contains("Lieferung"));
        assert!(phase_message("Italian", 7, "").contains("consegna"));
        assert!(phase_message("Dutch", 7, "").contains("Levering"));
        assert!(phase_message("Russian", 7, "").contains("доставке"));

        // Phase 3 (Test Writer) — now has custom messages per language
        assert!(phase_message("French", 3, "").contains("tests"));
        assert!(phase_message("German", 3, "").contains("Tests"));
        assert!(phase_message("Italian", 3, "").contains("test"));
        assert!(phase_message("Dutch", 3, "").contains("Tests"));
        assert!(phase_message("Russian", 3, "").contains("тестов"));
    }

    // =======================================================================
    // REQ-BAP-010 (Must): Preserve existing parse functions — regression
    // =======================================================================
    //
    // These tests lock the CURRENT behavior of parse functions. They must
    // pass both before and after the build agent pipeline implementation.

    // Requirement: REQ-BAP-010 (Must)
    // Acceptance: parse_project_brief remains functional
    // Edge case: extra whitespace around field values
    #[test]
    fn test_regression_parse_project_brief_whitespace_in_values() {
        let text = "PROJECT_NAME:   my-tool  \nLANGUAGE:  Python \nSCOPE: Does stuff";
        let brief = parse_project_brief(text).unwrap();
        assert_eq!(brief.name, "my-tool", "Should trim whitespace from name");
        assert_eq!(
            brief.language, "Python",
            "Should trim whitespace from language"
        );
    }

    // Requirement: REQ-BAP-010 (Must)
    // Edge case: multiline text with BUILD_COMPLETE embedded in middle
    #[test]
    fn test_regression_parse_project_brief_no_false_positive() {
        // Brief text that also happens to contain BUILD_COMPLETE should still
        // parse as a brief, not as a build summary.
        let text = "PROJECT_NAME: my-tool\nSCOPE: Does stuff\nBUILD_COMPLETE";
        let brief = parse_project_brief(text);
        assert!(brief.is_some(), "Brief should still parse even with BUILD_COMPLETE present");
    }

    // Requirement: REQ-BAP-010 (Must)
    // Edge case: VERIFICATION: PASS appears multiple times
    #[test]
    fn test_regression_parse_verification_multiple_pass_markers() {
        let text = "Phase 1: VERIFICATION: PASS\nPhase 2: VERIFICATION: PASS";
        assert!(matches!(
            parse_verification_result(text),
            VerificationResult::Pass
        ));
    }

    // Requirement: REQ-BAP-010 (Must)
    // Edge case: VERIFICATION: FAIL with REASON on non-adjacent line
    #[test]
    fn test_regression_parse_verification_reason_non_adjacent() {
        let text = "VERIFICATION: FAIL\nSome other output\nREASON: tests fail";
        match parse_verification_result(text) {
            VerificationResult::Fail(reason) => {
                assert!(
                    reason.contains("tests fail"),
                    "Should find REASON even if not adjacent to FAIL"
                );
            }
            _ => panic!("expected Fail"),
        }
    }

    // Requirement: REQ-BAP-010 (Must)
    // Edge case: empty text to all three parse functions
    #[test]
    fn test_regression_parse_functions_empty_input() {
        assert!(parse_project_brief("").is_none(), "Empty string -> None");
        match parse_verification_result("") {
            VerificationResult::Fail(reason) => {
                assert!(reason.contains("No verification marker"));
            }
            _ => panic!("Empty string should fail verification"),
        }
        assert!(
            parse_build_summary("").is_none(),
            "Empty string -> None for build summary"
        );
    }

    // Requirement: REQ-BAP-010 (Must)
    // Edge case: very large input (stress test)
    #[test]
    fn test_regression_parse_project_brief_large_input() {
        let mut text = String::from("PROJECT_NAME: huge-project\nSCOPE: Test\nCOMPONENTS:\n");
        for i in 0..1000 {
            text.push_str(&format!("- component {i}\n"));
        }
        let brief = parse_project_brief(&text).unwrap();
        assert_eq!(brief.name, "huge-project");
        assert_eq!(brief.components.len(), 1000);
    }

    // Requirement: REQ-BAP-010 (Must)
    // Edge case: unicode in project name
    #[test]
    fn test_regression_parse_project_brief_unicode_scope() {
        let text =
            "PROJECT_NAME: emoji-tracker\nSCOPE: Tracks emojis like \u{1f600} and \u{1f4a5}\nLANGUAGE: Rust";
        let brief = parse_project_brief(text).unwrap();
        assert_eq!(brief.name, "emoji-tracker");
        assert!(brief.scope.contains('\u{1f600}'));
    }

    // Requirement: REQ-BAP-010 (Must)
    // Edge case: BUILD_COMPLETE with missing fields
    #[test]
    fn test_regression_parse_build_summary_partial_fields() {
        let text = "BUILD_COMPLETE\nPROJECT: my-app";
        let summary = parse_build_summary(text).unwrap();
        assert_eq!(summary.project, "my-app");
        assert!(summary.location.is_empty(), "Missing field should default to empty");
        assert!(summary.language.is_empty());
        assert!(summary.summary.is_empty());
        assert!(summary.usage.is_empty());
        assert_eq!(summary.skill, None);
    }

    // Requirement: REQ-BAP-010 (Must)
    // Security: script injection in project name
    #[test]
    fn test_regression_parse_project_brief_script_injection_with_slash() {
        // Names with / are rejected — </script> contains a slash.
        let text = "PROJECT_NAME: <script>alert(1)</script>\nSCOPE: evil";
        assert!(
            parse_project_brief(text).is_none(),
            "Name containing / (from </script>) should be rejected"
        );
    }

    // Requirement: REQ-BAP-010 (Must)
    // Security: special chars in project name (no path separators)
    #[test]
    fn test_regression_parse_project_brief_special_chars_no_slash() {
        // Names without / \ .. or leading . are accepted by the parser.
        let text = "PROJECT_NAME: my-app-v2.0\nSCOPE: test";
        let brief = parse_project_brief(text);
        assert!(brief.is_some(), "Name with dots (not leading) should be accepted");
        assert_eq!(brief.unwrap().name, "my-app-v2.0");
    }

    // =======================================================================
    // REQ-BAP-009 (Must): Localized progress messages for all 7 phases
    // =======================================================================
    //
    // The current phase_message() handles phases 1 and 5 with custom messages
    // and uses a generic format for others. For the 7-phase pipeline, the
    // developer must extend this to handle all 7 phases with meaningful
    // localized messages.
    //
    // These tests will FAIL until the developer extends phase_message().

    // Requirement: REQ-BAP-009 (Must)
    // Acceptance: Each phase transition sends a dedicated localized message
    // (not just the generic "Phase N: action..." format).
    //
    // IMPORTANT: These tests call phase_message with an EMPTY action string
    // to ensure the function produces meaningful per-phase messages from its
    // own content, not by echoing the action parameter back.
    #[test]
    fn test_phase_message_7_phases_english_custom_messages() {
        // Phase 1: Analyst — already has custom message
        let msg1 = phase_message("English", 1, "");
        assert!(
            msg1.contains("Analyzing"),
            "Phase 1 must have custom English message: got '{msg1}'"
        );
        // Phase 2: Architect — needs custom message about architecture/design
        let msg2 = phase_message("English", 2, "");
        assert!(
            msg2.contains("architecture") || msg2.contains("Architecture")
                || msg2.contains("design") || msg2.contains("Design"),
            "Phase 2 must have custom English message about architecture: got '{msg2}'"
        );
        // Phase 3: Test Writer — needs custom message about tests
        let msg3 = phase_message("English", 3, "");
        assert!(
            msg3.contains("test") || msg3.contains("Test"),
            "Phase 3 must have custom English message about tests: got '{msg3}'"
        );
        // Phase 4: Developer — needs custom message about implementation
        let msg4 = phase_message("English", 4, "");
        assert!(
            msg4.contains("implement") || msg4.contains("Implement")
                || msg4.contains("build") || msg4.contains("Build")
                || msg4.contains("cod") || msg4.contains("Cod"),
            "Phase 4 must have custom English message about implementation: got '{msg4}'"
        );
        // Phase 5: QA — needs custom message about verification/quality
        let msg5 = phase_message("English", 5, "");
        assert!(
            msg5.contains("verif") || msg5.contains("Verif")
                || msg5.contains("quality") || msg5.contains("Quality")
                || msg5.contains("check") || msg5.contains("Check")
                || msg5.contains("test") || msg5.contains("Test"),
            "Phase 5 must have custom English message about QA: got '{msg5}'"
        );
        // Phase 6: Reviewer — needs custom message about review
        let msg6 = phase_message("English", 6, "");
        assert!(
            msg6.contains("review") || msg6.contains("Review")
                || msg6.contains("audit") || msg6.contains("Audit"),
            "Phase 6 must have custom English message about review: got '{msg6}'"
        );
        // Phase 7: Delivery — needs custom message about delivery
        let msg7 = phase_message("English", 7, "");
        assert!(
            msg7.contains("deliver") || msg7.contains("Deliver")
                || msg7.contains("Preparing") || msg7.contains("delivery"),
            "Phase 7 must have custom English message about delivery: got '{msg7}'"
        );
    }

    // Requirement: REQ-BAP-009 (Must)
    // Acceptance: All 8 languages supported for all 7 phases
    #[test]
    fn test_phase_message_7_phases_all_languages_non_empty() {
        let languages = [
            "English", "Spanish", "Portuguese", "French",
            "German", "Italian", "Dutch", "Russian",
        ];
        for lang in &languages {
            for phase in 1..=7 {
                let msg = phase_message(lang, phase, "action");
                assert!(
                    !msg.is_empty(),
                    "Phase {phase} message for {lang} must not be empty"
                );
            }
        }
    }

    // Requirement: REQ-BAP-009 (Must)
    // Acceptance: Spanish messages for all 7 phases
    #[test]
    fn test_phase_message_7_phases_spanish() {
        let msg1 = phase_message("Spanish", 1, "");
        assert!(msg1.contains("Analizando"), "Spanish phase 1: got '{msg1}'");

        let msg7 = phase_message("Spanish", 7, "");
        assert!(
            msg7.contains("entrega") || msg7.contains("Entrega") || msg7.contains("Preparando"),
            "Spanish phase 7 should mention delivery: got '{msg7}'"
        );
    }

    // Requirement: REQ-BAP-009 (Must)
    // Edge case: unknown language falls back to English-like behavior
    #[test]
    fn test_phase_message_unknown_language_all_phases() {
        for phase in 1..=7 {
            let msg = phase_message("Klingon", phase, "action");
            assert!(
                !msg.is_empty(),
                "Unknown language should still produce a message for phase {phase}"
            );
        }
    }

    // Requirement: REQ-BAP-009 (Must)
    // Edge case: phase 0 and phase 8 (out of range)
    #[test]
    fn test_phase_message_out_of_range_phases() {
        // Phase 0 and 8+ should produce a reasonable generic message.
        let msg0 = phase_message("English", 0, "unknown");
        assert!(
            !msg0.is_empty(),
            "Phase 0 should produce some output (generic fallback)"
        );
        let msg8 = phase_message("English", 8, "unknown");
        assert!(
            !msg8.is_empty(),
            "Phase 8 should produce some output (generic fallback)"
        );
    }

    // Requirement: REQ-BAP-009 (Must)
    // Regression: In the 7-phase pipeline, phase 7 is delivery.
    // The custom message must not depend on the action parameter.
    #[test]
    fn test_phase_message_delivery_phase_english() {
        // Phase 7 is delivery in the new pipeline.
        // Use EMPTY action to ensure the function has its own custom message.
        let msg = phase_message("English", 7, "");
        assert!(
            msg.contains("deliver") || msg.contains("Deliver")
                || msg.contains("Preparing") || msg.contains("delivery"),
            "Phase 7 (delivery) must have a custom English message: got '{msg}'"
        );
    }
}
