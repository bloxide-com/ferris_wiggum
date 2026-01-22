# Build Feature Skill

Autonomous story execution for Ralph iterations.

## Workflow

1. **Read Context**
   - Read `prd.json` to see all stories
   - Read `.ralph/progress.md` for past learnings
   - Read `.ralph/guardrails.md` for important rules
   - Check current git branch matches PRD

2. **Pick Next Story**
   - Filter to stories where `passes: false`
   - Sort by `priority` (ascending)
   - Take the first one

3. **Implement Story**
   - Follow acceptance criteria exactly
   - Use existing patterns from the codebase
   - Make minimal, focused changes
   - Follow guardrails

4. **Quality Checks**
   - Run dx build (always)
   - Run cargo clippy (if configured)
   - Run cargo test (if applicable)
   - Verify in browser (if UI change)

5. **Commit**
   - Format: `feat: [Story ID] - [Story Title]`
   - Include all related changes
   - Only commit if checks pass

6. **Update State**
   - Set `passes: true` in `prd.json` for completed story
   - Append to `.ralph/progress.md` with learnings
   - Update `AGENTS.md` if reusable patterns discovered

7. **Check Completion**
   - If all stories have `passes: true`, output `<ralph>COMPLETE</ralph>`
   - Otherwise, end normally (next iteration continues)

## Example Progress Entry

```markdown
## 2026-01-21 14:30 - US-003

Implemented status toggle dropdown on task list.

**Files Changed:**
- `app/components/TaskRow.tsx` - Added StatusDropdown component
- `app/actions/tasks.ts` - Added updateTaskStatus server action
- `app/types.ts` - Updated Task type with status field

**Learnings for future iterations:**
- Server actions in this project return { success, data, error } format
- All mutations use optimistic updates via useOptimistic hook
- Status changes trigger toast notification via useToast context
---
```

## Patterns to Follow

- **One story per iteration** - Don't start US-004 if working on US-003
- **Commit frequently** - Don't accumulate 10+ file changes
- **Test before commit** - Broken code compounds across iterations
- **Document learnings** - Future iterations benefit from your experience
- **Update AGENTS.md** - Help future developers understand the patterns
