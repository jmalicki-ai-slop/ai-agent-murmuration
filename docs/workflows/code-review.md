# Code Review Workflow

Murmuration supports code review through both specialized review agents and integration with external review tools. This guide covers the review workflow, configuring review automation, and best practices.

## Review Agent Types

Murmuration includes specialized agents for different review purposes:

### Reviewer Agent

Focused on code quality and correctness:
- Checks for bugs and logic errors
- Reviews code style and conventions
- Verifies tests are comprehensive
- Ensures documentation is adequate

### Test Agent (in Review Mode)

Reviews test quality specifically:
- Validates test coverage
- Checks for missing edge cases
- Reviews test clarity and maintainability
- Ensures tests follow best practices

### Security Agent (Future)

Specialized security review:
- Identifies security vulnerabilities
- Checks for common attack vectors
- Reviews authentication and authorization
- Validates input sanitization

## TDD Phase Gates with Reviews

In the TDD workflow, reviews act as gates between phases:

```
┌──────────────┐
│  WriteSpec   │
└──────┬───────┘
       │
       ▼
    [Spec Review] ←─── Review agent checks specification
       │
       ▼
┌──────────────┐
│  WriteTests  │
└──────┬───────┘
       │
       ▼
    [Test Review] ←─── Review agent validates tests
       │
       ▼
┌──────────────┐
│  VerifyRed   │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│  Implement   │
└──────┬───────┘
       │
       ▼
    [Code Review] ←─── Review agent examines implementation
       │
       ▼
┌──────────────┐
│ VerifyGreen  │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│   Refactor   │
└──────┬───────┘
       │
       ▼
   [Final Review] ←─── Review agent checks refactored code
       │
       ▼
┌──────────────┐
│   Complete   │
└──────────────┘
```

## Review Workflow Phases

### 1. Spec Review (Before WriteTests)

**Purpose:** Validate specification quality before writing tests.

**Review checks:**
- Is the specification clear and unambiguous?
- Are inputs and outputs well-defined?
- Are edge cases identified?
- Are error conditions documented?

**Review request:**
```json
{
  "phase": "spec_review",
  "spec_file": "SPEC.md",
  "behavior": "Add user authentication",
  "request_type": "approve_or_revise"
}
```

**Possible outcomes:**
- **Approved:** Proceed to WriteTests
- **Revise:** Provide feedback, return to WriteSpec

### 2. Test Review (Before VerifyRed)

**Purpose:** Ensure tests are comprehensive and well-written.

**Review checks:**
- Do tests cover the main functionality?
- Are edge cases tested?
- Are error conditions tested?
- Are tests clear and maintainable?
- Is there redundancy or missing coverage?

**Review request:**
```json
{
  "phase": "test_review",
  "test_files": ["tests/auth_test.rs"],
  "spec_file": "SPEC.md",
  "behavior": "Add user authentication",
  "request_type": "approve_or_revise"
}
```

**Possible outcomes:**
- **Approved:** Proceed to VerifyRed
- **Revise:** Provide feedback, return to WriteTests

### 3. Code Review (Before VerifyGreen)

**Purpose:** Check implementation quality before validation.

**Review checks:**
- Is the implementation correct?
- Does it follow project conventions?
- Are there obvious bugs or issues?
- Is error handling appropriate?
- Is the code readable?

**Review request:**
```json
{
  "phase": "code_review",
  "impl_files": ["src/auth.rs"],
  "test_files": ["tests/auth_test.rs"],
  "behavior": "Add user authentication",
  "request_type": "approve_or_revise"
}
```

**Possible outcomes:**
- **Approved:** Proceed to VerifyGreen
- **Revise:** Provide feedback, return to Implement

### 4. Final Review (After Refactor)

**Purpose:** Ensure refactored code meets quality standards.

**Review checks:**
- Is duplication removed?
- Are names clear and consistent?
- Is the code maintainable?
- Does it follow project patterns?
- Is documentation complete?

**Review request:**
```json
{
  "phase": "final_review",
  "impl_files": ["src/auth.rs"],
  "test_files": ["tests/auth_test.rs"],
  "behavior": "Add user authentication",
  "request_type": "approve_or_complete"
}
```

**Possible outcomes:**
- **Approved:** Mark as Complete
- **Revise:** Provide feedback, return to Refactor

## Review Feedback Loop

When a review agent requests changes:

### Feedback Structure

```json
{
  "approved": false,
  "issues": [
    {
      "severity": "high",
      "category": "correctness",
      "file": "src/auth.rs",
      "line": 42,
      "description": "Potential null pointer dereference",
      "suggestion": "Add null check before accessing user.email"
    },
    {
      "severity": "low",
      "category": "style",
      "file": "src/auth.rs",
      "line": 15,
      "description": "Variable name 'x' is not descriptive",
      "suggestion": "Rename to 'user_id' for clarity"
    }
  ],
  "summary": "Found 1 high-severity and 1 low-severity issue. Address the null pointer issue before proceeding."
}
```

### Routing Feedback to Coder

The coordinator agent:
1. Receives review feedback
2. Constructs a prompt for the coder agent
3. Provides context about what needs fixing
4. Spawns coder agent with the feedback

**Example feedback prompt:**
```
Code review identified issues that need to be addressed:

HIGH SEVERITY:
- File: src/auth.rs, Line 42
  Issue: Potential null pointer dereference
  Suggestion: Add null check before accessing user.email

LOW SEVERITY:
- File: src/auth.rs, Line 15
  Issue: Variable name 'x' is not descriptive
  Suggestion: Rename to 'user_id' for clarity

Please address these issues and resubmit for review.
Focus on the high-severity issues first.
```

### Iteration Tracking

The workflow tracks review iterations:
- Each review attempt is counted
- Maximum iterations configurable (default: 3)
- After max iterations, escalate to human

**Example:**
```
Code review iteration 1/3: 2 issues found, revising...
Code review iteration 2/3: 1 issue found, revising...
Code review iteration 3/3: 0 issues found, approved!
```

## Configuring Review Workflow

### Enable Review Gates

In `~/.config/murmur/config.toml`:

```toml
[workflow]
# Enable review at each TDD phase gate
enable_reviews = true

# Maximum review iterations before human escalation
max_review_iterations = 3

# Review agent configuration
[workflow.review]
# Require approval for spec before writing tests
review_spec = true

# Require approval for tests before VerifyRed
review_tests = true

# Require approval for implementation before VerifyGreen
review_code = true

# Require final approval after Refactor
review_final = true
```

### Review Agent Model

Use a different model for reviews:

```toml
[agent.review]
model = "claude-sonnet-4-5-20250929"
temperature = 0.1  # Lower temperature for more consistent reviews
```

## Using the Review Agent

### Standalone Review

Review code outside of the TDD workflow:

```bash
murmur review src/auth.rs
```

Output:
```
Reviewing: src/auth.rs

Issues found:

HIGH SEVERITY:
  Line 42: Potential null pointer dereference
  Suggestion: Add null check before accessing user.email

MEDIUM SEVERITY:
  Line 87: Error not propagated properly
  Suggestion: Use ? operator instead of unwrap()

LOW SEVERITY:
  Line 15: Variable name 'x' is not descriptive
  Suggestion: Rename to 'user_id'

Summary: 3 issues found (1 high, 1 medium, 1 low)
Recommendation: Address high and medium severity issues before merging.
```

### Review a Pull Request

Review all changes in a PR:

```bash
murmur review --pr 105
```

This:
1. Fetches PR diff from GitHub
2. Runs review agent on changed files
3. Posts review comments on the PR

### Review Specific Files

```bash
murmur review src/auth.rs tests/auth_test.rs
```

### Review with Custom Prompt

```bash
murmur review src/auth.rs --focus "Check for security vulnerabilities"
```

## Integration with External Review Tools

### GitHub Actions Integration

Add a review step to your CI/CD:

```yaml
# .github/workflows/review.yml
name: Code Review

on: [pull_request]

jobs:
  murmur-review:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2

      - name: Install murmur
        run: cargo install --path .

      - name: Run review agent
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: |
          murmur review --pr ${{ github.event.pull_request.number }} \
                        --post-comments
```

### GitLab CI Integration

```yaml
# .gitlab-ci.yml
review:
  stage: review
  script:
    - cargo install --path .
    - murmur review --mr $CI_MERGE_REQUEST_IID --post-comments
  only:
    - merge_requests
```

### Pre-commit Hook

Review changes before committing:

```bash
# .git/hooks/pre-commit
#!/bin/bash

# Get staged files
files=$(git diff --cached --name-only --diff-filter=ACM | grep '\.rs$')

if [ -n "$files" ]; then
    echo "Running review agent on staged files..."
    murmur review $files --strict

    if [ $? -ne 0 ]; then
        echo "Review found issues. Commit blocked."
        echo "Use git commit --no-verify to bypass."
        exit 1
    fi
fi
```

## Review Agent Prompts

### Reviewer Agent Prompt Structure

```markdown
You are a code reviewer. Review the following code for:

1. Correctness: Does the code do what it's supposed to do?
2. Style: Does it follow project conventions?
3. Maintainability: Is it readable and well-organized?
4. Performance: Are there obvious inefficiencies?
5. Security: Are there potential vulnerabilities?

Code to review:
<code>
...
</code>

Provide feedback in this format:
- Severity: [high|medium|low]
- Category: [correctness|style|maintainability|performance|security]
- Location: File and line number
- Description: What's the issue?
- Suggestion: How to fix it?

Conclude with: APPROVED or REVISE
```

### Customizing Review Criteria

Create custom review prompts in `prompts/reviewer.md`:

```markdown
# Custom Reviewer Prompt

Review code with emphasis on:

## Critical Checks
- [ ] No null pointer dereferences
- [ ] All errors are handled
- [ ] Input validation is present
- [ ] No hardcoded secrets

## Style Checks
- [ ] Variables have descriptive names
- [ ] Functions are < 50 lines
- [ ] Comments explain "why" not "what"
- [ ] Code follows team conventions

## Security Checks
- [ ] No SQL injection vulnerabilities
- [ ] User input is sanitized
- [ ] Authentication is required
- [ ] Authorization is checked
```

## Best Practices

### 1. Enable Reviews for Critical Code

```toml
[workflow.review]
# Always review security-critical code
review_code = true
review_final = true

# Can skip spec/test review for simple changes
review_spec = false
review_tests = false
```

### 2. Use Different Severity Levels

Train review agents to categorize issues:
- **High:** Correctness bugs, security vulnerabilities
- **Medium:** Style violations, maintainability issues
- **Low:** Minor style nitpicks

### 3. Limit Review Iterations

```toml
[workflow]
max_review_iterations = 2  # Escalate to human after 2 attempts
```

Prevents endless review loops.

### 4. Combine Automated and Human Review

- Use review agents for first-pass review
- Escalate to human for:
  - Complex architectural decisions
  - Security-critical code
  - API design choices

### 5. Review Tests Thoroughly

Test review is critical:
- Ensures tests actually test the right thing
- Catches missing edge cases early
- Validates test quality before implementation

### 6. Post-Merge Reviews

Review agent can also review merged code:

```bash
murmur review --commit abc123..def456
```

Use for:
- Post-mortem analysis
- Learning from past issues
- Identifying technical debt

## Auto-Review Loop (Future Feature)

Future versions will support automatic review feedback loops:

```toml
[workflow]
auto_review_loop = true
```

When enabled:
1. Agent completes work and creates PR
2. Review agent automatically reviews PR
3. If issues found, feedback is routed back to coder agent
4. Coder agent addresses feedback and updates PR
5. Loop continues until review passes or max iterations reached

See `PLAN.md` Phase 6 for implementation details.

## Troubleshooting

### Review agent too strict

**Problem:** Review agent rejects valid code.

**Solutions:**
1. Lower severity threshold: `--min-severity medium`
2. Customize review prompt to be more lenient
3. Use `--force-approve` to bypass specific reviews

### Review agent too lenient

**Problem:** Review agent approves problematic code.

**Solutions:**
1. Use a stronger model for reviews
2. Add explicit criteria to review prompt
3. Enable multiple review passes

### Review takes too long

**Problem:** Review agent is slow on large files.

**Solutions:**
1. Review only changed lines: `murmur review --diff-only`
2. Split large files into smaller modules
3. Increase agent timeout in config

### Review comments not posted to PR

**Problem:** `--post-comments` doesn't work.

**Solutions:**
1. Check `GITHUB_TOKEN` has `repo` scope
2. Verify `gh` CLI is authenticated: `gh auth status`
3. Check API rate limits: `gh api rate_limit`

## Summary

Code review workflow in Murmuration:

1. **Phase gates** ensure quality at each TDD step
2. **Review agents** provide automated feedback
3. **Feedback loops** route issues back to coder agents
4. **External integration** works with GitHub, GitLab, CI/CD
5. **Human escalation** for complex issues

This creates a robust quality assurance process while maintaining development velocity.
