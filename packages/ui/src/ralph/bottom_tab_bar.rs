use dioxus::prelude::*;

const BOTTOM_TAB_BAR_CSS: Asset = asset!("/assets/styling/bottom_tab_bar.css");

#[component]
pub fn BottomTabBar<Route: Routable + Clone + PartialEq>(
    dashboard_route: Route,
    new_session_route: Route,
) -> Element {
    let router = router();
    let current_route = router.current::<Route>();

    // Determine active tab based on current route
    let is_dashboard_active = current_route == dashboard_route;
    let is_new_session_active = current_route == new_session_route;

    rsx! {
        document::Link { rel: "stylesheet", href: BOTTOM_TAB_BAR_CSS }

        nav {
            id: "bottom-tab-bar",
            div {
                class: "tab-item",
                class: if is_dashboard_active { "active" },
                Link {
                    to: dashboard_route.clone(),
                    "Dashboard"
                }
            }
            div {
                class: "tab-item",
                class: if is_new_session_active { "active" },
                Link {
                    to: new_session_route.clone(),
                    "New Session"
                }
            }
        }
    }
}
