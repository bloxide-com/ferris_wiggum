use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Session {
    pub id: String,
    pub project_path: String,
    pub status: SessionStatus,
    pub config: SessionConfig,
    pub prd: Option<Prd>,
    pub current_iteration: u32,
    pub token_usage: TokenUsage,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionStatus {
    Idle,
    Running { story_id: String },
    Paused,
    WaitingForRotation,
    Gutter { reason: String },
    Complete,
    Failed { error: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionConfig {
    pub model: String,
    pub max_iterations: u32,
    pub warn_threshold: u32,
    pub rotate_threshold: u32,
    pub branch_name: Option<String>,
    pub open_pr: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            model: "opus-4.5-thinking".into(),
            max_iterations: 20,
            warn_threshold: 70_000,
            rotate_threshold: 80_000,
            branch_name: None,
            open_pr: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Prd {
    pub project: String,
    pub branch_name: String,
    pub description: String,
    pub stories: Vec<Story>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Story {
    pub id: String,
    pub title: String,
    pub description: String,
    pub acceptance_criteria: Vec<String>,
    pub priority: u32,
    pub passes: bool,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct TokenUsage {
    pub total: u32,
    pub read: u32,
    pub write: u32,
    pub assistant: u32,
    pub shell: u32,
}

impl TokenUsage {
    pub fn percentage(&self, threshold: u32) -> f32 {
        (self.total as f32 / threshold as f32 * 100.0).min(100.0)
    }

    pub fn health(&self, warn_threshold: u32, rotate_threshold: u32) -> ContextHealth {
        let percent = self.percentage(rotate_threshold);
        let warn_percent = warn_threshold as f32 / rotate_threshold as f32 * 100.0;
        
        match percent as u32 {
            p if p < warn_percent as u32 => ContextHealth::Healthy,
            p if p < 100 => ContextHealth::Warning,
            _ => ContextHealth::Critical,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Signal {
    Warn,
    Rotate,
    Gutter(String),
    Complete,
    StoryComplete(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActivityEntry {
    pub timestamp: SystemTime,
    pub iteration: u32,
    pub kind: ActivityKind,
    pub health: ContextHealth,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActivityKind {
    Read { path: String, lines: u32, bytes: u32 },
    Write { path: String, lines: u32, bytes: u32 },
    Shell { command: String, exit_code: i32 },
    TokenUpdate(TokenUsage),
    Signal(Signal),
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContextHealth {
    Healthy,
    Warning,
    Critical,
}

impl ContextHealth {
    pub fn as_str(&self) -> &str {
        match self {
            ContextHealth::Healthy => "healthy",
            ContextHealth::Warning => "warning",
            ContextHealth::Critical => "critical",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            ContextHealth::Healthy => "🟢",
            ContextHealth::Warning => "🟡",
            ContextHealth::Critical => "🔴",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Guardrail {
    pub id: String,
    pub title: String,
    pub trigger: String,
    pub instruction: String,
    pub added_after: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IterationResult {
    StoryComplete,
    Rotate,
    Gutter(String),
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum RalphError {
    #[error("IO error: {0}")]
    Io(String),
    
    #[error("Git error: {0}")]
    Git(String),
    
    #[error("Cursor agent error: {0}")]
    CursorAgent(String),
    
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("Session not found: {0}")]
    SessionNotFound(String),
    
    #[error("Invalid session state: {0}")]
    InvalidState(String),
}

impl From<std::io::Error> for RalphError {
    fn from(e: std::io::Error) -> Self {
        RalphError::Io(e.to_string())
    }
}
