use dioxus::prelude::*;
use ralph::{MessageRole, ConversationMessage};

#[component]
pub fn PrdConversation(
    session_id: String,
    on_prd_generated: EventHandler<String>,
) -> Element {
    let session_id = use_signal(|| session_id);
    let mut messages = use_signal(|| Vec::<ConversationMessage>::new());
    let mut input_text = use_signal(|| String::new());
    let mut loading = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let mut generated_prd = use_signal(|| None::<String>);
    let mut conversation_started = use_signal(|| false);

    // Start conversation on mount
    use_effect(move || {
        if !conversation_started() {
            conversation_started.set(true);
            let session_id = session_id();
            spawn(async move {
                loading.set(true);
                error.set(None);

                match api::ralph::start_prd_conversation(session_id).await {
                    Ok(conv) => {
                        // Filter out system messages for display
                        let display_messages: Vec<_> = conv
                            .messages
                            .into_iter()
                            .filter(|m| !matches!(m.role, MessageRole::System))
                            .collect();
                        messages.set(display_messages);
                        if let Some(prd) = conv.generated_prd {
                            generated_prd.set(Some(prd));
                        }
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to start conversation: {:?}", e)));
                    }
                }
                loading.set(false);
            });
        }
    });

    // Helper to perform the actual send
    let mut do_send = move || {
        let session_id = session_id();
        let message = input_text();
        if message.trim().is_empty() || loading() {
            return;
        }

        // Clear input immediately
        input_text.set(String::new());
        loading.set(true);
        error.set(None);

        spawn(async move {
            match api::ralph::send_prd_message(session_id, message).await {
                Ok(conv) => {
                    // Update with full conversation (excluding system messages)
                    let display_messages: Vec<_> = conv
                        .messages
                        .into_iter()
                        .filter(|m| !matches!(m.role, MessageRole::System))
                        .collect();
                    messages.set(display_messages);
                    if let Some(prd) = conv.generated_prd {
                        generated_prd.set(Some(prd));
                    }
                }
                Err(e) => {
                    error.set(Some(format!("Failed to send message: {:?}", e)));
                }
            }
            loading.set(false);
        });
    };

    let handle_keydown = move |e: KeyboardEvent| {
        if e.key() == Key::Enter && !e.modifiers().shift() {
            e.prevent_default();
            do_send();
        }
    };

    let use_generated_prd = move |_| {
        if let Some(prd) = generated_prd() {
            on_prd_generated.call(prd);
        }
    };

    rsx! {
        div { class: "prd-conversation",
            div { class: "conversation-messages",
                if messages().is_empty() && !loading() {
                    div { class: "conversation-empty",
                        p { "Starting conversation..." }
                    }
                }

                for (idx, message) in messages().iter().enumerate() {
                    MessageBubble {
                        key: "{idx}",
                        message: message.clone(),
                    }
                }

                if loading() {
                    div { class: "message-bubble assistant loading",
                        div { class: "typing-indicator",
                            span {}
                            span {}
                            span {}
                        }
                    }
                }
            }

            if let Some(err) = error() {
                div { class: "conversation-error",
                    "{err}"
                }
            }

            if generated_prd().is_some() {
                div { class: "prd-detected",
                    div { class: "prd-detected-content",
                        span { class: "prd-detected-icon", "✓" }
                        span { "PRD has been generated from the conversation" }
                    }
                    button {
                        onclick: use_generated_prd,
                        class: "btn btn-primary",
                        "Use Generated PRD"
                    }
                }
            }

            div { class: "conversation-input",
                textarea {
                    class: "message-input",
                    value: "{input_text}",
                    oninput: move |e| input_text.set(e.value()),
                    onkeydown: handle_keydown,
                    placeholder: "Describe your feature or answer the agent's questions...",
                    disabled: loading(),
                    rows: "3",
                }
                button {
                    onclick: move |_| do_send(),
                    disabled: loading() || input_text().trim().is_empty(),
                    class: "btn btn-send",
                    if loading() { "..." } else { "Send" }
                }
            }
        }
    }
}

#[component]
fn MessageBubble(message: ConversationMessage) -> Element {
    let role_class = match message.role {
        MessageRole::User => "user",
        MessageRole::Assistant => "assistant",
        MessageRole::System => "system",
    };

    rsx! {
        div { class: "message-bubble {role_class}",
            div { class: "message-content",
                // Render markdown-like content for assistant messages
                if matches!(message.role, MessageRole::Assistant) {
                    FormattedMessage { content: message.content }
                } else {
                    p { "{message.content}" }
                }
            }
        }
    }
}

#[component]
fn FormattedMessage(content: String) -> Element {
    // Simple markdown-like formatting for assistant messages
    let paragraphs: Vec<&str> = content.split("\n\n").collect();

    rsx! {
        div { class: "formatted-message",
            for (idx, para) in paragraphs.iter().enumerate() {
                if para.starts_with("# ") {
                    h3 { key: "{idx}", "{para.trim_start_matches(\"# \")}" }
                } else if para.starts_with("## ") {
                    h4 { key: "{idx}", "{para.trim_start_matches(\"## \")}" }
                } else if para.starts_with("### ") {
                    h5 { key: "{idx}", "{para.trim_start_matches(\"### \")}" }
                } else if para.starts_with("- ") || para.starts_with("* ") {
                    ul { key: "{idx}",
                        for (li_idx, line) in para.lines().enumerate() {
                            if line.starts_with("- ") || line.starts_with("* ") {
                                li { key: "{li_idx}", "{line.trim_start_matches(\"- \").trim_start_matches(\"* \")}" }
                            }
                        }
                    }
                } else if para.starts_with("```") {
                    pre { key: "{idx}",
                        code { "{para.trim_start_matches(\"```\").trim_start_matches(\"markdown\").trim_end_matches(\"```\").trim()}" }
                    }
                } else if !para.trim().is_empty() {
                    p { key: "{idx}", "{para}" }
                }
            }
        }
    }
}
