# PRD Generation Skill

Generate comprehensive Product Requirements Documents (PRDs) through a structured interview process.

## Process

1. **Understand the Feature**
   - Ask: "What problem does this feature solve?"
   - Ask: "Who are the users?"
   - Ask: "What's the expected outcome?"

2. **Gather Requirements**
   - Ask about core functionality
   - Ask about edge cases
   - Ask about constraints (performance, security, etc.)
   - Ask about dependencies on other features

3. **Define Success Criteria**
   - What does "done" look like?
   - What metrics matter?
   - How will we test this?

4. **Break Down User Stories**
   - Each story should be completable in one Ralph iteration
   - Order by dependency (schema → backend → UI)
   - Include clear acceptance criteria

## Output Format

```markdown
# [Feature Name]

## Problem Statement
[Brief description of the problem]

## Users
- [User type 1]: [What they need]
- [User type 2]: [What they need]

## Requirements

### Functional
1. [Requirement 1]
2. [Requirement 2]

### Non-Functional
- Performance: [requirement]
- Security: [requirement]
- Accessibility: [requirement]

## User Stories

### US-001: [Story Title]
**As a** [user type]
**I want** [feature]
**So that** [benefit]

**Acceptance Criteria:**
- [ ] [Criterion 1]
- [ ] [Criterion 2]
- [ ] Typecheck passes
- [ ] Tests pass (if applicable)
- [ ] Verify in browser (if UI change)

**Priority:** 1
**Dependencies:** None

[Continue for all stories...]

## Out of Scope
- [What we're NOT doing]

## Open Questions
- [Any unresolved questions]
```

## Tips for Good PRDs

- **Small stories** - Each story = one Ralph iteration
- **Clear criteria** - Must be verifiable, not vague
- **Order matters** - Earlier stories can't depend on later ones
- **Include testing** - Always add "Typecheck passes" as final criterion
- **Frontend verification** - UI stories need "Verify in browser" criterion
