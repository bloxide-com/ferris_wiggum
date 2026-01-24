use super::{ActivityLog, GuardrailsPanel, StoryProgress, TokenMeter};
use dioxus::prelude::*;
use ralph::Session;

#[component]
pub fn SessionDashboard(session_id: ReadSignal<String>) -> Element {
    let session = use_resource(move || async move {
        let result: Result<Session, _> = api::ralph::get_session(session_id()).await;
        result.ok()
    });

    rsx! {
        div { class: "ralph-dashboard",
            match session() {
                Some(Some(sess)) => {
                    let stories = sess.prd.as_ref().map(|p| p.stories.clone()).unwrap_or_default();
                    rsx! {
                        SessionHeader { session: sess.clone() }

                        div { class: "ralph-main",
                            div { class: "ralph-content",
                                if !stories.is_empty() {
                                    StoryProgress { stories }
                                }

                                ActivityLog { session_id }
                            }

                            div { class: "ralph-sidebar",
                                TokenMeter {
                                    usage: sess.token_usage.clone(),
                                    warn_threshold: sess.config.warn_threshold,
                                    rotate_threshold: sess.config.rotate_threshold,
                                }
                                GuardrailsPanel { session_id }
                            }
                        }
                    }
                },
                _ => rsx! {
                    div { class: "loading", "Loading session..." }
                }
            }
        }
    }
}

#[component]
fn SessionHeader(session: Session) -> Element {
    let mut starting = use_signal(|| false);
    let mut pausing = use_signal(|| false);
    let mut stopping = use_signal(|| false);
    let session_id = session.id.clone();
    let session_id2 = session.id.clone();
    let session_id3 = session.id.clone();

    let start_session = move |_| {
        let id = session_id.clone();
        spawn(async move {
            starting.set(true);
            let _ = api::ralph::start_session(id).await;
            starting.set(false);
        });
    };

    let pause_session = move |_| {
        let id = session_id2.clone();
        spawn(async move {
            pausing.set(true);
            let _ = api::ralph::pause_session(id).await;
            pausing.set(false);
        });
    };

    let stop_session = move |_| {
        let id = session_id3.clone();
        spawn(async move {
            stopping.set(true);
            let _ = api::ralph::stop_session(id).await;
            stopping.set(false);
        });
    };

    rsx! {
        div { class: "session-header",
            div { class: "session-title",
                h1 { "{session.project_path}" }
                span { class: "session-id", "ID: {session.id}" }
            }

            div { class: "session-controls",
                button {
                    onclick: start_session,
                    disabled: starting() || matches!(session.status, ralph::SessionStatus::Running { .. }),
                    class: "btn btn-start",
                    if starting() { "Starting..." } else { "Start" }
                }

                button {
                    onclick: pause_session,
                    disabled: pausing() || !matches!(session.status, ralph::SessionStatus::Running { .. }),
                    class: "btn btn-pause",
                    if pausing() { "Pausing..." } else { "Pause" }
                }

                button {
                    onclick: stop_session,
                    disabled: stopping(),
                    class: "btn btn-stop",
                    if stopping() { "Stopping..." } else { "Stop" }
                }
            }

            div { class: "session-stats",
                div { class: "stat",
                    span { class: "stat-label", "Iteration" }
                    span { class: "stat-value", "{session.current_iteration}" }
                }
                div { class: "stat",
                    span { class: "stat-label", "PRD Model" }
                    span { class: "stat-value", "{session.config.prd_model}" }
                }
                div { class: "stat",
                    span { class: "stat-label", "Execution Model" }
                    span { class: "stat-value", "{session.config.execution_model}" }
                }
                div { class: "stat",
                    span { class: "stat-label", "Max Iterations" }
                    span { class: "stat-value", "{session.config.max_iterations}" }
                }
            }
        }
    }
}
