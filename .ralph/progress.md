## 2026-01-24 - US-001
- Created `use_persisted_signal` hook in `packages/web/src/hooks.rs`
- Hook reads from localStorage on first render (after hydration)
- Hook writes to localStorage when signal changes with 100ms debounce using `gloo-timers`
- Hook handles JSON serialization/deserialization errors gracefully (falls back to initial value)
- Added serde and serde_json dependencies to `packages/web/Cargo.toml`
- Exported hook from `packages/web/src/main.rs`
- Files changed:
  - `packages/web/src/hooks.rs` (new)
  - `packages/web/Cargo.toml`
  - `packages/web/src/main.rs`
  - `prd.json`

**Learnings for future iterations:**
- Dioxus 0.7 `use_effect` is `FnMut` and can be called multiple times, so captured values must be clonable
- `use_effect` automatically tracks signals that are read inside the closure - reading `signal()` inside the effect makes it reactive
- localStorage access must be conditional on `#[cfg(feature = "web")]` to avoid unused import warnings when compiling server-only
- The web package uses feature flags: `web` for client-side and `server` for server-side - both should compile
- `gloo-timers` provides `TimeoutFuture` for async debouncing in web contexts
- When using `move` closures with multiple `use_effect` calls, clone values before the closures to avoid move errors
---

## 2026-01-24 - US-002
- Created `NewSessionDraft` struct in `packages/web/src/views/ralph/new_session.rs`
- Struct includes all 13 required fields: project_path_input, locked_project_path, prd_model, execution_model, max_iterations, warn_threshold, rotate_threshold, branch_name, open_pr, session_id, step, prd_mode, generated_prd_markdown
- Struct derives Serialize, Deserialize, Clone, PartialEq
- Made `SetupStep` and `PrdMode` enums public and added Serialize/Deserialize derives so they can be used in the public struct
- Added serde imports to the file
- Files changed:
  - `packages/web/src/views/ralph/new_session.rs`
  - `prd.json`

**Learnings for future iterations:**
- When a public struct contains enum fields, those enums must also be public to avoid visibility warnings
- The struct is currently unused (expected dead_code warning) - it will be integrated in US-003
- All enum variants must be serializable for the struct to work with localStorage persistence
---

## 2026-01-24 - US-003
- Integrated `use_persisted_signal` hook with `NewSessionDraft` struct in `RalphNewSession` component
- Replaced all 13 individual signals with a single persisted signal using key `ralph_new_session_draft`
- All form fields (project_path_input, locked_project_path, prd_model, execution_model, max_iterations, warn_threshold, rotate_threshold, branch_name, open_pr, session_id, step, prd_mode, generated_prd_markdown) now sync to localStorage automatically
- Form restores values on page load from localStorage
- localStorage is cleared when navigating to session page (successful completion) in `on_prd_set` callback
- Created local signal for FilePicker's `project_path_input` that syncs bidirectionally with draft using `use_effect` hooks
- Files changed:
  - `packages/web/src/views/ralph/new_session.rs`

**Learnings for future iterations:**
- When using `use_persisted_signal` with a struct, all field reads/writes must use `draft()` to read and `draft.write()` to mutate
- Signals captured in closures need to be cloned before the closure if used in multiple places (e.g., `create_session`, `on_prd_set`, `on_prd_generated`)
- FilePicker component writes to its `value` signal when user types, so a local signal synced with the draft is needed (can't use `Signal::derive` which is read-only)
- Bidirectional sync between signals requires careful `use_effect` conditions to prevent infinite loops - each effect should only update if values differ
- `web_sys::window().local_storage()` returns `Result<Option<Storage>, JsValue>`, not just `Option<Storage>` - need proper error handling
- Signals captured in `move` closures must be declared as `mut` if you need to call `.write()` on them
- When clearing localStorage on navigation, use `#[cfg(feature = "web")]` to conditionally compile the code
---

## 2026-01-24 - US-004
- Added Page Visibility API listener to `RalphNewSession` component
- Created a visibility state signal that tracks `document.visibilityState`
- Set up `visibilitychange` event listener using `wasm_bindgen::closure::Closure` in a `use_effect`
- When document becomes visible (`VisibilityState::Visible`), re-reads from localStorage and updates draft signal if values differ
- Added `wasm-bindgen` dependency and `VisibilityState` feature to `web-sys` in `Cargo.toml`
- Files changed:
  - `packages/web/src/views/ralph/new_session.rs`
  - `packages/web/Cargo.toml`
  - `prd.json`

**Learnings for future iterations:**
- `document.visibility_state()` returns a `VisibilityState` enum, not a string - need to compare with `VisibilityState::Visible`
- `wasm_bindgen::closure::Closure` requires closures to be `'static`, so signals can't be directly captured - use a signal to track state and a separate `use_effect` to react to changes
- Need to import `wasm_bindgen::JsCast` trait to use `unchecked_ref()` on closures for event listeners
- The `visibilitychange` event fires when the tab becomes visible/hidden, which handles phone lock/unlock scenarios
- Use `web_sys::window()` inside the closure instead of trying to capture the window value (which isn't `'static`)
- The closure must be kept alive with `closure.forget()` to prevent it from being dropped
- Pattern: use a signal to track browser state, update signal in event listener closure, react to signal changes in `use_effect`
---

## 2026-01-24 - US-005
- Modified `PrdConversation` component to restore conversation state from server on mount
- Component now calls `get_prd_conversation` first to check for existing conversation
- If conversation exists, displays existing messages and generated PRD (if any)
- If no conversation exists (returns `None`), starts a new conversation using `start_prd_conversation`
- Handles errors gracefully for both get and start operations
- Files changed:
  - `packages/ui/src/ralph/prd_conversation.rs`
  - `prd.json`

**Learnings for future iterations:**
- The `get_prd_conversation` server function returns `Result<Option<PrdConversation>, ServerFnError>` - need to handle both `Ok(None)` (no conversation) and `Ok(Some(conv))` (existing conversation) cases
- When restoring conversation, need to filter out system messages for display (same pattern as when starting new conversation)
- The `generated_prd` field should be restored if present in the existing conversation
- The restoration logic should only run once on mount (using `conversation_started` signal to prevent re-running)
- Both `get_prd_conversation` and `start_prd_conversation` return the same `PrdConversation` type, so the message filtering and PRD extraction logic can be shared
---

## 2026-01-26 - US-001
- Added mobile viewport meta tag to web app HTML head using `document::Meta` component
- Meta tag configured with `name="viewport"` and `content="width=device-width, initial-scale=1"`
- Placed in `App` component alongside other document resources (favicon, stylesheets)
- Files changed:
  - `packages/web/src/main.rs`
  - `prd.json`

**Learnings for future iterations:**
- Dioxus 0.7 uses `document::Meta` component to add meta tags to the HTML head, following the same pattern as `document::Link` for stylesheets
- The `document::Meta` component accepts `name` and `content` props for standard meta tags
- Meta tags should be placed in the root `App` component to ensure they're present on all pages
- Typecheck passes with `cargo check -p web --features web` validates the implementation
---

## 2026-01-26 - US-002
- Added CSS custom properties for responsive breakpoints in `packages/web/assets/main.css`
- Defined `--bp-mobile: 480px` and `--bp-tablet: 768px` in `:root` selector
- Added documentation comments explaining breakpoint usage in media queries
- Files changed:
  - `packages/web/assets/main.css`
  - `prd.json`

**Learnings for future iterations:**
- CSS custom properties for breakpoints should be defined in the `:root` selector in `main.css` for global availability
- Breakpoint variables can be used in media queries: `@media (max-width: var(--bp-mobile)) { ... }`
- These breakpoints provide consistent responsive thresholds across all components (mobile: 480px, tablet: 768px)
- Typecheck passes with `cargo check -p web --features web` validates the implementation
---

## 2026-01-26 - US-003
- Updated `.form-row` media query to use 480px breakpoint (mobile) instead of 960px
- Form rows now stack vertically (single-column) below 480px width
- Form inputs already had `width: 100%` so they're full-width
- Labels have adequate spacing (`margin-bottom: 0.4rem`)
- Verified in browser at 390px width using headless Chrome screenshot
- Files changed:
  - `packages/web/assets/styling/ralph.css`
  - `prd.json`

**Learnings for future iterations:**
- The existing media query at 960px was for `.ralph-main` layout, not form rows
- Form rows needed a separate media query at the mobile breakpoint (480px) to stack vertically
- Form inputs in `ralph.css` already have `width: 100%` so they're automatically full-width on mobile
- Browser verification can be done with headless Chrome: `google-chrome-stable --headless=new --window-size=390,900 --screenshot=path.png http://localhost:5080/new`
- The `--bp-mobile` CSS variable (480px) should be used consistently for mobile breakpoints across all components
---

## 2026-01-26 - US-001
- Added media query to reduce body padding from 24px to 12px at viewport width ≤ 480px
- Media query placed in `packages/web/assets/main.css` after the body styles
- Uses 480px breakpoint (matching `--bp-mobile` variable) for consistency
- Verified in browser at 390px width using headless Chrome screenshot
- Files changed:
  - `packages/web/assets/main.css`
  - `prd.json`

**Learnings for future iterations:**
- Body padding is defined in `packages/web/assets/main.css` at line 51
- Media queries for mobile should use the 480px breakpoint to match the `--bp-mobile` CSS variable
- CSS-only changes don't require Rust typechecking, but `cargo check -p web --features web` still validates the overall build
- Browser verification with headless Chrome confirms the padding reduction works correctly at mobile viewport widths
---

## 2026-01-26 - US-001
- Added `--mobile-breakpoint: 640px` CSS variable to `packages/web/assets/main.css`
- Added mobile-specific spacing variables: `--spacing-mobile-sm: 8px` and `--spacing-mobile-md: 16px`
- Added `--touch-target-min: 44px` variable for mobile accessibility compliance
- All variables added to `:root` selector for global availability
- Typecheck passes with `cargo check -p web --features web`
- Files changed:
  - `packages/web/assets/main.css`
  - `prd.json`

**Learnings for future iterations:**
- The PRD specifies `--mobile-breakpoint: 640px` which is different from the existing `--bp-mobile: 480px` variable - both serve different purposes (640px for mobile-first design, 480px for small mobile screens)
- Mobile spacing variables (`--spacing-mobile-sm`, `--spacing-mobile-md`) should be used consistently across mobile components for consistent spacing
- The `--touch-target-min: 44px` variable follows WCAG accessibility guidelines for minimum touch target size on mobile devices
- CSS variables in `:root` are globally available and can be referenced in any component stylesheet
- Typecheck validates the overall build even for CSS-only changes
---

## 2026-01-26 - US-002
- Created `BottomTabBar` component in `packages/ui/src/ralph/bottom_tab_bar.rs`
- Component accepts generic `Route` type and two route props (dashboard and new session)
- Uses `router()` function from Dioxus 0.7 to get current route and highlight active tab
- Tabs: Dashboard and New Session (Settings route doesn't exist, so only these two)
- Active tab visually highlighted with `active` class (blue color and background highlight)
- Fixed to bottom of viewport using `position: fixed; bottom: 0`
- Hidden on desktop (>640px) using media query `@media (min-width: 641px)`
- Visible on mobile (≤640px) using media query `@media (max-width: 640px)`
- Integrated into `WebNavbar` layout component in `packages/web/src/main.rs`
- All tabs meet minimum touch target size (44px) using `--touch-target-min` variable
- Verified in browser at 390px width using headless Chrome screenshots
- Files changed:
  - `packages/ui/src/ralph/bottom_tab_bar.rs` (new)
  - `packages/ui/assets/styling/bottom_tab_bar.css` (new)
  - `packages/ui/src/ralph/mod.rs`
  - `packages/web/src/main.rs`
  - `prd.json`

**Learnings for future iterations:**
- In Dioxus 0.7, use `router()` function (not `use_router()`) to get router context - it takes no generic parameters
- `router.current::<Route>()` returns the current route directly (not `Option<Route>`), so can be compared directly with route props
- The `BottomTabBar` component is generic over `Route: Routable + Clone + PartialEq` to work with any route enum
- CSS media queries should use the `--mobile-breakpoint: 640px` variable for mobile-first responsive design
- Fixed bottom navigation requires `z-index` to ensure it appears above content (set to 1000)
- Touch targets should use `min-height: var(--touch-target-min, 44px)` for accessibility compliance
- The component uses CSS variables with fallbacks (e.g., `var(--surface, #151a23)`) so it works even if web `main.css` isn't loaded
- Browser verification with headless Chrome confirms the tab bar appears on mobile and is hidden on desktop
---
