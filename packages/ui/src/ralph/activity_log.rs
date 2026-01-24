use dioxus::prelude::*;
use ralph::{ActivityEntry, ActivityKind, ContextHealth, Signal};

#[component]
pub fn ActivityLog(session_id: ReadSignal<String>) -> Element {
    let entries = use_signal(|| Vec::<ActivityEntry>::new());

    // TODO: Implement SSE streaming when Dioxus supports it better
    // For now, we'll poll periodically
    use_effect(move || {
        spawn(async move {
            // Placeholder for SSE connection
            // In a real implementation, we'd connect to an SSE endpoint
            // and stream ActivityEntry events
        });
    });

    rsx! {
        div { class: "activity-log",
            h3 { "Activity Log" }

            div { class: "log-entries",
                if entries().is_empty() {
                    div { class: "log-empty", "No activity yet. Start a session to see activity." }
                } else {
                    for entry in entries() {
                        ActivityRow { entry }
                    }
                }
            }
        }
    }
}

#[component]
fn ActivityRow(entry: ActivityEntry) -> Element {
    let (icon, description) = match &entry.kind {
        ActivityKind::Read { path, lines, bytes } => (
            "👁️",
            format!("READ {} ({} lines, {} bytes)", path, lines, bytes),
        ),
        ActivityKind::Write { path, lines, bytes } => (
            "✏️",
            format!("WRITE {} ({} lines, {} bytes)", path, lines, bytes),
        ),
        ActivityKind::Shell { command, exit_code } => {
            let icon = if *exit_code == 0 { "✅" } else { "❌" };
            (icon, format!("SHELL {} → exit {}", command, exit_code))
        }
        ActivityKind::TokenUpdate(usage) => ("📊", format!("TOKENS: {} total", usage.total)),
        ActivityKind::Signal(signal) => match signal {
            Signal::Warn => ("⚠️", "WARN: Approaching token limit".to_string()),
            Signal::Rotate => ("🔄", "ROTATE: Starting fresh iteration".to_string()),
            Signal::Gutter(reason) => ("🚨", format!("GUTTER: {}", reason)),
            Signal::Complete => ("🎉", "COMPLETE: All stories pass!".to_string()),
            Signal::StoryComplete(id) => ("✓", format!("Story {} completed", id)),
        },
        ActivityKind::Error(msg) => ("❌", format!("ERROR: {}", msg)),
    };

    let health_icon = match entry.health {
        ContextHealth::Healthy => "🟢",
        ContextHealth::Warning => "🟡",
        ContextHealth::Critical => "🔴",
    };

    let health_class = match entry.health {
        ContextHealth::Healthy => "health-green",
        ContextHealth::Warning => "health-yellow",
        ContextHealth::Critical => "health-red",
    };

    // Format timestamp
    let timestamp = format!("{:?}", entry.timestamp); // Simple format for now

    rsx! {
        div { class: "activity-row {health_class}",
            span { class: "activity-health", "{health_icon}" }
            span { class: "activity-icon", "{icon}" }
            span { class: "activity-time", "{timestamp}" }
            span { class: "activity-description", "{description}" }
        }
    }
}
