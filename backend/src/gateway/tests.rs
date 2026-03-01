use super::keywords::*;
use super::*;
use omega_core::context::ContextNeeds;

#[test]
fn test_prompts_default_welcome_all_languages() {
    let prompts = Prompts::default();
    let languages = [
        "English",
        "Spanish",
        "Portuguese",
        "French",
        "German",
        "Italian",
        "Dutch",
        "Russian",
    ];
    for lang in &languages {
        let msg = prompts.welcome.get(*lang);
        assert!(msg.is_some(), "welcome for {lang} should exist");
        assert!(
            msg.unwrap().contains("*OMEGA \u{03a9}*"),
            "welcome for {lang} should mention *OMEGA \u{03a9}*"
        );
    }
}

#[test]
fn test_prompts_default_welcome_fallback() {
    let prompts = Prompts::default();
    let default = prompts.welcome.get("English").cloned().unwrap_or_default();
    let msg = prompts.welcome.get("Klingon").unwrap_or(&default);
    assert!(msg.contains("*OMEGA \u{03a9}*"));
    assert!(msg.contains("honor"));
}

#[test]
fn test_bundled_system_prompt_contains_identity_soul_system() {
    let content = include_str!("../../../prompts/SYSTEM_PROMPT.md");
    assert!(
        content.contains("## Identity"),
        "bundled system prompt should contain Identity section"
    );
    assert!(
        content.contains("## Soul"),
        "bundled system prompt should contain Soul section"
    );
    assert!(
        content.contains("## System"),
        "bundled system prompt should contain System section"
    );
    assert!(
        content.contains("quietly confident"),
        "bundled system prompt should contain personality principles"
    );
}

#[test]
fn test_bundled_facts_prompt_guided_schema() {
    let content = include_str!("../../../prompts/SYSTEM_PROMPT.md");
    assert!(
        content.contains("preferred_name"),
        "bundled facts section should list preferred_name"
    );
    assert!(
        content.contains("timezone"),
        "bundled facts section should list timezone"
    );
    assert!(
        content.contains("pronouns"),
        "bundled facts section should list pronouns"
    );
    assert!(
        content.contains("PERSON"),
        "bundled facts section should emphasize personal facts"
    );
}

// --- Prompt injection integration tests ---

/// Simulate the gateway's keyword detection + prompt assembly logic.
fn assemble_test_prompt(
    prompts: &Prompts,
    msg: &str,
    _has_active_project: bool,
) -> (String, ContextNeeds) {
    let msg_lower = msg.to_lowercase();
    let needs_scheduling = kw_match(&msg_lower, SCHEDULING_KW);
    let needs_recall = kw_match(&msg_lower, RECALL_KW);
    let needs_tasks = needs_scheduling || kw_match(&msg_lower, TASKS_KW);
    let needs_projects = kw_match(&msg_lower, PROJECTS_KW);
    let needs_meta = kw_match(&msg_lower, META_KW);
    let needs_profile =
        kw_match(&msg_lower, PROFILE_KW) || needs_scheduling || needs_recall || needs_tasks;
    let needs_summaries = needs_recall;
    let needs_outcomes = kw_match(&msg_lower, OUTCOMES_KW);

    let mut prompt = format!(
        "{}\n\n{}\n\n{}",
        prompts.identity, prompts.soul, prompts.system
    );

    if needs_scheduling {
        prompt.push_str("\n\n");
        prompt.push_str(&prompts.scheduling);
    }
    if needs_projects {
        prompt.push_str("\n\n");
        prompt.push_str(&prompts.projects_rules);
    }
    if needs_meta {
        prompt.push_str("\n\n");
        prompt.push_str(&prompts.meta);
    }

    let context_needs = ContextNeeds {
        recall: needs_recall,
        pending_tasks: needs_tasks,
        profile: needs_profile,
        summaries: needs_summaries,
        outcomes: needs_outcomes,
    };

    (prompt, context_needs)
}

#[test]
fn test_prompt_injection_simple_greeting() {
    let prompts = Prompts::default();
    let (prompt, needs) = assemble_test_prompt(&prompts, "good morning", false);

    // Core sections always present
    assert!(prompt.contains("OMEGA"));
    assert!(prompt.contains("precise, warm"));

    // Conditional sections NOT injected
    assert!(
        !prompt.contains("scheduler"),
        "scheduling should not be in greeting prompt"
    );
    assert!(
        !prompt.contains("Projects path"),
        "projects should not be in greeting prompt"
    );
    assert!(
        !prompt.contains("SKILL_IMPROVE"),
        "meta should not be in greeting prompt"
    );

    // ContextNeeds: skip both expensive queries
    assert!(!needs.recall);
    assert!(!needs.pending_tasks);
}

#[test]
fn test_prompt_injection_scheduling_keyword() {
    let prompts = Prompts::default();
    let (prompt, needs) =
        assemble_test_prompt(&prompts, "remind me to call mom tomorrow at 5pm", false);

    // Scheduling section injected
    assert!(
        prompt.contains("scheduler"),
        "scheduling section should be injected for 'remind'"
    );

    // Other conditional sections NOT injected
    assert!(!prompt.contains("Projects path"));
    assert!(!prompt.contains("SKILL_IMPROVE"));

    // ContextNeeds: scheduling implies pending_tasks
    assert!(!needs.recall);
    assert!(needs.pending_tasks, "scheduling should imply pending_tasks");
}

#[test]
fn test_prompt_injection_recall_keyword() {
    let prompts = Prompts::default();
    let (prompt, needs) = assemble_test_prompt(
        &prompts,
        "do you remember what we discussed yesterday?",
        false,
    );

    // No conditional prompt sections injected (recall only affects ContextNeeds)
    assert!(!prompt.contains("scheduler"));
    assert!(!prompt.contains("Projects path"));
    assert!(!prompt.contains("SKILL_IMPROVE"));

    // ContextNeeds: recall enabled
    assert!(
        needs.recall,
        "recall should be enabled for 'remember' + 'yesterday'"
    );
    assert!(!needs.pending_tasks);
}

#[test]
fn test_prompt_injection_tasks_keyword() {
    let prompts = Prompts::default();
    let (prompt, needs) = assemble_test_prompt(&prompts, "show me my pending tasks", false);

    // No conditional prompt sections (tasks only affects ContextNeeds)
    assert!(!prompt.contains("scheduler"));
    assert!(!prompt.contains("Projects path"));
    assert!(!prompt.contains("SKILL_IMPROVE"));

    // ContextNeeds: pending_tasks enabled
    assert!(!needs.recall);
    assert!(
        needs.pending_tasks,
        "pending_tasks should be enabled for 'task' + 'pending'"
    );
}

#[test]
fn test_prompt_injection_scheduling_implies_tasks() {
    let prompts = Prompts::default();
    let (_, needs) = assemble_test_prompt(&prompts, "schedule a daily alarm for 7am", false);

    // Scheduling always implies pending_tasks (need task awareness)
    assert!(
        needs.pending_tasks,
        "scheduling keyword should always enable pending_tasks"
    );
}

#[test]
fn test_prompt_injection_project_keyword() {
    let prompts = Prompts::default();
    let (prompt, needs) = assemble_test_prompt(&prompts, "activate project trader", false);

    // Projects section injected
    assert!(
        prompt.contains("Projects path"),
        "projects section should be injected for 'project' + 'activate'"
    );

    // Others not injected
    assert!(!prompt.contains("scheduler"));
    assert!(!prompt.contains("SKILL_IMPROVE"));

    // ContextNeeds: neither recall nor tasks
    assert!(!needs.recall);
    assert!(!needs.pending_tasks);
}

#[test]
fn test_prompt_injection_active_project_no_keyword() {
    let prompts = Prompts::default();
    let (prompt, _) = assemble_test_prompt(&prompts, "how is the weather today", true);

    // Projects section NOT injected without keyword — keyword-gated since contextual injection
    assert!(
        !prompt.contains("Projects path"),
        "projects section should NOT be injected without project keywords"
    );

    // Others not injected
    assert!(!prompt.contains("scheduler"));
    assert!(!prompt.contains("SKILL_IMPROVE"));
}

#[test]
fn test_prompt_injection_meta_keyword() {
    let prompts = Prompts::default();
    let (prompt, needs) = assemble_test_prompt(&prompts, "improve this skill please", false);

    // Meta section injected
    assert!(
        prompt.contains("SKILL_IMPROVE"),
        "meta section should be injected for 'improve' + 'skill'"
    );

    // Others not injected
    assert!(!prompt.contains("scheduler"));
    assert!(!prompt.contains("Projects path"));

    // ContextNeeds: neither recall nor tasks
    assert!(!needs.recall);
    assert!(!needs.pending_tasks);
}

#[test]
fn test_prompt_injection_combined_scheduling_and_meta() {
    let prompts = Prompts::default();
    let (prompt, needs) =
        assemble_test_prompt(&prompts, "remind me to improve my skill tomorrow", false);

    // Both scheduling and meta injected
    assert!(
        prompt.contains("scheduler"),
        "scheduling should be injected"
    );
    assert!(prompt.contains("SKILL_IMPROVE"), "meta should be injected");

    // Projects NOT injected
    assert!(!prompt.contains("Projects path"));

    // ContextNeeds: pending_tasks from scheduling, no recall
    assert!(!needs.recall);
    assert!(needs.pending_tasks);
}

#[test]
fn test_prompt_injection_all_sections() {
    let prompts = Prompts::default();
    let (prompt, needs) = assemble_test_prompt(
        &prompts,
        "remember to schedule a project skill improvement for tomorrow",
        true,
    );

    // All conditional sections injected
    assert!(
        prompt.contains("scheduler"),
        "scheduling should be injected"
    );
    assert!(
        prompt.contains("Projects path"),
        "projects should be injected"
    );
    assert!(prompt.contains("SKILL_IMPROVE"), "meta should be injected");

    // ContextNeeds: both enabled
    assert!(needs.recall, "recall should be enabled for 'remember'");
    assert!(
        needs.pending_tasks,
        "pending_tasks should be enabled for scheduling"
    );
}

#[test]
fn test_prompt_injection_token_reduction() {
    let prompts = Prompts::default();
    let (lean_prompt, _) = assemble_test_prompt(&prompts, "hello", false);
    let (full_prompt, _) = assemble_test_prompt(
        &prompts,
        "remind me about the project skill improvement tomorrow",
        true,
    );

    // Full prompt should be significantly larger than lean prompt
    assert!(
        full_prompt.len() > lean_prompt.len(),
        "full prompt ({}) should be larger than lean prompt ({})",
        full_prompt.len(),
        lean_prompt.len()
    );

    // Difference should be at least the size of the conditional sections
    let conditional_size =
        prompts.scheduling.len() + prompts.projects_rules.len() + prompts.meta.len();
    let diff = full_prompt.len() - lean_prompt.len();
    assert!(
        diff >= conditional_size,
        "prompt size difference ({diff}) should be >= conditional sections ({conditional_size})"
    );
}

#[test]
fn test_prompt_injection_multilingual_spanish() {
    let prompts = Prompts::default();
    let (prompt, needs) =
        assemble_test_prompt(&prompts, "recuérdame agendar una cita mañana", false);

    // Spanish scheduling keywords should trigger scheduling
    assert!(
        prompt.contains("scheduler"),
        "scheduling should be injected for Spanish keywords"
    );
    assert!(
        needs.pending_tasks,
        "pending_tasks should be enabled for Spanish scheduling"
    );
}

#[test]
fn test_prompt_injection_multilingual_portuguese() {
    let prompts = Prompts::default();
    let (prompt, needs) =
        assemble_test_prompt(&prompts, "lembre-me de verificar o projeto amanhã", false);

    // Portuguese keywords trigger scheduling + recall + projects
    assert!(
        prompt.contains("scheduler"),
        "scheduling should be injected for Portuguese 'lembr'"
    );
    assert!(
        prompt.contains("Projects path"),
        "projects should be injected for 'projeto'"
    );
    assert!(
        needs.recall,
        "recall should be enabled for Portuguese 'lembr'"
    );
    assert!(needs.pending_tasks);
}

#[test]
fn test_bundled_prompt_has_conditional_sections() {
    let content = include_str!("../../../prompts/SYSTEM_PROMPT.md");
    assert!(
        content.contains("## Scheduling"),
        "bundled prompt should have ## Scheduling section"
    );
    assert!(
        content.contains("## Projects"),
        "bundled prompt should have ## Projects section"
    );
    assert!(
        content.contains("## Meta"),
        "bundled prompt should have ## Meta section"
    );
}

// --- Pipeline helper function tests (P2-TEST-004) ---

#[test]
fn test_kw_match_basic() {
    assert!(kw_match("remind me to call mom", SCHEDULING_KW));
    assert!(kw_match("schedule a meeting", SCHEDULING_KW));
    assert!(!kw_match("hello world", SCHEDULING_KW));
}

#[test]
fn test_kw_match_recall_keywords() {
    assert!(kw_match("do you remember what we talked about", RECALL_KW));
    assert!(kw_match("yesterday we discussed the project", RECALL_KW));
    assert!(!kw_match("what is the weather like", RECALL_KW));
}

#[test]
fn test_kw_match_tasks_keywords() {
    assert!(kw_match("show me my pending tasks", TASKS_KW));
    assert!(kw_match("what are my reminders", TASKS_KW));
    assert!(!kw_match("hello how are you", TASKS_KW));
}

#[test]
fn test_kw_match_meta_keywords() {
    assert!(kw_match("improve this skill please", META_KW));
    assert!(kw_match("report a bug in the system", META_KW));
    assert!(!kw_match("tell me a joke", META_KW));
}

#[test]
fn test_is_build_confirmed_multilingual() {
    // English
    assert!(is_build_confirmed("yes"));
    assert!(is_build_confirmed("Yes"));
    assert!(is_build_confirmed("go ahead"));
    // Spanish
    assert!(is_build_confirmed("sí"));
    assert!(is_build_confirmed("dale"));
    // Portuguese
    assert!(is_build_confirmed("sim"));
    // French
    assert!(is_build_confirmed("oui"));
    // German
    assert!(is_build_confirmed("ja"));
    // Not a confirmation
    assert!(!is_build_confirmed("maybe later"));
    assert!(!is_build_confirmed("tell me more"));
}

#[test]
fn test_is_build_cancelled_multilingual() {
    assert!(is_build_cancelled("no"));
    assert!(is_build_cancelled("cancel"));
    assert!(is_build_cancelled("cancelar"));
    assert!(is_build_cancelled("annuler"));
    assert!(is_build_cancelled("nein"));
    assert!(!is_build_cancelled("yes please"));
}

#[test]
fn test_is_valid_fact_rejects_system_keys() {
    assert!(!is_valid_fact("welcomed", "true"));
    assert!(!is_valid_fact("preferred_language", "English"));
    assert!(!is_valid_fact("active_project", "trader"));
    assert!(!is_valid_fact("onboarding_stage", "3"));
}

#[test]
fn test_is_valid_fact_accepts_user_facts() {
    assert!(is_valid_fact("name", "Alice"));
    assert!(is_valid_fact("occupation", "Software Engineer"));
    assert!(is_valid_fact("hobby", "Painting"));
}

#[test]
fn test_is_valid_fact_rejects_too_long() {
    let long_key = "k".repeat(51);
    assert!(!is_valid_fact(&long_key, "value"));
    let long_val = "v".repeat(201);
    assert!(!is_valid_fact("key", &long_val));
}

#[test]
fn test_is_valid_fact_rejects_numeric_key() {
    assert!(!is_valid_fact("123", "value"));
    assert!(!is_valid_fact("1key", "value"));
}

#[test]
fn test_is_valid_fact_rejects_dollar_value() {
    assert!(!is_valid_fact("price", "$100"));
}

#[test]
fn test_setup_help_message_all_languages() {
    let languages = [
        "English",
        "Spanish",
        "Portuguese",
        "French",
        "German",
        "Italian",
        "Dutch",
        "Russian",
    ];
    for lang in &languages {
        let msg = setup_help_message(lang);
        assert!(
            msg.contains("/setup"),
            "setup help for {lang} should mention /setup"
        );
        assert!(
            msg.contains("OMEGA"),
            "setup help for {lang} should mention OMEGA"
        );
    }
}

#[test]
fn test_build_cancelled_message_all_languages() {
    let languages = [
        "English",
        "Spanish",
        "Portuguese",
        "French",
        "German",
        "Italian",
        "Dutch",
        "Russian",
    ];
    for lang in &languages {
        let msg = build_cancelled_message(lang);
        assert!(
            !msg.is_empty(),
            "cancel message for {lang} should not be empty"
        );
    }
}

#[test]
fn test_context_needs_scheduling_implies_tasks_and_profile() {
    let prompts = Prompts::default();
    let (_, needs) = assemble_test_prompt(&prompts, "schedule a reminder for 3pm", false);
    assert!(needs.pending_tasks, "scheduling should imply pending_tasks");
    assert!(needs.profile, "scheduling should imply profile");
}

#[test]
fn test_context_needs_recall_implies_profile_and_summaries() {
    let prompts = Prompts::default();
    let (_, needs) = assemble_test_prompt(
        &prompts,
        "do you remember what we discussed yesterday?",
        false,
    );
    assert!(needs.recall, "recall keyword should enable recall");
    assert!(needs.profile, "recall should imply profile");
    assert!(needs.summaries, "recall should imply summaries");
}
