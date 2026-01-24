use dioxus::prelude::*;
use ralph::Story;

#[component]
pub fn StoryProgress(stories: Vec<Story>) -> Element {
    let completed = stories.iter().filter(|s| s.passes).count();
    let total = stories.len();
    let percentage = if total > 0 {
        (completed as f32 / total as f32 * 100.0) as u32
    } else {
        0
    };

    rsx! {
        div { class: "story-progress",
            h3 { "User Stories ({completed}/{total})" }

            div { class: "progress-bar",
                div {
                    class: "progress-fill",
                    style: "width: {percentage}%"
                }
            }

            div { class: "story-list",
                for story in stories {
                    StoryCard { story }
                }
            }
        }
    }
}

#[component]
fn StoryCard(story: Story) -> Element {
    let status_icon = if story.passes { "✅" } else { "⏳" };
    let status_class = if story.passes {
        "story-complete"
    } else {
        "story-pending"
    };

    rsx! {
        div { class: "story-card {status_class}",
            div { class: "story-header",
                span { class: "story-id", "{story.id}" }
                span { class: "story-status", "{status_icon}" }
                span { class: "story-priority", "Priority: {story.priority}" }
            }

            h4 { class: "story-title", "{story.title}" }
            p { class: "story-description", "{story.description}" }

            div { class: "story-criteria",
                h5 { "Acceptance Criteria:" }
                ul {
                    for criterion in story.acceptance_criteria {
                        li { "{criterion}" }
                    }
                }
            }

            if !story.notes.is_empty() {
                div { class: "story-notes",
                    h5 { "Notes:" }
                    p { "{story.notes}" }
                }
            }
        }
    }
}
