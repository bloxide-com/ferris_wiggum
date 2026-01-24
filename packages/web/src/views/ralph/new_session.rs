use dioxus::prelude::*;
use ralph::{Prd, SessionConfig};
use ui::ralph::{FilePicker, PrdConversation, PrdEditor};

#[component]
pub fn RalphNewSession() -> Element {
    // `project_path_input` tracks transient browsing/typing state.
    // `locked_project_path` is only updated when the user explicitly clicks "Select" in the FilePicker.
    // Session creation must use the locked value.
    let project_path_input = use_signal(|| String::new());
    let mut locked_project_path = use_signal(|| None::<String>);
    let mut model = use_signal(|| "opus-4.5-thinking".to_string());
    let mut max_iterations = use_signal(|| 20);
    let mut warn_threshold = use_signal(|| 70_000);
    let mut rotate_threshold = use_signal(|| 80_000);
    let mut branch_name = use_signal(|| String::new());
    let mut open_pr = use_signal(|| false);
    let mut creating = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let mut session_id = use_signal(|| None::<String>);
    let mut step = use_signal(|| SetupStep::Config);
    let mut prd_mode = use_signal(|| PrdMode::Conversation);
    let mut generated_prd_markdown = use_signal(|| None::<String>);

    let can_create_session = use_memo(move || {
        let locked = locked_project_path();
        if let Some(locked) = locked {
            !locked.trim().is_empty() && locked == project_path_input()
        } else {
            false
        }
    });

    let create_session = move |_| {
        spawn(async move {
            creating.set(true);
            error.set(None);

            let locked = locked_project_path();
            let Some(project_path) = locked else {
                error.set(Some(
                    "Please click “Select” in the file picker to confirm a valid repo path."
                        .to_string(),
                ));
                creating.set(false);
                return;
            };

            if project_path != project_path_input() {
                error.set(Some(
                    "The project path changed since you clicked “Select”. Click “Select” again to confirm the updated path."
                        .to_string(),
                ));
                creating.set(false);
                return;
            }

            let config = SessionConfig {
                model: model(),
                max_iterations: max_iterations(),
                warn_threshold: warn_threshold(),
                rotate_threshold: rotate_threshold(),
                branch_name: if branch_name().is_empty() {
                    None
                } else {
                    Some(branch_name())
                },
                open_pr: open_pr(),
            };

            match api::ralph::create_session(project_path, config).await {
                Ok(session) => {
                    session_id.set(Some(session.id));
                    step.set(SetupStep::Prd);
                    creating.set(false);
                }
                Err(e) => {
                    let err_msg: String = format!("{:?}", e);
                    error.set(Some(err_msg));
                    creating.set(false);
                }
            }
        });
    };

    let on_prd_set = move |_prd: Prd| {
        // Navigate to session page
        if let Some(id) = session_id() {
            let nav = navigator();
            nav.push(format!("/{}", id).as_str());
        }
    };

    let on_prd_generated = move |prd_markdown: String| {
        // Store the generated PRD markdown and switch to paste mode for preview
        generated_prd_markdown.set(Some(prd_markdown));
        prd_mode.set(PrdMode::Paste);
    };

    rsx! {
        div { class: "ralph-new-session-page",
            h1 { "Create New Ralph Session" }

            div { class: "setup-steps",
                div { class: if matches!(step(), SetupStep::Config) { "step active" } else { "step" },
                    "1. Configure Session"
                }
                div { class: if matches!(step(), SetupStep::Prd) { "step active" } else { "step" },
                    "2. Set PRD"
                }
            }

            if matches!(step(), SetupStep::Config) {
                div {
                    class: "session-form",

                    div { class: "form-group",
                    label { "for": "project-path", "Project Path" }
                    FilePicker {
                        value: project_path_input,
                        on_select: move |path: String| {
                            locked_project_path.set(Some(path));
                        },
                    }
                    p { class: "form-help", "Browse and select your project's git repository directory" }
                    if let Some(locked) = locked_project_path() {
                        p { class: "form-help",
                            strong { "Locked path: " }
                            "{locked}"
                        }
                        if locked != project_path_input() {
                            p { class: "form-help",
                                "Path changed. Click “Select” again to update the locked path."
                            }
                        }
                    } else {
                        p { class: "form-help",
                            strong { "Locked path: " }
                            "none (click “Select” to confirm)"
                        }
                    }
                }

                div { class: "form-group",
                    label { "for": "model", "Model" }
                    select {
                        id: "model",
                        value: "{model}",
                        onchange: move |e| model.set(e.value()),
                        option { value: "opus-4.5-thinking", "Claude Opus 4.5 (thinking)" }
                        option { value: "sonnet-4.5-thinking", "Claude Sonnet 4.5 (thinking)" }
                        option { value: "gpt-5.2-high", "GPT 5.2 High" }
                        option { value: "composer-1", "Composer 1" }
                    }
                }

                div { class: "form-row",
                    div { class: "form-group",
                        label { "for": "max-iterations", "Max Iterations" }
                        input {
                            id: "max-iterations",
                            r#type: "number",
                            value: "{max_iterations}",
                            oninput: move |e| {
                                if let Ok(val) = e.value().parse::<u32>() {
                                    max_iterations.set(val);
                                }
                            },
                            min: "1",
                            max: "100",
                        }
                    }

                    div { class: "form-group",
                        label { "for": "warn-threshold", "Warn Threshold (tokens)" }
                        input {
                            id: "warn-threshold",
                            r#type: "number",
                            value: "{warn_threshold}",
                            oninput: move |e| {
                                if let Ok(val) = e.value().parse::<u32>() {
                                    warn_threshold.set(val);
                                }
                            },
                            step: "1000",
                        }
                    }

                    div { class: "form-group",
                        label { "for": "rotate-threshold", "Rotate Threshold (tokens)" }
                        input {
                            id: "rotate-threshold",
                            r#type: "number",
                            value: "{rotate_threshold}",
                            oninput: move |e| {
                                if let Ok(val) = e.value().parse::<u32>() {
                                    rotate_threshold.set(val);
                                }
                            },
                            step: "1000",
                        }
                    }
                }

                div { class: "form-group",
                    label { "for": "branch-name", "Branch Name (optional)" }
                    input {
                        id: "branch-name",
                        r#type: "text",
                        value: "{branch_name}",
                        oninput: move |e| branch_name.set(e.value()),
                        placeholder: "ralph/my-feature",
                    }
                    p { class: "form-help", "Leave empty to work on current branch" }
                }

                div { class: "form-group",
                    label { class: "checkbox-label",
                        input {
                            r#type: "checkbox",
                            checked: open_pr(),
                            onchange: move |e| open_pr.set(e.checked()),
                        }
                        " Open PR when complete"
                    }
                }

                    if let Some(err) = error() {
                        div { class: "error-message",
                            "{err}"
                        }
                    }

                    div { class: "form-actions",
                        button {
                            r#type: "button",
                            onclick: create_session,
                            disabled: creating() || !can_create_session(),
                            class: "btn btn-primary",
                            if creating() { "Creating..." } else { "Create session" }
                        }

                        Link {
                            to: "/",
                            class: "btn btn-secondary",
                            "Cancel"
                        }
                    }
                }
            }

            if matches!(step(), SetupStep::Prd) {
                if let Some(id) = session_id() {
                    div { class: "prd-step",
                        h2 { "Set Product Requirements Document" }
                        p {
                            "Define the stories you want Ralph to work on. "
                            "Use the conversation mode to build your PRD interactively, or paste your own markdown."
                        }

                        // Mode selector tabs
                        div { class: "prd-mode-selector",
                            button {
                                class: if matches!(prd_mode(), PrdMode::Conversation) { "prd-mode-btn active" } else { "prd-mode-btn" },
                                onclick: move |_| prd_mode.set(PrdMode::Conversation),
                                "Conversation"
                            }
                            button {
                                class: if matches!(prd_mode(), PrdMode::Paste) { "prd-mode-btn active" } else { "prd-mode-btn" },
                                onclick: move |_| prd_mode.set(PrdMode::Paste),
                                "Paste Markdown"
                            }
                        }

                        match prd_mode() {
                            PrdMode::Conversation => rsx! {
                                PrdConversation {
                                    session_id: id.clone(),
                                    on_prd_generated: on_prd_generated
                                }
                            },
                            PrdMode::Paste => rsx! {
                                PrdEditor {
                                    session_id: id.clone(),
                                    on_prd_set: on_prd_set,
                                    initial_markdown: generated_prd_markdown()
                                }
                            }
                        }

                        div { class: "step-actions",
                            button {
                                onclick: move |_| step.set(SetupStep::Config),
                                class: "btn btn-secondary",
                                "Back"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum SetupStep {
    Config,
    Prd,
}

#[derive(Clone, Copy, PartialEq)]
enum PrdMode {
    Conversation,
    Paste,
}
