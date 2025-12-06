# Project Bootstrap Workflow

This guide covers how to bootstrap a new project using Murmuration's plan-based workflow. You'll learn how to create a PLAN.md, import it into GitHub issues, and work through the phases systematically.

## What is a Bootstrap Plan?

A bootstrap plan is a structured roadmap for building a software project:

- Organized into **Phases** (high-level milestones)
- Each phase contains **PRs** (individual tasks or issues)
- PRs can have **sub-PRs** (smaller subtasks)
- Dependencies are inferred from ordering and can be explicit

The plan serves as:
- A project roadmap
- Source of truth for work to be done
- Input for GitHub issue creation
- Documentation of the project structure

## Creating a PLAN.md

### Basic Structure

```markdown
# Project Name: Description

Brief overview of the project goals.

---

## Phase 1: Phase Name
*Goal: What this phase achieves*

| PR | Description | Files |
|----|-------------|-------|
| PR-001 | Task description | `file1.rs`, `file2.rs` |
| PR-002 | Another task | `file3.rs` |

**Checkpoint:** Describe what should be working after this phase.

---

## Phase 2: Another Phase
*Goal: Next milestone*

| PR | Description | Files |
|----|-------------|-------|
| PR-003 | Task description | `file4.rs` |
| PR-003a | Sub-task (indented) | `file5.rs` |
| PR-003b | Another sub-task | `file6.rs` |
| PR-004 | Independent task | `file7.rs` |

**Checkpoint:** Another milestone description.

---
```

### Example: Authentication System

```markdown
# Authentication System

A secure user authentication system with JWT tokens.

---

## Phase 1: Core Infrastructure
*Goal: Basic user model and database setup*

| PR | Description | Files |
|----|-------------|-------|
| PR-001 | Database schema and migrations | `migrations/001_users.sql` |
| PR-002 | User model and repository | `src/models/user.rs` |
| PR-003 | Password hashing utilities | `src/crypto/hash.rs` |

**Checkpoint:** Can create users and store them in database with hashed passwords.

---

## Phase 2: Authentication Endpoints
*Goal: Login and signup functionality*

| PR | Description | Files |
|----|-------------|-------|
| PR-004 | Signup endpoint | `src/routes/auth.rs` |
| PR-004a | Input validation | `src/validators/auth.rs` |
| PR-004b | Email verification | `src/services/email.rs` |
| PR-005 | Login endpoint | `src/routes/auth.rs` |
| PR-006 | JWT token generation | `src/crypto/jwt.rs` |

**Checkpoint:** Users can sign up and log in, receiving JWT tokens.

---

## Phase 3: Authorization
*Goal: Protect routes with authentication*

| PR | Description | Files |
|----|-------------|-------|
| PR-007 | Auth middleware | `src/middleware/auth.rs` |
| PR-008 | Protected route examples | `src/routes/protected.rs` |
| PR-009 | Token refresh endpoint | `src/routes/auth.rs` |

**Checkpoint:** Can protect routes and refresh tokens.

---
```

### PR Naming Conventions

- **Sequential:** PR-001, PR-002, PR-003
- **Sub-PRs:** PR-003a, PR-003b, PR-003c
- **Indented in table:** Sub-PRs are visually nested

### File References

List files that will be created or modified:
- Helps agents understand scope
- Provides context for implementation
- Useful for code review

## PLAN.md Best Practices

### 1. Start with High-Level Phases

```markdown
## Phase 1: Core Infrastructure
## Phase 2: Basic Features
## Phase 3: Advanced Features
## Phase 4: Polish & Production
```

### 2. Each Phase Should Have a Clear Goal

```markdown
## Phase 2: Authentication
*Goal: Users can sign up, log in, and access protected routes*
```

### 3. Include Checkpoints

After each phase, describe what should be working:

```markdown
**Checkpoint:** Can run `murmur run "task"` and see Claude work.
```

This helps verify progress.

### 4. Break Down Complex Tasks

Use sub-PRs for complex tasks:

```markdown
| PR-015 | Dependency parsing | `src/deps.rs` |
| PR-015a | "Depends on #X" parsing | |
| PR-015b | "Blocked by #X" parsing | |
| PR-015c | Build dependency graph | |
```

### 5. Infer Dependencies from Order

PRs are assumed to depend on previous PRs in the same phase:

```markdown
# This ordering means:
# PR-002 depends on PR-001
# PR-003 depends on PR-002

| PR-001 | First task |
| PR-002 | Second task (depends on PR-001) |
| PR-003 | Third task (depends on PR-002) |
```

### 6. Mark Independent Tasks

If PRs are independent, use comments:

```markdown
| PR-004 | Feature A (independent) |
| PR-005 | Feature B (independent) |
| PR-006 | Integration (depends on PR-004, PR-005) |
```

## Importing Plan to GitHub

### Dry Run

Always start with a dry run to preview what will be created:

```bash
murmur plan import
```

Output:
```
Parsed PLAN.md: 3 phases, 9 PRs

Would create:

  📁 Epic: Phase 1: Core Infrastructure
     Goal: Basic user model and database setup

  📝 PR-001: Database schema and migrations
  📝 PR-002: User model and repository
  📝 PR-003: Password hashing utilities

  📁 Epic: Phase 2: Authentication Endpoints
     Goal: Login and signup functionality

  📝 PR-004: Signup endpoint
      📝 PR-004a: Input validation
      📝 PR-004b: Email verification
  📝 PR-005: Login endpoint
  📝 PR-006: JWT token generation

Run with --execute to create issues.
```

### Execute Import

Create the issues:

```bash
murmur plan import --execute --repo owner/repo
```

Output:
```
Creating issues in owner/repo...

Created 12 issue(s):
  ✅ #10 Phase 1: Core Infrastructure
  ✅ #11 PR-001: Database schema and migrations
  ✅ #12 PR-002: User model and repository
  ✅ #13 PR-003: Password hashing utilities
  ✅ #14 Phase 2: Authentication Endpoints
  ✅ #15 PR-004: Signup endpoint
  ✅ #16 PR-004a: Input validation
  ✅ #17 PR-004b: Email verification
  ✅ #18 PR-005: Login endpoint
  ✅ #19 PR-006: JWT token generation
  ✅ #20 Phase 3: Authorization
  ✅ #21 PR-007: Auth middleware

Summary: 12 created, 0 skipped, 0 errors
```

### Custom Labels

Add labels during import:

```bash
murmur plan import --execute --label enhancement --label bootstrapping
```

### Skip Existing Issues

Re-importing won't duplicate issues:

```bash
murmur plan import --execute
```

Output:
```
Skipped 5 existing issue(s).
Created 3 new issue(s).
```

## Viewing Plan Status

Check progress on the plan:

```bash
murmur plan status
```

Output:
```
Plan status for owner/repo:

✅ Phase 1: Core Infrastructure
  ✅ PR-001 #11 Database schema and migrations
  ✅ PR-002 #12 User model and repository
  ✅ PR-003 #13 Password hashing utilities

🔄 Phase 2: Authentication Endpoints
  🔄 PR-004 #15 Signup endpoint
    ✅ PR-004a #16 Input validation
    🔄 PR-004b #17 Email verification
  ❌ PR-005 #18 Login endpoint
  ❌ PR-006 #19 JWT token generation

❌ Phase 3: Authorization
  ❌ PR-007 #21 Auth middleware

Progress: 5/10 PRs completed (50%)
```

Legend:
- ✅ Completed (issue closed)
- 🔄 In progress (issue open, has PR)
- ❌ Not started (issue open, no PR)

## Working Through the Plan

### 1. Find Ready Issues

```bash
murmur issue list --ready
```

Output:
```
Ready to work (no unmet dependencies):

#11: PR-001: Database schema and migrations
#16: PR-004a: Input validation (parent: #15)
```

### 2. Start Working on an Issue

```bash
murmur work 11
```

This:
1. Checks dependencies (should be satisfied for ready issues)
2. Creates a worktree
3. Spawns an agent
4. Agent completes the work
5. Creates a PR

### 3. Review and Merge

After the agent creates a PR:
1. Review the PR manually or with review agent
2. Merge if approved
3. Issue is automatically closed

### 4. Next Issue

When an issue is closed, dependent issues become unblocked:

```bash
# After merging PR from issue #11
murmur work 12  # Now ready (depended on #11)
```

### 5. Parallel Work

Work on multiple independent issues:

```bash
# Terminal 1
murmur work 11

# Terminal 2
murmur work 16  # Independent sub-task
```

## Issue Structure from Plan Import

### Epic Issues

Each phase becomes an epic issue:

```markdown
# Phase 1: Core Infrastructure

Goal: Basic user model and database setup

## Child Issues

- [ ] #11 PR-001: Database schema and migrations
- [ ] #12 PR-002: User model and repository
- [ ] #13 PR-003: Password hashing utilities

## Checkpoint

Can create users and store them in database with hashed passwords.

<!-- murmur:metadata
{
  "type": "epic",
  "phase": 1
}
-->
```

### Task Issues

Each PR becomes a task issue:

```markdown
# PR-001: Database schema and migrations

Files: `migrations/001_users.sql`

Parent: #10

Depends on: (previous PRs if any)

<!-- murmur:metadata
{
  "phase": 1,
  "pr": "001",
  "parent": 10,
  "status": "ready"
}
-->
```

### Sub-task Issues

Sub-PRs become child issues:

```markdown
# PR-004a: Input validation

Files: `src/validators/auth.rs`

Parent: #15

<!-- murmur:metadata
{
  "phase": 2,
  "pr": "004a",
  "parent": 15,
  "status": "ready"
}
-->
```

## Bootstrap Workflow Pattern

### Phase-by-Phase Approach

Work through phases sequentially:

```bash
# Complete Phase 1
murmur work 11  # PR-001
murmur work 12  # PR-002
murmur work 13  # PR-003

# Check phase 1 completion
murmur plan status | grep "Phase 1"

# Start Phase 2
murmur work 15  # PR-004
```

### Parallel Sub-tasks

Work on sub-PRs in parallel:

```bash
# Main task
murmur work 15  # PR-004: Signup endpoint

# Simultaneously work on sub-tasks
murmur work 16  # PR-004a: Input validation
murmur work 17  # PR-004b: Email verification
```

### Checkpoint Verification

After each phase, verify the checkpoint:

```bash
# Phase 1 checkpoint: "Can create users and store them in database"
cargo test user::test_create_user

# Phase 2 checkpoint: "Users can sign up and log in"
curl -X POST http://localhost:8000/auth/signup
curl -X POST http://localhost:8000/auth/login
```

## Updating the Plan

### Adding New PRs

1. Edit PLAN.md to add new tasks
2. Re-import (skips existing issues):

```bash
murmur plan import --execute
```

### Revising Phase Structure

If you need to restructure:
1. Close existing issues
2. Update PLAN.md
3. Re-import with new structure

### Tracking Deviations

Document deviations in issue comments:

```markdown
# Original plan: PR-015
# Actual work: Split into PR-015 and PR-015-extra

See #42 for additional work not in original plan.
```

## Complete Bootstrap Example

### 1. Create PLAN.md

```bash
# Create initial plan
vim PLAN.md
```

### 2. Dry Run Import

```bash
murmur plan import
# Review output, ensure structure looks correct
```

### 3. Execute Import

```bash
murmur plan import --execute --repo owner/repo --label bootstrap
```

### 4. Check Status

```bash
murmur plan status
```

Output:
```
Progress: 0/15 PRs completed (0%)
```

### 5. Work Through Phase 1

```bash
# Find ready issues
murmur issue list --ready

# Work on first issue
murmur work 11

# After PR merged
murmur work 12
murmur work 13
```

### 6. Verify Phase 1 Checkpoint

```bash
# Run checkpoint validation
cargo test
cargo build

# Check phase 1 status
murmur plan status | grep "Phase 1"
```

Output:
```
✅ Phase 1: Core Infrastructure
```

### 7. Continue with Phase 2

```bash
murmur issue list --ready
murmur work 15
# ... continue ...
```

### 8. Monitor Overall Progress

```bash
murmur plan status
```

Output after several phases:
```
Progress: 12/15 PRs completed (80%)
```

## Self-Hosting Milestone

For projects like Murmuration itself, the goal is to reach "self-hosting" where the system can build itself:

```markdown
## 🎯 BOOTSTRAP MILESTONE
*Murmuration can now build itself!*

At this point:
- `murmur orchestrate 42` runs full TDD workflow on issue #42
- Coordinator spawns coder, test, reviewer agents as needed
- Worktrees created from fresh main
- Red-green validation enforced
- Reviews gate each phase

**Start using Murmuration to build remaining features.**
```

After bootstrap:
- All features listed after this point are built using Murmuration
- The system dogfoods itself
- Development velocity increases

## Best Practices

### 1. Keep PLAN.md Updated

Treat it as living documentation:
- Update as you learn
- Add new phases as needed
- Document decisions

### 2. Use Descriptive PR Descriptions

```markdown
# Good
| PR-015 | Parse "Depends on #X" syntax from issue body | `src/deps.rs` |

# Less helpful
| PR-015 | Add dependency parsing | |
```

### 3. Balance Phase Size

- **Too small:** Overhead of many checkpoints
- **Too large:** Hard to track progress
- **Just right:** 5-10 PRs per phase

### 4. Include File References

Helps agents understand scope:

```markdown
| PR-015 | Dependency parsing | `src/deps.rs`, `tests/deps_test.rs` |
```

### 5. Celebrate Checkpoints

After each phase:
- Run tests
- Review progress
- Update stakeholders
- Take a moment to appreciate the milestone!

## Troubleshooting

### Import creates duplicate issues

**Problem:** Re-importing creates new issues.

**Solution:** Use `--skip-existing` flag (enabled by default)

### Circular dependencies detected

**Problem:** Plan has circular dependencies in PR ordering.

**Solution:**
1. Review PLAN.md structure
2. Ensure PRs flow in logical order
3. Break circular dependencies

### Too many issues created

**Problem:** Plan import created hundreds of issues.

**Solution:**
1. Start with fewer phases
2. Import incrementally: one phase at a time
3. Close issues that aren't needed

### Can't find ready issues

**Problem:** All issues are blocked.

**Solution:**
1. Check dependency structure: `murmur issue deps <issue>`
2. Work on the earliest unblocked issue
3. May need to complete dependencies in order

## Summary

Project bootstrap workflow:

1. **Create PLAN.md** with phases, PRs, and checkpoints
2. **Dry run import** to preview issues
3. **Execute import** to create GitHub issues
4. **Work through phases** sequentially or in parallel
5. **Verify checkpoints** after each phase
6. **Monitor progress** with `plan status`
7. **Reach bootstrap milestone** where system builds itself

This structured approach enables:
- Clear project roadmap
- Automatic issue creation
- Dependency management
- Progress tracking
- Team coordination

Perfect for bootstrapping new projects or adding complex features to existing ones!
