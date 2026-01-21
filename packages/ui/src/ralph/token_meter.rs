use dioxus::prelude::*;
use ralph::TokenUsage;

#[component]
pub fn TokenMeter(
    usage: TokenUsage,
    warn_threshold: u32,
    rotate_threshold: u32,
) -> Element {
    let percentage = (usage.total as f32 / rotate_threshold as f32 * 100.0).min(100.0);
    let warn_percentage = warn_threshold as f32 / rotate_threshold as f32 * 100.0;
    
    let health_class = if percentage < warn_percentage {
        "healthy"
    } else if percentage < 100.0 {
        "warning"
    } else {
        "critical"
    };

    let icon = if percentage < warn_percentage {
        "🟢"
    } else if percentage < 100.0 {
        "🟡"
    } else {
        "🔴"
    };

    rsx! {
        div { class: "ralph-token-meter {health_class}",
            h3 { "{icon} Context Usage" }
            
            div { class: "meter-bar",
                div {
                    class: "meter-fill {health_class}",
                    style: "width: {percentage}%"
                }
                div {
                    class: "meter-warn-line",
                    style: "left: {warn_percentage}%"
                }
            }

            div { class: "meter-labels",
                span { class: "meter-total", "{usage.total} / {rotate_threshold} tokens" }
                span { class: "meter-percentage", "{percentage:.1}%" }
            }

            div { class: "meter-breakdown",
                div { class: "breakdown-item",
                    span { class: "breakdown-label", "Read:" }
                    span { class: "breakdown-value", "{usage.read}" }
                }
                div { class: "breakdown-item",
                    span { class: "breakdown-label", "Write:" }
                    span { class: "breakdown-value", "{usage.write}" }
                }
                div { class: "breakdown-item",
                    span { class: "breakdown-label", "Assistant:" }
                    span { class: "breakdown-value", "{usage.assistant}" }
                }
                div { class: "breakdown-item",
                    span { class: "breakdown-label", "Shell:" }
                    span { class: "breakdown-value", "{usage.shell}" }
                }
            }

            if percentage >= warn_percentage && percentage < 100.0 {
                div { class: "meter-warning",
                    "Approaching limit - agent will wrap up current work"
                }
            } else if percentage >= 100.0 {
                div { class: "meter-critical",
                    "Rotation threshold reached - context will rotate"
                }
            }
        }
    }
}
