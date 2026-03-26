# PR Merge

Verify GitHub Actions CI, fix failures if any, and merge the PR.

## Arguments
- $ARGUMENTS: Optional — PR number (e.g., "3") or branch name. If omitted, uses the current branch's PR.

## Instructions

### 1. Identify the PR
- If $ARGUMENTS is a number, use it as the PR number
- If $ARGUMENTS is a branch name, find the PR for that branch
- If $ARGUMENTS is empty, find the PR for the current git branch
- Run: `gh pr view <PR> --json number,title,state,headRefName,statusCheckRollup,mergeable,reviewDecision`
- If no PR found, report and stop

### 2. Check CI Status
- Run: `gh pr checks <PR>`
- Report each job's status:
  - ✓ Check
  - ✓ Format
  - ✓ Clippy
  - ✓ Test
  - ✓ Conformance Tests
- If all checks pass, go to Step 4
- If any check is **pending/in_progress**, wait and re-check (up to 2 retries with 30s interval)
- If any check **failed**, go to Step 3

### 3. Fix Failures
For each failed check, diagnose and fix:

- **Check failed**: Run `cargo check --workspace`, fix compile errors
- **Format failed**: Run `cargo fmt --all`
- **Clippy failed**: Run `cargo clippy --workspace -- -D warnings`, fix warnings
- **Test failed**: Run `cargo test --workspace`, examine failures, fix code
- **Conformance failed**: Run `cargo test --test conformance`, compare outputs, fix discrepancies

After fixing:
- Commit the fix: `git commit -m "fix: address CI failure — <description>"`
- Push: `git push`
- Re-check CI status (go back to Step 2)

### 4. Merge Decision
When all checks pass:
- Show PR summary: title, branch, number of commits, check results
- Ask the user whether to merge
- If user approves, run: `gh pr merge <PR> --squash --delete-branch`
- Report the merge result

### 5. Post-merge
- Switch back to main: `git checkout main && git pull`
- Confirm clean state: `git status`
