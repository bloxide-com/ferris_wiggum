## 2025-01-27 - US-001
- Implemented file system browsing functionality with a FilePicker component
- Created server functions `list_directory` and `get_parent_directory` in `packages/api/src/ralph.rs` for file system operations
- Built FilePicker component in `packages/ui/src/ralph/file_picker.rs` with directory navigation UI
- Integrated FilePicker into new session form (`packages/web/src/views/ralph/new_session.rs`) replacing the text input
- Added CSS styling for file picker in `packages/web/assets/styling/ralph.css`
- Files changed:
  - `packages/api/src/ralph.rs` - Added file system browsing server functions
  - `packages/api/Cargo.toml` - Added `dirs` dependency
  - `packages/ui/src/ralph/file_picker.rs` - New FilePicker component
  - `packages/ui/src/ralph/mod.rs` - Exported FilePicker
  - `packages/web/src/views/ralph/new_session.rs` - Integrated FilePicker
  - `packages/web/assets/styling/ralph.css` - Added file picker styles
  - `prd.json` - Updated US-001 to passes: true

**Learnings for future iterations:**
- Dioxus 0.7 requires using `rsx!` macro for match arms when returning JSX from match expressions
- When using closures in `for` loops within `rsx!`, create separate components to avoid lifetime issues with captured variables
- Server functions in Dioxus fullstack automatically handle serialization - types used in server functions are available on both client and server
- Use `use_memo` to memoize computed values that depend on resources to avoid lifetime issues
- The `dirs` crate provides cross-platform access to standard directories (home, documents, etc.)
- File picker component uses `use_resource` hook which automatically reloads when dependencies (like `current_path` signal) change
---

## 2025-01-27 - US-002
- Implemented directory selection functionality in FilePicker component
- Added single-click to select directories (double-click navigates into them)
- Added visual highlighting for selected directories (blue background with left border)
- Added "Select" button that appears when a directory is selected
- Implemented validation before confirming selection (checks directory exists via API)
- Selected path is returned to parent component via value signal and optional on_select handler
- Files changed:
  - `packages/ui/src/ralph/file_picker.rs` - Added selection state, handlers, and DirectoryEntryWrapper component
  - `packages/web/assets/styling/ralph.css` - Added styles for selected directory and selection preview
  - `prd.json` - Updated US-002 to passes: true

**Learnings for future iterations:**
- When using multiple event handlers (onclick, ondoubleclick) on the same element, clone values before creating closures to avoid move issues
- Use `ondoubleclick` instead of deprecated `ondblclick` in Dioxus 0.7
- Cannot use `let` statements directly inside `for` loops within `rsx!` - create wrapper components instead
- Selection state should be cleared when navigating to maintain clean UI state
- Validate selections server-side before confirming to ensure directory still exists and is accessible
---

## 2026-01-22 - US-003
- Implemented search and filter functionality for directories in FilePicker component
- Added search input field that filters directories as user types (case-insensitive)
- Added clear button (✕) that appears when search query is not empty
- Filtered entries are computed using `use_memo` for performance (no delay)
- Search filters directories by name matching (case-insensitive)
- Empty state message changes based on whether search is active or directory is empty
- Files changed:
  - `packages/ui/src/ralph/file_picker.rs` - Added search_query signal, filtered_entries memo, search UI, and clear handler
  - `packages/web/assets/styling/ralph.css` - Added styles for search input and clear button
  - `prd.json` - Updated US-003 to passes: true

**Learnings for future iterations:**
- Use `use_memo` to filter collections based on reactive state (like search queries) - this ensures efficient re-computation only when dependencies change
- Case-insensitive search can be implemented by converting both query and entry names to lowercase before comparison
- Conditional rendering of UI elements (like clear button) can be done with `if` statements directly in `rsx!`
- When filtering entries, show different empty state messages based on whether filtering is active or the source is empty
- Search input should have focus styles for better UX (border color change and subtle shadow)
---

## 2026-01-22 - US-004
- Implemented quick access sidebar with common directory shortcuts in FilePicker component
- Added server function `get_common_directories` that returns platform-appropriate common locations (Home, Documents, Desktop, Downloads, and common project folders)
- Created sidebar UI with clickable shortcuts that navigate to the selected directory
- Shortcuts display with icons and names for better visual identification
- Sidebar is positioned on the left side of the file picker with proper styling
- Files changed:
  - `packages/api/src/ralph.rs` - Added `get_common_directories` server function and `CommonDirectory` struct
  - `packages/ui/src/ralph/file_picker.rs` - Added sidebar with shortcuts, `ShortcutItem` component, and navigation handler
  - `packages/web/assets/styling/ralph.css` - Added styles for sidebar, shortcuts, and shortcut items
  - `prd.json` - Updated US-004 to passes: true

**Learnings for future iterations:**
- The `dirs` crate provides platform-specific directory access: `home_dir()`, `document_dir()`, `desktop_dir()`, `download_dir()` - these automatically work on Linux, macOS, and Windows
- When checking for common project folders, iterate through common names and only add the first one that exists to avoid duplicates
- Sidebar layout uses flexbox with `flex-shrink: 0` to maintain fixed width while main content area uses `flex: 1` to fill remaining space
- Shortcut items should use hover states for better UX feedback
- Create wrapper components (like `ShortcutItem`) when you need to use `let` statements or complex logic within `for` loops in `rsx!`
- The `use_resource` hook automatically handles loading states - use `None` for loading, `Some(Ok(...))` for success, and `Some(Err(...))` for errors
---

## 2026-01-22 - US-005
- Implemented permission error handling in FilePicker component
- Enhanced `list_directory` server function to detect permission errors specifically and mark protected directories
- Added `is_protected` field to `DirectoryEntry` struct to track directories with permission restrictions
- Updated DirectoryEntry component to show lock icon (🔒) for protected directories and prevent selection/navigation
- Improved error messages with clear titles, details, and helpful guidance for permission errors
- File picker remains fully functional after errors - users can navigate to alternate directories using sidebar or path navigation
- Added CSS styles for protected directories (reduced opacity, red hover state, lock indicator)
- Enhanced error message display with icon, structured layout, and contextual help text
- Files changed:
  - `packages/api/src/ralph.rs` - Added `is_protected` field to DirectoryEntry, enhanced permission detection in `list_directory`
  - `packages/ui/src/ralph/file_picker.rs` - Updated DirectoryEntry to show lock icons, prevent interaction with protected dirs, improved error rendering
  - `packages/web/assets/styling/ralph.css` - Added styles for protected directories and enhanced error messages
  - `prd.json` - Updated US-005 to passes: true

**Learnings for future iterations:**
- Use `std::fs::read_dir()` on directories to check permissions before marking them as protected - this allows showing lock icons proactively
- Check `std::io::ErrorKind::PermissionDenied` to specifically identify permission errors vs other I/O errors for better error messages
- When rendering conditional content inside `rsx!` blocks (especially in match arms), compute values outside the rsx! block first, then use simple conditionals inside
- Protected directories should have `cursor: not-allowed` and prevent onclick/ondoubleclick handlers from executing
- Error messages should be structured with icon, title, detail, and optional help text for better UX
- Always ensure the file picker remains functional after errors - show error message but keep navigation controls (sidebar, up button) working
- Use visual indicators (lock icons, reduced opacity, different colors) to clearly distinguish protected directories from accessible ones
---

## 2026-01-22 - US-006
- Implemented localStorage persistence for last selected/browsed directory path in FilePicker component
- Added `web-sys` dependency to web package and created "web" feature flag in ui package for conditional compilation
- Created `load_last_path()` and `save_last_path()` helper functions that use browser localStorage (web) or no-op (other platforms)
- File picker now loads last used directory from localStorage on component mount (after hydration)
- Path is automatically saved to localStorage whenever user navigates (up, into directory, via shortcuts) or confirms a selection
- Added "Home" button (🏠) in header to allow users to override and navigate to home directory
- Falls back gracefully to home directory if last path doesn't exist or is invalid
- Files changed:
  - `packages/ui/Cargo.toml` - Added web-sys as optional dependency and "web" feature flag
  - `packages/web/Cargo.toml` - Added web-sys dependency and enabled "web" feature on ui dependency
  - `packages/ui/src/ralph/file_picker.rs` - Added localStorage save/load functions, updated initialization and navigation handlers to persist path, added Home button
  - `prd.json` - Updated US-006 to passes: true

**Learnings for future iterations:**
- Browser-specific APIs (like localStorage) must be accessed after hydration - use `use_effect` hook which runs after hydration
- Use conditional compilation (`#[cfg(feature = "web")]`) to make platform-specific code only compile for the appropriate platform
- When sharing code between platforms (like ui crate), add platform-specific dependencies as optional and create feature flags
- localStorage access should be wrapped in error handling since it may not be available (private browsing mode, etc.)
- Save path to localStorage on all navigation actions (up, into directory, shortcuts, selection) to ensure it's always up-to-date
- Provide a way for users to override persisted state (like Home button) for better UX
- The `use_effect` hook runs after hydration, so browser APIs are safe to access inside it
---

## 2026-01-22 - US-007
- Implemented visual Git repository indicators in FilePicker component
- Added `is_git_repository` field to `DirectoryEntry` struct to track directories containing `.git` folders
- Updated `list_directory` server function to detect Git repositories by checking for `.git` directory existence
- Added Git indicator icon (🔀) that appears next to directories containing Git repositories
- Added filter toggle checkbox to show only Git repositories when enabled
- Updated `filtered_entries` memo to respect both search query and Git filter
- Added CSS styles for Git indicator (green color) and filter toggle checkbox
- Files changed:
  - `packages/api/src/ralph.rs` - Added `is_git_repository` field to DirectoryEntry, added Git detection logic in `list_directory`
  - `packages/ui/src/ralph/file_picker.rs` - Added Git indicator to DirectoryEntry component, added `show_only_git` signal and filter toggle UI, updated filtering logic
  - `packages/web/assets/styling/ralph.css` - Added styles for Git indicator and filter toggle
  - `prd.json` - Updated US-007 to passes: true

**Learnings for future iterations:**
- Check for Git repositories by testing if `.git` directory exists and is a directory (not just a file)
- Git detection should only run on accessible directories (skip protected directories to avoid unnecessary checks)
- When adding filter toggles, combine them with existing filters (like search) in a single memoized computation for efficiency
- Use checkbox inputs with labels for better accessibility and UX
- Git indicators should be visually distinct (green color) and positioned consistently (right side before protected indicator)
- Filter toggles should be placed near search inputs for logical grouping of filtering controls
---

## 2026-01-24 09:24:09 EST - US-005
- Added an explicit “Create session” button (separate from “Select”) and gated it on a locked repo path matching the currently displayed path, preventing accidental/implicit session creation.
- Files changed:
  - `AGENTS.md`
  - `Cargo.lock`
  - `packages/api/src/ralph.rs`
  - `packages/ralph/src/conversation.rs`
  - `packages/ralph/src/cursor.rs`
  - `packages/ralph/src/git.rs`
  - `packages/ralph/src/guardrails.rs`
  - `packages/ralph/src/lib.rs`
  - `packages/ralph/src/parser.rs`
  - `packages/ralph/src/session.rs`
  - `packages/ralph/src/signals.rs`
  - `packages/ralph/src/types.rs`
  - `packages/ui/src/ralph/activity_log.rs`
  - `packages/ui/src/ralph/file_picker.rs`
  - `packages/ui/src/ralph/guardrails_panel.rs`
  - `packages/ui/src/ralph/mod.rs`
  - `packages/ui/src/ralph/prd_conversation.rs`
  - `packages/ui/src/ralph/session_dashboard.rs`
  - `packages/ui/src/ralph/session_list.rs`
  - `packages/ui/src/ralph/story_progress.rs`
  - `packages/ui/src/ralph/token_meter.rs`
  - `packages/web/Cargo.toml`
  - `packages/web/src/main.rs`
  - `packages/web/src/views/mod.rs`
  - `packages/web/src/views/ralph/mod.rs`
  - `packages/web/src/views/ralph/new_session.rs`
  - `prd.json`
- **Learnings for future iterations:**
  - Treat the file picker’s path as transient and only proceed from an explicitly “locked” path; also gate “Create session” if the visible path has changed since locking.
  - There is no `.ralph/guardrails.md` in this repo currently; guardrails knowledge is likely captured in code (`packages/ralph/src/guardrails.rs`) and in this progress log instead.
  - No browser automation/e2e harness (Playwright/Cypress/etc.) appears configured in the repo right now; “Verify in browser” is currently a manual step.
---
