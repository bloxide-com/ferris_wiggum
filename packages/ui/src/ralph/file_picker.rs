use dioxus::prelude::*;

#[component]
fn DirectoryEntry(
    name: String,
    path: String,
    is_selected: bool,
    on_select: EventHandler<String>,
    on_navigate: EventHandler<String>,
) -> Element {
    let path_for_select = path.clone();
    let path_for_navigate = path.clone();
    rsx! {
        div {
            class: if is_selected { "file-picker-entry directory selected" } else { "file-picker-entry directory" },
            onclick: move |_| on_select.call(path_for_select.clone()),
            ondoubleclick: move |_| on_navigate.call(path_for_navigate.clone()),
            span { class: "entry-icon", "📁" }
            span { class: "entry-name", "{name}" }
        }
    }
}

#[component]
fn DirectoryEntryWrapper(
    entry: api::ralph::DirectoryEntry,
    selected_path: Signal<Option<String>>,
    on_select: EventHandler<String>,
    on_navigate: EventHandler<String>,
) -> Element {
    let is_selected = selected_path() == Some(entry.path.clone());
    rsx! {
        DirectoryEntry {
            name: entry.name.clone(),
            path: entry.path.clone(),
            is_selected,
            on_select,
            on_navigate,
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
    let mut selected_path = use_signal(|| None::<String>);
    let mut error = use_signal(|| None::<String>);
    let mut search_query = use_signal(|| String::new());

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

    // Filter entries based on search query (case-insensitive)
    let filtered_entries = use_memo(move || {
        let query = search_query().to_lowercase();
        if query.is_empty() {
            entries().clone()
        } else {
            entries()
                .iter()
                .filter(|entry| entry.name.to_lowercase().contains(&query))
                .cloned()
                .collect::<Vec<_>>()
        }
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
                selected_path.set(None); // Clear selection when navigating
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

    let handle_directory_select = move |path: String| {
        selected_path.set(Some(path.clone()));
    };

    let handle_directory_navigate = move |path: String| {
        error.set(None);
        selected_path.set(None); // Clear selection when navigating
        current_path.set(Some(path));
    };

    let confirm_selection = move |_| {
        if let Some(path) = selected_path() {
            // Validate that the path is a directory
            let path_clone = path.clone();
            spawn(async move {
                error.set(None);
                match api::ralph::list_directory(Some(path_clone.clone())).await {
                    Ok(_) => {
                        // Path is valid directory, confirm selection
                        value.set(path_clone.clone());
                        selected_path.set(None); // Clear selection after confirming
                        if let Some(handler) = on_select {
                            handler.call(path_clone);
                        }
                    }
                    Err(e) => {
                        error.set(Some(format!("Cannot select: {}", e)));
                        selected_path.set(None);
                    }
                }
            });
        }
    };

    let clear_search = move |_| {
        search_query.set(String::new());
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

            // Search input
            div { class: "file-picker-search",
                input {
                    r#type: "text",
                    placeholder: "Search directories...",
                    value: "{search_query}",
                    oninput: move |e| search_query.set(e.value()),
                    class: "search-input",
                }
                if !search_query().is_empty() {
                    button {
                        class: "btn-icon search-clear",
                        onclick: clear_search,
                        "✕"
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
                        if filtered_entries().is_empty() {
                            div { class: "empty-directory",
                                if search_query().is_empty() {
                                    "This directory is empty"
                                } else {
                                    "No directories match your search"
                                }
                            }
                        } else {
                            for entry in filtered_entries().iter() {
                                if entry.is_directory {
                                    DirectoryEntryWrapper {
                                        entry: entry.clone(),
                                        selected_path,
                                        on_select: handle_directory_select,
                                        on_navigate: handle_directory_navigate,
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

            // Selected path display and confirmation
            div { class: "file-picker-selection",
                label { "Selected Path:" }
                input {
                    r#type: "text",
                    value: "{value}",
                    oninput: move |e| value.set(e.value()),
                    placeholder: "No directory selected",
                }
                if let Some(selected) = selected_path() {
                    div { class: "selection-preview",
                        span { "Selected: " }
                        span { class: "selected-path-preview", "{selected}" }
                    }
                    button {
                        class: "btn btn-primary",
                        onclick: confirm_selection,
                        "Select"
                    }
                }
            }
        }
    }
}
