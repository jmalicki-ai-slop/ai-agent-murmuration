# Working with GitHub Issues

Murmuration uses GitHub issues as the primary interface for task management. This guide covers how to structure issues, define dependencies, and work with epics.

## Issue Format

Issues in Murmuration can be enhanced with metadata and dependency information to enable intelligent workflow orchestration.

### Basic Issue Structure

```markdown
# Issue Title

## Description

Clear description of what needs to be done.

## Dependencies

Depends on #15
Blocked by #23

## Metadata

<!-- murmur:metadata
{
  "phase": 3,
  "pr": "023",
  "status": "ready"
}
-->
```

## Issue Metadata

Metadata is embedded in HTML comments and parsed by Murmuration to track workflow state.

### Metadata Fields

- `phase` (number): The phase number this issue belongs to (e.g., 3 for Phase 3)
- `pr` (string): PR identifier from PLAN.md (e.g., "023" for PR-023)
- `status` (string): Current status - "ready", "blocked", "in_progress", "completed"
- `type` (string): Type of issue - "epic", "pr", "task"
- `parent` (number): Parent epic issue number
- `depends_on` (array): Array of issue numbers this depends on

### Example Metadata Block

```markdown
<!-- murmur:metadata
{
  "phase": 5,
  "pr": "037",
  "depends_on": [35, 36],
  "status": "blocked",
  "type": "pr"
}
-->
```

## Dependency Syntax

Murmuration supports both GitHub native issue tracking and markdown-based dependency declarations.

### Markdown Dependencies

Use these patterns in your issue body:

```markdown
## Dependencies

Depends on #15
Depends on #16, #17
Blocked by #23
```

### Supported Patterns

- `Depends on #X` - This issue depends on issue X
- `depends on #X` - Case insensitive
- `Blocked by #X` - This issue is blocked by issue X
- `blocked by #X` - Case insensitive

### Comma-Separated Dependencies

You can list multiple dependencies on a single line:

```markdown
Depends on #12, #13, #14
```

### Cross-Repository Dependencies

Reference issues in other repositories:

```markdown
Depends on other-org/other-repo#456
```

**Note:** Cross-repository dependencies are parsed but not validated by the `murmur work` command. Only local (same-repo) dependencies are checked.

### Invalid Dependency References

Dependency references must use the `#123` or `owner/repo#123` format. Other formats will cause an error:

```markdown
# This will fail:
Depends on: PR-023

# Use this instead:
Depends on #23
```

## Epics and Child Issues

Murmuration supports hierarchical issue organization through epics.

### Creating an Epic

Mark an issue as an epic using metadata:

```markdown
# Phase 3: GitHub Integration

Epic for all GitHub-related features.

## Child Issues

- [ ] #15 GitHub API client
- [ ] #16 Issue fetching
- [ ] #17 Dependency parsing

<!-- murmur:metadata
{
  "type": "epic",
  "phase": 3
}
-->
```

### Linking Child Issues to Parent

Child issues should reference their parent:

```markdown
# GitHub API Client

Implementation of GitHub API client.

Parent: #14

<!-- murmur:metadata
{
  "parent": 14,
  "phase": 3,
  "pr": "012"
}
-->
```

## Working with Issues

### Checking Dependencies

Use `murmur work` to automatically check dependencies before starting work:

```bash
$ murmur work 42

Working on issue #42 in owner/repo

#42: Implement TDD workflow

Checking dependencies...

  ✅ #38: Agent type definitions [complete]
  ✅ #39: System prompt loading [complete]
  ❌ #40: Context building [PR #53 open]

❌ Blocked by 1 unmet dependency.

Options:
  1. Wait for PR #53 to merge
  2. Run `murmur work 40` to start the blocking issue
  3. Run `murmur work 42 --force` to proceed anyway
```

### Dependency Resolution

When you run `murmur work <issue>`, Murmuration:

1. Parses the issue body for dependency declarations
2. Checks each dependency's status:
   - Is the issue closed?
   - Does it have a linked PR?
   - Is that PR merged?
3. Blocks work if any dependencies are unmet
4. Provides options to resolve blockers

### Forcing Work on Blocked Issues

Skip dependency checking with `--force`:

```bash
murmur work 42 --force
```

Use this when:
- You're confident the dependency isn't critical
- You're working on a prototype
- You need to make progress while waiting for reviews

## Viewing Issues and Dependencies

### List Open Issues

```bash
murmur issue list
```

Output:
```
Open issues for owner/repo:

#45: Implement review workflow [Phase 6]
     Labels: enhancement, phase-6
     Dependencies: 2

#42: Implement TDD workflow [Phase 5]
     Labels: enhancement, phase-5
     Dependencies: 3

#40: Add context building [Phase 4]
     Labels: enhancement, phase-4
     In Progress (PR #53)
```

### Show Issue Details

```bash
murmur issue show 42
```

Output:
```
Issue #42: Implement TDD workflow

Status: Open
Labels: enhancement, phase-5
Created: 2025-11-15

Description:
Implement the full TDD cycle with enforced red-green validation.

Dependencies:
  ✅ #38: Agent type definitions
  ✅ #39: System prompt loading
  ❌ #40: Context building (blocking)

Metadata:
  Phase: 5
  PR: 035
  Status: blocked
```

### View Dependency Graph

```bash
murmur issue deps 42
```

Output:
```
Dependency graph for #42:

  ┌─ #38 (completed)
  │
#42 ┼─ #39 (completed)
  │
  └─ #40 (in progress) → #35 → #33
                              └─ #30

Transitive dependencies: #40, #35, #33, #30
Direct dependencies: #38, #39, #40
Ready to work: #30, #33
```

## Best Practices

### 1. Clear Descriptions

Write issue descriptions that:
- Explain what needs to be done
- Define success criteria
- List any edge cases or special considerations
- Reference relevant documentation or design docs

### 2. Explicit Dependencies

Always declare dependencies explicitly:
- Use `Depends on #X` for sequential work
- Use `Blocked by #X` for external blockers
- Include cross-repo dependencies when relevant

### 3. Update Status

Keep metadata status current:
- `ready` - All dependencies met, can start
- `blocked` - Waiting on dependencies
- `in_progress` - Currently being worked on
- `completed` - Work finished, PR merged

### 4. Organize with Epics

For large features:
- Create an epic issue with `"type": "epic"`
- Break work into smaller child issues
- Use `Parent: #X` to link children to epic
- Include checklist in epic body

### 5. Avoid Circular Dependencies

Murmuration detects circular dependencies during plan import:

```markdown
# Invalid:
#10 depends on #11
#11 depends on #10

# Valid:
#10 → #11 → #12
```

### 6. Test Dependency Checking

Before creating many issues, test dependency syntax:

```bash
# This should succeed:
murmur work <issue> --no-agent

# This tests dependency parsing without starting an agent
```

## Common Patterns

### Sequential Feature Development

```markdown
# Issue #10
Depends on #9

# Issue #11
Depends on #10

# Issue #12
Depends on #11
```

### Parallel Work with Merge Point

```markdown
# Issue #15 - Feature A
Depends on #10

# Issue #16 - Feature B
Depends on #10

# Issue #17 - Integration
Depends on #15, #16
```

### Epic with Independent Tasks

```markdown
# Epic #20: Dashboard Feature

## Child Issues
- [ ] #21 Backend API
- [ ] #22 Frontend components
- [ ] #23 Integration tests

# Issue #21
Parent: #20

# Issue #22
Parent: #20

# Issue #23
Parent: #20
Depends on #21, #22
```

## GitHub Native Tracking

Murmuration also supports GitHub's native issue tracking (beta feature):

When issues are linked using GitHub's UI, Murmuration automatically:
- Reads tracked issues from the GitHub GraphQL API
- Treats tracked issues as dependencies
- Validates their status before allowing work

This works alongside markdown dependencies, with native tracking taking precedence when available.

## Troubleshooting

### "Invalid dependency references found"

**Problem:** Dependency reference format is incorrect.

**Solution:** Use `#123` or `owner/repo#123` format. Avoid "PR-023" or other formats.

### "Blocked by unmet dependency"

**Problem:** Dependency issue's PR isn't merged yet.

**Solutions:**
1. Work on the blocking issue first: `murmur work <dep-issue>`
2. Wait for the PR to merge
3. Use `--force` if you're confident the dependency isn't critical

### Dependencies not detected

**Problem:** Markdown syntax not recognized.

**Solutions:**
1. Check for typos: `Depends on` (not `Depend on` or `Dependency:`)
2. Ensure `#` prefix on issue numbers
3. Verify metadata JSON is valid (use a JSON validator)

### Circular dependency error during import

**Problem:** Issues depend on each other in a loop.

**Solution:** Restructure dependencies to be acyclic. Review PLAN.md for logical ordering.
