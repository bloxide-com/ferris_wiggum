pub mod parser;
pub mod signals;
pub mod types;

#[cfg(feature = "server")]
pub mod conversation;
#[cfg(feature = "server")]
pub mod cursor;
#[cfg(feature = "server")]
pub mod git;
#[cfg(feature = "server")]
pub mod guardrails;
#[cfg(feature = "server")]
pub mod memory;
#[cfg(feature = "server")]
pub mod session;

#[cfg(feature = "server")]
pub use conversation::PrdConversationManager;
#[cfg(feature = "server")]
pub use cursor::CursorRunner;
#[cfg(feature = "server")]
pub use git::GitOperations;
#[cfg(feature = "server")]
pub use guardrails::GuardrailManager;
#[cfg(feature = "server")]
pub use memory::{run_memory_monitor, run_health_watchdog, MemoryMonitor, MemorySnapshot, MemoryStatus};
pub use parser::StreamParser;
#[cfg(feature = "server")]
pub use session::SessionManager;
pub use signals::SignalHandler;
#[cfg(feature = "server")]
pub use signals::shutdown_signal;
pub use types::*;
