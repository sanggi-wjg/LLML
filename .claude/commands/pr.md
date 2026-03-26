# Create Pull Request

Create a feature branch, commit changes, push, and open a GitHub PR following the project's git workflow.

## Arguments
- $ARGUMENTS: Optional — branch name suffix or PR description hint (e.g., "add-repl" or "fix parser bug")

## Instructions

Follow the CLAUDE.md git workflow strictly. Run each step in sequence:

### 1. Pre-flight Checks
- Run `cargo fmt --all` — fix any formatting
- Run `cargo clippy --workspace -- -D warnings` — fix any warnings
- Run `cargo test --workspace` — all tests must pass
- If any step fails, fix the issue before proceeding. Do NOT skip.

### 2. Branch
- Check current branch. If already on `main`, create a new feature branch:
  - Infer prefix from changes: `feat/`, `fix/`, `docs/`, `refactor/`, `ci/`
  - Use $ARGUMENTS as suffix if provided, otherwise infer from changes
- If already on a feature branch, stay on it.

### 3. Commit
- Run `git status` and `git diff --stat` to understand all changes
- Group related changes into logical commits (prefer fewer, well-scoped commits over many tiny ones)
- Each commit message should be concise and explain **why**, not just what
- Always include the `Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>` trailer
- Never commit `.env`, credentials, or secrets

### 4. Push
- `git push -u origin <branch-name>`

### 5. Create PR
- Use `gh pr create` with:
  - Short title (under 70 characters)
  - Body with `## Summary` (bullet points) and `## Test plan` (checklist)
  - End body with `🤖 Generated with [Claude Code](https://claude.com/claude-code)`
- Report the PR URL when done

### 6. Summary
Report:
- Branch name
- Number of commits
- PR URL
- Test results (pass/fail counts)
