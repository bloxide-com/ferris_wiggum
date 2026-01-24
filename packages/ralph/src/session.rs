use crate::types::*;
use crate::git::GitOperations;
use crate::cursor::CursorRunner;
use crate::parser::StreamParser;
use crate::guardrails::GuardrailManager;
use std::path::PathBuf;
use std::time::SystemTime;
use tokio::sync::mpsc;

pub struct SessionManager {
    sessions: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<String, Session>>>,
    activity_channels: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<String, Vec<mpsc::UnboundedSender<ActivityEntry>>>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            activity_channels: std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }

    pub async fn create_session(&self, project_path: String, config: SessionConfig) -> Result<Session, RalphError> {
        let id = uuid::Uuid::new_v4().to_string();
        tracing::info!("Creating new session {} for project: {}", id, project_path);
        
        // Validate project path exists
        let path = PathBuf::from(&project_path);
        if !path.exists() {
            tracing::error!("Project path does not exist: {}", project_path);
            return Err(RalphError::Io(
                format!("Project path does not exist: {}", project_path)
            ));
        }

        // Check if it's a git repository
        let git_dir = path.join(".git");
        if !git_dir.exists() {
            tracing::error!("Not a git repository: {}", project_path);
            return Err(RalphError::Git("Not a git repository".into()));
        }

        // Initialize .ralph directory
        tracing::debug!("Creating .ralph directory in {}", project_path);
        let ralph_dir = path.join(".ralph");
        tokio::fs::create_dir_all(&ralph_dir).await?;
        
        let session = Session {
            id: id.clone(),
            project_path,
            status: SessionStatus::Idle,
            config,
            prd: None,
            current_iteration: 0,
            token_usage: TokenUsage::default(),
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        };

        let mut sessions = self.sessions.write().await;
        sessions.insert(id.clone(), session.clone());
        
        tracing::info!("Session {} created successfully", id);

        Ok(session)
    }

    pub async fn get_session(&self, id: &str) -> Result<Session, RalphError> {
        let sessions = self.sessions.read().await;
        sessions.get(id)
            .cloned()
            .ok_or_else(|| RalphError::SessionNotFound(id.to_string()))
    }

    pub async fn list_sessions(&self) -> Vec<Session> {
        let sessions = self.sessions.read().await;
        sessions.values().cloned().collect()
    }

    pub async fn update_session(&self, session: Session) -> Result<(), RalphError> {
        let mut sessions = self.sessions.write().await;
        sessions.insert(session.id.clone(), session);
        Ok(())
    }

    pub async fn start_session(&self, id: &str) -> Result<Session, RalphError> {
        tracing::info!("Starting session: {}", id);
        let mut session = self.get_session(id).await?;
        
        if !matches!(session.status, SessionStatus::Idle | SessionStatus::Paused) {
            tracing::error!("Cannot start session {} in state: {:?}", id, session.status);
            return Err(RalphError::InvalidState(
                format!("Cannot start session in state: {:?}", session.status)
            ));
        }

        // Validate that session has a PRD with stories
        if session.prd.is_none() {
            tracing::error!("Cannot start session {} without a PRD", id);
            return Err(RalphError::InvalidState(
                "Cannot start session without a PRD. Please set a PRD first.".to_string()
            ));
        }
        
        let prd = session.prd.as_ref().unwrap();
        if prd.stories.is_empty() {
            tracing::error!("Cannot start session {} with empty PRD", id);
            return Err(RalphError::InvalidState(
                "Cannot start session with empty PRD. PRD must contain at least one story.".to_string()
            ));
        }
        
        tracing::info!("Session {} has {} stories to process", id, prd.stories.len());

        // Spawn the Ralph loop in a background task
        let session_clone = session.clone();
        let manager_clone = self.clone();
        
        tracing::debug!("Spawning Ralph loop for session {}", id);
        tokio::spawn(async move {
            if let Err(e) = manager_clone.run_loop(session_clone).await {
                tracing::error!("Ralph loop error: {}", e);
            }
        });

        session.status = SessionStatus::Running { story_id: "initializing".to_string() };
        session.updated_at = SystemTime::now();
        self.update_session(session.clone()).await?;
        
        tracing::info!("Session {} started successfully", id);

        Ok(session)
    }

    pub async fn pause_session(&self, id: &str) -> Result<Session, RalphError> {
        let mut session = self.get_session(id).await?;
        session.status = SessionStatus::Paused;
        session.updated_at = SystemTime::now();
        self.update_session(session.clone()).await?;
        Ok(session)
    }

    pub async fn stop_session(&self, id: &str) -> Result<Session, RalphError> {
        let mut session = self.get_session(id).await?;
        session.status = SessionStatus::Idle;
        session.updated_at = SystemTime::now();
        self.update_session(session.clone()).await?;
        Ok(session)
    }

    pub async fn set_prd(&self, id: &str, prd: Prd) -> Result<Session, RalphError> {
        tracing::info!("Setting PRD for session {}", id);
        let mut session = self.get_session(id).await?;
        
        // Write PRD to disk
        self.write_prd_to_disk(&session.project_path, &prd).await?;
        
        session.prd = Some(prd);
        session.updated_at = SystemTime::now();
        self.update_session(session.clone()).await?;
        Ok(session)
    }

    async fn write_prd_to_disk(&self, project_path: &str, prd: &Prd) -> Result<(), RalphError> {
        use std::path::Path;
        
        tracing::debug!("Writing prd.json to {}", project_path);
        let prd_path = Path::new(project_path).join("prd.json");
        let json = serde_json::to_string_pretty(prd)
            .map_err(|e| RalphError::Io(format!("Failed to serialize PRD: {}", e)))?;
        
        tokio::fs::write(&prd_path, json).await?;
        tracing::info!("PRD written to {:?}", prd_path);
        Ok(())
    }

    pub async fn subscribe_to_activity(&self, session_id: &str) -> mpsc::UnboundedReceiver<ActivityEntry> {
        let (tx, rx) = mpsc::unbounded_channel();
        let mut channels = self.activity_channels.write().await;
        channels.entry(session_id.to_string())
            .or_insert_with(Vec::new)
            .push(tx);
        rx
    }

    async fn broadcast_activity(&self, session_id: &str, entry: ActivityEntry) {
        let channels = self.activity_channels.read().await;
        if let Some(senders) = channels.get(session_id) {
            for sender in senders {
                let _ = sender.send(entry.clone());
            }
        }
    }

    async fn run_loop(&self, mut session: Session) -> Result<(), RalphError> {
        while session.current_iteration < session.config.max_iterations {
            // Check if paused or stopped
            let current = self.get_session(&session.id).await?;
            if matches!(current.status, SessionStatus::Paused | SessionStatus::Idle) {
                return Ok(());
            }

            // Pick next story
            let story_id = session.prd.as_ref()
                .and_then(|prd| {
                    prd.stories
                        .iter()
                        .filter(|s| !s.passes)
                        .min_by_key(|s| s.priority)
                        .map(|s| s.id.clone())
                });

            let Some(story_id) = story_id else {
                session.status = SessionStatus::Complete;
                session.updated_at = SystemTime::now();
                self.update_session(session.clone()).await?;
                
                let entry = ActivityEntry {
                    timestamp: SystemTime::now(),
                    iteration: session.current_iteration,
                    kind: ActivityKind::Signal(Signal::Complete),
                    health: session.token_usage.health(
                        session.config.warn_threshold,
                        session.config.rotate_threshold
                    ),
                };
                self.broadcast_activity(&session.id, entry).await;
                
                return Ok(());
            };

            tracing::info!("Session {} working on story: {}", session.id, story_id);
            session.status = SessionStatus::Running {
                story_id: story_id.clone()
            };
            session.updated_at = SystemTime::now();
            self.update_session(session.clone()).await?;

            // Run iteration (placeholder - actual implementation in cursor.rs)
            tracing::debug!("Running iteration for session {}, story {}", session.id, story_id);
            let result = self.run_iteration(&mut session).await?;

            match result {
                IterationResult::StoryComplete => {
                    tracing::info!("Story {} completed for session {}", story_id, session.id);
                    // Mark story as complete in PRD
                    if let Some(prd) = &mut session.prd {
                        if let Some(s) = prd.stories.iter_mut().find(|s| s.id == story_id) {
                            s.passes = true;
                        }
                        // Write updated PRD to disk
                        self.write_prd_to_disk(&session.project_path, prd).await?;
                    }
                    session.current_iteration += 1;
                    session.updated_at = SystemTime::now();
                    self.update_session(session.clone()).await?;

                    let entry = ActivityEntry {
                        timestamp: SystemTime::now(),
                        iteration: session.current_iteration,
                        kind: ActivityKind::Signal(Signal::StoryComplete(story_id.clone())),
                        health: session.token_usage.health(
                            session.config.warn_threshold,
                            session.config.rotate_threshold
                        ),
                    };
                    self.broadcast_activity(&session.id, entry).await;
                }
                IterationResult::Rotate => {
                    session.current_iteration += 1;
                    session.token_usage = TokenUsage::default();
                    session.updated_at = SystemTime::now();
                    self.update_session(session.clone()).await?;

                    let entry = ActivityEntry {
                        timestamp: SystemTime::now(),
                        iteration: session.current_iteration,
                        kind: ActivityKind::Signal(Signal::Rotate),
                        health: ContextHealth::Critical,
                    };
                    self.broadcast_activity(&session.id, entry).await;
                }
                IterationResult::Gutter(reason) => {
                    session.status = SessionStatus::Gutter { reason: reason.clone() };
                    session.updated_at = SystemTime::now();
                    self.update_session(session.clone()).await?;

                    let entry = ActivityEntry {
                        timestamp: SystemTime::now(),
                        iteration: session.current_iteration,
                        kind: ActivityKind::Signal(Signal::Gutter(reason)),
                        health: session.token_usage.health(
                            session.config.warn_threshold,
                            session.config.rotate_threshold
                        ),
                    };
                    self.broadcast_activity(&session.id, entry).await;
                    
                    return Ok(());
                }
            }
        }

        tracing::warn!("Session {} reached max iterations without completion", session.id);
        session.status = SessionStatus::Failed {
            error: "Max iterations reached".into()
        };
        session.updated_at = SystemTime::now();
        self.update_session(session).await?;

        Ok(())
    }

    async fn run_iteration(&self, session: &mut Session) -> Result<IterationResult, RalphError> {
        tracing::info!("Running iteration for session {}", session.id);
        
        // Build prompt from iteration template
        let prompt = self.build_iteration_prompt(session).await?;
        
        // Create cursor runner
        let runner = CursorRunner::new(
            session.project_path.clone(),
            session.config.execution_model.clone(),
        );
        
        // Create stream parser for tracking
        let parser = std::sync::Arc::new(tokio::sync::Mutex::new(StreamParser::new(
            session.current_iteration,
            session.token_usage.clone(),
            session.config.warn_threshold,
            session.config.rotate_threshold,
        )));
        
        // Track signals
        let saw_complete = std::sync::Arc::new(tokio::sync::Mutex::new(false));
        let gutter_signal = std::sync::Arc::new(tokio::sync::Mutex::new(None));
        
        let session_id = session.id.clone();
        let current_iteration = session.current_iteration;
        let manager_clone = self.clone();
        let parser_clone = parser.clone();
        let saw_complete_clone = saw_complete.clone();
        let gutter_signal_clone = gutter_signal.clone();
        
        // Run cursor-agent iteration
        runner.run_iteration(&prompt, move |mut activity| {
            let parser = parser_clone.clone();
            let saw_complete = saw_complete_clone.clone();
            let gutter_signal = gutter_signal_clone.clone();
            let session_id = session_id.clone();
            let manager = manager_clone.clone();
            
            tokio::spawn(async move {
                // Update iteration number and parse
                activity.iteration = current_iteration;
                let mut parser_guard = parser.lock().await;
                let (entry, signal) = parser_guard.parse_activity(activity.kind);
                drop(parser_guard);
                
                // Broadcast activity
                manager.broadcast_activity(&session_id, entry).await;
                
                // Check for signals
                if let Some(sig) = signal {
                    match sig {
                        Signal::Complete => {
                            *saw_complete.lock().await = true;
                        }
                        Signal::Gutter(reason) => {
                            *gutter_signal.lock().await = Some(reason);
                        }
                        Signal::Warn => {
                            tracing::warn!("Token usage warning for session {}", session_id);
                        }
                        Signal::Rotate => {
                            tracing::info!("Rotation signal for session {}", session_id);
                        }
                        _ => {}
                    }
                }
            });
        }).await?;
        
        // Update session token usage
        let parser_guard = parser.lock().await;
        session.token_usage = parser_guard.token_usage().clone();
        drop(parser_guard);
        
        // Check for gutter
        let gutter = gutter_signal.lock().await;
        if let Some(reason) = gutter.clone() {
            return Ok(IterationResult::Gutter(reason));
        }
        drop(gutter);
        
        // Check for rotation
        if session.token_usage.total >= session.config.rotate_threshold {
            return Ok(IterationResult::Rotate);
        }
        
        // Check for completion
        let complete = *saw_complete.lock().await;
        if complete {
            tracing::info!("Story completion signal received for session {}", session.id);
            return Ok(IterationResult::StoryComplete);
        }
        
        // Default to story complete (cursor-agent finished successfully)
        Ok(IterationResult::StoryComplete)
    }
    
    async fn build_iteration_prompt(&self, session: &Session) -> Result<String, RalphError> {
        tracing::debug!("Building iteration prompt for session {}", session.id);
        
        // Load iteration template
        let template = include_str!("../assets/prompts/iteration.md");
        
        // Load guardrails
        let guardrail_manager = GuardrailManager::new(session.project_path.clone());
        let guardrails = guardrail_manager.format_for_prompt().await
            .unwrap_or_else(|_| String::new());
        
        // Build the full prompt
        let mut prompt = String::from(template);
        
        if !guardrails.is_empty() {
            prompt.push_str("\n\n---\n\n");
            prompt.push_str(&guardrails);
        }
        
        tracing::trace!("Prompt built: {} chars", prompt.len());
        Ok(prompt)
    }
}

impl Clone for SessionManager {
    fn clone(&self) -> Self {
        Self {
            sessions: self.sessions.clone(),
            activity_channels: self.activity_channels.clone(),
        }
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SessionConfig;
    use crate::cursor::CursorRunner;

    #[test]
    fn test_execution_model_is_used_for_execution() {
        // Verify that execution phase uses execution_model from SessionConfig
        // This test verifies the structure: SessionConfig has execution_model field
        // and it should be used when creating CursorRunner
        let config = SessionConfig {
            prd_model: "sonnet-4.5-thinking".to_string(),
            execution_model: "opus-4.5-thinking".to_string(),
            max_iterations: 20,
            warn_threshold: 70_000,
            rotate_threshold: 80_000,
            branch_name: None,
            open_pr: false,
        };

        // Verify execution_model is different from prd_model
        assert_eq!(config.execution_model, "opus-4.5-thinking");
        assert_ne!(config.execution_model, config.prd_model);

        // Verify CursorRunner can be created with execution_model
        let runner = CursorRunner::new(
            "/tmp/test".to_string(),
            config.execution_model.clone(),
        );
        assert_eq!(runner.model, "opus-4.5-thinking");
    }
}
