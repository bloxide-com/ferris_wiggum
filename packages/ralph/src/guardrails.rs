use crate::types::{Guardrail, RalphError};
use std::path::Path;

pub struct GuardrailManager {
    project_path: String,
}

impl GuardrailManager {
    pub fn new(project_path: String) -> Self {
        Self { project_path }
    }

    pub async fn load_guardrails(&self) -> Result<Vec<Guardrail>, RalphError> {
        let guardrails_path = Path::new(&self.project_path).join(".ralph/guardrails.md");

        if !guardrails_path.exists() {
            return Ok(Vec::new());
        }

        let content = tokio::fs::read_to_string(&guardrails_path).await?;
        let guardrails = self.parse_guardrails(&content);

        Ok(guardrails)
    }

    pub async fn add_guardrail(&self, guardrail: &Guardrail) -> Result<(), RalphError> {
        let guardrails_path = Path::new(&self.project_path).join(".ralph/guardrails.md");

        let mut content = if guardrails_path.exists() {
            tokio::fs::read_to_string(&guardrails_path).await?
        } else {
            "# Ralph Guardrails (Signs)\n\n".to_string()
        };

        // Format guardrail as markdown
        content.push_str(&format!(
            "\n## Sign: {}\n\n- **Trigger**: {}\n- **Instruction**: {}\n- **Added after**: {}\n\n",
            guardrail.title, guardrail.trigger, guardrail.instruction, guardrail.added_after
        ));

        tokio::fs::write(&guardrails_path, content).await?;

        Ok(())
    }

    fn parse_guardrails(&self, content: &str) -> Vec<Guardrail> {
        let mut guardrails = Vec::new();
        let mut current_id = None;
        let mut current_title = None;
        let mut current_trigger = None;
        let mut current_instruction = None;
        let mut current_added_after = None;

        for line in content.lines() {
            if line.starts_with("## Sign:") {
                // Save previous guardrail if complete
                if let (
                    Some(id),
                    Some(title),
                    Some(trigger),
                    Some(instruction),
                    Some(added_after),
                ) = (
                    current_id.take(),
                    current_title.take(),
                    current_trigger.take(),
                    current_instruction.take(),
                    current_added_after.take(),
                ) {
                    guardrails.push(Guardrail {
                        id,
                        title,
                        trigger,
                        instruction,
                        added_after,
                    });
                }

                // Start new guardrail
                let title = line.trim_start_matches("## Sign:").trim().to_string();
                current_id = Some(uuid::Uuid::new_v4().to_string());
                current_title = Some(title);
            } else if line.starts_with("- **Trigger**:") {
                current_trigger =
                    Some(line.trim_start_matches("- **Trigger**:").trim().to_string());
            } else if line.starts_with("- **Instruction**:") {
                current_instruction = Some(
                    line.trim_start_matches("- **Instruction**:")
                        .trim()
                        .to_string(),
                );
            } else if line.starts_with("- **Added after**:") {
                current_added_after = Some(
                    line.trim_start_matches("- **Added after**:")
                        .trim()
                        .to_string(),
                );
            }
        }

        // Save last guardrail if complete
        if let (Some(id), Some(title), Some(trigger), Some(instruction), Some(added_after)) = (
            current_id,
            current_title,
            current_trigger,
            current_instruction,
            current_added_after,
        ) {
            guardrails.push(Guardrail {
                id,
                title,
                trigger,
                instruction,
                added_after,
            });
        }

        guardrails
    }

    pub async fn format_for_prompt(&self) -> Result<String, RalphError> {
        let guardrails = self.load_guardrails().await?;

        if guardrails.is_empty() {
            return Ok(String::new());
        }

        let mut output = String::from("# Guardrails (Signs to Follow)\n\n");
        output.push_str("The following guardrails were learned from previous iterations. Please follow them:\n\n");

        for guardrail in guardrails {
            output.push_str(&format!(
                "## {}\n- **When**: {}\n- **Do**: {}\n- **Context**: {}\n\n",
                guardrail.title, guardrail.trigger, guardrail.instruction, guardrail.added_after
            ));
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_guardrails() {
        let manager = GuardrailManager::new("/tmp/test".to_string());

        let content = r#"# Ralph Guardrails (Signs)

## Sign: Check imports before adding

- **Trigger**: Adding a new import statement
- **Instruction**: First verify import doesn't already exist
- **Added after**: Iteration 3 - duplicate import caused build failure

## Sign: Run tests before commit

- **Trigger**: About to commit code
- **Instruction**: Always run `npm test` first
- **Added after**: Iteration 5 - committed broken tests
"#;

        let guardrails = manager.parse_guardrails(content);
        assert_eq!(guardrails.len(), 2);
        assert_eq!(guardrails[0].title, "Check imports before adding");
        assert_eq!(guardrails[1].title, "Run tests before commit");
    }
}
