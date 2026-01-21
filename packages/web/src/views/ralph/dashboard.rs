use dioxus::prelude::*;
use ui::ralph::SessionList;

#[component]
pub fn RalphDashboard() -> Element {
    rsx! {
        div { class: "ralph-dashboard-page",
            SessionList {}
        }
    }
}
