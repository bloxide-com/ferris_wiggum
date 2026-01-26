use super::BranchSelector;
use dioxus::prelude::*;

const GIT_PANEL_CSS: Asset = asset!("/assets/styling/git_panel.css");

#[component]
pub fn GitPanel(session_id: String, project_path: String) -> Element {
    // session_id is currently unused but kept to match the plan and to allow future
    // session-scoped git behaviors.
    let _ = session_id;

    let project_path = use_signal(|| project_path);
    let mut refresh_nonce = use_signal(|| 0_u32);

    let mut selected_branch = use_signal(|| String::new());
    let mut action_error = use_signal(|| None::<String>);
    let mut action_success = use_signal(|| None::<String>);

    let mut pr_title = use_signal(|| String::new());
    let mut pr_body = use_signal(|| String::new());
    let mut pr_url = use_signal(|| None::<String>);

    let mut merge_source = use_signal(|| String::new());
    let mut confirm_merge = use_signal(|| false);

    let mut creating_pr = use_signal(|| false);
    let mut merging = use_signal(|| false);
    let mut pushing = use_signal(|| false);

    let branches = use_resource(move || {
        let project_path = project_path();
        let _nonce = refresh_nonce();
        async move { api::ralph::list_branches(project_path).await }
    });

    let current_branch = use_memo(move || match branches() {
        Some(Ok(branches)) => branches
            .iter()
            .find(|b| b.is_current)
            .map(|b| b.name.clone())
            .unwrap_or_default(),
        _ => String::new(),
    });

    // Keep local selected_branch in sync with current branch.
    use_effect(move || {
        let current = current_branch();
        if !current.is_empty() && selected_branch().is_empty() {
            selected_branch.set(current);
        }
    });

    let merge_candidates = use_memo(move || match branches() {
        Some(Ok(branches)) => {
            let current = current_branch();
            branches
                .iter()
                .filter(|b| b.name != current)
                .map(|b| b.name.clone())
                .collect::<Vec<_>>()
        }
        _ => Vec::new(),
    });

    let on_branch_change = move |branch: String| {
        action_error.set(None);
        action_success.set(None);
        pr_url.set(None);
        selected_branch.set(branch);
        confirm_merge.set(false);
        refresh_nonce.with_mut(|n| *n = n.wrapping_add(1));
    };

    let on_create_pr = move |_| {
        let project_path = project_path();
        let branch = selected_branch();
        let title = pr_title();
        let body = pr_body();

        if branch.trim().is_empty() || title.trim().is_empty() {
            action_error.set(Some("PR requires a branch and a title.".to_string()));
            return;
        }

        spawn(async move {
            action_error.set(None);
            action_success.set(None);
            pr_url.set(None);
            creating_pr.set(true);

            let result: Result<String, _> =
                api::ralph::create_pull_request(project_path, branch.clone(), title, body).await;

            creating_pr.set(false);
            match result {
                Ok(url) => {
                    pr_url.set(Some(url.clone()));
                    action_success.set(Some("Pull request created.".to_string()));
                }
                Err(e) => action_error.set(Some(e.to_string())),
            }
        });
    };

    let on_merge = move |_| {
        let project_path = project_path();
        let source = merge_source();

        if source.trim().is_empty() {
            action_error.set(Some("Select a source branch to merge.".to_string()));
            return;
        }

        if !confirm_merge() {
            action_error.set(Some("Please confirm the merge before proceeding.".to_string()));
            return;
        }

        spawn(async move {
            action_error.set(None);
            action_success.set(None);
            merging.set(true);
            let result: Result<(), _> = api::ralph::merge_branches(project_path, source.clone()).await;
            merging.set(false);

            match result {
                Ok(()) => {
                    action_success.set(Some("Merge completed.".to_string()));
                    confirm_merge.set(false);
                    refresh_nonce.with_mut(|n| *n = n.wrapping_add(1));
                }
                Err(e) => action_error.set(Some(e.to_string())),
            }
        });
    };

    let on_push = move |_| {
        let project_path = project_path();
        let branch = selected_branch();
        let branch_opt = if branch.trim().is_empty() {
            None
        } else {
            Some(branch)
        };

        spawn(async move {
            action_error.set(None);
            action_success.set(None);
            pushing.set(true);
            let result: Result<(), _> = api::ralph::push_branch(project_path, branch_opt).await;
            pushing.set(false);

            match result {
                Ok(()) => action_success.set(Some("Push completed.".to_string())),
                Err(e) => action_error.set(Some(e.to_string())),
            }
        });
    };

    rsx! {
        document::Stylesheet { href: GIT_PANEL_CSS }

        div { class: "git-panel",
            div { class: "git-panel-header",
                h3 { "Git" }
                if !current_branch().is_empty() {
                    span { class: "git-branch-badge", "On {current_branch}" }
                }
            }

            match branches() {
                Some(Ok(_branches)) => rsx! {
                    div { class: "git-panel-section",
                        label { class: "git-label", "Branch" }
                        BranchSelector {
                            project_path,
                            on_branch_change
                        }
                    }

                    div { class: "git-panel-section",
                        label { class: "git-label", "Pull request" }
                        input {
                            r#type: "text",
                            placeholder: "Title",
                            value: "{pr_title}",
                            oninput: move |e| pr_title.set(e.value()),
                            class: "git-input",
                        }
                        textarea {
                            placeholder: "Body",
                            value: "{pr_body}",
                            oninput: move |e| pr_body.set(e.value()),
                            class: "git-textarea",
                        }
                        button {
                            class: "btn btn-primary",
                            onclick: on_create_pr,
                            disabled: creating_pr(),
                            if creating_pr() { "Creating PR..." } else { "Create PR" }
                        }
                        if let Some(url) = pr_url() {
                            div { class: "git-pr-url", "{url}" }
                        }
                    }

                    div { class: "git-panel-section",
                        label { class: "git-label", "Merge" }
                        select {
                            class: "git-select",
                            value: "{merge_source}",
                            onchange: move |e| {
                                merge_source.set(e.value());
                                confirm_merge.set(false);
                            },
                            option { value: "", "Select source branch..." }
                            for name in merge_candidates().into_iter() {
                                option { value: "{name}", "{name}" }
                            }
                        }
                        label { class: "git-merge-confirm",
                            input {
                                r#type: "checkbox",
                                checked: confirm_merge(),
                                onchange: move |e| confirm_merge.set(e.checked()),
                            }
                            span { "I understand this will merge into the current branch." }
                        }
                        button {
                            class: "btn btn-secondary",
                            onclick: on_merge,
                            disabled: merging() || merge_source().trim().is_empty() || !confirm_merge(),
                            if merging() { "Merging..." } else { "Merge into current" }
                        }
                    }

                    div { class: "git-panel-actions",
                        button {
                            class: "btn btn-secondary",
                            onclick: on_push,
                            disabled: pushing(),
                            if pushing() { "Pushing..." } else { "Push" }
                        }
                    }
                },
                Some(Err(e)) => rsx! {
                    div { class: "git-error", "Failed to load git info: {e}" }
                },
                None => rsx! {
                    div { class: "git-loading", "Loading git info..." }
                }
            }

            if let Some(msg) = action_success() {
                div { class: "git-success", "{msg}" }
            }
            if let Some(msg) = action_error() {
                div { class: "git-error", "{msg}" }
            }
        }
    }
}

