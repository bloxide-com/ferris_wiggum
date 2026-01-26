use dioxus::prelude::*;
use ralph::{Branch, Guardrail, Prd, PrdConversation, Session, SessionConfig};

#[cfg(feature = "server")]
use ralph::GitOperations;

#[cfg(feature = "server")]
use ralph::{GuardrailManager, PrdConversationManager, SessionManager};
#[cfg(feature = "server")]
use std::sync::Arc;

// Global session manager
#[cfg(feature = "server")]
lazy_static::lazy_static! {
    static ref SESSION_MANAGER: Arc<SessionManager> = Arc::new(SessionManager::new());
    static ref CONVERSATION_MANAGER: Arc<PrdConversationManager> = Arc::new(PrdConversationManager::new());
}

// Session Management

#[server]
pub async fn create_session(
    project_path: String,
    config: SessionConfig,
) -> Result<Session, ServerFnError> {
    tracing::info!("Creating session for project: {}", project_path);
    tracing::debug!("Session config: {:?}", config);

    let session = SESSION_MANAGER
        .create_session(project_path.clone(), config)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create session for {}: {}", project_path, e);
            ServerFnError::new(e.to_string())
        })?;

    tracing::info!("Session created successfully: {}", session.id);
    Ok(session)
}

#[server]
pub async fn list_sessions() -> Result<Vec<Session>, ServerFnError> {
    tracing::debug!("Listing all sessions");
    let sessions = SESSION_MANAGER.list_sessions().await;
    tracing::info!("Found {} sessions", sessions.len());
    Ok(sessions)
}

#[server]
pub async fn get_session(id: String) -> Result<Session, ServerFnError> {
    SESSION_MANAGER
        .get_session(&id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn start_session(id: String) -> Result<Session, ServerFnError> {
    SESSION_MANAGER
        .start_session(&id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn pause_session(id: String) -> Result<Session, ServerFnError> {
    tracing::info!("Pausing session: {}", id);
    SESSION_MANAGER
        .pause_session(&id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to pause session {}: {}", id, e);
            ServerFnError::new(e.to_string())
        })
        .inspect(|_| {
            tracing::info!("Session {} paused successfully", id);
        })
}

#[server]
pub async fn stop_session(id: String) -> Result<Session, ServerFnError> {
    SESSION_MANAGER
        .stop_session(&id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

// Git Operations

#[server]
pub async fn get_current_branch(project_path: String) -> Result<String, ServerFnError> {
    #[cfg(feature = "server")]
    {
    let git = GitOperations::new(project_path);
    git.get_current_branch()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
    }

    #[cfg(not(feature = "server"))]
    {
        let _ = project_path;
        Err(ServerFnError::new(
            "Git operations are only available on the server".to_string(),
        ))
    }
}

#[server]
pub async fn list_branches(project_path: String) -> Result<Vec<Branch>, ServerFnError> {
    #[cfg(feature = "server")]
    {
    let git = GitOperations::new(project_path);
    git.list_branches()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
    }

    #[cfg(not(feature = "server"))]
    {
        let _ = project_path;
        Err(ServerFnError::new(
            "Git operations are only available on the server".to_string(),
        ))
    }
}

#[server]
pub async fn checkout_branch(project_path: String, branch: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
    tracing::info!("Checking out branch '{}'", branch);
    let git = GitOperations::new(project_path);
    git.checkout(&branch)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
    }

    #[cfg(not(feature = "server"))]
    {
        let _ = (project_path, branch);
        Err(ServerFnError::new(
            "Git operations are only available on the server".to_string(),
        ))
    }
}

#[server]
pub async fn merge_branches(project_path: String, source: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
    tracing::info!("Merging source branch '{}'", source);
    let git = GitOperations::new(project_path);
    git.merge(&source)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
    }

    #[cfg(not(feature = "server"))]
    {
        let _ = (project_path, source);
        Err(ServerFnError::new(
            "Git operations are only available on the server".to_string(),
        ))
    }
}

#[server]
pub async fn create_pull_request(
    project_path: String,
    branch: String,
    title: String,
    body: String,
) -> Result<String, ServerFnError> {
    #[cfg(feature = "server")]
    {
    tracing::info!("Creating PR for branch '{}'", branch);
    let git = GitOperations::new(project_path);
    git.create_pr(&branch, &title, &body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
    }

    #[cfg(not(feature = "server"))]
    {
        let _ = (project_path, branch, title, body);
        Err(ServerFnError::new(
            "Git operations are only available on the server".to_string(),
        ))
    }
}

#[server]
pub async fn push_branch(
    project_path: String,
    branch: Option<String>,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
    let git = GitOperations::new(project_path);
    git.push(branch.as_deref())
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
    }

    #[cfg(not(feature = "server"))]
    {
        let _ = (project_path, branch);
        Err(ServerFnError::new(
            "Git operations are only available on the server".to_string(),
        ))
    }
}

// PRD Management

#[server]
pub async fn set_prd(id: String, prd: Prd) -> Result<Session, ServerFnError> {
    SESSION_MANAGER
        .set_prd(&id, prd)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn convert_prd(id: String, markdown: String) -> Result<Prd, ServerFnError> {
    tracing::info!("Converting PRD markdown for session: {}", id);
    tracing::debug!("Markdown length: {} bytes", markdown.len());

    let session = SESSION_MANAGER.get_session(&id).await.map_err(|e| {
        tracing::error!("Failed to get session {} for PRD conversion: {}", id, e);
        ServerFnError::new(e.to_string())
    })?;

    // Parse markdown PRD
    let prd = parse_markdown_prd(
        &markdown,
        &session.project_path,
        session.config.branch_name.as_deref(),
    )
    .map_err(|e| {
        tracing::error!("Failed to parse PRD markdown: {}", e);
        ServerFnError::new(format!("Failed to parse PRD: {}", e))
    })?;

    tracing::info!(
        "Successfully converted PRD with {} stories",
        prd.stories.len()
    );
    Ok(prd)
}

#[cfg(feature = "server")]
fn parse_markdown_prd(
    markdown: &str,
    project_path: &str,
    branch_name: Option<&str>,
) -> Result<Prd, String> {
    use std::path::Path;

    let mut lines = markdown.lines().peekable();
    let mut project_name = Path::new(project_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Project")
        .to_string();
    let mut description = String::new();
    let mut stories = Vec::new();

    // Parse header and description
    while let Some(line) = lines.peek() {
        if line.starts_with("# ") {
            project_name = line.trim_start_matches("# ").trim().to_string();
            lines.next();
        } else if line.starts_with("## Problem Statement") || line.starts_with("## Description") {
            lines.next();
            // Read description until next section
            while let Some(desc_line) = lines.peek() {
                if desc_line.starts_with("##") {
                    break;
                }
                if !desc_line.is_empty() {
                    description.push_str(desc_line);
                    description.push(' ');
                }
                lines.next();
            }
            break;
        } else {
            lines.next();
        }
    }

    // Find User Stories section
    while let Some(line) = lines.next() {
        if line.starts_with("## User Stories") || line.starts_with("## Stories") {
            break;
        }
    }

    // Parse stories
    let mut current_story: Option<ralph::Story> = None;
    let mut in_acceptance = false;
    let mut acceptance_criteria = Vec::new();

    for line in lines {
        let trimmed = line.trim();

        if trimmed.starts_with("### ") && trimmed.contains(':') {
            // Save previous story
            if let Some(mut story) = current_story.take() {
                story.acceptance_criteria = acceptance_criteria.clone();
                stories.push(story);
                acceptance_criteria.clear();
            }

            // Parse new story: "### US-001: Story Title"
            let parts: Vec<&str> = trimmed.trim_start_matches("### ").splitn(2, ':').collect();
            if parts.len() == 2 {
                current_story = Some(ralph::Story {
                    id: parts[0].trim().to_string(),
                    title: parts[1].trim().to_string(),
                    description: String::new(),
                    acceptance_criteria: Vec::new(),
                    priority: stories.len() as u32 + 1,
                    passes: false,
                    notes: String::new(),
                });
                in_acceptance = false;
            }
        } else if trimmed.starts_with("**As a**")
            || trimmed.starts_with("**I want**")
            || trimmed.starts_with("**So that**")
        {
            // Parse user story description
            if let Some(ref mut story) = current_story {
                if !story.description.is_empty() {
                    story.description.push(' ');
                }
                story
                    .description
                    .push_str(trimmed.trim_start_matches("**").trim_end_matches("**"));
            }
        } else if trimmed.starts_with("**Acceptance Criteria:**") {
            in_acceptance = true;
        } else if trimmed.starts_with("**Priority:**") && current_story.is_some() {
            let priority_str = trimmed.trim_start_matches("**Priority:**").trim();
            if let Ok(priority) = priority_str.parse::<u32>() {
                current_story.as_mut().unwrap().priority = priority;
            }
            in_acceptance = false;
        } else if in_acceptance
            && (trimmed.starts_with("- [ ]")
                || trimmed.starts_with("- [x]")
                || trimmed.starts_with("-"))
        {
            let criterion = trimmed
                .trim_start_matches("- [ ]")
                .trim_start_matches("- [x]")
                .trim_start_matches("-")
                .trim()
                .to_string();
            if !criterion.is_empty() {
                acceptance_criteria.push(criterion);
            }
        } else if trimmed.starts_with("**") {
            // End of acceptance criteria
            in_acceptance = false;
        }
    }

    // Save last story
    if let Some(mut story) = current_story {
        story.acceptance_criteria = acceptance_criteria;
        stories.push(story);
    }

    if stories.is_empty() {
        return Err("No stories found in PRD markdown".to_string());
    }

    let branch = branch_name
        .map(|b| b.to_string())
        .unwrap_or_else(|| format!("ralph/{}", project_name.to_lowercase().replace(' ', "-")));

    Ok(Prd {
        project: project_name,
        branch_name: branch,
        description: description.trim().to_string(),
        stories,
    })
}

// PRD Conversation

#[server]
pub async fn start_prd_conversation(session_id: String) -> Result<PrdConversation, ServerFnError> {
    tracing::info!("Starting PRD conversation for session: {}", session_id);
    
    // Get session to retrieve model from config and root_path
    let session = SESSION_MANAGER
        .get_session(&session_id)
        .await
        .map_err(|e| {
            tracing::error!("Session not found for PRD conversation: {}", session_id);
            ServerFnError::new(e.to_string())
        })?;
    
    let model = session.config.prd_model.clone();
    let root_path = session.project_path.clone();
    tracing::debug!("Using PRD model '{}' from session config", model);
    tracing::debug!("Using root_path '{}' for PRD generation", root_path);
    
    CONVERSATION_MANAGER
        .start_conversation(session_id.clone(), model, root_path)
        .await
        .map_err(|e| {
            tracing::error!("Failed to start PRD conversation for {}: {}", session_id, e);
            ServerFnError::new(e.to_string())
        })
        .inspect(|conv| {
            tracing::info!(
                "PRD conversation started with {} messages",
                conv.messages.len()
            );
        })
}

#[server]
pub async fn send_prd_message(
    session_id: String,
    message: String,
) -> Result<PrdConversation, ServerFnError> {
    tracing::info!(
        "Sending message to PRD conversation for session: {}",
        session_id
    );
    tracing::debug!("Message length: {} chars", message.len());
    
    // Get session to retrieve model from config and root_path
    let session = SESSION_MANAGER
        .get_session(&session_id)
        .await
        .map_err(|e| {
            tracing::error!("Session not found for PRD message: {}", session_id);
            ServerFnError::new(e.to_string())
        })?;
    
    let model = session.config.prd_model.clone();
    let root_path = session.project_path.clone();
    tracing::debug!("Using PRD model '{}' from session config", model);
    tracing::debug!("Using root_path '{}' for PRD generation", root_path);
    
    CONVERSATION_MANAGER
        .send_message(&session_id, message, model, root_path)
        .await
        .map_err(|e| {
            tracing::error!("Failed to send message for {}: {}", session_id, e);
            ServerFnError::new(e.to_string())
        })
        .inspect(|conv| {
            tracing::info!(
                "Conversation updated, {} messages, PRD generated: {}",
                conv.messages.len(),
                conv.generated_prd.is_some()
            );
        })
}

#[server]
pub async fn get_prd_conversation(
    session_id: String,
) -> Result<Option<PrdConversation>, ServerFnError> {
    tracing::debug!("Getting PRD conversation for session: {}", session_id);
    Ok(CONVERSATION_MANAGER.get_conversation(&session_id).await)
}

// Guardrails

#[server]
pub async fn get_guardrails(id: String) -> Result<Vec<Guardrail>, ServerFnError> {
    let session = SESSION_MANAGER
        .get_session(&id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let manager = GuardrailManager::new(session.project_path);
    manager
        .load_guardrails()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn add_guardrail(id: String, guardrail: Guardrail) -> Result<(), ServerFnError> {
    tracing::info!("Adding guardrail '{}' for session: {}", guardrail.title, id);
    let session = SESSION_MANAGER.get_session(&id).await.map_err(|e| {
        tracing::error!("Failed to get session {} for adding guardrail: {}", id, e);
        ServerFnError::new(e.to_string())
    })?;

    let manager = GuardrailManager::new(session.project_path.clone());
    manager
        .add_guardrail(&guardrail)
        .await
        .map_err(|e| {
            tracing::error!(
                "Failed to add guardrail for {}: {}",
                session.project_path,
                e
            );
            ServerFnError::new(e.to_string())
        })
        .inspect(|_| {
            tracing::info!(
                "Guardrail '{}' added successfully for session {}",
                guardrail.title,
                id
            );
        })
}

// File System Browsing

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct DirectoryEntry {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub is_protected: bool,
    pub is_git_repository: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct DirectoryListing {
    pub current_path: String,
    pub entries: Vec<DirectoryEntry>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct ProjectPathValidation {
    pub path: String,
    pub exists: bool,
    pub is_directory: bool,
    pub is_git_repository: bool,
    pub message: Option<String>,
}

#[server]
pub async fn validate_project_path(path: String) -> Result<ProjectPathValidation, ServerFnError> {
    use std::path::Path;

    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Ok(ProjectPathValidation {
            path,
            exists: false,
            is_directory: false,
            is_git_repository: false,
            message: Some("Path is empty".to_string()),
        });
    }

    let target_path = Path::new(trimmed);
    let exists = target_path.exists();
    let is_directory = exists && target_path.is_dir();

    if !exists {
        return Ok(ProjectPathValidation {
            path: trimmed.to_string(),
            exists,
            is_directory,
            is_git_repository: false,
            message: Some(format!("Path does not exist: {}", target_path.display())),
        });
    }

    if !is_directory {
        return Ok(ProjectPathValidation {
            path: trimmed.to_string(),
            exists,
            is_directory,
            is_git_repository: false,
            message: Some(format!(
                "Path is not a directory: {}",
                target_path.display()
            )),
        });
    }

    // Confirm Git repo using a git command (more reliable than just checking `.git`).
    // NOTE: Non-zero exit from git simply means "not a repo"; it is not treated as a server error.
    let git_output = tokio::process::Command::new("git")
        .arg("-C")
        .arg(trimmed)
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .output()
        .await;

    let (is_git_repository, message) = match git_output {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let inside = stdout.trim().eq_ignore_ascii_case("true");
                if inside {
                    (true, None)
                } else {
                    (false, Some("Directory is not a git repository".to_string()))
                }
            } else {
                // git returns non-zero when not a repository; keep message user-friendly.
                (false, Some("Directory is not a git repository".to_string()))
            }
        }
        Err(e) => (
            false,
            Some(format!("Failed to run git for validation: {}", e)),
        ),
    };

    Ok(ProjectPathValidation {
        path: trimmed.to_string(),
        exists,
        is_directory,
        is_git_repository,
        message,
    })
}

#[server]
pub async fn list_directory(path: Option<String>) -> Result<DirectoryListing, ServerFnError> {
    use std::path::Path;

    let target_path = if let Some(p) = path {
        Path::new(&p).to_path_buf()
    } else {
        // Default to home directory
        dirs::home_dir()
            .ok_or_else(|| ServerFnError::new("Could not determine home directory".to_string()))?
    };

    // Validate that the path exists and is a directory
    if !target_path.exists() {
        return Err(ServerFnError::new(format!(
            "Path does not exist: {}",
            target_path.display()
        )));
    }

    if !target_path.is_dir() {
        return Err(ServerFnError::new(format!(
            "Path is not a directory: {}",
            target_path.display()
        )));
    }

    // Read directory entries
    let mut entries = Vec::new();

    match std::fs::read_dir(&target_path) {
        Ok(dir_entries) => {
            for entry_result in dir_entries {
                match entry_result {
                    Ok(entry) => {
                        let entry_path = entry.path();
                        let name = entry_path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("")
                            .to_string();

                        // Skip hidden files/directories (starting with .)
                        if name.starts_with('.') {
                            continue;
                        }

                        let is_directory = entry_path.is_dir();
                        let full_path = entry_path.to_string_lossy().to_string();

                        // Check if directory is protected (permission denied)
                        let is_protected = if is_directory {
                            // Try to read the directory to check permissions
                            std::fs::read_dir(&entry_path).is_err()
                        } else {
                            false
                        };

                        // Check if directory contains a Git repository (.git folder)
                        let is_git_repository = if is_directory && !is_protected {
                            let git_path = entry_path.join(".git");
                            git_path.exists() && git_path.is_dir()
                        } else {
                            false
                        };

                        entries.push(DirectoryEntry {
                            name,
                            path: full_path,
                            is_directory,
                            is_protected,
                            is_git_repository,
                        });
                    }
                    Err(e) => {
                        tracing::warn!("Error reading directory entry: {}", e);
                        // Continue with other entries
                    }
                }
            }
        }
        Err(e) => {
            // Check if this is a permission error
            let error_kind = e.kind();
            let error_msg = if error_kind == std::io::ErrorKind::PermissionDenied {
                format!(
                    "Permission denied: You don't have access to read this directory ({})",
                    target_path.display()
                )
            } else {
                format!("Error reading directory: {}", e)
            };
            return Err(ServerFnError::new(error_msg));
        }
    }

    // Sort entries: directories first, then files, both alphabetically
    entries.sort_by(|a, b| match (a.is_directory, b.is_directory) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.cmp(&b.name),
    });

    Ok(DirectoryListing {
        current_path: target_path.to_string_lossy().to_string(),
        entries,
    })
}

#[server]
pub async fn get_parent_directory(path: String) -> Result<Option<String>, ServerFnError> {
    use std::path::Path;

    let path_buf = Path::new(&path);

    if let Some(parent) = path_buf.parent() {
        Ok(Some(parent.to_string_lossy().to_string()))
    } else {
        Ok(None)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct CommonDirectory {
    pub name: String,
    pub path: String,
    pub icon: String,
}

#[server]
pub async fn get_common_directories() -> Result<Vec<CommonDirectory>, ServerFnError> {
    let mut directories = Vec::new();

    // Home directory
    if let Some(home) = dirs::home_dir() {
        let home_str = home.to_string_lossy().to_string();
        directories.push(CommonDirectory {
            name: "Home".to_string(),
            path: home_str.clone(),
            icon: "🏠".to_string(),
        });

        // Documents directory
        if let Some(documents) = dirs::document_dir() {
            directories.push(CommonDirectory {
                name: "Documents".to_string(),
                path: documents.to_string_lossy().to_string(),
                icon: "📄".to_string(),
            });
        }

        // Desktop directory
        if let Some(desktop) = dirs::desktop_dir() {
            directories.push(CommonDirectory {
                name: "Desktop".to_string(),
                path: desktop.to_string_lossy().to_string(),
                icon: "🖥️".to_string(),
            });
        }

        // Downloads directory
        if let Some(downloads) = dirs::download_dir() {
            directories.push(CommonDirectory {
                name: "Downloads".to_string(),
                path: downloads.to_string_lossy().to_string(),
                icon: "⬇️".to_string(),
            });
        }

        // Common project folders (check if they exist)
        let project_folders = vec![
            ("Projects", "projects"),
            ("Projects", "Projects"),
            ("Code", "code"),
            ("Code", "Code"),
            ("Workspace", "workspace"),
            ("Workspace", "Workspace"),
            ("Dev", "dev"),
            ("Dev", "Dev"),
        ];

        for (display_name, folder_name) in project_folders {
            let project_path = home.join(folder_name);
            if project_path.exists() && project_path.is_dir() {
                directories.push(CommonDirectory {
                    name: display_name.to_string(),
                    path: project_path.to_string_lossy().to_string(),
                    icon: "💻".to_string(),
                });
                break; // Only add one project folder to avoid duplicates
            }
        }
    }

    Ok(directories)
}

// Activity Streaming
// Note: SSE streaming will be implemented in the web package using use_resource
