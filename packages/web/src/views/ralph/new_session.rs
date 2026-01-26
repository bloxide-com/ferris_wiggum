use dioxus::prelude::*;
use ralph::SessionConfig;
use serde::{Deserialize, Serialize};
use ui::ralph::FilePicker;

#[cfg(feature = "web")]
use web_sys::{window, Event, EventTarget, VisibilityState};
#[cfg(feature = "web")]
use wasm_bindgen::JsCast;

use crate::use_persisted_signal;

#[component]
pub fn RalphNewSession() -> Element {
    // `project_path_input` tracks transient browsing/typing state.
    // `locked_project_path` is only updated when the user explicitly clicks "Select" in the FilePicker.
    // Session creation must use the locked value.
    // All form state is persisted to localStorage under key `ralph_new_session_draft`
    let mut draft = use_persisted_signal(
        "ralph_new_session_draft",
        || NewSessionDraft {
            project_path_input: String::new(),
            locked_project_path: None,
            prd_model: "opus-4.5-thinking".to_string(),
            execution_model: "opus-4.5-thinking".to_string(),
            max_iterations: 20,
            warn_threshold: 70_000,
            rotate_threshold: 80_000,
            branch_name: String::new(),
            open_pr: false,
        },
    );

    let mut creating = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let nav = navigator();

    // Create a local signal for FilePicker that syncs with draft
    let mut project_path_input = use_signal(|| draft().project_path_input.clone());

    // Sync project_path_input signal with draft when draft changes
    use_effect(move || {
        let draft_value = draft().project_path_input.clone();
        if project_path_input() != draft_value {
            project_path_input.set(draft_value);
        }
    });

    // Update draft when project_path_input changes (user typing in FilePicker)
    use_effect(move || {
        let input_value = project_path_input();
        let current_draft = draft();
        if current_draft.project_path_input != input_value {
            draft.write().project_path_input = input_value;
        }
    });

    // Listen for visibility changes to restore state when tab becomes visible
    #[cfg(feature = "web")]
    {
        // Track visibility state with a signal
        let visibility_state = use_signal(|| {
            if let Some(window) = window() {
                if let Some(document) = window.document() {
                    document.visibility_state()
                } else {
                    VisibilityState::Visible
                }
            } else {
                VisibilityState::Visible
            }
        });

        // Set up visibility change listener
        use_effect(move || {
            let window = window();
            let Some(window) = window else {
                return;
            };
            let Some(document) = window.document() else {
                return;
            };
            let event_target: &EventTarget = document.as_ref();

            let mut visibility_signal = visibility_state;
            let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move |_e: Event| {
                let Some(window) = web_sys::window() else {
                    return;
                };
                let Some(document) = window.document() else {
                    return;
                };
                
                // Update visibility state signal when it changes
                visibility_signal.set(document.visibility_state());
            }) as Box<dyn FnMut(Event)>);

            let _ = event_target
                .add_event_listener_with_callback("visibilitychange", closure.as_ref().unchecked_ref());

            // Keep the closure alive for the lifetime of the component
            closure.forget();
        });

        // Re-read from localStorage when visibility becomes "visible"
        use_effect(move || {
            // Read visibility state to track it
            let is_visible = visibility_state() == VisibilityState::Visible;
            
            if is_visible {
                // Re-read from localStorage and update signal if needed
                let window = window();
                if let Some(window) = window {
                    if let Ok(Some(storage)) = window.local_storage() {
                        if let Ok(Some(stored)) = storage.get_item("ralph_new_session_draft") {
                            if let Ok(deserialized) = serde_json::from_str::<NewSessionDraft>(&stored) {
                                let current = draft();
                                // Only update if the stored value differs
                                if deserialized != current {
                                    draft.set(deserialized);
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    let can_create_session = use_memo(move || {
        let draft = draft();
        let locked = draft.locked_project_path.clone();
        if let Some(locked) = locked {
            !locked.trim().is_empty() && locked == draft.project_path_input
        } else {
            false
        }
    });

    let create_session = move |_| {
        let draft_signal = draft;
        let nav = nav.clone();
        spawn(async move {
            creating.set(true);
            error.set(None);

            let draft = draft_signal();
            let locked = draft.locked_project_path.clone();
            let Some(project_path) = locked else {
                error.set(Some(
                    "Please click “Select” in the file picker to confirm a valid repo path."
                        .to_string(),
                ));
                creating.set(false);
                return;
            };

            if project_path != draft.project_path_input {
                error.set(Some(
                    "The project path changed since you clicked “Select”. Click “Select” again to confirm the updated path."
                        .to_string(),
                ));
                creating.set(false);
                return;
            }

            let config = SessionConfig {
                prd_model: draft.prd_model.clone(),
                execution_model: draft.execution_model.clone(),
                max_iterations: draft.max_iterations,
                warn_threshold: draft.warn_threshold,
                rotate_threshold: draft.rotate_threshold,
                branch_name: if draft.branch_name.is_empty() {
                    None
                } else {
                    Some(draft.branch_name.clone())
                },
                open_pr: draft.open_pr,
            };

            match api::ralph::create_session(project_path, config).await {
                Ok(session) => {
                    // Clear localStorage when navigating to session page (successful completion)
                    #[cfg(feature = "web")]
                    {
                        if let Some(window) = web_sys::window() {
                            if let Ok(Some(storage)) = window.local_storage() {
                                let _ = storage.remove_item("ralph_new_session_draft");
                            }
                        }
                    }
                    nav.push(format!("/{}", session.id).as_str());
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

    rsx! {
        div { class: "ralph-new-session-page",
            h1 { "Create New Ralph Session" }
            div {
                class: "session-form",

                div { class: "form-group",
                    label { "for": "project-path", "Project Path" }
                    FilePicker {
                        value: project_path_input,
                        on_select: move |path: String| {
                            project_path_input.set(path.clone());
                            draft.write().project_path_input = path.clone();
                            draft.write().locked_project_path = Some(path);
                        },
                    }
                    p { class: "form-help", "Browse and select your project's git repository directory" }
                }

                div { class: "form-row",
                    div { class: "form-group",
                        label { "for": "prd-model", "PRD Model" }
                        select {
                            id: "prd-model",
                            value: "{draft().prd_model}",
                            onchange: move |e| draft.write().prd_model = e.value(),
                            option { value: "auto", "Auto (cursor-agent picks best model)" }
                            option { value: "opus-4.5-thinking", "Claude Opus 4.5 (thinking)" }
                            option { value: "sonnet-4.5-thinking", "Claude Sonnet 4.5 (thinking)" }
                            option { value: "gpt-5.2-high", "GPT 5.2 High" }
                            option { value: "composer-1", "Composer 1" }
                        }
                        p { class: "form-help", "Model used for PRD generation" }
                    }

                    div { class: "form-group",
                        label { "for": "execution-model", "Execution Model" }
                        select {
                            id: "execution-model",
                            value: "{draft().execution_model}",
                            onchange: move |e| draft.write().execution_model = e.value(),
                            option { value: "auto", "Auto (cursor-agent picks best model)" }
                            option { value: "opus-4.5-thinking", "Claude Opus 4.5 (thinking)" }
                            option { value: "sonnet-4.5-thinking", "Claude Sonnet 4.5 (thinking)" }
                            option { value: "gpt-5.2-high", "GPT 5.2 High" }
                            option { value: "composer-1", "Composer 1" }
                        }
                        p { class: "form-help", "Model used for code execution" }
                    }
                }

                div { class: "form-row",
                    div { class: "form-group",
                        label { "for": "max-iterations", "Max Iterations" }
                        input {
                            id: "max-iterations",
                            r#type: "number",
                            value: "{draft().max_iterations}",
                            oninput: move |e| {
                                if let Ok(val) = e.value().parse::<u32>() {
                                    draft.write().max_iterations = val;
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
                            value: "{draft().warn_threshold}",
                            oninput: move |e| {
                                if let Ok(val) = e.value().parse::<u32>() {
                                    draft.write().warn_threshold = val;
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
                            value: "{draft().rotate_threshold}",
                            oninput: move |e| {
                                if let Ok(val) = e.value().parse::<u32>() {
                                    draft.write().rotate_threshold = val;
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
                        value: "{draft().branch_name}",
                        oninput: move |e| draft.write().branch_name = e.value(),
                        placeholder: "ralph/my-feature",
                    }
                    p { class: "form-help", "Leave empty to work on current branch" }
                }

                div { class: "form-group",
                    label { class: "checkbox-label",
                        input {
                            r#type: "checkbox",
                            checked: draft().open_pr,
                            onchange: move |e| draft.write().open_pr = e.checked(),
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
    }
}

/// Draft state for the new session form, serializable for localStorage persistence.
#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct NewSessionDraft {
    pub project_path_input: String,
    pub locked_project_path: Option<String>,
    pub prd_model: String,
    pub execution_model: String,
    pub max_iterations: u32,
    pub warn_threshold: u32,
    pub rotate_threshold: u32,
    pub branch_name: String,
    pub open_pr: bool,
}
