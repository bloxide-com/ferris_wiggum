use dioxus::prelude::*;
use dioxus::router::Link;
use ralph::{Session, SessionStatus};

#[component]
pub fn SessionList() -> Element {
    let sessions = use_resource(|| async move {
        let result: Result<Vec<Session>, _> = api::ralph::list_sessions().await;
        result.unwrap_or_default()
    });

    rsx! {
        div { class: "ralph-session-list",
            h2 { "Ralph Sessions" }

            match sessions() {
                Some(session_list) => rsx! {
                    for session in session_list {
                        SessionCard { session }
                    }
                },
                None => rsx! {
                    div { class: "loading", "Loading sessions..." }
                }
            }

            Link { 
                to: "/new", 
                class: "new-session-btn",
                "New Session"
            }
        }
    }
}

#[component]
fn SessionCard(session: Session) -> Element {
    let status_class = match &session.status {
        SessionStatus::Running { .. } => "status-running",
        SessionStatus::Complete => "status-complete",
        SessionStatus::Gutter { .. } => "status-gutter",
        SessionStatus::Failed { .. } => "status-failed",
        SessionStatus::Paused => "status-paused",
        _ => "status-idle",
    };

    let status_text = match &session.status {
        SessionStatus::Idle => "Idle".to_string(),
        SessionStatus::Running { story_id } => format!("Running: {}", story_id),
        SessionStatus::Paused => "Paused".to_string(),
        SessionStatus::WaitingForRotation => "Waiting for Rotation".to_string(),
        SessionStatus::Gutter { reason } => format!("Gutter: {}", reason),
        SessionStatus::Complete => "Complete".to_string(),
        SessionStatus::Failed { error } => format!("Failed: {}", error),
    };

    let stories_info = session.prd.as_ref().map(|prd| {
        let completed = prd.stories.iter().filter(|s| s.passes).count();
        let total = prd.stories.len();
        format!("Stories: {}/{}", completed, total)
    });

    rsx! {
        Link {
            to: format!("/{}", session.id),
            class: "session-card",
            
            div { class: "session-header",
                h3 { "{session.project_path}" }
                span { class: "session-status {status_class}", "{status_text}" }
            }
            
            div { class: "session-info",
                span { "Iteration: {session.current_iteration}" }
                span { "Tokens: {session.token_usage.total}" }
                
                if let Some(info) = stories_info {
                    span { "{info}" }
                }
            }
        }
    }
}
