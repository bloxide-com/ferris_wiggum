use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "web")]
use web_sys::window;

/// A hook that persists a signal's value to localStorage with automatic sync.
///
/// The hook:
/// - Reads from localStorage on first render (after hydration)
/// - Writes to localStorage when the signal changes (debounced 100ms)
/// - Handles JSON serialization errors gracefully (falls back to default)
///
/// # Arguments
/// * `key` - The localStorage key to use
/// * `initial` - The initial value if localStorage is empty or invalid
///
/// # Returns
/// A `Signal<T>` that automatically syncs with localStorage
#[cfg(feature = "web")]
pub fn use_persisted_signal<T>(key: impl Into<String>, initial: impl FnOnce() -> T) -> Signal<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Clone + PartialEq + 'static,
{
    let key = key.into();
    let key_for_read = key.clone();
    let key_for_write = key.clone();
    let mut signal = use_signal(initial);
    let mut is_initial_load = use_signal(|| true);

    // Read from localStorage on mount (after hydration) - only once
    use_effect(move || {
        if !is_initial_load() {
            return;
        }
        is_initial_load.set(false);

        let key = key_for_read.clone();
        let window = window();
        if let Some(window) = window {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(Some(stored)) = storage.get_item(&key) {
                    if let Ok(deserialized) = serde_json::from_str::<T>(&stored) {
                        signal.set(deserialized);
                    }
                    // If deserialization fails, fall back to initial value (already set)
                }
            }
        }
    });

    // Debounced write to localStorage when signal changes
    // This effect re-runs whenever signal() changes (because we read it here)
    use_effect(move || {
        // Skip write on initial load (we just read from storage)
        if is_initial_load() {
            return;
        }

        let key = key_for_write.clone();
        // Read signal here so the effect tracks it and re-runs on changes
        let value = signal().clone();

        use gloo_timers::future::TimeoutFuture;
        spawn(async move {
            // Debounce: wait 100ms before writing
            TimeoutFuture::new(100).await;

            let window = window();
            if let Some(window) = window {
                if let Ok(Some(storage)) = window.local_storage() {
                    // Serialize and store, ignoring errors
                    if let Ok(json) = serde_json::to_string(&value) {
                        let _ = storage.set_item(&key, &json);
                    }
                    // If serialization fails, silently ignore (value stays in signal)
                }
            }
        });
    });

    signal
}

/// Fallback implementation for non-web platforms (returns a regular signal)
#[cfg(not(feature = "web"))]
pub fn use_persisted_signal<T>(_key: impl Into<String>, initial: impl FnOnce() -> T) -> Signal<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Clone + PartialEq + 'static,
{
    use_signal(initial)
}
