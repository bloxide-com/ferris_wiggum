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

## 2026-01-24 - US-001
- Fixed hardcoded model bug in PrdConversationManager
- Removed `model` field from `PrdConversationManager` struct - model is now passed dynamically to methods
- Updated `start_conversation` and `send_message` methods to accept `model` parameter
- Updated `generate_response` to accept `model` parameter instead of using hardcoded value
- Modified API functions `start_prd_conversation` and `send_prd_message` to extract model from `SessionConfig` and pass it to manager methods
- Updated static `CONVERSATION_MANAGER` initialization to not require a model parameter
- Added unit test to verify manager accepts model parameter dynamically
- Files changed:
  - `packages/ralph/src/conversation.rs` - Removed model field, updated methods to accept model parameter, updated tests
  - `packages/api/src/ralph.rs` - Updated API functions to get model from session config and pass to manager methods
  - `prd.json` - Updated US-001 to passes: true

**Learnings for future iterations:**
- When removing hardcoded values, pass them as parameters to methods rather than storing in struct fields to allow dynamic configuration
- API server functions should extract configuration from session/context and pass to underlying managers rather than using global defaults
- The `PrdConversationManager` maintains conversation state in a HashMap, so it should remain a singleton, but configuration (like model) should be passed per-operation
- When updating method signatures, remember to update all call sites including tests
- Type system ensures model flows correctly from SessionConfig -> API -> Manager -> cursor-agent command
---

## 2026-01-24 - US-002
- Added "Auto" option to model dropdown in new session form
- The "Auto" option appears first in the dropdown list with descriptive text "Auto (cursor-agent picks best model)"
- Verified that selecting "auto" passes `--model auto` to cursor-agent (model string is passed directly via Command::arg())
- Added unit test `test_auto_model_handling` to verify auto model option is accepted and passed through correctly
- Files changed:
  - `packages/web/src/views/ralph/new_session.rs` - Added "auto" option to model dropdown
  - `packages/ralph/src/conversation.rs` - Added unit test for auto model handling
  - `prd.json` - Updated US-002 to passes: true

**Learnings for future iterations:**
- Model dropdown options can be added simply by adding new `option` elements to the `select` component
- The model value is passed as a string directly to cursor-agent via `Command::arg()`, so any string value (including "auto") will be passed through correctly without special handling
- When adding new model options, place the most commonly used or recommended option first in the dropdown for better UX
- Unit tests for model handling can verify that string values are accepted without needing to actually spawn cursor-agent processes
---

## 2026-01-24 - US-003
- Split model settings into separate `prd_model` and `execution_model` fields in `SessionConfig`
- Updated PRD generation functions (`start_prd_conversation`, `send_prd_message`) to use `prd_model` from session config
- Updated execution phase (`run_iteration` in `SessionManager`) to use `execution_model` when creating `CursorRunner`
- Updated UI to set both fields (currently both set to same value from single dropdown; UI split will come in US-004)
- Updated session dashboard to display both PRD Model and Execution Model stats
- Added unit tests to verify `SessionConfig` structure supports separate models
- Files changed:
  - `packages/ralph/src/types.rs` - Added `prd_model` and `execution_model` fields to `SessionConfig`, updated `Default` implementation
  - `packages/api/src/ralph.rs` - Updated `start_prd_conversation` and `send_prd_message` to use `prd_model`
  - `packages/ralph/src/session.rs` - Updated `run_iteration` to use `execution_model`, added unit test
  - `packages/ralph/src/conversation.rs` - Added unit test for PRD model usage
  - `packages/web/src/views/ralph/new_session.rs` - Updated to set both `prd_model` and `execution_model` fields
  - `packages/ui/src/ralph/session_dashboard.rs` - Updated to display both model fields in stats
  - `prd.json` - Updated US-003 to passes: true

**Learnings for future iterations:**
- When splitting a single field into multiple fields, update all access points: struct definition, Default impl, API functions, execution code, UI creation, and UI display
- The UI currently sets both fields to the same value from a single dropdown - this maintains backward compatibility while preparing for the UI split in US-004
- Session dashboard now shows both "PRD Model" and "Execution Model" stats to make the separation visible to users
- Unit tests verify the structure supports separate models even if integration tests aren't running - the types.rs tests confirm the config structure is correct
- Both models default to "opus-4.5-thinking" in the Default implementation for consistency
---

## 2026-01-24 - US-004
- Split model dropdown into two separate dropdowns: "PRD Model" and "Execution Model"
- Replaced single `model` signal with `prd_model` and `execution_model` signals
- Both dropdowns include all model options including "Auto" option
- Updated form submission to use both model values independently
- Added form styles to ralph.css for proper layout (form-group, form-row, form-help, etc.)
- Form now displays both dropdowns side-by-side in a form-row layout
- Files changed:
  - `packages/web/src/views/ralph/new_session.rs` - Split model dropdown into two separate dropdowns, updated signals and form submission
  - `packages/web/assets/styling/ralph.css` - Added form styles for new session page (form-group, form-row, form-help, session-form, etc.)
  - `prd.json` - Updated US-004 to passes: true

**Learnings for future iterations:**
- When splitting UI controls, replace the single signal with separate signals for each control
- Use `form-row` class with CSS grid (`grid-template-columns: 1fr 1fr`) to display related form fields side-by-side
- Both dropdowns should have identical option lists to maintain consistency
- Form help text (`form-help` class) provides context for each field and improves UX
- The form submission already uses `SessionConfig` with separate fields, so no backend changes were needed
- Default values for both models are set to "opus-4.5-thinking" to match the backend defaults
---
