use dioxus::prelude::*;
use ralph::{Prd, Story};

#[component]
pub fn PrdEditor(session_id: String, on_prd_set: EventHandler<Prd>) -> Element {
    let session_id = use_signal(|| session_id);
    let mut markdown = use_signal(|| String::new());
    let mut prd_preview = use_signal(|| None::<Prd>);
    let mut converting = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let mut mode = use_signal(|| EditorMode::Markdown);

    let convert_prd = move |_| {
        let session_id = session_id();
        spawn(async move {
            converting.set(true);
            error.set(None);

            match api::ralph::convert_prd(session_id, markdown()).await {
                Ok(prd) => {
                    prd_preview.set(Some(prd));
                    mode.set(EditorMode::Preview);
                }
                Err(e) => {
                    error.set(Some(format!("Conversion failed: {:?}", e)));
                }
            }
            converting.set(false);
        });
    };

    let set_prd = move |_| {
        let session_id = session_id();
        if let Some(prd) = prd_preview() {
            spawn(async move {
                match api::ralph::set_prd(session_id, prd.clone()).await {
                    Ok(_) => {
                        on_prd_set.call(prd);
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to set PRD: {:?}", e)));
                    }
                }
            });
        }
    };

    rsx! {
        div { class: "prd-editor",
            div { class: "editor-tabs",
                button {
                    class: if matches!(mode(), EditorMode::Markdown) { "tab active" } else { "tab" },
                    onclick: move |_| mode.set(EditorMode::Markdown),
                    "Edit Markdown"
                }
                button {
                    class: if matches!(mode(), EditorMode::Preview) { "tab active" } else { "tab" },
                    onclick: move |_| mode.set(EditorMode::Preview),
                    disabled: prd_preview().is_none(),
                    "Preview Stories"
                }
            }

            match mode() {
                EditorMode::Markdown => rsx! {
                    div { class: "markdown-editor",
                        textarea {
                            class: "prd-markdown-input",
                            value: "{markdown}",
                            oninput: move |e| markdown.set(e.value()),
                            placeholder: r#"# Feature Name

## Problem Statement
Brief description of what problem this feature solves.

## User Stories

### US-001: Story Title
**As a** user type
**I want** feature description
**So that** benefit

**Acceptance Criteria:**
- [ ] Criterion 1
- [ ] Criterion 2
- [ ] Typecheck passes

**Priority:** 1

### US-002: Another Story
..."#,
                            rows: "20",
                        }

                        div { class: "editor-actions",
                            button {
                                onclick: convert_prd,
                                disabled: converting() || markdown().trim().is_empty(),
                                class: "btn btn-primary",
                                if converting() { "Converting..." } else { "Convert & Preview" }
                            }
                        }
                    }
                },
                EditorMode::Preview => rsx! {
                    if let Some(prd) = prd_preview() {
                        div { class: "prd-preview",
                            div { class: "prd-header",
                                h3 { "{prd.project}" }
                                p { class: "prd-description", "{prd.description}" }
                                p { class: "prd-branch", "Branch: {prd.branch_name}" }
                            }

                            div { class: "stories-list",
                                h4 { "Stories ({prd.stories.len()})" }
                                for story in prd.stories.iter() {
                                    StoryCard { story: story.clone() }
                                }
                            }

                            div { class: "editor-actions",
                                button {
                                    onclick: move |_| mode.set(EditorMode::Markdown),
                                    class: "btn btn-secondary",
                                    "Back to Edit"
                                }
                                button {
                                    onclick: set_prd,
                                    class: "btn btn-primary",
                                    "Use This PRD"
                                }
                            }
                        }
                    }
                }
            }

            if let Some(err) = error() {
                div { class: "error-message",
                    "{err}"
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum EditorMode {
    Markdown,
    Preview,
}

#[component]
fn StoryCard(story: Story) -> Element {
    rsx! {
        div { class: "story-card",
            div { class: "story-header",
                span { class: "story-id", "{story.id}" }
                h5 { "{story.title}" }
                span { class: "story-priority", "Priority: {story.priority}" }
            }
            p { class: "story-description", "{story.description}" }
            div { class: "acceptance-criteria",
                strong { "Acceptance Criteria:" }
                ul {
                    for criterion in story.acceptance_criteria.iter() {
                        li { "{criterion}" }
                    }
                }
            }
        }
    }
}
