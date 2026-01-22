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
