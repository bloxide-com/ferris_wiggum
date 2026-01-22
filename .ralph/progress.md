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
