use dioxus::prelude::*;

#[component]
fn DirectoryEntry(
    name: String,
    path: String,
    on_navigate: EventHandler<String>,
) -> Element {
    rsx! {
        div {
            class: "file-picker-entry directory",
            onclick: move |_| on_navigate.call(path.clone()),
            span { class: "entry-icon", "📁" }
            span { class: "entry-name", "{name}" }
        }
    }
}

#[component]
fn FileEntry(name: String) -> Element {
    rsx! {
        div {
            class: "file-picker-entry file",
            span { class: "entry-icon", "📄" }
            span { class: "entry-name", "{name}" }
        }
    }
}

#[component]
pub fn FilePicker(
    value: Signal<String>,
    on_select: Option<EventHandler<String>>,
) -> Element {
    let mut current_path = use_signal(|| None::<String>);
    let mut error = use_signal(|| None::<String>);

    // Resource that reloads when current_path changes
    let directory_listing = use_resource(move || {
        let path = current_path();
        async move {
            api::ralph::list_directory(path).await
        }
    });

    // Memoize entries to avoid lifetime issues
    let entries = use_memo(move || {
        directory_listing()
            .and_then(|r| r.ok())
            .map(|l| l.entries.clone())
            .unwrap_or_default()
    });

    // Initialize with home directory if not set
    use_effect(move || {
        if current_path().is_none() {
            spawn(async move {
                error.set(None);
                match api::ralph::list_directory(None).await {
                    Ok(listing) => {
                        current_path.set(Some(listing.current_path.clone()));
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to load directory: {}", e)));
                    }
                }
            });
        }
    });

    let navigate_up = move |_| {
        let current = current_path();
        if let Some(path) = current {
            spawn(async move {
                error.set(None);
                match api::ralph::get_parent_directory(path).await {
                    Ok(Some(parent_path)) => {
                        current_path.set(Some(parent_path));
                    }
                    Ok(None) => {
                        // Already at root, can't go up
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to navigate up: {}", e)));
                    }
                }
            });
        }
    };

    rsx! {
        div { class: "file-picker",
            // Current path display and navigation
            div { class: "file-picker-header",
                div { class: "file-picker-path",
                    button {
                        class: "btn-icon",
                        onclick: navigate_up,
                        disabled: current_path().is_none(),
                        "↑"
                    }
                    span { class: "current-path",
                        if let Some(path) = current_path() {
                            "{path}"
                        } else {
                            {"Loading..."}
                        }
                    }
                }
            }

            // Error message
            if let Some(err) = error() {
                div { class: "error-message",
                    "{err}"
                }
            }

            // Directory listing
            div { class: "file-picker-list",
                match directory_listing() {
                    Some(Ok(_)) => rsx! {
                        if entries().is_empty() {
                            div { class: "empty-directory",
                                "This directory is empty"
                            }
                        } else {
                            for entry in entries().iter() {
                                if entry.is_directory {
                                    DirectoryEntry {
                                        name: entry.name.clone(),
                                        path: entry.path.clone(),
                                        on_navigate: move |path| {
                                            error.set(None);
                                            current_path.set(Some(path));
                                        }
                                    }
                                } else {
                                    FileEntry {
                                        name: entry.name.clone()
                                    }
                                }
                            }
                        }
                    },
                    Some(Err(e)) => rsx! {
                        div { class: "error-message",
                            "Error loading directory: {e}"
                        }
                    },
                    None => rsx! {
                        div { class: "loading",
                            "Loading directory..."
                        }
                    }
                }
            }

            // Selected path display
            div { class: "file-picker-selection",
                label { "Selected Path:" }
                input {
                    r#type: "text",
                    value: "{value}",
                    oninput: move |e| value.set(e.value()),
                    placeholder: "No directory selected",
                }
                if !value().is_empty() {
                    button {
                        class: "btn btn-primary",
                        onclick: move |_| {
                            if let Some(path) = current_path() {
                                value.set(path.clone());
                                if let Some(handler) = on_select {
                                    handler.call(path);
                                }
                            }
                        },
                        "Select Current Directory"
                    }
                }
            }
        }
    }
}
