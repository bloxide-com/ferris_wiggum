use dioxus::prelude::*;
use ralph::{Session, SessionConfig, Prd, Guardrail};

#[cfg(feature = "server")]
use ralph::{SessionManager, GuardrailManager};
#[cfg(feature = "server")]
use std::sync::Arc;

// Global session manager
#[cfg(feature = "server")]
lazy_static::lazy_static! {
    static ref SESSION_MANAGER: Arc<SessionManager> = Arc::new(SessionManager::new());
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

// PRD Management

#[server]
pub async fn set_prd(id: String, prd: Prd) -> Result<Session, ServerFnError> {
    SESSION_MANAGER
        .set_prd(&id, prd)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn convert_prd(
    id: String,
    markdown: String,
) -> Result<Prd, ServerFnError> {
    tracing::info!("Converting PRD markdown for session: {}", id);
    tracing::debug!("Markdown length: {} bytes", markdown.len());
    
    let session = SESSION_MANAGER
        .get_session(&id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get session {} for PRD conversion: {}", id, e);
            ServerFnError::new(e.to_string())
        })?;
    
    // Parse markdown PRD
    let prd = parse_markdown_prd(&markdown, &session.project_path, session.config.branch_name.as_deref())
        .map_err(|e| {
            tracing::error!("Failed to parse PRD markdown: {}", e);
            ServerFnError::new(format!("Failed to parse PRD: {}", e))
        })?;
    
    tracing::info!("Successfully converted PRD with {} stories", prd.stories.len());
    Ok(prd)
}

#[cfg(feature = "server")]
fn parse_markdown_prd(markdown: &str, project_path: &str, branch_name: Option<&str>) -> Result<Prd, String> {
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
        } else if trimmed.starts_with("**As a**") || trimmed.starts_with("**I want**") || trimmed.starts_with("**So that**") {
            // Parse user story description
            if let Some(ref mut story) = current_story {
                if !story.description.is_empty() {
                    story.description.push(' ');
                }
                story.description.push_str(trimmed.trim_start_matches("**").trim_end_matches("**"));
            }
        } else if trimmed.starts_with("**Acceptance Criteria:**") {
            in_acceptance = true;
        } else if trimmed.starts_with("**Priority:**") && current_story.is_some() {
            let priority_str = trimmed.trim_start_matches("**Priority:**").trim();
            if let Ok(priority) = priority_str.parse::<u32>() {
                current_story.as_mut().unwrap().priority = priority;
            }
            in_acceptance = false;
        } else if in_acceptance && (trimmed.starts_with("- [ ]") || trimmed.starts_with("- [x]") || trimmed.starts_with("-")) {
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
pub async fn add_guardrail(
    id: String,
    guardrail: Guardrail,
) -> Result<(), ServerFnError> {
    tracing::info!("Adding guardrail '{}' for session: {}", guardrail.title, id);
    let session = SESSION_MANAGER
        .get_session(&id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get session {} for adding guardrail: {}", id, e);
            ServerFnError::new(e.to_string())
        })?;
    
    let manager = GuardrailManager::new(session.project_path.clone());
    manager
        .add_guardrail(&guardrail)
        .await
        .map_err(|e| {
            tracing::error!("Failed to add guardrail for {}: {}", session.project_path, e);
            ServerFnError::new(e.to_string())
        })
        .inspect(|_| {
            tracing::info!("Guardrail '{}' added successfully for session {}", guardrail.title, id);
        })
}

// Activity Streaming
// Note: SSE streaming will be implemented in the web package using use_resource
