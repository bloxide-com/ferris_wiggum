use dioxus::prelude::*;
use ralph::{SessionConfig, Prd};
use ui::ralph::{PrdEditor, FilePicker};

#[component]
pub fn RalphNewSession() -> Element {
    let project_path = use_signal(|| String::new());
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

    let create_session = move |e: FormEvent| {
        e.prevent_default();
        spawn(async move {
            creating.set(true);
            error.set(None);

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

            match api::ralph::create_session(project_path(), config).await {
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
                form { 
                    class: "session-form",
                    onsubmit: create_session,

                    div { class: "form-group",
                    label { "for": "project-path", "Project Path" }
                    FilePicker {
                        value: project_path,
                        on_select: None,
                    }
                    p { class: "form-help", "Browse and select your project's git repository directory" }
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
                            r#type: "submit",
                            disabled: creating() || project_path().is_empty(),
                            class: "btn btn-primary",
                            if creating() { "Creating..." } else { "Next: Set PRD" }
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
                        p { "Define the stories you want Ralph to work on. You can write in markdown and preview before saving." }
                        
                        PrdEditor {
                            session_id: id,
                            on_prd_set: on_prd_set
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
