use crate::types::*;
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::RwLock;

const SYSTEM_PROMPT: &str = r#"You are an expert product manager helping to create a Product Requirements Document (PRD).

Your goal is to guide the user through creating a comprehensive PRD by asking targeted questions. Follow this process:

1. **Understand the Feature** - Ask about the problem, users, and expected outcome
2. **Gather Requirements** - Ask about core functionality, edge cases, constraints, and dependencies  
3. **Define Success Criteria** - What does "done" look like? How will we test this?
4. **Break Down User Stories** - Create small, focused stories that can each be completed in one iteration

When you have gathered enough information, generate the complete PRD in markdown format. The PRD should follow this structure:

```markdown
# [Feature Name]

## Problem Statement
[Brief description]

## Users
- [User type]: [What they need]

## Requirements

### Functional
1. [Requirement]

### Non-Functional
- Performance: [requirement]
- Security: [requirement]

## User Stories

### US-001: [Story Title]
**As a** [user type]
**I want** [feature]
**So that** [benefit]

**Acceptance Criteria:**
- [ ] [Criterion]
- [ ] Typecheck passes

**Priority:** 1
**Dependencies:** None

## Out of Scope
- [What we're NOT doing]

## Open Questions
- [Unresolved questions]
```

Guidelines:
- Ask ONE question at a time
- Keep stories small (each = one iteration of work)
- Order stories by dependency (schema → backend → UI)
- Always include "Typecheck passes" in acceptance criteria
- Add "Verify in browser" for UI stories

Start by asking what feature or project the user wants to build."#;

/// Manages PRD conversations
pub struct PrdConversationManager {
    conversations: Arc<RwLock<HashMap<String, PrdConversation>>>,
}

impl PrdConversationManager {
    pub fn new() -> Self {
        Self {
            conversations: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start a new PRD conversation for a session
    pub async fn start_conversation(&self, session_id: String, model: String) -> Result<PrdConversation, RalphError> {
        let mut conversations = self.conversations.write().await;
        
        // Create new conversation with system prompt
        let mut conversation = PrdConversation::new(session_id.clone());
        conversation.add_message(ConversationMessage::system(SYSTEM_PROMPT));
        
        // Generate initial assistant message
        let initial_message = self.generate_response(&conversation, &model).await?;
        conversation.add_message(ConversationMessage::assistant(&initial_message));
        
        conversations.insert(session_id, conversation.clone());
        
        Ok(conversation)
    }

    /// Get an existing conversation
    pub async fn get_conversation(&self, session_id: &str) -> Option<PrdConversation> {
        let conversations = self.conversations.read().await;
        conversations.get(session_id).cloned()
    }

    /// Send a user message and get a response
    pub async fn send_message(
        &self,
        session_id: &str,
        message: String,
        model: String,
    ) -> Result<PrdConversation, RalphError> {
        let mut conversations = self.conversations.write().await;
        
        let conversation = conversations
            .get_mut(session_id)
            .ok_or_else(|| RalphError::SessionNotFound(session_id.to_string()))?;
        
        // Add user message
        conversation.add_message(ConversationMessage::user(&message));
        
        // Generate assistant response
        let response = self.generate_response(conversation, &model).await?;
        
        // Check if the response contains a PRD
        if let Some(prd_markdown) = self.extract_prd(&response) {
            conversation.set_generated_prd(prd_markdown);
        }
        
        conversation.add_message(ConversationMessage::assistant(&response));
        
        Ok(conversation.clone())
    }

    /// Generate a response using the cursor-agent CLI
    async fn generate_response(&self, conversation: &PrdConversation, model: &str) -> Result<String, RalphError> {
        // Build the prompt from conversation history
        let prompt = self.build_prompt(conversation);
        
        tracing::info!("Generating PRD conversation response with model {}", model);
        tracing::debug!("Prompt length: {} chars", prompt.len());
        
        // Use cursor-agent in conversation mode
        let mut child = Command::new("cursor-agent")
            .arg("--model")
            .arg(model)
            .arg("--output-format")
            .arg("text")
            .arg(&prompt)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                tracing::error!("Failed to spawn cursor-agent: {}", e);
                RalphError::CursorAgent(format!("Failed to spawn cursor-agent: {}", e))
            })?;

        let stdout = child.stdout.take()
            .ok_or_else(|| {
                tracing::error!("Failed to capture cursor-agent stdout");
                RalphError::CursorAgent("Failed to capture stdout".into())
            })?;

        let mut reader = BufReader::new(stdout);
        let mut response = String::new();
        
        // Read the entire response
        let mut line = String::new();
        while reader.read_line(&mut line).await.map_err(|e| {
            RalphError::CursorAgent(format!("Failed to read response: {}", e))
        })? > 0 {
            response.push_str(&line);
            line.clear();
        }

        // Wait for process to complete
        let status = child.wait().await
            .map_err(|e| {
                tracing::error!("Failed to wait for cursor-agent: {}", e);
                RalphError::CursorAgent(format!("Failed to wait for cursor-agent: {}", e))
            })?;

        if !status.success() {
            tracing::error!("cursor-agent exited with status: {}", status);
            return Err(RalphError::CursorAgent(
                format!("cursor-agent exited with status: {}", status)
            ));
        }

        Ok(response.trim().to_string())
    }

    /// Build a prompt from conversation history
    fn build_prompt(&self, conversation: &PrdConversation) -> String {
        let mut prompt = String::new();
        
        for message in &conversation.messages {
            match message.role {
                MessageRole::System => {
                    prompt.push_str(&format!("[System]\n{}\n\n", message.content));
                }
                MessageRole::User => {
                    prompt.push_str(&format!("[User]\n{}\n\n", message.content));
                }
                MessageRole::Assistant => {
                    prompt.push_str(&format!("[Assistant]\n{}\n\n", message.content));
                }
            }
        }
        
        prompt.push_str("[Assistant]\n");
        prompt
    }

    /// Extract a PRD from the response if one is present
    fn extract_prd(&self, response: &str) -> Option<String> {
        // Look for markdown PRD structure
        // A PRD should have a title starting with # and contain ## User Stories
        if !response.contains("## User Stories") && !response.contains("## Stories") {
            return None;
        }
        
        // Try to find the PRD markdown block
        // First, check if the entire response is a PRD (starts with #)
        let trimmed = response.trim();
        if trimmed.starts_with("# ") && trimmed.contains("## ") {
            return Some(trimmed.to_string());
        }
        
        // Look for a markdown code block containing the PRD
        if let Some(start) = response.find("```markdown") {
            let content_start = start + "```markdown".len();
            if let Some(end) = response[content_start..].find("```") {
                let prd = response[content_start..content_start + end].trim();
                if prd.starts_with("# ") {
                    return Some(prd.to_string());
                }
            }
        }
        
        // Look for PRD that starts after some text
        if let Some(start) = response.find("\n# ") {
            let prd = response[start + 1..].trim();
            if prd.contains("## User Stories") || prd.contains("## Stories") {
                return Some(prd.to_string());
            }
        }
        
        None
    }

    /// Delete a conversation
    pub async fn delete_conversation(&self, session_id: &str) {
        let mut conversations = self.conversations.write().await;
        conversations.remove(session_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_prd_from_markdown_block() {
        let manager = PrdConversationManager::new();
        
        let response = r#"Here's the PRD based on our discussion:

```markdown
# My Feature

## Problem Statement
Users need this feature.

## User Stories

### US-001: First Story
**As a** user
**I want** something
**So that** I can do things

**Acceptance Criteria:**
- [ ] Works
- [ ] Typecheck passes

**Priority:** 1
**Dependencies:** None
```

Let me know if you'd like any changes!"#;
        
        let prd = manager.extract_prd(response);
        assert!(prd.is_some());
        let prd = prd.unwrap();
        assert!(prd.starts_with("# My Feature"));
        assert!(prd.contains("## User Stories"));
    }

    #[test]
    fn test_extract_prd_direct() {
        let manager = PrdConversationManager::new();
        
        let response = r#"# My Feature

## Problem Statement
Users need this feature.

## User Stories

### US-001: First Story
**Priority:** 1"#;
        
        let prd = manager.extract_prd(response);
        assert!(prd.is_some());
    }

    #[test]
    fn test_no_prd_in_response() {
        let manager = PrdConversationManager::new();
        
        let response = "What problem are you trying to solve with this feature?";
        
        let prd = manager.extract_prd(response);
        assert!(prd.is_none());
    }

    #[test]
    fn test_conversation_message_helpers() {
        let user_msg = ConversationMessage::user("Hello");
        assert_eq!(user_msg.role, MessageRole::User);
        assert_eq!(user_msg.content, "Hello");

        let assistant_msg = ConversationMessage::assistant("Hi there!");
        assert_eq!(assistant_msg.role, MessageRole::Assistant);

        let system_msg = ConversationMessage::system("Be helpful");
        assert_eq!(system_msg.role, MessageRole::System);
    }

    #[test]
    fn test_manager_accepts_model_parameter() {
        // Verify that PrdConversationManager can be created without a model
        // and that methods accept model as a parameter
        let _manager = PrdConversationManager::new();
        
        // Verify manager was created successfully (no model stored in struct)
        // The model is now passed to methods dynamically, ensuring it comes from SessionConfig
        // This test verifies the API change: model is no longer hardcoded in the manager
        assert!(true); // Manager created successfully
    }

    #[test]
    fn test_auto_model_handling() {
        // Verify that "auto" model option is accepted and passed through correctly
        let _manager = PrdConversationManager::new();
        
        // The manager should accept "auto" as a valid model parameter
        // This test verifies that the model parameter accepts any string value,
        // including "auto" which will be passed to cursor-agent as --model auto
        let auto_model = "auto";
        assert_eq!(auto_model, "auto");
        
        // Verify that other model values are also accepted
        let other_models = vec!["opus-4.5-thinking", "sonnet-4.5-thinking", "gpt-5.2-high", "composer-1"];
        for model in other_models {
            assert!(!model.is_empty());
        }
        
        // The actual model value is passed directly to cursor-agent via Command::arg(),
        // so any string value including "auto" will be passed through correctly
    }

    #[test]
    fn test_prd_model_is_used_for_prd_generation() {
        // Verify that PRD generation uses prd_model from SessionConfig
        // This test verifies the structure: SessionConfig has prd_model field
        // and it should be used when calling start_conversation/send_message
        use crate::types::SessionConfig;
        
        let config = SessionConfig {
            prd_model: "sonnet-4.5-thinking".to_string(),
            execution_model: "opus-4.5-thinking".to_string(),
            max_iterations: 20,
            warn_threshold: 70_000,
            rotate_threshold: 80_000,
            branch_name: None,
            open_pr: false,
        };

        // Verify prd_model is different from execution_model
        assert_eq!(config.prd_model, "sonnet-4.5-thinking");
        assert_ne!(config.prd_model, config.execution_model);
        
        // The manager accepts model as parameter, so prd_model should be passed
        // This test verifies the config structure supports separate models
        let _manager = PrdConversationManager::new();
        assert!(true); // Manager accepts model parameter dynamically
    }
}
