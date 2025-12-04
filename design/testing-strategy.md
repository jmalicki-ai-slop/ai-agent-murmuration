# Testing Strategy

## Overview

Comprehensive testing approach for the Dispatch system including unit tests, integration tests, property-based tests (proptests), and end-to-end tests with network API mocking.

---

## Testing Pyramid

```
                    ┌─────────┐
                    │   E2E   │  Few, slow, high confidence
                    │  Tests  │
                    ├─────────┤
                   ╱           ╲
                  ╱ Integration ╲  More, medium speed
                 ╱    Tests      ╲
                ├─────────────────┤
               ╱                   ╲
              ╱    Property Tests   ╲  Exhaustive edge cases
             ╱      (Proptests)      ╲
            ├─────────────────────────┤
           ╱                           ╲
          ╱        Unit Tests           ╲  Many, fast, isolated
         ╱                               ╲
        └─────────────────────────────────┘
```

---

## Test Organization

```
dispatch/
├── crates/
│   ├── dispatch-core/
│   │   ├── src/
│   │   │   └── types/
│   │   │       └── issue.rs         # Inline unit tests (#[cfg(test)])
│   │   └── tests/
│   │       ├── issue_props.rs       # Property tests
│   │       └── state_machine.rs     # State transition tests
│   │
│   ├── dispatch-db/
│   │   └── tests/
│   │       ├── repos/               # Repository integration tests
│   │       └── migrations.rs        # Migration tests
│   │
│   ├── dispatch-github/
│   │   └── tests/
│   │       ├── api_mock.rs          # GitHub API mocking
│   │       ├── sync_test.rs         # Sync integration tests
│   │       └── webhook_test.rs      # Webhook handler tests
│   │
│   ├── dispatch-agents/
│   │   └── tests/
│   │       └── lifecycle_test.rs    # Agent lifecycle tests
│   │
│   └── dispatch-governance/
│       └── tests/
│           ├── consensus_props.rs   # Consensus property tests
│           ├── voting_test.rs       # Voting integration tests
│           └── workflow_test.rs     # TDD workflow tests
│
└── tests/                           # End-to-end tests
    ├── e2e/
    │   ├── issue_workflow.rs
    │   ├── epic_workflow.rs
    │   └── full_cycle.rs
    └── fixtures/
        ├── github_responses/        # Mock API response fixtures
        └── test_repos/              # Test git repositories
```

---

## Unit Tests

### Inline Module Tests

```rust
// dispatch-core/src/types/issue.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_issue_creation() {
        let issue = Issue::new(
            PathBuf::from("/repo"),
            "Test issue".into(),
            "Description".into(),
            IssueType::Feature,
        );

        assert_eq!(issue.status, IssueStatus::Unassigned);
        assert_eq!(issue.priority, Priority::Medium);
        assert!(issue.assigned_agent_id.is_none());
    }

    #[test]
    fn test_valid_state_transitions() {
        let mut issue = Issue::new(
            PathBuf::from("/repo"),
            "Test".into(),
            "Desc".into(),
            IssueType::Feature,
        );

        // Unassigned -> Queued is valid
        assert!(issue.transition_to(IssueStatus::Queued).is_ok());
        assert_eq!(issue.status, IssueStatus::Queued);

        // Queued -> Assigned is valid
        assert!(issue.transition_to(IssueStatus::Assigned).is_ok());
        assert_eq!(issue.status, IssueStatus::Assigned);
    }

    #[test]
    fn test_invalid_state_transitions() {
        let mut issue = Issue::new(
            PathBuf::from("/repo"),
            "Test".into(),
            "Desc".into(),
            IssueType::Feature,
        );

        // Unassigned -> Done is invalid
        let result = issue.transition_to(IssueStatus::Done);
        assert!(matches!(result, Err(DispatchError::InvalidStateTransition { .. })));
        assert_eq!(issue.status, IssueStatus::Unassigned); // Unchanged
    }

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::Critical < Priority::High);
        assert!(Priority::High < Priority::Medium);
        assert!(Priority::Medium < Priority::Low);

        let mut priorities = vec![Priority::Low, Priority::Critical, Priority::Medium, Priority::High];
        priorities.sort();
        assert_eq!(priorities, vec![Priority::Critical, Priority::High, Priority::Medium, Priority::Low]);
    }
}
```

---

## Property-Based Testing (Proptests)

Use `proptest` for exhaustive testing of compute functions, state machines, and algorithms.

### Consensus Calculator Properties

```rust
// dispatch-governance/tests/consensus_props.rs

use proptest::prelude::*;
use dispatch_core::types::proposal::*;
use dispatch_core::types::vote::*;
use dispatch_governance::consensus::*;

// Strategy for generating random votes
fn vote_decision_strategy() -> impl Strategy<Value = VoteDecision> {
    prop_oneof![
        Just(VoteDecision::Approve),
        Just(VoteDecision::Reject),
        Just(VoteDecision::Abstain),
        Just(VoteDecision::NeedMoreInfo),
    ]
}

fn agent_type_strategy() -> impl Strategy<Value = AgentType> {
    prop_oneof![
        Just(AgentType::Coder),
        Just(AgentType::Reviewer),
        Just(AgentType::Architect),
        Just(AgentType::Security),
        Just(AgentType::Test),
        Just(AgentType::Pm),
        Just(AgentType::Docs),
    ]
}

fn vote_strategy() -> impl Strategy<Value = Vote> {
    (
        any::<u128>().prop_map(|n| VoteId::from_uuid(Uuid::from_u128(n))),
        any::<u128>().prop_map(|n| ProposalId::from_uuid(Uuid::from_u128(n))),
        any::<u128>().prop_map(|n| AgentId::from_uuid(Uuid::from_u128(n))),
        agent_type_strategy(),
        vote_decision_strategy(),
        ".*".prop_map(String::from),
        0.0..=1.0f64,
    ).prop_map(|(id, proposal_id, voter_id, voter_type, decision, reasoning, confidence)| {
        Vote {
            id,
            proposal_id,
            voter_id,
            voter_type,
            decision,
            reasoning,
            confidence,
            preferred_option: None,
            option_ranking: None,
            created_at: Utc::now(),
        }
    })
}

proptest! {
    /// Unanimous threshold requires zero rejections
    #[test]
    fn unanimous_requires_no_rejections(votes in prop::collection::vec(vote_strategy(), 1..10)) {
        let proposal = create_test_proposal(ConsensusThreshold::Unanimous);
        let result = ConsensusCalculator::calculate(&proposal, &votes);

        let has_rejection = votes.iter().any(|v| v.decision == VoteDecision::Reject);

        if let ConsensusResult::Approved { .. } = result {
            // If approved, there should be no rejections
            prop_assert!(!has_rejection, "Unanimous approval with rejections");
        }
    }

    /// Simple majority requires >50% approval
    #[test]
    fn simple_majority_over_50_percent(votes in prop::collection::vec(vote_strategy(), 1..20)) {
        let proposal = create_test_proposal(ConsensusThreshold::SimpleMajority);
        let result = ConsensusCalculator::calculate(&proposal, &votes);

        let voting_votes: Vec<_> = votes.iter()
            .filter(|v| v.decision != VoteDecision::Abstain && v.decision != VoteDecision::NeedMoreInfo)
            .collect();

        if voting_votes.is_empty() {
            return Ok(()); // Skip - no valid votes
        }

        let approvals = voting_votes.iter().filter(|v| v.decision == VoteDecision::Approve).count();
        let ratio = approvals as f64 / voting_votes.len() as f64;

        match result {
            ConsensusResult::Approved { approval_ratio, .. } => {
                prop_assert!(ratio > 0.5, "Approved with {}% < 50%", ratio * 100.0);
                prop_assert!((approval_ratio - ratio).abs() < 0.01, "Ratio mismatch");
            }
            ConsensusResult::Rejected { approval_ratio } => {
                prop_assert!(ratio <= 0.5, "Rejected with {}% > 50%", ratio * 100.0);
                prop_assert!((approval_ratio - ratio).abs() < 0.01, "Ratio mismatch");
            }
            _ => {} // Other states are valid
        }
    }

    /// Approval ratio is always between 0 and 1
    #[test]
    fn approval_ratio_bounded(votes in prop::collection::vec(vote_strategy(), 0..50)) {
        let proposal = create_test_proposal(ConsensusThreshold::SimpleMajority);
        let result = ConsensusCalculator::calculate(&proposal, &votes);

        match result {
            ConsensusResult::Approved { approval_ratio, .. } |
            ConsensusResult::Rejected { approval_ratio } => {
                prop_assert!(approval_ratio >= 0.0 && approval_ratio <= 1.0,
                    "Approval ratio {} out of bounds", approval_ratio);
            }
            _ => {}
        }
    }

    /// NeedMoreInfo always results in NeedsMoreInfo state
    #[test]
    fn need_more_info_halts_voting(
        base_votes in prop::collection::vec(vote_strategy(), 0..5),
        need_info_vote in vote_strategy()
    ) {
        let proposal = create_test_proposal(ConsensusThreshold::SimpleMajority);

        // Create a vote that needs more info
        let mut info_vote = need_info_vote;
        info_vote.decision = VoteDecision::NeedMoreInfo;

        let mut all_votes = base_votes;
        all_votes.push(info_vote);

        let result = ConsensusCalculator::calculate(&proposal, &all_votes);

        prop_assert!(
            matches!(result, ConsensusResult::NeedsMoreInfo { .. }),
            "Expected NeedsMoreInfo, got {:?}", result
        );
    }
}

fn create_test_proposal(threshold: ConsensusThreshold) -> Proposal {
    Proposal {
        id: ProposalId::new(),
        proposal_type: ProposalType::ImplementationApproach,
        proposer_id: AgentId::new(),
        title: "Test".into(),
        description: "Test".into(),
        rationale: "Test".into(),
        related_issue_id: None,
        affected_components: vec![],
        options: None,
        chosen_option: None,
        status: ProposalStatus::Voting,
        required_voters: vec![AgentType::Coder, AgentType::Reviewer],
        threshold,
        voting_deadline: None,
        implementation_plan: None,
        rollback_plan: None,
        execution_result: None,
        forced_by: None,
        force_reason: None,
        vetoed_by: None,
        veto_reason: None,
        created_at: Utc::now(),
        resolved_at: None,
        executed_at: None,
    }
}
```

### State Machine Properties

```rust
// dispatch-core/tests/state_machine_props.rs

use proptest::prelude::*;

prop_compose! {
    fn issue_status_strategy()(
        idx in 0..9usize
    ) -> IssueStatus {
        [
            IssueStatus::Unassigned,
            IssueStatus::Queued,
            IssueStatus::Assigned,
            IssueStatus::InProgress,
            IssueStatus::AwaitingReview,
            IssueStatus::InReview,
            IssueStatus::Done,
            IssueStatus::Blocked,
            IssueStatus::Cancelled,
        ][idx]
    }
}

proptest! {
    /// Terminal states have no valid transitions
    #[test]
    fn terminal_states_are_final(target in issue_status_strategy()) {
        let terminal_states = [IssueStatus::Done, IssueStatus::Cancelled];

        for terminal in &terminal_states {
            let transitions = terminal.valid_transitions();
            prop_assert!(
                transitions.is_empty(),
                "{:?} should have no transitions but has {:?}",
                terminal, transitions
            );
        }
    }

    /// can_transition_to is consistent with valid_transitions
    #[test]
    fn transition_methods_consistent(
        from in issue_status_strategy(),
        to in issue_status_strategy()
    ) {
        let valid = from.valid_transitions();
        let can = from.can_transition_to(to);

        prop_assert_eq!(
            valid.contains(&to),
            can,
            "Inconsistency: {:?}.valid_transitions()={:?}, can_transition_to({:?})={}",
            from, valid, to, can
        );
    }

    /// All non-terminal states have at least one valid transition
    #[test]
    fn non_terminal_have_transitions(status in issue_status_strategy()) {
        let terminal = [IssueStatus::Done, IssueStatus::Cancelled];

        if !terminal.contains(&status) {
            prop_assert!(
                !status.valid_transitions().is_empty(),
                "{:?} is non-terminal but has no transitions", status
            );
        }
    }
}
```

### Branch Name Generation Properties

```rust
// dispatch-git/tests/branch_props.rs

use proptest::prelude::*;

proptest! {
    /// Branch names are always valid git refs
    #[test]
    fn branch_names_are_valid_git_refs(
        issue_id in "[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}",
        title in "[ -~]{1,100}"  // Printable ASCII
    ) {
        let issue_id: IssueId = issue_id.parse().unwrap();
        let branch = WorktreeManager::branch_name(&issue_id, &title);

        // Valid git ref: no spaces, no consecutive dots, no leading/trailing dots
        prop_assert!(!branch.contains(' '), "Branch contains space: {}", branch);
        prop_assert!(!branch.contains(".."), "Branch contains '..': {}", branch);
        prop_assert!(!branch.starts_with('.'), "Branch starts with dot: {}", branch);
        prop_assert!(!branch.ends_with('.'), "Branch ends with dot: {}", branch);
        prop_assert!(!branch.ends_with('/'), "Branch ends with slash: {}", branch);
        prop_assert!(branch.len() <= 255, "Branch too long: {} chars", branch.len());
    }

    /// Branch names are deterministic
    #[test]
    fn branch_names_deterministic(
        issue_id in "[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}",
        title in ".*"
    ) {
        let issue_id: IssueId = issue_id.parse().unwrap();
        let branch1 = WorktreeManager::branch_name(&issue_id, &title);
        let branch2 = WorktreeManager::branch_name(&issue_id, &title);
        prop_assert_eq!(branch1, branch2);
    }
}
```

---

## Network API Mocking

### GitHub API Mock Server

```rust
// dispatch-github/tests/api_mock.rs

use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path, path_regex, header, query_param};

/// Setup a mock GitHub API server
pub async fn setup_github_mock() -> MockServer {
    let mock_server = MockServer::start().await;

    // Default rate limit headers
    let rate_limit_headers = vec![
        ("X-RateLimit-Limit", "5000"),
        ("X-RateLimit-Remaining", "4999"),
        ("X-RateLimit-Reset", "1234567890"),
    ];

    mock_server
}

/// Mock GitHub issue list endpoint
pub async fn mock_list_issues(server: &MockServer, issues: Vec<serde_json::Value>) {
    Mock::given(method("GET"))
        .and(path("/repos/test-owner/test-repo/issues"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(issues)
            .insert_header("X-RateLimit-Remaining", "4999"))
        .mount(server)
        .await;
}

/// Mock GitHub issue get endpoint
pub async fn mock_get_issue(server: &MockServer, number: u64, issue: serde_json::Value) {
    Mock::given(method("GET"))
        .and(path(format!("/repos/test-owner/test-repo/issues/{}", number)))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(issue))
        .mount(server)
        .await;
}

/// Mock GitHub issue create endpoint
pub async fn mock_create_issue(server: &MockServer, response_number: u64) {
    Mock::given(method("POST"))
        .and(path("/repos/test-owner/test-repo/issues"))
        .respond_with(ResponseTemplate::new(201)
            .set_body_json(serde_json::json!({
                "number": response_number,
                "title": "Created Issue",
                "state": "open",
                "html_url": format!("https://github.com/test-owner/test-repo/issues/{}", response_number)
            })))
        .mount(server)
        .await;
}

/// Mock GitHub rate limit exceeded
pub async fn mock_rate_limited(server: &MockServer, reset_at: u64) {
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(403)
            .set_body_json(serde_json::json!({
                "message": "API rate limit exceeded",
                "documentation_url": "https://docs.github.com/rest/overview/resources-in-the-rest-api#rate-limiting"
            }))
            .insert_header("X-RateLimit-Remaining", "0")
            .insert_header("X-RateLimit-Reset", reset_at.to_string()))
        .mount(server)
        .await;
}

/// Mock GitHub 404 not found
pub async fn mock_not_found(server: &MockServer, path_pattern: &str) {
    Mock::given(method("GET"))
        .and(path_regex(path_pattern))
        .respond_with(ResponseTemplate::new(404)
            .set_body_json(serde_json::json!({
                "message": "Not Found",
                "documentation_url": "https://docs.github.com/rest"
            })))
        .mount(server)
        .await;
}

/// Mock GitHub webhook delivery
pub async fn mock_webhook_delivery(server: &MockServer) {
    Mock::given(method("POST"))
        .and(path("/webhook"))
        .and(header("X-GitHub-Event", "issues"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(server)
        .await;
}
```

### GitHub API Integration Tests

```rust
// dispatch-github/tests/sync_test.rs

use crate::api_mock::*;

#[tokio::test]
async fn test_sync_pulls_new_issues() {
    let mock_server = setup_github_mock().await;

    // Setup mock response
    mock_list_issues(&mock_server, vec![
        serde_json::json!({
            "number": 1,
            "title": "Test Issue",
            "body": "Description",
            "state": "open",
            "labels": [],
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z"
        })
    ]).await;

    // Create client pointing to mock server
    let client = GitHubClient::new_with_base_url(
        "test-token",
        "test-owner".into(),
        "test-repo".into(),
        mock_server.uri(),
    ).unwrap();

    // Test sync
    let issues = client.list_issues(None, None, None).await.unwrap();

    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].title, "Test Issue");
}

#[tokio::test]
async fn test_handles_rate_limiting() {
    let mock_server = setup_github_mock().await;
    let reset_time = (Utc::now() + chrono::Duration::hours(1)).timestamp() as u64;

    mock_rate_limited(&mock_server, reset_time).await;

    let client = GitHubClient::new_with_base_url(
        "test-token",
        "test-owner".into(),
        "test-repo".into(),
        mock_server.uri(),
    ).unwrap();

    let result = client.list_issues(None, None, None).await;

    assert!(matches!(result, Err(DispatchError::RateLimited { .. })));
}

#[tokio::test]
async fn test_creates_issue_with_metadata() {
    let mock_server = setup_github_mock().await;

    mock_create_issue(&mock_server, 42).await;

    let client = GitHubClient::new_with_base_url(
        "test-token",
        "test-owner".into(),
        "test-repo".into(),
        mock_server.uri(),
    ).unwrap();

    let issue = Issue::new(
        PathBuf::from("/repo"),
        "New Feature".into(),
        "Implement something".into(),
        IssueType::Feature,
    );

    let github_id = client.create_issue(&issue).await.unwrap();

    assert_eq!(github_id, 42);
}

#[tokio::test]
async fn test_handles_not_found() {
    let mock_server = setup_github_mock().await;

    mock_not_found(&mock_server, r"/repos/.*/issues/\d+").await;

    let client = GitHubClient::new_with_base_url(
        "test-token",
        "test-owner".into(),
        "test-repo".into(),
        mock_server.uri(),
    ).unwrap();

    let result = client.get_issue(999).await;

    assert!(result.unwrap().is_none());
}
```

### Webhook Handler Tests

```rust
// dispatch-github/tests/webhook_test.rs

use axum::http::StatusCode;
use axum_test::TestServer;

#[tokio::test]
async fn test_webhook_validates_signature() {
    let app = create_test_app().await;
    let server = TestServer::new(app).unwrap();

    // Request without signature should fail
    let response = server
        .post("/webhooks/github")
        .add_header("X-GitHub-Event", "issues")
        .json(&serde_json::json!({"action": "opened"}))
        .await;

    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_webhook_handles_issue_opened() {
    let app = create_test_app().await;
    let server = TestServer::new(app).unwrap();

    let payload = serde_json::json!({
        "action": "opened",
        "issue": {
            "number": 1,
            "title": "New Issue",
            "body": "Description",
            "state": "open",
            "labels": [],
            "user": {"login": "testuser"}
        },
        "repository": {
            "full_name": "test-owner/test-repo"
        },
        "sender": {"login": "testuser"}
    });

    let signature = compute_webhook_signature(&payload, "test-secret");

    let response = server
        .post("/webhooks/github")
        .add_header("X-GitHub-Event", "issues")
        .add_header("X-Hub-Signature-256", format!("sha256={}", signature))
        .json(&payload)
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
}
```

---

## Database Integration Tests

```rust
// dispatch-db/tests/repos/issue_test.rs

use sqlx::SqlitePool;

#[sqlx::test(migrations = "../../migrations")]
async fn test_issue_crud(pool: SqlitePool) {
    let repo = IssueRepository::new(pool.clone());

    // Create
    let issue = Issue::new(
        PathBuf::from("/repo"),
        "Test issue".into(),
        "Description".into(),
        IssueType::Feature,
    );
    repo.create(&issue).await.unwrap();

    // Read
    let fetched = repo.get(&issue.id).await.unwrap().unwrap();
    assert_eq!(fetched.title, "Test issue");
    assert_eq!(fetched.status, IssueStatus::Unassigned);

    // Update
    let mut updated = fetched;
    updated.title = "Updated title".into();
    updated.status = IssueStatus::Queued;
    repo.update(&updated).await.unwrap();

    let fetched = repo.get(&issue.id).await.unwrap().unwrap();
    assert_eq!(fetched.title, "Updated title");
    assert_eq!(fetched.status, IssueStatus::Queued);

    // Delete
    repo.delete(&issue.id).await.unwrap();
    assert!(repo.get(&issue.id).await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_issue_queries(pool: SqlitePool) {
    let repo = IssueRepository::new(pool.clone());

    // Create multiple issues
    for i in 0..5 {
        let mut issue = Issue::new(
            PathBuf::from("/repo"),
            format!("Issue {}", i),
            "Desc".into(),
            IssueType::Feature,
        );
        issue.priority = if i < 2 { Priority::High } else { Priority::Medium };
        repo.create(&issue).await.unwrap();
    }

    // Query by status
    let unassigned = repo.list_by_status(IssueStatus::Unassigned).await.unwrap();
    assert_eq!(unassigned.len(), 5);

    // Results should be ordered by priority
    assert_eq!(unassigned[0].priority, Priority::High);
    assert_eq!(unassigned[1].priority, Priority::High);
    assert_eq!(unassigned[2].priority, Priority::Medium);
}
```

---

## End-to-End Tests

```rust
// tests/e2e/issue_workflow.rs

use dispatch_test_helpers::*;

#[tokio::test]
async fn test_full_issue_workflow() {
    let ctx = TestContext::new().await;

    // Create an issue
    let issue_id = ctx.create_issue("Implement feature", "Do something").await;

    // Assign to coder
    let agent_id = ctx.assign_issue(&issue_id, AgentType::Coder).await;

    // Verify agent starts
    ctx.wait_for_agent_status(&agent_id, AgentStatus::Working).await;

    // Simulate agent completing work
    ctx.simulate_agent_completion(&agent_id).await;

    // Verify issue transitions
    let issue = ctx.get_issue(&issue_id).await;
    assert_eq!(issue.status, IssueStatus::AwaitingReview);

    // Cleanup
    ctx.cleanup().await;
}

#[tokio::test]
async fn test_tdd_workflow() {
    let ctx = TestContext::new().await;

    // Create issue with TDD workflow
    let issue_id = ctx.create_issue_with_workflow(
        "Add validation",
        "Add input validation",
        WorkflowType::Tdd,
    ).await;

    // Phase 1: Specification
    ctx.advance_workflow(&issue_id).await;

    // Phase 2: Design Review - simulate approval
    ctx.simulate_review_approval(&issue_id, ReviewType::DesignReview).await;

    // Phase 3: Write Tests (RED)
    let test_agent = ctx.assign_for_phase(&issue_id, TddPhase::WriteTests).await;
    ctx.simulate_agent_completion(&test_agent).await;

    // Verify tests fail (RED phase)
    let workflow = ctx.get_workflow(&issue_id).await;
    assert!(workflow.red_phase_results.as_ref().unwrap().failed > 0);

    // Phase 4: Implementation (GREEN)
    let coder_agent = ctx.assign_for_phase(&issue_id, TddPhase::Implementation).await;
    ctx.simulate_agent_completion(&coder_agent).await;

    // Verify tests pass (GREEN phase)
    let workflow = ctx.get_workflow(&issue_id).await;
    assert_eq!(workflow.green_phase_results.as_ref().unwrap().failed, 0);

    ctx.cleanup().await;
}
```

---

## Test Fixtures

### GitHub Response Fixtures

```json
// tests/fixtures/github_responses/issue_list.json
[
  {
    "number": 1,
    "title": "First Issue",
    "body": "Description for first issue\n\n<!-- dispatch:metadata\n{\"dispatch_id\": \"123\", \"status\": \"in_progress\"}\n-->",
    "state": "open",
    "labels": [{"name": "bug"}, {"name": "priority:high"}],
    "user": {"login": "testuser"},
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-02T00:00:00Z"
  }
]
```

### Test Repository Setup

```rust
// tests/fixtures/mod.rs

pub fn create_test_repo() -> TempDir {
    let temp = TempDir::new().unwrap();
    let repo = Repository::init(temp.path()).unwrap();

    // Create initial commit
    let sig = Signature::now("Test", "test@example.com").unwrap();
    let tree_id = repo.index().unwrap().write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[]).unwrap();

    temp
}
```

---

## CI Configuration

```yaml
# .github/workflows/ci.yml

name: CI

on:
  push:
    branches: [main]
  pull_request:

env:
  CARGO_TERM_COLOR: always
  RUSTFLAGS: -Dwarnings

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable

      - name: Cache
        uses: Swatinem/rust-cache@v2

      - name: Run unit tests
        run: cargo test --lib --all-features

      - name: Run integration tests
        run: cargo test --test '*' --all-features

      - name: Run property tests
        run: cargo test --test '*_props' --all-features
        env:
          PROPTEST_CASES: 1000

  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable
        with:
          components: llvm-tools-preview

      - name: Install cargo-llvm-cov
        uses: taiki-e/install-action@cargo-llvm-cov

      - name: Generate coverage
        run: cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info

      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: lcov.info
```

---

## Test Dependencies

```toml
# Cargo.toml [workspace.dev-dependencies]

[workspace.dev-dependencies]
# Testing framework
proptest = "1"
test-case = "3"

# Async testing
tokio-test = "0.4"

# HTTP mocking
wiremock = "0.6"

# Database testing
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite", "migrate"] }

# Temporary directories
tempfile = "3"

# Assertions
pretty_assertions = "1"

# Axum testing
axum-test = "14"
```

---

## Implementation PRs

| PR | Description | Files |
|----|-------------|-------|
| PR-002a | CI test setup | `.github/workflows/ci.yml` |
| PR-004b | Core type unit tests | `dispatch-core/src/types/*_test.rs` |
| PR-005a | Database integration tests | `dispatch-db/tests/*.rs` |
| PR-037a | GitHub API mock infrastructure | `dispatch-github/tests/api_mock.rs` |
| PR-047a | Consensus property tests | `dispatch-governance/tests/consensus_props.rs` |
| PR-047b | State machine property tests | `dispatch-core/tests/state_machine_props.rs` |
| PR-086a | E2E test infrastructure | `tests/e2e/*.rs`, `tests/fixtures/*` |
