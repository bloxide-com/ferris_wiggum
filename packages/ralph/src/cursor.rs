use crate::types::*;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::broadcast;

/// Retry subprocess spawn with exponential backoff for transient failures
async fn spawn_with_retry(
    mut command: Command,
    max_retries: u32,
) -> Result<tokio::process::Child, std::io::Error> {
    let mut last_error = None;
    
    for attempt in 0..=max_retries {
        match command.spawn() {
            Ok(child) => return Ok(child),
            Err(e) => {
                // Check if error is transient (too many open files, resource unavailable, etc.)
                let is_transient = matches!(
                    e.kind(),
                    std::io::ErrorKind::WouldBlock
                        | std::io::ErrorKind::ResourceBusy
                        | std::io::ErrorKind::Interrupted
                );
                
                // Also check raw OS error codes
                let is_transient = is_transient || matches!(
                    e.raw_os_error(),
                    Some(11) | // EAGAIN
                    Some(24) | // EMFILE (too many open files)
                    Some(23)   // ENFILE (system file table overflow)
                );
                
                if !is_transient || attempt == max_retries {
                    return Err(e);
                }
                
                last_error = Some(e);
                let delay_ms = 100 * 2u64.pow(attempt);
                tracing::warn!(
                    "Transient error spawning subprocess (attempt {}/{}): {:?}. Retrying in {}ms...",
                    attempt + 1,
                    max_retries + 1,
                    last_error.as_ref().unwrap(),
                    delay_ms
                );
                tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
            }
        }
    }
    
    Err(last_error.unwrap())
}

pub struct CursorRunner {
    project_path: String,
    model: String,
}

impl CursorRunner {
    pub fn new(project_path: String, model: String) -> Self {
        Self {
            project_path,
            model,
        }
    }

    pub async fn run_iteration(
        &self,
        prompt: &str,
        mut shutdown_rx: broadcast::Receiver<()>,
        mut activity_callback: impl FnMut(ActivityEntry),
    ) -> Result<(), RalphError> {
        tracing::info!("=== Starting cursor-agent iteration ===");
        tracing::info!("Model: {}", self.model);
        tracing::info!("Project path: {}", self.project_path);
        tracing::info!("Prompt length: {} chars", prompt.len());
        tracing::debug!("First 200 chars of prompt: {}", &prompt.chars().take(200).collect::<String>());

        // Spawn cursor-agent CLI with retry for transient failures
        tracing::debug!("Spawning cursor-agent in {}", self.project_path);
        
        let mut command = Command::new("cursor-agent");
        command
            .arg("-p")
            .arg("--output-format")
            .arg("stream-json")
            .arg("--force")
            .arg("--model")
            .arg(&self.model)
            .arg(prompt)
            .current_dir(&self.project_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        
        let mut child = match spawn_with_retry(command, 3).await {
            Ok(child) => {
                tracing::info!("cursor-agent spawned successfully (PID: {:?})", child.id());
                child
            }
            Err(e) => {
                tracing::error!("Failed to spawn cursor-agent: {}", e);
                tracing::error!("Error kind: {:?}", e.kind());
                tracing::error!("Working directory: {}", self.project_path);
                tracing::error!("Model: {}", self.model);
                
                // Check if cursor-agent is in PATH
                if let Err(which_err) = std::process::Command::new("which").arg("cursor-agent").output() {
                    tracing::error!("Failed to check cursor-agent availability: {}", which_err);
                }
                
                return Err(RalphError::CursorAgent(format!(
                    "Failed to spawn cursor-agent: {} (kind: {:?})", e, e.kind()
                )));
            }
        };

        let stdout = child.stdout.take().ok_or_else(|| {
            tracing::error!("Failed to capture cursor-agent stdout");
            RalphError::CursorAgent("Failed to capture stdout".into())
        })?;

        // Drain stderr to prevent buffer deadlock
        if let Some(stderr) = child.stderr.take() {
            let pid = child.id();
            tokio::spawn(async move {
                tracing::debug!("Started stderr reader for cursor-agent PID {:?}", pid);
                let mut reader = BufReader::new(stderr).lines();
                let mut line_count = 0;
                while let Ok(Some(line)) = reader.next_line().await {
                    if !line.is_empty() {
                        line_count += 1;
                        tracing::warn!("cursor-agent[{:?}] stderr line {}: {}", pid, line_count, line);
                    }
                }
                tracing::debug!("Stderr reader completed for PID {:?} ({} lines)", pid, line_count);
            });
        }

        let mut reader = BufReader::new(stdout).lines();

        // Read stream-json output line by line
        tracing::info!("📖 Reading cursor-agent output stream...");
        let mut line_count = 0;
        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    tracing::info!("Shutdown received, terminating cursor-agent");
                    self.terminate(&mut child).await?;
                    return Err(RalphError::CursorAgent("cursor-agent terminated due to shutdown".into()));
                }
                line = reader.next_line() => {
                    let Ok(line) = line else { 
                        tracing::debug!("stdout reader: end of stream or error");
                        break 
                    };
                    let Some(line) = line else { 
                        tracing::debug!("stdout reader: no more lines");
                        break 
                    };
                    if line.trim().is_empty() {
                        continue;
                    }

                    line_count += 1;
                    if line_count % 10 == 0 {
                        tracing::debug!("Processed {} lines from cursor-agent", line_count);
                    }

                    // Parse JSON line
                    match serde_json::from_str::<serde_json::Value>(&line) {
                        Ok(json) => {
                            if let Some(activity) = self.parse_activity(&json) {
                                tracing::debug!("Activity #{}: {:?}", line_count, activity.kind);
                                activity_callback(activity);
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Failed to parse JSON line #{}: {} - {}", line_count, e, line);
                        }
                    }
                }
            }
        }

        // Wait for process to complete
        tracing::info!("📥 Processed {} output lines, waiting for cursor-agent to complete", line_count);
        let status = tokio::select! {
            _ = shutdown_rx.recv() => {
                tracing::info!("Shutdown received while waiting, terminating cursor-agent");
                self.terminate(&mut child).await?;
                return Err(RalphError::CursorAgent("cursor-agent terminated due to shutdown".into()));
            }
            result = tokio::time::timeout(std::time::Duration::from_secs(600), child.wait()) => {
                match result {
                    Ok(status) => {
                        status.map_err(|e| {
                            tracing::error!("Failed to wait for cursor-agent: {}", e);
                            RalphError::CursorAgent(format!("Failed to wait for cursor-agent: {}", e))
                        })?
                    }
                    Err(_) => {
                        tracing::error!("cursor-agent timed out after 10 minutes");
                        self.terminate(&mut child).await?;
                        return Err(RalphError::CursorAgent("cursor-agent timed out after 10 minutes".into()));
                    }
                }
            }
        };

        if !status.success() {
            tracing::error!("❌ cursor-agent exited with non-zero status: {}", status);
            if let Some(code) = status.code() {
                tracing::error!("Exit code: {}", code);
            }
            return Err(RalphError::CursorAgent(format!(
                "cursor-agent exited with status: {}",
                status
            )));
        }

        tracing::info!("✅ cursor-agent iteration completed successfully ({} lines processed)", line_count);
        Ok(())
    }

    fn parse_activity(&self, json: &serde_json::Value) -> Option<ActivityEntry> {
        let kind = json.get("type")?.as_str()?;

        let activity_kind = match kind {
            "read" => {
                let path = json.get("path")?.as_str()?.to_string();
                let lines = json.get("lines")?.as_u64()? as u32;
                let bytes = json.get("bytes")?.as_u64()? as u32;
                tracing::debug!("Read: {} ({} lines, {} bytes)", path, lines, bytes);
                ActivityKind::Read { path, lines, bytes }
            }
            "write" => {
                let path = json.get("path")?.as_str()?.to_string();
                let lines = json.get("lines")?.as_u64()? as u32;
                let bytes = json.get("bytes")?.as_u64()? as u32;
                tracing::debug!("Write: {} ({} lines, {} bytes)", path, lines, bytes);
                ActivityKind::Write { path, lines, bytes }
            }
            "shell" => {
                let command = json.get("command")?.as_str()?.to_string();
                let exit_code = json.get("exit_code")?.as_i64()? as i32;
                tracing::debug!("Shell: {} (exit code: {})", command, exit_code);
                ActivityKind::Shell { command, exit_code }
            }
            "error" => {
                let message = json.get("message")?.as_str()?.to_string();
                tracing::warn!("Error from cursor-agent: {}", message);
                ActivityKind::Error(message)
            }
            _ => {
                tracing::trace!("Ignoring unknown activity type: {}", kind);
                return None;
            }
        };

        Some(ActivityEntry {
            timestamp: std::time::SystemTime::now(),
            iteration: 0, // Will be set by caller
            kind: activity_kind,
            health: ContextHealth::Healthy, // Will be updated by caller
        })
    }

    pub async fn terminate(&self, child: &mut tokio::process::Child) -> Result<(), RalphError> {
        // Best-effort termination of the running cursor-agent process.
        if let Err(e) = child.kill().await {
            tracing::warn!("Failed to kill cursor-agent process: {}", e);
            return Err(RalphError::CursorAgent(format!(
                "Failed to kill cursor-agent process: {}",
                e
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cursor_runner_creation() {
        let runner = CursorRunner::new(
            "/tmp/test-project".to_string(),
            "opus-4.5-thinking".to_string(),
        );

        assert_eq!(runner.project_path, "/tmp/test-project");
        assert_eq!(runner.model, "opus-4.5-thinking");
    }
}
