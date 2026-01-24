use crate::types::*;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

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
        mut activity_callback: impl FnMut(ActivityEntry),
    ) -> Result<(), RalphError> {
        tracing::info!("Starting cursor-agent iteration with model {}", self.model);
        tracing::debug!("Prompt length: {} chars", prompt.len());

        // Spawn cursor-agent CLI
        tracing::debug!("Spawning cursor-agent in {}", self.project_path);
        let mut child = Command::new("cursor-agent")
            .arg("-p")
            .arg("--output-format")
            .arg("stream-json")
            .arg("--force")
            .arg("--model")
            .arg(&self.model)
            .arg(prompt)
            .current_dir(&self.project_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                tracing::error!("Failed to spawn cursor-agent: {}", e);
                RalphError::CursorAgent(format!("Failed to spawn cursor-agent: {}", e))
            })?;

        let stdout = child.stdout.take().ok_or_else(|| {
            tracing::error!("Failed to capture cursor-agent stdout");
            RalphError::CursorAgent("Failed to capture stdout".into())
        })?;

        let mut reader = BufReader::new(stdout).lines();

        // Read stream-json output line by line
        tracing::debug!("Reading cursor-agent output stream");
        while let Ok(Some(line)) = reader.next_line().await {
            if line.trim().is_empty() {
                continue;
            }

            // Parse JSON line
            match serde_json::from_str::<serde_json::Value>(&line) {
                Ok(json) => {
                    if let Some(activity) = self.parse_activity(&json) {
                        tracing::trace!("Activity: {:?}", activity.kind);
                        activity_callback(activity);
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to parse JSON line: {} - {}", e, line);
                }
            }
        }

        // Wait for process to complete
        tracing::debug!("Waiting for cursor-agent to complete");
        let status = child.wait().await.map_err(|e| {
            tracing::error!("Failed to wait for cursor-agent: {}", e);
            RalphError::CursorAgent(format!("Failed to wait for cursor-agent: {}", e))
        })?;

        if !status.success() {
            tracing::error!("cursor-agent exited with status: {}", status);
            return Err(RalphError::CursorAgent(format!(
                "cursor-agent exited with status: {}",
                status
            )));
        }

        tracing::info!("cursor-agent iteration completed successfully");
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

    pub async fn terminate(&self) -> Result<(), RalphError> {
        // Kill any running cursor-agent processes for this project
        // This is a placeholder - actual implementation would need process tracking
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
