# Context Rotation - Commit and Exit

🔄 **ROTATE THRESHOLD REACHED**

You have reached 80,000 tokens (rotation threshold). You must commit and exit NOW:

1. **Commit immediately** - Even if work is incomplete:
   - If story complete: `feat: [Story ID] - [Story Title]`
   - If work in progress: `wip: [Story ID] - partial implementation`

2. **Update `.ralph/progress.md`** - Document what was done AND what remains

3. **DO NOT start new work** - The next iteration will continue from git

## Why Rotation Matters

At 80k+ tokens, the LLM's context is full and performance degrades. Starting fresh is more effective than continuing.

The next iteration will:
- Have 0 tokens (fresh context)
- Read your commit(s) from git
- Read your progress notes
- Continue where you left off

Better to commit partial work than to produce degraded code.
