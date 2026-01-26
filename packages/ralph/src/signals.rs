use crate::types::Signal;

pub struct SignalHandler;

impl SignalHandler {
    pub fn should_warn(total_tokens: u32, threshold: u32) -> bool {
        total_tokens >= threshold
    }

    pub fn should_rotate(total_tokens: u32, threshold: u32) -> bool {
        total_tokens >= threshold
    }

    pub fn format_signal(signal: &Signal) -> String {
        match signal {
            Signal::Warn => {
                "⚠️  WARN: Approaching token limit. Wrap up current work and commit.".to_string()
            }
            Signal::Rotate => {
                "🔄 ROTATE: Token limit reached. Committing and starting fresh iteration."
                    .to_string()
            }
            Signal::Gutter(reason) => {
                format!("🚨 GUTTER: Stuck state detected. {}", reason)
            }
            Signal::Complete => "✅ COMPLETE: All stories have passed!".to_string(),
            Signal::StoryComplete(id) => {
                format!("✓ Story {} completed", id)
            }
        }
    }
}

#[cfg(feature = "server")]
pub async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(e) = tokio::signal::ctrl_c().await {
            tracing::error!("Failed to listen for Ctrl+C: {}", e);
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut sigterm) => {
                sigterm.recv().await;
            }
            Err(e) => {
                tracing::error!("Failed to install SIGTERM handler: {}", e);
                // If we can't install SIGTERM, don't block shutdown forever.
                tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_warn() {
        assert!(!SignalHandler::should_warn(60_000, 70_000));
        assert!(SignalHandler::should_warn(70_000, 70_000));
        assert!(SignalHandler::should_warn(75_000, 70_000));
    }

    #[test]
    fn test_should_rotate() {
        assert!(!SignalHandler::should_rotate(75_000, 80_000));
        assert!(SignalHandler::should_rotate(80_000, 80_000));
        assert!(SignalHandler::should_rotate(85_000, 80_000));
    }

    #[test]
    fn test_format_signal() {
        assert!(SignalHandler::format_signal(&Signal::Warn).contains("WARN"));
        assert!(SignalHandler::format_signal(&Signal::Rotate).contains("ROTATE"));
        assert!(SignalHandler::format_signal(&Signal::Complete).contains("COMPLETE"));

        let gutter = Signal::Gutter("test reason".to_string());
        assert!(SignalHandler::format_signal(&gutter).contains("GUTTER"));
        assert!(SignalHandler::format_signal(&gutter).contains("test reason"));
    }
}
