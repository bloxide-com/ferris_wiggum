use dioxus::prelude::*;

use ui::Navbar;
use views::{RalphDashboard, RalphNewSession, RalphSession};
#[cfg(feature = "server")]
use api::ralph::init_background_tasks;

mod hooks;
mod views;

pub use hooks::use_persisted_signal;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(WebNavbar)]
    #[route("/")]
    RalphDashboard {},
    #[route("/new")]
    RalphNewSession {},
    #[route("/:id")]
    RalphSession { id: String },
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const RALPH_CSS: Asset = asset!("/assets/styling/ralph.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    // Build cool things ✌️

    #[cfg(feature = "server")]
    init_background_tasks();

    rsx! {
        // Global app resources
        document::Meta { name: "viewport", content: "width=device-width, initial-scale=1" }
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: RALPH_CSS }

        Router::<Route> {}
    }
}

/// A web-specific Router around the shared `Navbar` component
/// which allows us to use the web-specific `Route` enum.
#[component]
fn WebNavbar() -> Element {
    rsx! {
        Navbar {
            Link {
                to: Route::RalphDashboard {},
                "Dashboard"
            }
        }

        Outlet::<Route> {}
    }
}
