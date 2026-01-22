use dioxus::prelude::*;

#[cfg(feature = "web")]
use web_sys::window;

const LAST_PATH_KEY: &str = "ralph_file_picker_last_path";

#[cfg(feature = "web")]
fn load_last_path() -> Option<String> {
    let window = window()?;
    let storage = window.local_storage().ok()??;
    storage.get_item(LAST_PATH_KEY).ok()?
}

#[cfg(feature = "web")]
fn save_last_path(path: &str) {
    if let Some(window) = window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item(LAST_PATH_KEY, path);
        }
    }
}

#[cfg(not(feature = "web"))]
fn load_last_path() -> Option<String> {
    None
}

#[cfg(not(feature = "web"))]
fn save_last_path(_path: &str) {
    // No-op for non-web platforms
}

#[component]
fn DirectoryEntry(
    name: String,
    path: String,
    is_selected: bool,
    is_protected: bool,
    on_select: EventHandler<String>,
    on_navigate: EventHandler<String>,
) -> Element {
    let path_for_select = path.clone();
    let path_for_navigate = path.clone();
    let class_str = if is_protected {
        if is_selected {
            "file-picker-entry directory protected selected"
        } else {
            "file-picker-entry directory protected"
        }
    } else if is_selected {
        "file-picker-entry directory selected"
    } else {
        "file-picker-entry directory"
    };
    
    rsx! {
        div {
            class: class_str,
            onclick: move |_| {
                if !is_protected {
                    on_select.call(path_for_select.clone());
                }
            },
            ondoubleclick: move |_| {
                if !is_protected {
                    on_navigate.call(path_for_navigate.clone());
                }
            },
            span { class: "entry-icon", 
                if is_protected { "🔒" } else { "📁" }
            }
            span { class: "entry-name", "{name}" }
            if is_protected {
                span { class: "protected-indicator", title: "Permission denied: You don't have access to this directory", "🔒" }
            }
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
            is_protected: entry.is_protected,
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
fn ShortcutItem(
    dir: api::ralph::CommonDirectory,
    on_navigate: EventHandler<String>,
) -> Element {
    let dir_path = dir.path.clone();
    rsx! {
        div {
            class: "shortcut-item",
            onclick: move |_| on_navigate.call(dir_path.clone()),
            span { class: "shortcut-icon", "{dir.icon}" }
            span { class: "shortcut-name", "{dir.name}" }
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

    // Resource for common directories
    let common_dirs = use_resource(move || async move {
        api::ralph::get_common_directories().await
    });

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

    // Initialize with last used path or home directory
    use_effect(move || {
        if current_path().is_none() {
            spawn(async move {
                error.set(None);
                // Try to load last path from localStorage (web only)
                let initial_path = load_last_path();
                
                match api::ralph::list_directory(initial_path.clone()).await {
                    Ok(listing) => {
                        let path = listing.current_path.clone();
                        current_path.set(Some(path.clone()));
                        // Save to localStorage if we loaded from it
                        if initial_path.is_some() {
                            save_last_path(&path);
                        }
                    }
                    Err(_) => {
                        // If last path doesn't exist or is invalid, fall back to home
                        match api::ralph::list_directory(None).await {
                            Ok(listing) => {
                                current_path.set(Some(listing.current_path.clone()));
                            }
                            Err(e) => {
                                error.set(Some(format!("Failed to load directory: {}", e)));
                            }
                        }
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
                        current_path.set(Some(parent_path.clone()));
                        save_last_path(&parent_path);
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
        // Trigger resource reload by setting current_path
        // The resource will handle permission errors
        current_path.set(Some(path.clone()));
        save_last_path(&path);
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
                        save_last_path(&path_clone);
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

    let handle_shortcut_navigate = move |path: String| {
        error.set(None);
        selected_path.set(None); // Clear selection when navigating
        current_path.set(Some(path.clone()));
        save_last_path(&path);
    };

    let navigate_to_home = move |_| {
        error.set(None);
        selected_path.set(None);
        spawn(async move {
            match api::ralph::list_directory(None).await {
                Ok(listing) => {
                    let path = listing.current_path.clone();
                    current_path.set(Some(path.clone()));
                    save_last_path(&path);
                }
                Err(e) => {
                    error.set(Some(format!("Failed to load home directory: {}", e)));
                }
            }
        });
    };

    rsx! {
        div { class: "file-picker",
            // Sidebar with common directories
            div { class: "file-picker-sidebar",
                div { class: "file-picker-shortcuts-header",
                    "Quick Access"
                }
                div { class: "file-picker-shortcuts",
                    match common_dirs() {
                        Some(Ok(dirs)) => rsx! {
                            if dirs.is_empty() {
                                div { class: "shortcuts-empty",
                                    "No shortcuts available"
                                }
                            } else {
                                for dir in dirs.iter() {
                                    ShortcutItem {
                                        dir: dir.clone(),
                                        on_navigate: handle_shortcut_navigate,
                                    }
                                }
                            }
                        },
                        Some(Err(_e)) => rsx! {
                            div { class: "shortcuts-error",
                                "Failed to load shortcuts"
                            }
                        },
                        None => rsx! {
                            div { class: "shortcuts-loading",
                                "Loading shortcuts..."
                            }
                        }
                    }
                }
            }

            // Main file browser
            div { class: "file-picker-main",
            // Current path display and navigation
            div { class: "file-picker-header",
                div { class: "file-picker-path",
                    button {
                        class: "btn-icon",
                        onclick: navigate_up,
                        disabled: current_path().is_none(),
                        title: "Go up one directory",
                        "↑"
                    }
                    button {
                        class: "btn-icon",
                        onclick: navigate_to_home,
                        title: "Go to home directory",
                        "🏠"
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
                    Some(Err(e)) => {
                        let error_str = e.to_string();
                        let is_permission_error = error_str.contains("Permission denied");
                        rsx! {
                            div { class: "error-message",
                                div { class: "error-icon", "⚠️" }
                                div { class: "error-text",
                                    div { class: "error-title",
                                        if is_permission_error {
                                            "Permission Denied"
                                        } else {
                                            "Error Loading Directory"
                                        }
                                    }
                                    div { class: "error-detail", "{error_str}" }
                                    if is_permission_error {
                                        div { class: "error-help",
                                            "You can navigate to other directories using the sidebar or path navigation."
                                        }
                                    }
                                }
                            }
                            // Show empty directory list but keep navigation functional
                            div { class: "file-picker-list",
                                div { class: "empty-directory",
                                    "Unable to load directory contents. Try navigating to another directory."
                                }
                            }
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
}
