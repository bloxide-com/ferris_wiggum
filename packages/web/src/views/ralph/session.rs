use dioxus::prelude::*;
use ui::ralph::SessionDashboard;

#[component]
pub fn RalphSession(id: String) -> Element {
    let session_id = use_signal(|| id);

    rsx! {
        div { class: "ralph-session-page",
            SessionDashboard { session_id }
        }
    }
}
