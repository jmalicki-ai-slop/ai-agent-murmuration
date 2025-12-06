# Multi-Agent Orchestration

Murmuration enables parallel development through isolated git worktrees and intelligent dependency management. This guide covers how to run multiple agents concurrently and coordinate their work.

## Why Multi-Agent Development?

Traditional single-agent workflows create bottlenecks:
- Only one feature can be worked on at a time
- Blocked issues delay all downstream work
- No parallelization of independent tasks

Murmuration's multi-agent approach:
- Run multiple agents in parallel on independent issues
- Automatic dependency checking prevents conflicts
- Each agent works in an isolated git worktree
- Intelligent branch management from latest main

## Git Worktrees for Isolation

Worktrees provide isolated working directories sharing the same repository.

### What is a Worktree?

A git worktree is a linked working directory:
- Shares the same `.git` repository
- Has its own branch and working files
- Changes don't affect other worktrees
- Can be created, removed, and managed independently

### Benefits for Multi-Agent Work

1. **Isolation:** Each agent works in its own directory
2. **Parallelism:** Multiple agents can run simultaneously
3. **Branch Safety:** No conflicts from concurrent branch updates
4. **Fast Creation:** Worktrees are lightweight (hard links, not full clones)

### Worktree Structure

```
Main repository:
/Users/dev/murmuration/.git

Cached worktrees:
~/.cache/murmur/worktrees/
├── owner-repo/
│   ├── issue-15/      # Working on issue #15
│   ├── issue-16/      # Working on issue #16
│   └── issue-17/      # Working on issue #17
```

## Creating Worktrees

### Automatic Creation with `murmur work`

The easiest way is to use `murmur work`, which creates worktrees automatically:

```bash
# Terminal 1
murmur work 15

# Terminal 2
murmur work 16

# Terminal 3
murmur work 17
```

Each command:
1. Checks dependencies
2. Creates a worktree from latest main
3. Spawns an agent in that worktree
4. Tracks the worktree in the database

### Manual Worktree Creation

Create a worktree without starting an agent:

```bash
murmur worktree create issue-15
```

Output:
```
Repository: murmuration
Branching from: origin/main (a1b2c3d4)

Created worktree:
  Path:   /Users/dev/.cache/murmur/worktrees/murmuration/issue-15
  Branch: murmur/issue-15
  Base:   origin/main (a1b2c3d4)
```

Then work in it manually:

```bash
cd ~/.cache/murmur/worktrees/murmuration/issue-15
# ... make changes ...
```

### Worktree Naming Convention

Worktrees are named `murmur/<task-id>`:
- Issue-based: `murmur/issue-15`
- Custom task: `murmur/add-caching`
- Feature: `murmur/feature-auth`

## Managing Multiple PRs

### Parallel PR Development

Each worktree can produce its own PR:

```bash
# Terminal 1: Work on issue 15
$ murmur work 15
# ... agent completes work ...
# Creates PR #101

# Terminal 2: Work on issue 16 (independent of 15)
$ murmur work 16
# ... agent completes work ...
# Creates PR #102

# Terminal 3: Work on issue 17 (depends on 15)
$ murmur work 17
❌ Blocked by 1 unmet dependency.
  ❌ #15: GitHub API client [PR #101 open]
```

### Auto-Commit and Auto-Push

Configure automatic PR creation in `~/.config/murmur/config.toml`:

```toml
[workflow]
auto_commit = true
auto_push = true
auto_pr = true
```

When an agent completes successfully:
1. Changes are committed with a descriptive message
2. Branch is pushed to origin
3. PR is created with issue details

Example output:
```
✅ Agent completed successfully

Detected uncommitted changes. Committing...
✅ Changes committed successfully

Pushing branch to origin...
✅ Branch pushed successfully

Creating pull request...
✅ Pull request created successfully
https://github.com/owner/repo/pull/105
```

### Review and Merge Workflow

1. **Parallel Development:** Multiple agents work on independent issues
2. **Auto-PR Creation:** Each agent creates a PR when done
3. **Review:** Human or review agent examines PRs
4. **Merge:** PRs are merged when approved
5. **Unblock:** Dependent issues can now proceed

```
Issue #15 (no deps)  →  PR #101  →  Merge  →  Unblocks #17
Issue #16 (no deps)  →  PR #102  →  Merge
Issue #17 (depends on #15)  →  Waits for #101  →  PR #103
```

## Coordinating Dependent Issues

### Dependency-Based Workflow

Murmuration respects issue dependencies:

```markdown
# Issue #17
Depends on #15
```

When you try to work on #17:

```bash
$ murmur work 17

Checking dependencies...
  ❌ #15: GitHub API client [PR #101 open, not merged]

❌ Blocked by 1 unmet dependency.

Options:
  1. Wait for PR #101 to merge
  2. Run `murmur work 15` to help finish the blocking issue
  3. Run `murmur work 17 --force` to proceed anyway
```

### Parallel Work Tree

Work proceeds in this order:

```
Phase 1: Start independent issues
├─ murmur work 15 (no dependencies) ✅ Start immediately
├─ murmur work 16 (no dependencies) ✅ Start immediately
└─ murmur work 17 (depends on #15) ❌ Blocked

Phase 2: Complete independent work
├─ Issue 15 completes → PR #101 created
└─ Issue 16 completes → PR #102 created

Phase 3: Review and merge
├─ PR #101 reviewed and merged ✅
└─ PR #102 reviewed and merged ✅

Phase 4: Unblocked work proceeds
└─ murmur work 17 ✅ Now unblocked, starts working
```

### Identifying Ready Issues

Find issues that can be worked on in parallel:

```bash
murmur issue list --ready
```

Output:
```
Ready to work (no unmet dependencies):

#15: GitHub API client
#16: Parse issue metadata
#18: Git repository detection

Blocked (waiting on dependencies):

#17: Fetch issues from repo (blocked by #15)
#19: Dependency graph (blocked by #16)
```

## Worktree Lifecycle

### 1. Creation

Created automatically by `murmur work` or manually by `murmur worktree create`.

### 2. Active Use

Agent works in the worktree:
- Makes commits
- Runs tests
- Modifies files

Worktree status: `Active`

### 3. Completion

Agent completes successfully:
- PR is created (if auto-PR enabled)
- Worktree status: `Completed`

### 4. Cleanup

Remove completed worktrees:

```bash
# Clean completed worktrees
murmur worktree clean

# Clean all non-active worktrees
murmur worktree clean --all

# Clean worktrees older than 7 days
murmur worktree clean --older-than 7

# Also delete git branches
murmur worktree clean --all --delete-branches
```

### 5. Orphaned Worktrees

Worktrees become orphaned when:
- Agent crashed or was killed
- Manual termination of agent process
- Agent completed but worktree remains

These are worktrees that exist on disk but have no running agent associated with them (as shown by `murmur status`).

Clean them up:

```bash
murmur worktree clean --stale-only
```

## Listing and Inspecting Worktrees

### List All Worktrees

```bash
murmur worktree list
```

Output:
```
Repository: owner-repo

  issue-15 [active] - task: 15
    Branch: murmur/issue-15
    Base:   a1b2c3d4

  issue-16 [completed] - task: 16
    Branch: murmur/issue-16
    Base:   a1b2c3d4

  issue-17 [active] - task: 17
    Branch: murmur/issue-17
    Base:   b2c3d4e5
```

### Show Worktree Details

```bash
murmur worktree show 15
```

Output:
```
Worktree Details
================

Repository: owner-repo
Path:       /Users/dev/.cache/murmur/worktrees/owner-repo/issue-15
Task:       15
Branch:     murmur/issue-15
Base:       a1b2c3d4e5f6...
Status:     Active
Dirty:      yes
```

### Filter by Repository

```bash
murmur worktree list --repo murmuration
```

## Best Practices

### 1. Work on Independent Issues First

Maximize parallelism:

```bash
# Good: All independent
murmur work 10 &
murmur work 11 &
murmur work 12 &

# Less efficient: Sequential dependencies
murmur work 20  # Depends on nothing
# Wait for 20 to complete
murmur work 21  # Depends on 20
# Wait for 21 to complete
murmur work 22  # Depends on 21
```

### 2. Clean Up Regularly

Don't let worktrees accumulate:

```bash
# Weekly cleanup
murmur worktree clean --older-than 7 --delete-branches
```

### 3. Monitor Worktree Status

Check what's in progress:

```bash
murmur worktree list | grep active
```

### 4. Use Descriptive Task IDs

```bash
# Good
murmur worktree create auth-feature

# Less clear
murmur worktree create feature
```

### 5. Don't Force Dependencies Unnecessarily

```bash
# Only use --force if you're confident
murmur work 42 --force
```

Forcing can lead to:
- Merge conflicts later
- Duplicate work
- Integration issues

## Advanced Patterns

### Parallel Feature Development

Work on multiple sub-features simultaneously:

```bash
# Epic: Add authentication (#20)
# Child issues: #21, #22, #23 (all independent)

# Terminal 1
murmur work 21  # Implement login endpoint

# Terminal 2
murmur work 22  # Implement signup endpoint

# Terminal 3
murmur work 23  # Implement token validation

# All three can work in parallel!
```

### Staggered Release

Create PRs in sequence even though work is parallel:

```bash
# Work in parallel
murmur work 15 &  # Feature A
murmur work 16 &  # Feature B

# Feature A completes first → PR #101
# Feature B completes later → PR #102

# Review and merge #101 first
# Then review and merge #102

# Controlled release schedule
```

### Cross-Repository Workflows

Work on issues in different repositories:

```bash
# Terminal 1: Backend work
murmur work 15 --repo backend-org/api

# Terminal 2: Frontend work
murmur work 32 --repo frontend-org/web

# Note: Cross-repo dependencies are not validated
```

## Troubleshooting

### "Worktree already exists and is active"

**Problem:** Trying to create a worktree that already exists.

**Solutions:**
```bash
# Force recreate
murmur work 42 --force

# Or clean up first
murmur worktree clean --stale-only
```

### "No space left on device"

**Problem:** Too many worktrees consuming disk space.

**Solution:**
```bash
# Clean old worktrees
murmur worktree clean --older-than 14

# Or clean all completed
murmur worktree clean --all
```

### Merge Conflicts Between PRs

**Problem:** Two parallel PRs modify the same files.

**Prevention:**
- Design issues to minimize file overlap
- Review dependency structure
- Merge PRs in dependency order

**Resolution:**
```bash
# Rebase PR branch on latest main
cd ~/.cache/murmur/worktrees/owner-repo/issue-42
git fetch origin
git rebase origin/main
# Resolve conflicts
git push --force-with-lease
```

### Agent Crashes Leave Orphaned Worktrees

**Problem:** Agent fails or is killed, worktree remains on disk with no running agent.

**Solution:**
```bash
murmur worktree clean --stale-only
```

This cleans up worktrees that exist on disk but have no running agent (matching what `murmur status` shows as "Stale Worktrees").

## Future Enhancements

Future versions of Murmuration will support:

- **Coordinator Agent:** Automatically spawns agents on ready issues
- **Concurrent Agent Limits:** Configure max parallel agents
- **Dependency-Aware Scheduling:** Work on issues as dependencies complete
- **Automatic Worktree Rebase:** Keep worktrees up-to-date with main
- **Inter-Agent Communication:** Coordinate work across agents

See `PLAN.md` Phase 9 (Background Daemon) for details.

## Example Multi-Agent Session

### Scenario: Three Independent Features

```bash
# Check which issues are ready
$ murmur issue list --ready

Ready to work:
  #15: GitHub API client
  #16: Parse issue metadata
  #17: Git repository detection

# Start three agents in parallel

# Terminal 1
$ murmur work 15
Creating worktree for #15...
  Created: /Users/dev/.cache/murmur/worktrees/owner-repo/issue-15
Starting agent...

# Terminal 2
$ murmur work 16
Creating worktree for #16...
  Created: /Users/dev/.cache/murmur/worktrees/owner-repo/issue-16
Starting agent...

# Terminal 3
$ murmur work 17
Creating worktree for #17...
  Created: /Users/dev/.cache/murmur/worktrees/owner-repo/issue-17
Starting agent...

# All three agents are now working in parallel!

# Monitor progress
$ murmur worktree list
Repository: owner-repo
  issue-15 [active] - task: 15
  issue-16 [active] - task: 16
  issue-17 [active] - task: 17

# As agents complete, PRs are created
# Terminal 1: Issue #15 completes → PR #101
# Terminal 2: Issue #16 completes → PR #102
# Terminal 3: Issue #17 completes → PR #103

# Review and merge PRs
# Dependent issues can now proceed

$ murmur work 18  # Depends on #15, #16
✅ All dependencies satisfied!
Creating worktree...
```

## Summary

Multi-agent orchestration with Murmuration:

1. **Use worktrees** for parallel, isolated development
2. **Respect dependencies** to prevent conflicts
3. **Automate PR creation** with workflow config
4. **Clean up regularly** to manage disk space
5. **Monitor progress** with `worktree list` and `issue list`

This enables massive parallelization while maintaining code quality and integration safety.
