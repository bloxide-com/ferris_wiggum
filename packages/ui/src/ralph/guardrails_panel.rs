use dioxus::prelude::*;
use ralph::Guardrail;

#[component]
pub fn GuardrailsPanel(session_id: ReadSignal<String>) -> Element {
    let guardrails = use_resource(move || async move {
        let result: Result<Vec<Guardrail>, _> = api::ralph::get_guardrails(session_id()).await;
        result.unwrap_or_default()
    });

    rsx! {
        div { class: "guardrails-panel",
            h3 { "Guardrails (Signs)" }

            match guardrails() {
                Some(list) => {
                    if list.is_empty() {
                        rsx! {
                            div { class: "guardrails-empty",
                                "No guardrails yet. They'll be added as Ralph learns from failures."
                            }
                        }
                    } else {
                        rsx! {
                            div { class: "guardrails-list",
                                for guardrail in list {
                                    GuardrailCard { guardrail }
                                }
                            }
                        }
                    }
                },
                None => rsx! {
                    div { class: "loading", "Loading guardrails..." }
                }
            }
        }
    }
}

#[component]
fn GuardrailCard(guardrail: Guardrail) -> Element {
    rsx! {
        div { class: "guardrail-card",
            h4 { "🚧 {guardrail.title}" }

            div { class: "guardrail-content",
                div { class: "guardrail-item",
                    strong { "Trigger: " }
                    span { "{guardrail.trigger}" }
                }

                div { class: "guardrail-item",
                    strong { "Do: " }
                    span { "{guardrail.instruction}" }
                }

                div { class: "guardrail-context",
                    em { "Added after: {guardrail.added_after}" }
                }
            }
        }
    }
}
