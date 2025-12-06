# TDD Workflow Guide

Murmuration implements a rigorous Test-Driven Development (TDD) workflow with enforced red-green validation. This guide explains when to use TDD, how the phases work, and how to run TDD workflows.

## When to Use TDD Workflow

Use the TDD workflow when:

- Implementing new features with clear, testable requirements
- Building library code or APIs with well-defined interfaces
- Working on critical business logic that needs comprehensive test coverage
- Developing components where regression prevention is important

Skip TDD for:

- Exploratory prototypes
- UI/UX work requiring rapid iteration
- Integration with third-party services (hard to test in isolation)
- Documentation or configuration-only changes

## The 7 TDD Phases

The TDD workflow consists of 7 sequential phases:

1. **WriteSpec** - Write specification document
2. **WriteTests** - Write tests based on spec
3. **VerifyRed** - Verify tests fail (red phase)
4. **Implement** - Write minimal code to make tests pass
5. **VerifyGreen** - Verify tests pass (green phase)
6. **Refactor** - Clean up code while keeping tests green
7. **Complete** - TDD cycle finished

### Phase Flow Diagram

```
┌──────────────┐
│  WriteSpec   │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│  WriteTests  │◄──┐
└──────┬───────┘   │
       │           │ (retry if tests pass)
       ▼           │
┌──────────────┐   │
│  VerifyRed   │───┘
└──────┬───────┘
       │
       ▼
┌──────────────┐
│  Implement   │◄──┐
└──────┬───────┘   │
       │           │ (retry if tests fail)
       ▼           │
┌──────────────┐   │
│ VerifyGreen  │───┘
└──────┬───────┘
       │
       ▼
┌──────────────┐
│   Refactor   │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│   Complete   │
└──────────────┘
```

## Phase-by-Phase Explanation

### Phase 1: WriteSpec

**Purpose:** Document the expected behavior before writing any code.

**What happens:**
- An Implement agent writes a specification document
- The spec describes inputs, outputs, edge cases, and error conditions
- The spec provides enough detail to write tests

**Agent prompt:**
```
Write a specification document for the following behavior:

<behavior description>

The specification should:
- Clearly describe the expected behavior
- Define inputs and outputs
- List edge cases and error conditions
- Be detailed enough to write tests from
```

**Output:** A specification file (e.g., `SPEC.md` or comments in code)

**Skip this phase:** Use `--skip-spec` if requirements are already clear

### Phase 2: WriteTests

**Purpose:** Write tests that define success criteria.

**What happens:**
- A Test agent writes test cases based on the specification
- Tests cover main functionality, edge cases, and error conditions
- Tests should NOT include any implementation code

**Agent prompt:**
```
Write tests for the following behavior:

<behavior description>

Refer to the specification in: <spec file>

The tests should:
- Cover the main functionality
- Include edge cases
- Be clear and readable
- NOT include any implementation code
```

**Output:** Test files in the appropriate framework (cargo test, pytest, jest, etc.)

**Test framework detection:** Murmuration automatically detects the test framework based on project files

### Phase 3: VerifyRed

**Purpose:** Ensure tests actually test the new behavior.

**What happens:**
- Murmuration runs the test suite
- Tests MUST fail (because the behavior isn't implemented yet)
- If tests pass, they might not be testing the right thing

**Validation:**
```bash
Running tests (expecting failures)...

Test results: 0 passed, 5 failed, 0 skipped

✅ Tests failed as expected (red phase)
```

**If tests pass unexpectedly:**
```bash
⚠️ Tests passed unexpectedly - tests may not be testing new behavior

Going back to WriteTests phase...
```

The workflow returns to WriteTests to revise the tests.

### Phase 4: Implement

**Purpose:** Write minimal code to make tests pass.

**What happens:**
- An Implement agent writes the simplest code that makes tests pass
- Focus is on making tests green, not perfect code
- No extra features or premature optimization

**Agent prompt:**
```
Implement the MINIMAL code to make the tests pass:

<behavior description>

Guidelines:
- Write only enough code to make the tests pass
- Do not add extra features or optimizations
- Do not refactor yet - that comes later
- Focus on making tests green, not on perfect code
```

**Output:** Implementation code

**Iteration tracking:** Each attempt is counted toward max iterations (default: 3)

### Phase 5: VerifyGreen

**Purpose:** Ensure the implementation makes all tests pass.

**What happens:**
- Murmuration runs the test suite
- All tests MUST pass
- If tests fail, return to Implement phase

**Validation:**
```bash
Running tests (iteration 1/3)...

Test results: 5 passed, 0 failed, 0 skipped

✅ All tests pass (green phase)
```

**If tests still fail:**
```bash
❌ 2 tests still failing

Returning to Implement phase...
```

**Max iterations:**
- Default: 3 attempts to make tests pass
- Configurable with `--max-iterations`
- If exceeded, workflow gives up and reports failure

### Phase 6: Refactor

**Purpose:** Improve code quality while maintaining green tests.

**What happens:**
- An Implement agent reviews and refactors the code
- Remove duplication, improve naming, simplify logic
- Tests are run after each change to ensure they stay green

**Agent prompt:**
```
Refactor the code while keeping tests green:

<behavior description>

Review and improve the code:
- Remove duplication
- Improve naming
- Simplify complex logic
- Ensure code follows project conventions

Run tests after each change to ensure they stay green.
```

**Output:** Refactored implementation

**Skip this phase:** Use `--skip-refactor` to go straight to Complete

### Phase 7: Complete

**Purpose:** Mark the TDD cycle as finished.

**What happens:**
- Workflow completes successfully
- Code is ready for review and PR

**Final output:**
```
═══════════════════════════════════════
✅ TDD workflow completed successfully!
═══════════════════════════════════════
```

## Handling Iteration Loops

### WriteTests → VerifyRed Loop

Tests might pass unexpectedly in VerifyRed if:
- Tests don't actually test the new behavior
- Implementation already exists elsewhere
- Test setup is incorrect

**Action:** Workflow returns to WriteTests to revise tests

### Implement → VerifyGreen Loop

Tests might fail in VerifyGreen if:
- Implementation is incomplete
- Implementation has bugs
- Tests revealed edge cases not handled

**Action:** Workflow returns to Implement for another attempt

**Max iterations example:**
```bash
# Iteration 1
Implement → VerifyGreen (2 tests failing)

# Iteration 2
Implement → VerifyGreen (1 test failing)

# Iteration 3
Implement → VerifyGreen (0 tests failing) ✅

# Success! Proceed to Refactor
```

**Exceeding max iterations:**
```bash
# Iteration 3
Implement → VerifyGreen (1 test still failing)

🛑 Maximum iterations reached, giving up

═══════════════════════════════════════
💥 TDD workflow failed after 3 iterations
═══════════════════════════════════════
```

## Running TDD Workflow

### Basic Usage

```bash
murmur tdd "Add user authentication"
```

This runs the full 7-phase TDD cycle in the current directory.

### Skip Spec Phase

Start directly from WriteTests:

```bash
murmur tdd "Add login endpoint" --skip-spec
```

Use when:
- Requirements are already well-documented
- Specification exists elsewhere
- Working on a simple, self-evident feature

### Skip Refactor Phase

Go straight to Complete after tests pass:

```bash
murmur tdd "Fix validation bug" --skip-refactor
```

Use when:
- Code is already clean
- Minimal implementation needed
- Time constraints

### Custom Working Directory

```bash
murmur tdd "Parse config file" -d ./src/config
```

### Maximum Iterations

```bash
murmur tdd "Complex algorithm" --max-iterations 5
```

Increase for:
- Complex features
- Experimental implementations
- Learning scenarios

Decrease for:
- Simple features
- Strict deadlines
- CI/CD pipelines

### Dry Run

Preview the workflow without executing:

```bash
murmur tdd "Add caching" --dry-run
```

Output:
```
[Dry run] Would execute TDD workflow with the following phases:

  1. 📝 Writing specification document (implement agent)
  2. 🧪 Writing tests based on spec (test agent)
  3. 🔴 Verifying tests fail (test runner)
  4. 🔨 Implementing to make tests pass (implement agent)
  5. 🟢 Verifying tests pass (test runner)
  6. ✨ Refactoring while keeping tests green (implement agent)
  7. 🎉 TDD cycle complete (n/a)

Current prompt for first phase:
---
Write a specification document for the following behavior:

Add caching
...
---
```

## Example TDD Session

### Full Session Output

```bash
$ murmur tdd "Implement fibonacci function" --skip-spec

TDD Workflow
============

Behavior: Implement fibonacci function
Working directory: /Users/dev/project
Model: claude-sonnet-4-5-20250929

Detected test framework: cargo test

Phase 1/6: 🧪 Writing tests based on spec

Starting agent...

[Agent writes tests for fibonacci function]

✅ Phase completed

Phase 2/6: 🔴 Verifying tests fail (red phase)

Running tests (expecting failures)...

Test results: 0 passed, 4 failed, 0 skipped

✅ Tests failed as expected (red phase)

Phase 3/6: 🔨 Implementing to make tests pass

Starting agent...

[Agent implements fibonacci function]

✅ Phase completed

Phase 4/6: 🟢 Verifying tests pass (green phase)

Running tests (iteration 1/3)...

Test results: 4 passed, 0 failed, 0 skipped

✅ All tests pass (green phase)

Phase 5/6: ✨ Refactoring while keeping tests green

Starting agent...

[Agent refactors implementation]

✅ Phase completed

Phase 6/6: 🎉 TDD cycle complete

═══════════════════════════════════════
✅ TDD workflow completed successfully!
═══════════════════════════════════════
```

### Session with Iteration

```bash
Phase 4/6: 🟢 Verifying tests pass (green phase)

Running tests (iteration 1/3)...

Test results: 3 passed, 1 failed, 0 skipped

❌ 1 tests still failing

Returning to Implement phase...

Phase 3/6: 🔨 Implementing to make tests pass

Starting agent...

[Agent fixes failing test]

✅ Phase completed

Phase 4/6: 🟢 Verifying tests pass (green phase)

Running tests (iteration 2/3)...

Test results: 4 passed, 0 failed, 0 skipped

✅ All tests pass (green phase)
```

## Test Framework Support

Murmuration automatically detects common test frameworks:

### Rust (Cargo)

Detection: `Cargo.toml` file
Command: `cargo test`

### Python (pytest)

Detection: `pytest.ini`, `pyproject.toml`, or `tests/` directory
Command: `pytest`

### JavaScript/TypeScript (Jest)

Detection: `package.json` with jest dependency
Command: `npm test` or `yarn test`

### Go

Detection: `go.mod` file
Command: `go test ./...`

## Best Practices

### 1. Write Failing Tests First

Never skip VerifyRed. It proves your tests actually test something:

```bash
# Good: Tests fail in VerifyRed
🔴 Tests failed as expected

# Bad: Tests pass in VerifyRed
⚠️ Tests passed unexpectedly
```

### 2. Keep Implementation Minimal

In the Implement phase, resist the urge to:
- Add features not covered by tests
- Optimize prematurely
- Refactor (that's phase 6!)

### 3. Refactor Confidently

With green tests, you can refactor safely:
- Tests act as a safety net
- Run tests frequently during refactoring
- Stop if tests break and investigate

### 4. Use Appropriate Iteration Limits

```bash
# Simple feature: 1-2 iterations
--max-iterations 2

# Medium complexity: 3 iterations (default)
# (no flag needed)

# Complex feature: 4-5 iterations
--max-iterations 5
```

### 5. Document Non-Obvious Behavior

Enhance the behavior description:

```bash
# Too vague
murmur tdd "Add validation"

# Better
murmur tdd "Add email validation that checks format and blocks disposable domains"
```

## Troubleshooting

### Tests pass in VerifyRed

**Problem:** Tests don't fail even though behavior isn't implemented.

**Possible causes:**
- Tests are too lenient (e.g., always checking `true == true`)
- Implementation already exists elsewhere
- Tests are testing the wrong thing

**Solution:** Review test output, revise tests in WriteTests phase

### Tests never pass after max iterations

**Problem:** Exceeded `--max-iterations` in Implement → VerifyGreen loop.

**Possible causes:**
- Feature is more complex than anticipated
- Tests have conflicting requirements
- Implementation approach is flawed

**Solutions:**
1. Increase `--max-iterations`
2. Review failing tests manually
3. Simplify the behavior into smaller features
4. Run with `--skip-refactor` to focus on core implementation

### Agent times out or hangs

**Problem:** Agent doesn't complete a phase.

**Solutions:**
1. Check for syntax errors in generated code
2. Verify test framework is properly configured
3. Reduce scope of the behavior
4. Check agent logs with `-v` flag

### Wrong test framework detected

**Problem:** Murmuration uses the wrong test command.

**Solution:** Ensure project files are properly configured (e.g., `Cargo.toml` for Rust, `package.json` for JavaScript).

## Advanced Usage

### Integrate with `murmur work`

Combine TDD with issue workflow:

```bash
# Work on an issue using TDD
murmur work 42

# Then manually run TDD in the worktree:
cd ~/.cache/murmur/worktrees/owner-repo/issue-42
murmur tdd "$(cat .murmur/issue-description.txt)" --skip-spec
```

(Future versions may integrate TDD directly into `murmur work`)

### Custom Agent Configuration

Configure agent behavior in `~/.config/murmur/config.toml`:

```toml
[agent]
model = "claude-sonnet-4-5-20250929"
timeout = 300  # seconds
```

### Verbose Output

See detailed agent activity:

```bash
murmur tdd "Add feature" -v
```

Shows:
- Agent prompts
- Tool usage
- Test output details
