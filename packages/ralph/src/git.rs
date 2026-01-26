use crate::types::*;
use std::path::Path;
use tokio::process::Command;

pub struct GitOperations {
    project_path: String,
}

impl GitOperations {
    pub fn new(project_path: String) -> Self {
        Self { project_path }
    }

    pub async fn fetch(&self) -> Result<(), RalphError> {
        self.run_git_command(&["fetch", "--all", "--prune"]).await?;
        Ok(())
    }

    pub async fn list_branches(&self) -> Result<Vec<Branch>, RalphError> {
        // git branch output format:
        // * main
        //   feature/foo
        let output = self.run_git_command(&["branch"]).await?;
        let mut branches = Vec::new();

        for line in output.lines() {
            let trimmed = line.trim_end();
            if trimmed.is_empty() {
                continue;
            }

            let is_current = trimmed.starts_with('*');
            let name = trimmed
                .trim_start_matches('*')
                .trim_start()
                .to_string();

            if name.is_empty() {
                continue;
            }

            branches.push(Branch {
                name,
                is_current,
                is_remote: false,
            });
        }

        Ok(branches)
    }

    pub async fn list_remote_branches(&self) -> Result<Vec<String>, RalphError> {
        // Ensure we have up-to-date remotes.
        self.fetch().await?;

        // git branch -r output format:
        //   origin/HEAD -> origin/main
        //   origin/main
        //   origin/feature/foo
        let output = self.run_git_command(&["branch", "-r"]).await?;
        let mut branches = Vec::new();

        for line in output.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Skip symbolic ref line: origin/HEAD -> origin/main
            if trimmed.contains("->") {
                continue;
            }

            branches.push(trimmed.to_string());
        }

        Ok(branches)
    }

    pub async fn checkout(&self, branch: &str) -> Result<(), RalphError> {
        self.run_git_command(&["checkout", branch]).await?;
        Ok(())
    }

    pub async fn merge(&self, source_branch: &str) -> Result<(), RalphError> {
        self.run_git_command(&["merge", source_branch]).await?;
        Ok(())
    }

    pub async fn commit(&self, message: &str) -> Result<(), RalphError> {
        tracing::info!("Committing changes: {}", message);
        // Stage all changes
        tracing::debug!("Staging all changes in {}", self.project_path);
        self.run_git_command(&["add", "-A"]).await?;

        // Commit
        self.run_git_command(&["commit", "-m", message]).await?;
        tracing::info!("Commit successful");

        Ok(())
    }

    pub async fn create_branch(&self, branch_name: &str) -> Result<(), RalphError> {
        // Check if branch exists
        let output = self
            .run_git_command(&["branch", "--list", branch_name])
            .await?;

        if output.trim().is_empty() {
            // Branch doesn't exist, create it
            self.run_git_command(&["checkout", "-b", branch_name])
                .await?;
        } else {
            // Branch exists, checkout
            self.run_git_command(&["checkout", branch_name]).await?;
        }

        Ok(())
    }

    pub async fn push(&self, branch: Option<&str>) -> Result<(), RalphError> {
        let args = if let Some(b) = branch {
            vec!["push", "-u", "origin", b]
        } else {
            vec!["push"]
        };

        self.run_git_command(&args).await?;
        Ok(())
    }

    pub async fn create_pr(
        &self,
        branch: &str,
        title: &str,
        body: &str,
    ) -> Result<String, RalphError> {
        // Use gh CLI to create PR
        let output = Command::new("gh")
            .args(&[
                "pr", "create", "--head", branch, "--title", title, "--body", body,
            ])
            .current_dir(&self.project_path)
            .output()
            .await
            .map_err(|e| RalphError::Git(format!("Failed to create PR: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(RalphError::Git(format!("gh pr create failed: {}", stderr)));
        }

        let pr_url = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(pr_url)
    }

    pub async fn get_current_branch(&self) -> Result<String, RalphError> {
        let output = self.run_git_command(&["branch", "--show-current"]).await?;
        Ok(output.trim().to_string())
    }

    pub async fn has_changes(&self) -> Result<bool, RalphError> {
        let output = self.run_git_command(&["status", "--porcelain"]).await?;
        Ok(!output.trim().is_empty())
    }

    pub async fn get_last_commit_message(&self) -> Result<String, RalphError> {
        let output = self.run_git_command(&["log", "-1", "--pretty=%B"]).await?;
        Ok(output.trim().to_string())
    }

    async fn run_git_command(&self, args: &[&str]) -> Result<String, RalphError> {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.project_path)
            .output()
            .await
            .map_err(|e| RalphError::Git(format!("Failed to run git command: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(RalphError::Git(format!(
                "git {} failed: {}",
                args[0], stderr
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub async fn init_ralph_directory(&self) -> Result<(), RalphError> {
        tracing::info!("Initializing .ralph directory in {}", self.project_path);
        let ralph_dir = Path::new(&self.project_path).join(".ralph");
        tokio::fs::create_dir_all(&ralph_dir).await?;

        // Create default files
        let progress_file = ralph_dir.join("progress.md");
        if !progress_file.exists() {
            tracing::debug!("Creating progress.md");
            tokio::fs::write(&progress_file, "# Ralph Progress Log\n\n").await?;
        }

        let guardrails_file = ralph_dir.join("guardrails.md");
        if !guardrails_file.exists() {
            tracing::debug!("Creating guardrails.md");
            tokio::fs::write(&guardrails_file, "# Ralph Guardrails (Signs)\n\n").await?;
        }

        let activity_log = ralph_dir.join("activity.log");
        if !activity_log.exists() {
            tracing::debug!("Creating activity.log");
            tokio::fs::write(&activity_log, "").await?;
        }

        tracing::info!(".ralph directory initialized successfully");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_operations_creation() {
        let git = GitOperations::new("/tmp/test-project".to_string());
        assert_eq!(git.project_path, "/tmp/test-project");
    }
}
