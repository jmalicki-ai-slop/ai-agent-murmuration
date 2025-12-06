# Core Concepts

This document explains the fundamental concepts behind Murmuration's architecture and workflow.

## Agent Types

Murmuration uses specialized agent types, each with distinct responsibilities and prompts. This separation ensures focused, high-quality work at each stage of development.

### Implement Agent

**Purpose**: Writes code to implement features and fix bugs.

**When to Use**:
- Writing new functionality
- Implementing fixes for bugs
- Creating minimal code to make tests pass (TDD)
- Refactoring existing code

**Characteristics**:
- Has full read/write access to codebase
- Can run commands (build, test, lint)
- Works in active git worktree
- Focuses on making tests pass, not perfect code

**Prompt Focus**:
- Understand the requirements clearly
- Write minimal code to satisfy tests
- Follow project conventions and patterns
- Keep changes focused and small

**Example Usage**:
```bash
murmur agent --type implement
```

### Test Agent

**Purpose**: Writes tests and validates implementations.

**When to Use**:
- Writing tests based on specifications
- Creating test cases for edge conditions
- Verifying test quality (coverage, assertions)
- Ensuring tests actually fail before implementation

**Characteristics**:
- Specializes in test framework knowledge
- Understands testing patterns and best practices
- Can write tests without implementation code
- Validates tests fail for the right reasons

**Prompt Focus**:
- Write clear, readable tests
- Cover main functionality and edge cases
- Use appropriate assertions
- Do NOT write implementation code

**Example Usage**:
```bash
murmur agent --type test
```

### Review Agent

**Purpose**: Reviews code changes and provides feedback.

**When to Use**:
- Reviewing specs before writing tests
- Reviewing tests before implementation
- Reviewing implementation before marking complete
- Final review before PR creation

**Characteristics**:
- Runs in isolated mode (sees diffs, doesn't modify code)
- Focuses on quality, patterns, and potential issues
- Provides actionable feedback
- Can approve or request changes

**Prompt Focus**:
- Check for correctness and completeness
- Identify potential bugs or edge cases
- Verify adherence to project conventions
- Provide specific, actionable feedback

**Example Usage**:
```bash
murmur agent --type review
```

### Coordinator Agent

**Purpose**: Orchestrates workflow and delegates to other agents.

**When to Use**:
- Running full TDD workflows
- Managing multi-phase development cycles
- Routing feedback between agents
- Deciding when to escalate to human

**Characteristics**:
- Can spawn and monitor other agents
- Manages workflow state transitions
- Interprets review feedback
- Makes decisions about iteration vs. escalation

**Prompt Focus**:
- Understand overall workflow state
- Route tasks to appropriate agent types
- Integrate feedback from reviewers
- Know when to ask for human help

**Example Usage**:
```bash
murmur orchestrate <issue-number>
```

## Git Worktrees

### What are Worktrees?

Git worktrees allow multiple working directories for a single repository, each with its own branch and files. Think of them as "lightweight clones" that share the same git history but have independent working states.

### Why Worktrees Instead of Clones?

**Advantages**:
- **Fast**: No need to clone the entire repository
- **Disk Efficient**: Share .git objects, only duplicate working files
- **No Conflicts**: Each worktree has independent working state
- **Parallel Work**: Multiple agents work on different issues simultaneously

**Traditional Approach (Clones)**:
```bash
git clone repo.git issue-42/     # 500MB download
git clone repo.git issue-43/     # Another 500MB
# Total: 1GB for 2 workspaces
```

**Worktree Approach**:
```bash
git clone repo.git main/         # 500MB once
git worktree add ../issue-42/    # 50MB (working files only)
git worktree add ../issue-43/    # 50MB
# Total: 600MB for 3 workspaces
```

### Worktree Naming Convention

Murmuration uses a consistent naming pattern:

```
~/.cache/murmur/worktrees/<repo-name>/<branch-name>/
```

Examples:
- `~/.cache/murmur/worktrees/murmuration/murmur-issue-42/`
- `~/.cache/murmur/worktrees/myproject/feature-login-page/`

Branch names follow the pattern: `murmur/<issue-number>` or `murmur/<description>`

### Worktree Lifecycle

1. **Creation**
   ```bash
   murmur work 42
   # Creates: ~/.cache/murmur/worktrees/repo/murmur-issue-42/
   # Branch: murmur/issue-42
   # Base: origin/main (or specified branching point)
   ```

2. **Active Use**
   - Agent works in the worktree directory
   - All git operations isolated to this worktree
   - Can commit, test, build without affecting main repo

3. **Completion**
   - Agent commits changes
   - Creates PR from branch
   - Worktree marked as "completed" in database

4. **Caching**
   - Completed worktrees kept in cache
   - Can be reused if working on same issue again
   - Metadata stored in database for tracking

5. **Cleanup**
   - Manual: `murmur worktree clean`
   - Automatic: LRU eviction when cache exceeds size limit
   - Stale detection: Directory missing or agent dead

### Branching Strategy

Worktrees always branch from a clean, up-to-date point:

```
origin/main ───┬──> murmur/issue-42 (worktree 1)
               ├──> murmur/issue-43 (worktree 2)
               └──> murmur/issue-44 (worktree 3)
```

Before creating a worktree:
1. Fetch latest from origin: `git fetch origin`
2. Find default branch (main/master)
3. Use `origin/main` as branching point
4. Create new branch from there

This ensures each worktree starts fresh, avoiding merge conflicts between concurrent work.

## TDD Phases

Murmuration enforces a rigorous Test-Driven Development workflow with 7 distinct phases.

### The 7-Phase Cycle

```
┌──────────────┐
│  WriteSpec   │  Agent writes specification document
└──────┬───────┘
       │
       v
┌──────────────┐
│  WriteTests  │  Agent writes tests based on spec
└──────┬───────┘
       │
       v
┌──────────────┐
│  VerifyRed   │  Tests MUST fail (proves they test something)
└──────┬───────┘
       │
       v
┌──────────────┐
│  Implement   │  Agent writes minimal code to pass tests
└──────┬───────┘
       │
       v
┌──────────────┐
│ VerifyGreen  │  Tests MUST pass (proves implementation works)
└──────┬───────┘
       │
       v
┌──────────────┐
│  Refactor    │  Agent improves code while keeping tests green
└──────┬───────┘
       │
       v
┌──────────────┐
│  Complete    │  TDD cycle finished, ready for PR
└──────────────┘
```

### Phase Details

#### Phase 1: WriteSpec

**Purpose**: Create a specification document describing expected behavior.

**Agent**: Implement

**Validation**: Spec file exists and describes inputs, outputs, edge cases

**Output**: Usually a SPEC.md or similar document

**Can Skip**: Yes, with `--skip-spec` flag

#### Phase 2: WriteTests

**Purpose**: Write tests that verify the specified behavior.

**Agent**: Test

**Validation**: Test files exist and compile

**Critical**: Tests must NOT include implementation code

**Transition**: Only to VerifyRed (can't skip validation)

#### Phase 3: VerifyRed

**Purpose**: Prove tests actually test something by ensuring they fail.

**Agent**: None (automated test runner)

**Validation**: Tests MUST fail
- If tests pass: Something is wrong, go back to WriteTests
- If tests fail with wrong errors: Fix test setup, go back to WriteTests
- If tests fail correctly: Advance to Implement

**Why This Matters**: Passing tests before implementation means they're not testing the right thing.

#### Phase 4: Implement

**Purpose**: Write minimal code to make tests pass.

**Agent**: Implement

**Prompt**: "Write MINIMAL code to make tests pass, don't refactor yet"

**Validation**: None (trust agent to implement)

**Transition**: Automatically to VerifyGreen

#### Phase 5: VerifyGreen

**Purpose**: Prove implementation works by ensuring tests pass.

**Agent**: None (automated test runner)

**Validation**: Tests MUST pass
- If tests fail: Go back to Implement (retry)
- If exceed max iterations: Escalate to human
- If tests pass: Advance to Refactor

**Iteration Tracking**: Counts Implement → VerifyGreen cycles

#### Phase 6: Refactor

**Purpose**: Improve code quality while keeping tests green.

**Agent**: Implement

**Prompt**: "Refactor to improve quality, run tests after each change"

**Validation**: Tests must stay green

**Can Skip**: Yes, with `--skip-refactor` flag

#### Phase 7: Complete

**Purpose**: TDD cycle finished, ready for PR.

**Validation**: All tests passing, implementation complete

**Next Steps**: Commit, create PR, link to issue

### Phase Transitions

Valid transitions (including backwards for iteration):

```
WriteSpec    → WriteTests
WriteTests   → VerifyRed, WriteSpec (restart)
VerifyRed    → Implement, WriteTests (tests wrong)
Implement    → VerifyGreen
VerifyGreen  → Refactor, Implement (tests failing)
Refactor     → Complete, VerifyGreen (broke tests)
Complete     → Refactor (more cleanup)
```

Any phase can restart from WriteSpec for major changes.

### Validation Enforcement

The workflow enforces red-green validation automatically:

```rust
// Phase 3: VerifyRed
let result = test_runner.run_tests(workdir)?;
if result.all_passed() {
    // Tests passed when they should fail!
    workflow.retry_tests(Some("Tests passed unexpectedly"));
    return Err("Tests must fail before implementation");
}

// Phase 5: VerifyGreen
let result = test_runner.run_tests(workdir)?;
if !result.all_passed() {
    if workflow.exceeded_max_iterations() {
        // Give up and ask human for help
        escalate_to_human();
    } else {
        // Try implementing again
        workflow.retry_implement(Some("Tests still failing"));
    }
}
```

### Configuration Options

```bash
# Full TDD cycle
murmur tdd 42

# Skip spec phase (start from WriteTests)
murmur tdd 42 --skip-spec

# Skip refactor phase (Complete after VerifyGreen)
murmur tdd 42 --skip-refactor

# Set max implementation iterations
murmur tdd 42 --max-iterations 5
```

## Dependencies

### How Issue Dependencies Work

Murmuration prevents working on blocked issues by checking dependency status before starting work.

### Dependency Syntax

In GitHub issue bodies:

```markdown
## Dependencies
Depends on #12
Blocked by #15
Parent: #8
```

Or using GitHub's native issue tracking (task lists).

### Dependency Types

**Depends on**: This issue requires another issue to be completed first
```markdown
Depends on #42
```

**Blocked by**: Synonym for "depends on"
```markdown
Blocked by #42
```

**Parent**: This issue is part of a larger epic
```markdown
Parent: #100
```

### Dependency Resolution

When running `murmur work 42`:

1. **Parse Dependencies**: Extract all "Depends on" references from issue body
2. **Check Each Dependency**:
   - Is the dependency issue closed?
   - Does it have a linked PR?
   - Is that PR merged?
3. **Block if Unmet**:
   ```
   Issue #42: Implement TDD workflow
   Status: blocked

   Dependencies:
     ✓ #38 - Agent types [PR #51 merged]
     ✓ #39 - Prompts [PR #52 merged]
     ✗ #40 - Context building [PR #53 open, not merged]

   Blocked by 1 unmerged dependency.

   Options:
     1. Wait for PR #53 to merge
     2. Run `murmur work 40` to help finish the blocking issue
   ```

4. **Proceed if Met**: All dependencies satisfied, create worktree and start work

### Dependency Graph

Murmuration builds a dependency graph to understand relationships:

```
#100 (Epic: Phase 4)
  ├─ #38 (Agent types) ✓ merged
  ├─ #39 (Prompts) ✓ merged
  ├─ #40 (Context) → in progress
  └─ #41 (Factory) ← blocked by #40
```

View with:
```bash
murmur issue deps 100
```

### Cross-Repository Dependencies

Dependencies can reference issues in other repositories:

```markdown
Depends on owner/other-repo#42
```

Murmuration will:
1. Fetch the external issue via GitHub API
2. Check its PR status
3. Block if not merged

### Circular Dependencies

Murmuration detects circular dependencies and reports them:

```
Error: Circular dependency detected:
  #42 depends on #43
  #43 depends on #44
  #44 depends on #42
```

## Database

### What's Stored

The SQLite database (`~/.murmur/state.db`) stores:

1. **Agent Runs**: History of all agent executions
2. **Conversations**: JSON logs of agent interactions
3. **Worktrees**: Active and cached worktree metadata
4. **Issues**: Local state tracking for GitHub issues

### Why SQLite?

**Advantages**:
- **Zero Configuration**: No server setup
- **Portable**: Single file database
- **Fast**: Sufficient for local tool usage
- **Reliable**: ACID transactions, battle-tested

**Design Decisions**:
- Read-heavy workload (status queries)
- Single-user (local development tool)
- Simple schema (normalized tables)
- No need for concurrent connections from multiple machines

### Schema Overview

#### agent_runs Table

Tracks every agent execution:

```sql
CREATE TABLE agent_runs (
    id INTEGER PRIMARY KEY,
    agent_type TEXT NOT NULL,
    issue_number INTEGER,
    prompt TEXT NOT NULL,
    workdir TEXT NOT NULL,
    config_json TEXT NOT NULL,
    pid INTEGER,
    start_time TIMESTAMP NOT NULL,
    end_time TIMESTAMP,
    exit_code INTEGER,
    duration_seconds REAL,
    created_at TIMESTAMP NOT NULL
);
```

**Use Cases**:
- Show currently running agents
- Calculate total time spent on issue
- Resume interrupted work
- Track success/failure rates

#### conversation_logs Table

Stores JSON output from Claude Code:

```sql
CREATE TABLE conversation_logs (
    id INTEGER PRIMARY KEY,
    agent_run_id INTEGER NOT NULL,
    sequence INTEGER NOT NULL,
    timestamp TIMESTAMP NOT NULL,
    message_type TEXT NOT NULL,
    message_json TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL,
    FOREIGN KEY (agent_run_id) REFERENCES agent_runs(id)
);
```

**Use Cases**:
- Replay agent interactions
- Debug agent behavior
- Extract tool usage patterns
- Calculate token costs

**Message Types**: system, user, assistant, tool_use, tool_result, result

#### worktrees Table

Tracks git worktrees:

```sql
CREATE TABLE worktrees (
    id INTEGER PRIMARY KEY,
    path TEXT NOT NULL UNIQUE,
    branch_name TEXT NOT NULL,
    issue_number INTEGER,
    agent_run_id INTEGER,
    main_repo_path TEXT,
    base_commit TEXT,
    status TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);
```

**Use Cases**:
- Find worktree for issue
- Clean up stale worktrees
- Reuse existing worktrees
- Track worktree age for LRU eviction

**Statuses**: active, completed, abandoned, stale

#### issues Table

Caches GitHub issue state:

```sql
CREATE TABLE issues (
    id INTEGER PRIMARY KEY,
    issue_number INTEGER NOT NULL,
    repository TEXT NOT NULL,
    title TEXT NOT NULL,
    status TEXT NOT NULL,
    labels_json TEXT,
    dependencies_json TEXT,
    last_agent_run_id INTEGER,
    last_worked_at TIMESTAMP,
    last_error TEXT,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    UNIQUE(issue_number, repository)
);
```

**Use Cases**:
- Track which issues are in progress
- Remember issue status across runs
- Cache dependency information
- Record errors for retries

**Statuses**: open, in_progress, blocked, completed, failed

### Database Location

Default: `~/.murmur/state.db`

Override via config:
```toml
[database]
path = "/custom/path/murmur.db"
```

Or environment variable:
```bash
export MURMUR_DB_PATH=/custom/path/murmur.db
```

### Migrations

Database schema is versioned and migrated automatically:

```
murmur-db/migrations/
├── 001_initial_schema.sql
├── 002_add_worktree_base_commit.sql
└── 003_add_issue_dependencies.sql
```

On first run, all migrations execute. On subsequent runs, only new migrations apply.

### Querying the Database

For debugging or analysis:

```bash
# Show running agents
sqlite3 ~/.murmur/state.db "
  SELECT agent_type, issue_number, start_time
  FROM agent_runs
  WHERE end_time IS NULL
"

# Show conversation for run #123
sqlite3 ~/.murmur/state.db "
  SELECT sequence, message_type, message_json
  FROM conversation_logs
  WHERE agent_run_id = 123
  ORDER BY sequence
"

# Show active worktrees
sqlite3 ~/.murmur/state.db "
  SELECT path, branch_name, status
  FROM worktrees
  WHERE status = 'active'
"
```

### Data Retention

**Agent Runs**: Kept indefinitely (useful for history)

**Conversation Logs**: Kept indefinitely (needed for resume)

**Worktrees**: Cleaned up when:
- Manual cleanup: `murmur worktree clean`
- LRU eviction: Cache exceeds configured size
- Stale detection: Directory missing or agent stopped

**Issues**: Updated when fetched from GitHub, never auto-deleted

### Privacy Considerations

The database stores:
- Prompts given to agents (may contain sensitive context)
- Full conversation logs (includes code and comments)
- File paths and worktree locations

**Recommendation**: Keep `~/.murmur/` private, don't commit to version control.

## Best Practices

### When to Use Which Agent Type

- **Quick fixes**: Use `murmur run` with Implement agent
- **New features**: Use `murmur tdd` for full TDD cycle
- **Code review**: Use Review agent on existing changes
- **Large refactors**: Use Coordinator to manage multi-phase work

### Worktree Management

- Let Murmuration manage worktrees automatically
- Use `murmur worktree list` to see active worktrees
- Clean up completed work: `murmur worktree clean --completed`
- Keep cache size reasonable (set in config)

### Dependency Hygiene

- Always declare dependencies in issue body
- Update dependency status when PRs merge
- Use GitHub task lists for native tracking
- Check `murmur issue deps` before starting work

### Database Maintenance

- Database grows with conversation logs
- Periodically archive old runs (manual SQL export)
- Vacuum database to reclaim space: `sqlite3 ~/.murmur/state.db "VACUUM"`
- Back up before major schema changes
