use dioxus::prelude::*;
use ralph::Branch;

#[component]
pub fn BranchSelector(
    project_path: Signal<String>,
    on_branch_change: EventHandler<String>,
) -> Element {
    let mut refresh_nonce = use_signal(|| 0_u32);
    let mut switching = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);

    let branches = use_resource(move || {
        let project_path = project_path();
        let _nonce = refresh_nonce();
        async move { api::ralph::list_branches(project_path).await }
    });

    let current_branch = use_memo(move || match branches() {
        Some(Ok(branches)) => branches
            .iter()
            .find(|b| b.is_current)
            .map(|b| b.name.clone()),
        _ => None,
    });

    let on_select_branch = move |e: Event<FormData>| {
        let branch = e.value();
        if branch.trim().is_empty() {
            return;
        }

        let project_path = project_path();
        spawn(async move {
            error.set(None);
            switching.set(true);
            let result: Result<(), _> = api::ralph::checkout_branch(project_path, branch.clone()).await;
            switching.set(false);

            match result {
                Ok(()) => {
                    refresh_nonce.with_mut(|n| *n = n.wrapping_add(1));
                    on_branch_change.call(branch);
                }
                Err(e) => {
                    error.set(Some(e.to_string()));
                }
            }
        });
    };

    rsx! {
        div { class: "branch-selector",
            match branches() {
                Some(Ok(branches)) => {
                    let current_value = current_branch().unwrap_or_default();
                    rsx! {
                        select {
                            class: "branch-select",
                            disabled: switching(),
                            value: "{current_value}",
                            onchange: on_select_branch,
                            for Branch { name, is_current: _, is_remote: _ } in branches.into_iter() {
                                option {
                                    value: "{name}",
                                    "{name}"
                                }
                            }
                        }
                    }
                }
                Some(Err(e)) => rsx! {
                    div { class: "git-error", "Failed to load branches: {e}" }
                },
                None => rsx! {
                    div { class: "git-loading", "Loading branches..." }
                }
            }

            if let Some(err) = error() {
                div { class: "git-error", "{err}" }
            }
        }
    }
}

