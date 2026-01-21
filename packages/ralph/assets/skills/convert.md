# PRD to JSON Conversion Skill

Convert markdown PRDs to `prd.json` format for Ralph's autonomous execution.

## Input

A markdown PRD file with user stories and acceptance criteria.

## Output Format

```json
{
  "project": "[Project Name]",
  "branchName": "ralph/[feature-name-kebab-case]",
  "description": "[Feature description]",
  "stories": [
    {
      "id": "US-001",
      "title": "[Story title]",
      "description": "As a [user], I want [feature] so that [benefit]",
      "acceptanceCriteria": [
        "Criterion 1",
        "Criterion 2",
        "Typecheck passes"
      ],
      "priority": 1,
      "passes": false,
      "notes": ""
    }
  ]
}
```

## Conversion Rules

1. **Story Size** - Each story must be completable in ONE iteration
   - Right size: Add a database column, create a UI component, add an endpoint
   - Too big: Build entire dashboard, implement authentication system

2. **Story Ordering** - Dependencies first
   - Correct: Schema → Backend → UI → Dashboard
   - Wrong: UI before schema exists

3. **Acceptance Criteria** - Must be verifiable
   - Good: "Filter dropdown has options: All, Active, Completed"
   - Bad: "Works correctly"

4. **Always Include**
   - "Typecheck passes" for every story
   - "Tests pass" for stories with logic
   - "Verify in browser using dev-browser skill" for UI stories

5. **Initial State** - All stories start with:
   - `passes: false`
   - `notes: ""`

## Branch Naming

Derive from feature name:
- Feature: "Add Task Status"
- Branch: `ralph/task-status`

## Example Conversion

**Input (Markdown):**
```markdown
# Task Status Feature

Add ability to mark tasks with status.

## Requirements
- Toggle between pending/in-progress/done
- Filter list by status
- Show status badge on each task
```

**Output (JSON):**
```json
{
  "project": "TaskApp",
  "branchName": "ralph/task-status",
  "description": "Task Status Feature - Track task progress",
  "stories": [
    {
      "id": "US-001",
      "title": "Add status field to tasks table",
      "description": "As a developer, I need to store task status.",
      "acceptanceCriteria": [
        "Add status column: 'pending' | 'in_progress' | 'done'",
        "Default value is 'pending'",
        "Generate and run migration",
        "Typecheck passes"
      ],
      "priority": 1,
      "passes": false,
      "notes": ""
    },
    {
      "id": "US-002",
      "title": "Display status badge on task cards",
      "description": "As a user, I want to see task status at a glance.",
      "acceptanceCriteria": [
        "Each task card shows colored badge",
        "Colors: gray=pending, blue=in_progress, green=done",
        "Typecheck passes",
        "Verify in browser using dev-browser skill"
      ],
      "priority": 2,
      "passes": false,
      "notes": ""
    }
  ]
}
```
