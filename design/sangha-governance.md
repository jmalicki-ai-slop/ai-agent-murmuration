# Sangha Governance Design

## Overview

The Sangha is a self-organizing collective of AI agents that can propose, vote on, and implement changes autonomously while remaining under human oversight. This document defines the governance model, voting mechanisms, and human override capabilities.

---

## Philosophy

The Sangha model is inspired by Buddhist monastic communities where decisions are made through collective deliberation. In this system:

1. **Agents are autonomous** - They can propose improvements and vote on proposals
2. **Decisions are collective** - No single agent can unilaterally make significant changes
3. **Humans retain oversight** - Any decision can be forced, vetoed, or overridden
4. **Transparency is key** - All proposals, votes, and decisions are logged and visible

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      Governance Layer                           │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐ │
│  │  Proposal   │  │   Voting    │  │      Execution          │ │
│  │  Manager    │  │   Engine    │  │      Engine             │ │
│  └──────┬──────┘  └──────┬──────┘  └───────────┬─────────────┘ │
│         │                │                     │                │
│         └────────────────┼─────────────────────┘                │
│                          │                                      │
│                          ▼                                      │
│                   ┌─────────────┐                               │
│                   │  Consensus  │                               │
│                   │ Calculator  │                               │
│                   └──────┬──────┘                               │
└──────────────────────────┼──────────────────────────────────────┘
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
        ▼                  ▼                  ▼
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Agents    │    │  Decisions  │    │   Human     │
│  (voters)   │    │    Log      │    │  Override   │
└─────────────┘    └─────────────┘    └─────────────┘
```

---

## Proposal Types

```rust
// dispatch-governance/src/proposals.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalType {
    // Implementation decisions
    ImplementationApproach,  // How to implement a specific feature
    TechStackChoice,         // Which library/framework to use
    ArchitectureDecision,    // System design choices

    // System improvements
    NewAgentType,            // Propose new agent specialization
    WorkflowChange,          // Modify dispatch workflows
    GovernanceRule,          // Change governance rules
    ToolIntegration,         // Add new tool/integration
    PromptImprovement,       // Improve agent prompts
}

impl ProposalType {
    /// Default consensus threshold for this proposal type
    pub fn default_threshold(&self) -> ConsensusThreshold {
        match self {
            // Implementation decisions need simple majority
            Self::ImplementationApproach => ConsensusThreshold::SimpleMajority,
            Self::TechStackChoice => ConsensusThreshold::SimpleMajority,

            // Architecture needs broader agreement
            Self::ArchitectureDecision => ConsensusThreshold::SuperMajority,

            // System changes need strong consensus
            Self::NewAgentType => ConsensusThreshold::SuperMajority,
            Self::WorkflowChange => ConsensusThreshold::SuperMajority,

            // Governance changes need unanimous agreement
            Self::GovernanceRule => ConsensusThreshold::Unanimous,

            // Tool/prompt changes are easier
            Self::ToolIntegration => ConsensusThreshold::SimpleMajority,
            Self::PromptImprovement => ConsensusThreshold::SimpleMajority,
        }
    }

    /// Which agent types should vote on this proposal
    pub fn required_voters(&self) -> Vec<AgentType> {
        match self {
            Self::ImplementationApproach => vec![AgentType::Coder, AgentType::Reviewer, AgentType::Architect],
            Self::TechStackChoice => vec![AgentType::Coder, AgentType::Architect],
            Self::ArchitectureDecision => vec![AgentType::Architect, AgentType::Coder, AgentType::Security],
            Self::NewAgentType => vec![AgentType::Pm, AgentType::Architect],
            Self::WorkflowChange => vec![AgentType::Pm, AgentType::Architect],
            Self::GovernanceRule => vec![AgentType::Pm, AgentType::Architect, AgentType::Security],
            Self::ToolIntegration => vec![AgentType::Coder, AgentType::Security],
            Self::PromptImprovement => vec![AgentType::Pm, AgentType::Architect],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsensusThreshold {
    Unanimous,       // All must approve
    SuperMajority,   // 67%+ must approve
    SimpleMajority,  // >50% must approve
    SingleApproval,  // Any one approval suffices
}
```

---

## Proposal Lifecycle

```
┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
│  Draft   │───>│   Open   │───>│  Voting  │───>│ Resolved │
└──────────┘    └──────────┘    └──────────┘    └────┬─────┘
                     │                               │
                     │                    ┌──────────┴──────────┐
                     │                    │                     │
                     ▼                    ▼                     ▼
              ┌──────────┐         ┌──────────┐         ┌──────────┐
              │ Vetoed   │         │ Approved │         │ Rejected │
              └──────────┘         └────┬─────┘         └──────────┘
                                        │
                                        ▼
                                 ┌──────────┐
                                 │Executing │
                                 └────┬─────┘
                                      │
                            ┌─────────┴─────────┐
                            │                   │
                            ▼                   ▼
                     ┌──────────┐        ┌───────────┐
                     │ Executed │        │Rolled Back│
                     └──────────┘        └───────────┘
```

### Status Definitions

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalStatus {
    Draft,       // Being composed, not yet submitted
    Open,        // Open for discussion, not yet voting
    Voting,      // Active voting period
    Approved,    // Consensus reached, awaiting execution
    Rejected,    // Failed to reach consensus
    Executing,   // Being implemented
    Executed,    // Successfully implemented
    RolledBack,  // Executed but reverted
    Vetoed,      // Human vetoed the proposal
}
```

---

## Proposal Model

```rust
// dispatch-core/src/types/proposal.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: ProposalId,

    // Type and proposer
    pub proposal_type: ProposalType,
    pub proposer_id: AgentId,

    // Content
    pub title: String,
    pub description: String,
    pub rationale: String,

    // Context
    pub related_issue_id: Option<IssueId>,
    pub affected_components: Vec<String>,

    // Options (for decisions with multiple choices)
    pub options: Option<Vec<ProposalOption>>,
    pub chosen_option: Option<String>,

    // Voting configuration
    pub status: ProposalStatus,
    pub required_voters: Vec<AgentType>,
    pub threshold: ConsensusThreshold,
    pub voting_deadline: Option<DateTime<Utc>>,

    // Execution
    pub implementation_plan: Option<String>,
    pub rollback_plan: Option<String>,
    pub execution_result: Option<ExecutionResult>,

    // Human override
    pub forced_by: Option<String>,
    pub force_reason: Option<String>,
    pub vetoed_by: Option<String>,
    pub veto_reason: Option<String>,

    // Timestamps
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub executed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalOption {
    pub id: String,
    pub title: String,
    pub description: String,
    pub pros: Vec<String>,
    pub cons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub output: String,
    pub artifacts: Vec<String>,  // Created files, PRs, etc.
    pub error: Option<String>,
}
```

---

## Voting Model

```rust
// dispatch-core/src/types/vote.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VoteDecision {
    Approve,
    Reject,
    Abstain,
    NeedMoreInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub id: VoteId,
    pub proposal_id: ProposalId,

    pub voter_id: AgentId,
    pub voter_type: AgentType,

    pub decision: VoteDecision,
    pub reasoning: String,
    pub confidence: f64,  // 0.0 - 1.0

    // For multi-option proposals
    pub preferred_option: Option<String>,
    pub option_ranking: Option<Vec<String>>,

    pub created_at: DateTime<Utc>,
}
```

---

## Consensus Calculator

```rust
// dispatch-governance/src/consensus.rs

pub struct ConsensusCalculator;

impl ConsensusCalculator {
    /// Calculate consensus result for a proposal
    pub fn calculate(
        proposal: &Proposal,
        votes: &[Vote],
    ) -> ConsensusResult {
        // Check if we have all required votes
        let required_types: HashSet<_> = proposal.required_voters.iter().collect();
        let voted_types: HashSet<_> = votes.iter().map(|v| &v.voter_type).collect();

        let missing_voters: Vec<_> = required_types
            .difference(&voted_types)
            .map(|t| (*t).clone())
            .collect();

        if !missing_voters.is_empty() {
            return ConsensusResult::Pending {
                missing_voters,
                votes_received: votes.len(),
            };
        }

        // Check for NeedMoreInfo
        let need_info: Vec<_> = votes
            .iter()
            .filter(|v| v.decision == VoteDecision::NeedMoreInfo)
            .map(|v| v.voter_id.clone())
            .collect();

        if !need_info.is_empty() {
            return ConsensusResult::NeedsMoreInfo { requesters: need_info };
        }

        // Calculate weighted votes
        let mut weighted_approve = 0.0;
        let mut weighted_reject = 0.0;
        let mut total_weight = 0.0;

        for vote in votes {
            if vote.decision == VoteDecision::Abstain {
                continue;
            }

            let weight = vote.voter_type.capabilities().vote_weight;
            total_weight += weight;

            match vote.decision {
                VoteDecision::Approve => weighted_approve += weight,
                VoteDecision::Reject => weighted_reject += weight,
                _ => {}
            }
        }

        if total_weight == 0.0 {
            return ConsensusResult::NoQuorum;
        }

        let approval_ratio = weighted_approve / total_weight;

        // Check against threshold
        let meets_threshold = match proposal.threshold {
            ConsensusThreshold::Unanimous => weighted_reject == 0.0,
            ConsensusThreshold::SuperMajority => approval_ratio >= 0.67,
            ConsensusThreshold::SimpleMajority => approval_ratio > 0.5,
            ConsensusThreshold::SingleApproval => weighted_approve > 0.0,
        };

        if meets_threshold {
            ConsensusResult::Approved {
                approval_ratio,
                chosen_option: Self::determine_chosen_option(proposal, votes),
            }
        } else {
            ConsensusResult::Rejected { approval_ratio }
        }
    }

    /// For multi-option proposals, determine which option won
    fn determine_chosen_option(proposal: &Proposal, votes: &[Vote]) -> Option<String> {
        let options = proposal.options.as_ref()?;
        if options.is_empty() {
            return None;
        }

        // Count weighted preferences
        let mut option_scores: HashMap<String, f64> = HashMap::new();

        for vote in votes {
            if let Some(ref pref) = vote.preferred_option {
                let weight = vote.voter_type.capabilities().vote_weight;
                *option_scores.entry(pref.clone()).or_default() += weight;
            }
        }

        // Return highest scoring option
        option_scores
            .into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(option, _)| option)
    }
}

#[derive(Debug, Clone)]
pub enum ConsensusResult {
    Pending {
        missing_voters: Vec<AgentType>,
        votes_received: usize,
    },
    NeedsMoreInfo {
        requesters: Vec<AgentId>,
    },
    NoQuorum,
    Approved {
        approval_ratio: f64,
        chosen_option: Option<String>,
    },
    Rejected {
        approval_ratio: f64,
    },
}

impl ConsensusResult {
    pub fn is_final(&self) -> bool {
        matches!(self, Self::Approved { .. } | Self::Rejected { .. } | Self::NoQuorum)
    }

    pub fn is_approved(&self) -> bool {
        matches!(self, Self::Approved { .. })
    }
}
```

---

## Proposal Manager

```rust
// dispatch-governance/src/proposals.rs

pub struct ProposalManager {
    proposal_repo: ProposalRepository,
    vote_repo: VoteRepository,
    decision_repo: DecisionRepository,
    agent_repo: AgentRepository,
    events: broadcast::Sender<DispatchEvent>,
}

impl ProposalManager {
    /// Create a new proposal
    pub async fn create_proposal(
        &self,
        proposer_id: &AgentId,
        proposal_type: ProposalType,
        title: String,
        description: String,
        rationale: String,
        related_issue_id: Option<IssueId>,
        options: Option<Vec<ProposalOption>>,
    ) -> Result<ProposalId> {
        let proposal = Proposal {
            id: ProposalId::new(),
            proposal_type,
            proposer_id: proposer_id.clone(),
            title,
            description,
            rationale,
            related_issue_id,
            affected_components: vec![],
            options,
            chosen_option: None,
            status: ProposalStatus::Open,
            required_voters: proposal_type.required_voters(),
            threshold: proposal_type.default_threshold(),
            voting_deadline: Some(Utc::now() + chrono::Duration::hours(24)),
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
        };

        self.proposal_repo.create(&proposal).await?;

        self.events.send(DispatchEvent::ProposalCreated {
            proposal_id: proposal.id.clone(),
        })?;

        // Notify required voters
        self.notify_voters(&proposal).await?;

        Ok(proposal.id)
    }

    /// Start voting period
    pub async fn start_voting(&self, proposal_id: &ProposalId) -> Result<()> {
        let mut proposal = self.get_proposal(proposal_id).await?;

        if proposal.status != ProposalStatus::Open {
            return Err(DispatchError::InvalidStateTransition {
                from: proposal.status.as_str().to_string(),
                to: "voting".to_string(),
            });
        }

        proposal.status = ProposalStatus::Voting;
        self.proposal_repo.update(&proposal).await?;

        Ok(())
    }

    /// Cast a vote
    pub async fn cast_vote(
        &self,
        proposal_id: &ProposalId,
        voter_id: &AgentId,
        decision: VoteDecision,
        reasoning: String,
        confidence: f64,
        preferred_option: Option<String>,
    ) -> Result<VoteId> {
        let proposal = self.get_proposal(proposal_id).await?;

        // Verify voting is open
        if proposal.status != ProposalStatus::Voting && proposal.status != ProposalStatus::Open {
            return Err(DispatchError::Validation(
                "Voting is not open for this proposal".to_string()
            ));
        }

        // Verify voter is eligible
        let voter = self.agent_repo.get(voter_id).await?.ok_or(
            DispatchError::NotFound { entity: "Agent", id: voter_id.to_string() }
        )?;

        if !proposal.required_voters.contains(&voter.agent_type) {
            return Err(DispatchError::Validation(
                format!("Agent type {} is not eligible to vote on this proposal", voter.agent_type.as_str())
            ));
        }

        // Check for existing vote
        if self.vote_repo.get_by_voter(proposal_id, voter_id).await?.is_some() {
            return Err(DispatchError::Validation(
                "Agent has already voted on this proposal".to_string()
            ));
        }

        let vote = Vote {
            id: VoteId::new(),
            proposal_id: proposal_id.clone(),
            voter_id: voter_id.clone(),
            voter_type: voter.agent_type,
            decision,
            reasoning,
            confidence: confidence.clamp(0.0, 1.0),
            preferred_option,
            option_ranking: None,
            created_at: Utc::now(),
        };

        self.vote_repo.create(&vote).await?;

        self.events.send(DispatchEvent::VoteCast {
            proposal_id: proposal_id.clone(),
            vote_id: vote.id.clone(),
        })?;

        // Check if consensus is reached
        self.check_consensus(proposal_id).await?;

        Ok(vote.id)
    }

    /// Check if consensus has been reached and update proposal
    async fn check_consensus(&self, proposal_id: &ProposalId) -> Result<()> {
        let proposal = self.get_proposal(proposal_id).await?;
        let votes = self.vote_repo.list_by_proposal(proposal_id).await?;

        let result = ConsensusCalculator::calculate(&proposal, &votes);

        if result.is_final() {
            let mut proposal = proposal;

            match result {
                ConsensusResult::Approved { chosen_option, .. } => {
                    proposal.status = ProposalStatus::Approved;
                    proposal.chosen_option = chosen_option;
                    proposal.resolved_at = Some(Utc::now());

                    self.events.send(DispatchEvent::ProposalApproved {
                        proposal_id: proposal_id.clone(),
                    })?;
                }
                ConsensusResult::Rejected { .. } | ConsensusResult::NoQuorum => {
                    proposal.status = ProposalStatus::Rejected;
                    proposal.resolved_at = Some(Utc::now());

                    self.events.send(DispatchEvent::ProposalRejected {
                        proposal_id: proposal_id.clone(),
                    })?;
                }
                _ => return Ok(()), // Not final
            }

            self.proposal_repo.update(&proposal).await?;

            // Log decision
            self.log_decision(&proposal, &votes).await?;
        }

        Ok(())
    }

    /// Notify agents that need to vote
    async fn notify_voters(&self, proposal: &Proposal) -> Result<()> {
        for agent_type in &proposal.required_voters {
            // Find active agents of this type
            let agents = self.agent_repo.list_by_type(*agent_type).await?;

            for agent in agents {
                if agent.is_working() {
                    // Agent needs to vote - this would trigger via the agent communication system
                    // For now, log it
                    tracing::info!(
                        "Agent {} ({}) needs to vote on proposal {}",
                        agent.id,
                        agent.agent_type.as_str(),
                        proposal.id
                    );
                }
            }
        }

        Ok(())
    }

    async fn log_decision(&self, proposal: &Proposal, votes: &[Vote]) -> Result<()> {
        let decision = Decision {
            id: DecisionId::new(),
            proposal_id: Some(proposal.id.clone()),
            issue_id: proposal.related_issue_id.clone(),
            epic_id: None,
            decision_type: if proposal.status == ProposalStatus::Approved {
                DecisionType::ProposalApproved
            } else {
                DecisionType::ProposalRejected
            },
            description: format!(
                "Proposal '{}' was {} with {:.0}% approval",
                proposal.title,
                if proposal.status == ProposalStatus::Approved { "approved" } else { "rejected" },
                // Calculate approval percentage
                votes.iter().filter(|v| v.decision == VoteDecision::Approve).count() as f64 /
                votes.iter().filter(|v| v.decision != VoteDecision::Abstain).count().max(1) as f64 * 100.0
            ),
            outcome: serde_json::json!({
                "proposal_id": proposal.id.to_string(),
                "proposal_type": proposal.proposal_type,
                "votes": votes.iter().map(|v| serde_json::json!({
                    "voter_type": v.voter_type,
                    "decision": v.decision,
                    "reasoning": v.reasoning,
                })).collect::<Vec<_>>(),
                "chosen_option": proposal.chosen_option,
            }),
            decided_by: "sangha".to_string(),
            created_at: Utc::now(),
        };

        self.decision_repo.create(&decision).await?;

        Ok(())
    }

    async fn get_proposal(&self, id: &ProposalId) -> Result<Proposal> {
        self.proposal_repo.get(id).await?.ok_or(
            DispatchError::NotFound { entity: "Proposal", id: id.to_string() }
        )
    }
}
```

---

## Human Override

```rust
// dispatch-governance/src/overrides.rs

pub struct HumanOverride {
    proposal_repo: ProposalRepository,
    decision_repo: DecisionRepository,
    events: broadcast::Sender<DispatchEvent>,
}

impl HumanOverride {
    /// Force a proposal decision (bypass voting)
    pub async fn force_decision(
        &self,
        proposal_id: &ProposalId,
        decision: ForceDecision,
        username: &str,
        reason: &str,
    ) -> Result<()> {
        let mut proposal = self.proposal_repo.get(proposal_id).await?.ok_or(
            DispatchError::NotFound { entity: "Proposal", id: proposal_id.to_string() }
        )?;

        // Can only force open/voting proposals
        if !matches!(proposal.status, ProposalStatus::Open | ProposalStatus::Voting) {
            return Err(DispatchError::InvalidStateTransition {
                from: proposal.status.as_str().to_string(),
                to: "forced".to_string(),
            });
        }

        proposal.forced_by = Some(username.to_string());
        proposal.force_reason = Some(reason.to_string());
        proposal.resolved_at = Some(Utc::now());

        match decision {
            ForceDecision::Approve => {
                proposal.status = ProposalStatus::Approved;
                self.events.send(DispatchEvent::ProposalApproved {
                    proposal_id: proposal_id.clone(),
                })?;
            }
            ForceDecision::Reject => {
                proposal.status = ProposalStatus::Rejected;
                self.events.send(DispatchEvent::ProposalRejected {
                    proposal_id: proposal_id.clone(),
                })?;
            }
        }

        self.proposal_repo.update(&proposal).await?;

        // Log the forced decision
        let decision_record = Decision {
            id: DecisionId::new(),
            proposal_id: Some(proposal_id.clone()),
            issue_id: proposal.related_issue_id,
            epic_id: None,
            decision_type: DecisionType::HumanOverride,
            description: format!(
                "Human '{}' forced {} proposal '{}': {}",
                username,
                match decision { ForceDecision::Approve => "approval of", ForceDecision::Reject => "rejection of" },
                proposal.title,
                reason
            ),
            outcome: serde_json::json!({
                "forced_decision": decision,
                "reason": reason,
            }),
            decided_by: format!("human:{}", username),
            created_at: Utc::now(),
        };

        self.decision_repo.create(&decision_record).await?;

        Ok(())
    }

    /// Veto an approved proposal
    pub async fn veto_proposal(
        &self,
        proposal_id: &ProposalId,
        username: &str,
        reason: &str,
    ) -> Result<()> {
        let mut proposal = self.proposal_repo.get(proposal_id).await?.ok_or(
            DispatchError::NotFound { entity: "Proposal", id: proposal_id.to_string() }
        )?;

        // Can only veto approved or executing proposals
        if !matches!(proposal.status, ProposalStatus::Approved | ProposalStatus::Executing) {
            return Err(DispatchError::InvalidStateTransition {
                from: proposal.status.as_str().to_string(),
                to: "vetoed".to_string(),
            });
        }

        proposal.vetoed_by = Some(username.to_string());
        proposal.veto_reason = Some(reason.to_string());
        proposal.status = ProposalStatus::Vetoed;

        self.proposal_repo.update(&proposal).await?;

        self.events.send(DispatchEvent::ProposalVetoed {
            proposal_id: proposal_id.clone(),
            vetoed_by: username.to_string(),
        })?;

        // Log the veto
        let decision_record = Decision {
            id: DecisionId::new(),
            proposal_id: Some(proposal_id.clone()),
            issue_id: proposal.related_issue_id,
            epic_id: None,
            decision_type: DecisionType::HumanVeto,
            description: format!(
                "Human '{}' vetoed proposal '{}': {}",
                username,
                proposal.title,
                reason
            ),
            outcome: serde_json::json!({
                "reason": reason,
            }),
            decided_by: format!("human:{}", username),
            created_at: Utc::now(),
        };

        self.decision_repo.create(&decision_record).await?;

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForceDecision {
    Approve,
    Reject,
}
```

---

## Execution Engine

```rust
// dispatch-governance/src/execution.rs

pub struct ExecutionEngine {
    proposal_repo: ProposalRepository,
    decision_repo: DecisionRepository,
    agent_lifecycle: AgentLifecycleManager,
    events: broadcast::Sender<DispatchEvent>,
}

impl ExecutionEngine {
    /// Execute an approved proposal
    pub async fn execute(&self, proposal_id: &ProposalId) -> Result<ExecutionResult> {
        let mut proposal = self.proposal_repo.get(proposal_id).await?.ok_or(
            DispatchError::NotFound { entity: "Proposal", id: proposal_id.to_string() }
        )?;

        if proposal.status != ProposalStatus::Approved {
            return Err(DispatchError::InvalidStateTransition {
                from: proposal.status.as_str().to_string(),
                to: "executing".to_string(),
            });
        }

        proposal.status = ProposalStatus::Executing;
        self.proposal_repo.update(&proposal).await?;

        // Execute based on proposal type
        let result = match proposal.proposal_type {
            ProposalType::ImplementationApproach => {
                self.execute_implementation(&proposal).await
            }
            ProposalType::TechStackChoice => {
                self.execute_tech_choice(&proposal).await
            }
            ProposalType::PromptImprovement => {
                self.execute_prompt_improvement(&proposal).await
            }
            ProposalType::WorkflowChange => {
                self.execute_workflow_change(&proposal).await
            }
            // Other types may be informational only
            _ => Ok(ExecutionResult {
                success: true,
                output: "Proposal recorded, no automatic execution".to_string(),
                artifacts: vec![],
                error: None,
            })
        };

        // Update proposal with result
        let mut proposal = self.proposal_repo.get(proposal_id).await?.unwrap();

        match &result {
            Ok(exec_result) => {
                proposal.status = ProposalStatus::Executed;
                proposal.execution_result = Some(exec_result.clone());
                proposal.executed_at = Some(Utc::now());
            }
            Err(e) => {
                proposal.status = ProposalStatus::Approved;  // Revert to approved
                proposal.execution_result = Some(ExecutionResult {
                    success: false,
                    output: String::new(),
                    artifacts: vec![],
                    error: Some(e.to_string()),
                });
            }
        }

        self.proposal_repo.update(&proposal).await?;

        result
    }

    async fn execute_implementation(&self, proposal: &Proposal) -> Result<ExecutionResult> {
        // This would typically create an issue for the chosen approach
        // and assign an agent to implement it

        if let Some(ref issue_id) = proposal.related_issue_id {
            // Update the issue with the chosen approach
            // Could add to issue prompt/description
            Ok(ExecutionResult {
                success: true,
                output: format!("Approach '{}' selected for issue", proposal.chosen_option.as_deref().unwrap_or("default")),
                artifacts: vec![issue_id.to_string()],
                error: None,
            })
        } else {
            Ok(ExecutionResult {
                success: true,
                output: "Implementation approach recorded".to_string(),
                artifacts: vec![],
                error: None,
            })
        }
    }

    async fn execute_tech_choice(&self, proposal: &Proposal) -> Result<ExecutionResult> {
        // Record the technology choice
        // Could update a configuration file or documentation
        Ok(ExecutionResult {
            success: true,
            output: format!("Technology choice '{}' recorded", proposal.chosen_option.as_deref().unwrap_or("default")),
            artifacts: vec![],
            error: None,
        })
    }

    async fn execute_prompt_improvement(&self, proposal: &Proposal) -> Result<ExecutionResult> {
        // Update agent prompt files
        // This would modify the prompts/*.md files
        Ok(ExecutionResult {
            success: true,
            output: "Prompt improvement noted for manual implementation".to_string(),
            artifacts: vec![],
            error: None,
        })
    }

    async fn execute_workflow_change(&self, proposal: &Proposal) -> Result<ExecutionResult> {
        // Workflow changes typically require manual implementation
        // but we record the decision
        Ok(ExecutionResult {
            success: true,
            output: "Workflow change approved and recorded".to_string(),
            artifacts: vec![],
            error: None,
        })
    }

    /// Rollback an executed proposal
    pub async fn rollback(&self, proposal_id: &ProposalId) -> Result<()> {
        let mut proposal = self.proposal_repo.get(proposal_id).await?.ok_or(
            DispatchError::NotFound { entity: "Proposal", id: proposal_id.to_string() }
        )?;

        if proposal.status != ProposalStatus::Executed {
            return Err(DispatchError::InvalidStateTransition {
                from: proposal.status.as_str().to_string(),
                to: "rolled_back".to_string(),
            });
        }

        // Execute rollback plan if available
        if let Some(ref plan) = proposal.rollback_plan {
            tracing::info!("Executing rollback plan: {}", plan);
            // Implementation would depend on what was executed
        }

        proposal.status = ProposalStatus::RolledBack;
        self.proposal_repo.update(&proposal).await?;

        Ok(())
    }
}
```

---

## Agent Broadcast

```rust
// dispatch-governance/src/broadcast.rs

use tokio::sync::broadcast;

/// Broadcast proposals to all active agents for voting
pub struct AgentBroadcast {
    agent_repo: AgentRepository,
    /// Channel for proposal notifications
    proposal_tx: broadcast::Sender<ProposalNotification>,
}

#[derive(Debug, Clone)]
pub struct ProposalNotification {
    pub proposal_id: ProposalId,
    pub proposal_type: ProposalType,
    pub title: String,
    pub summary: String,
    pub voting_deadline: Option<DateTime<Utc>>,
    pub required_voters: Vec<AgentType>,
}

impl AgentBroadcast {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            agent_repo: todo!(),
            proposal_tx: tx,
        }
    }

    /// Subscribe to proposal notifications
    pub fn subscribe(&self) -> broadcast::Receiver<ProposalNotification> {
        self.proposal_tx.subscribe()
    }

    /// Broadcast a new proposal to all agents
    pub async fn broadcast_proposal(&self, proposal: &Proposal) -> Result<()> {
        let notification = ProposalNotification {
            proposal_id: proposal.id.clone(),
            proposal_type: proposal.proposal_type,
            title: proposal.title.clone(),
            summary: proposal.description.chars().take(200).collect(),
            voting_deadline: proposal.voting_deadline,
            required_voters: proposal.required_voters.clone(),
        };

        // Broadcast to all listeners
        let _ = self.proposal_tx.send(notification);

        Ok(())
    }
}
```

---

## Decision Log

```rust
// dispatch-core/src/types/decision.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionType {
    ProposalApproved,
    ProposalRejected,
    HumanOverride,
    HumanVeto,
    GateApproved,
    GateRejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub id: DecisionId,

    // What was decided
    pub proposal_id: Option<ProposalId>,
    pub issue_id: Option<IssueId>,
    pub epic_id: Option<EpicId>,

    pub decision_type: DecisionType,
    pub description: String,

    // Outcome details (JSON)
    pub outcome: serde_json::Value,

    // Who made it: "sangha", "human:username", "agent:id"
    pub decided_by: String,

    pub created_at: DateTime<Utc>,
}
```

---

## Coordinator Agent

The Coordinator is a special meta-agent that orchestrates multi-agent workflows, shuttles feedback between agents, and ensures the red-green-refactor cycle is followed properly.

### Architecture

```
                          ┌─────────────────────────┐
                          │     Coordinator         │
                          │  (Orchestration Agent)  │
                          └───────────┬─────────────┘
                                      │
          ┌───────────────────────────┼───────────────────────────┐
          │                           │                           │
          ▼                           ▼                           ▼
   ┌─────────────┐            ┌─────────────┐            ┌─────────────┐
   │   Coder     │◄──────────►│  Reviewer   │◄──────────►│   Test      │
   │   Agent     │            │   Agent     │            │   Agent     │
   └─────────────┘            └─────────────┘            └─────────────┘
          │                           │                           │
          │                           │                           │
          ▼                           ▼                           ▼
   ┌─────────────┐            ┌─────────────┐            ┌─────────────┐
   │  Worktree   │            │  Feedback   │            │  Test       │
   │  (impl)     │            │  Documents  │            │  Results    │
   └─────────────┘            └─────────────┘            └─────────────┘
```

### Coordinator Responsibilities

```rust
// dispatch-agents/src/coordinator.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoordinatorRole {
    /// Oversees implementation workflows
    ImplementationCoordinator,
    /// Manages design review cycles
    DesignReviewCoordinator,
    /// Handles red-green-refactor cycles
    TddCoordinator,
    /// General orchestration
    GeneralCoordinator,
}

pub struct Coordinator {
    pub id: AgentId,
    pub role: CoordinatorRole,
    pub workflow_id: WorkflowId,

    // Agents being coordinated
    pub managed_agents: Vec<AgentId>,

    // Communication channels
    pub feedback_queue: VecDeque<FeedbackItem>,
    pub pending_reviews: Vec<ReviewRequest>,

    // Workflow state
    pub current_phase: WorkflowPhase,
    pub iteration_count: u32,
    pub max_iterations: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackItem {
    pub id: FeedbackId,
    pub from_agent: AgentId,
    pub to_agent: AgentId,
    pub feedback_type: FeedbackType,
    pub content: String,
    pub severity: FeedbackSeverity,
    pub requires_action: bool,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeedbackType {
    CodeReview,
    DesignReview,
    TestFailure,
    SecurityConcern,
    ArchitecturalIssue,
    StyleViolation,
    PerformanceConcern,
    DocumentationNeeded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeedbackSeverity {
    Blocker,     // Must be fixed before proceeding
    Major,       // Should be fixed, may block
    Minor,       // Should be fixed, won't block
    Suggestion,  // Nice to have
}

impl Coordinator {
    /// Route feedback from one agent to another
    pub async fn route_feedback(&mut self, feedback: FeedbackItem) -> Result<()> {
        // Log the feedback
        tracing::info!(
            "Routing feedback from {} to {}: {:?}",
            feedback.from_agent,
            feedback.to_agent,
            feedback.feedback_type
        );

        // Check if this is blocking
        if feedback.severity == FeedbackSeverity::Blocker {
            // Pause the target agent until resolved
            self.pause_agent_for_feedback(&feedback.to_agent).await?;
        }

        // Queue the feedback
        self.feedback_queue.push_back(feedback);

        // Notify the target agent
        self.notify_agent_of_feedback(&feedback.to_agent).await?;

        Ok(())
    }

    /// Check if workflow can proceed to next phase
    pub fn can_advance(&self) -> bool {
        // No blocking feedback pending
        let has_blockers = self.feedback_queue
            .iter()
            .any(|f| f.severity == FeedbackSeverity::Blocker && f.resolved_at.is_none());

        !has_blockers && self.pending_reviews.is_empty()
    }

    /// Advance to next workflow phase
    pub async fn advance_phase(&mut self) -> Result<WorkflowPhase> {
        if !self.can_advance() {
            return Err(DispatchError::Validation(
                "Cannot advance: blocking items pending".to_string()
            ));
        }

        self.current_phase = self.current_phase.next();
        self.iteration_count += 1;

        Ok(self.current_phase)
    }
}
```

### Coordinator System Prompt

```markdown
<!-- prompts/coordinator.md -->

# Coordinator Agent

You are the Coordinator - a meta-agent responsible for orchestrating multi-agent workflows. You don't write code directly; instead, you manage the flow of work between specialized agents.

## Core Responsibilities

1. **Workflow Orchestration**
   - Ensure the correct sequence of operations
   - Route feedback between agents
   - Track progress through workflow phases
   - Decide when to iterate vs. proceed

2. **Feedback Management**
   - Collect feedback from reviewers
   - Prioritize and route feedback to implementers
   - Track feedback resolution
   - Escalate blockers appropriately

3. **Quality Gates**
   - Enforce the red-green-refactor cycle
   - Ensure design reviews happen before implementation
   - Verify test coverage before completion
   - Request security review when needed

4. **Communication**
   - Summarize feedback for agents
   - Provide context when routing work
   - Escalate to humans when stuck
   - Document decisions and rationale

## Workflow Phases

You manage these standard phases:

1. **Spec/Design** → Design Review
2. **Test Writing** → Test Review (RED phase - tests should fail)
3. **Implementation** → Code Review (GREEN phase - make tests pass)
4. **Refactor** → Final Review (REFACTOR phase - clean up)

## Commands Available

- `dispatch_route_feedback` - Send feedback to an agent
- `dispatch_request_review` - Request review from specific agent type
- `dispatch_advance_phase` - Move to next workflow phase
- `dispatch_iterate` - Request another iteration
- `dispatch_escalate` - Escalate to human oversight

## Decision Making

When deciding whether to iterate or proceed:

- **Iterate if:** Blocking feedback exists, tests fail, security concerns raised
- **Proceed if:** All reviews approved, tests pass, no blockers

Maximum iterations per phase: 3 (escalate to human after)
```

---

## Red-Green-Refactor Workflow

The system implements Test-Driven Development (TDD) through a structured red-green-refactor workflow.

### Workflow Phases

```
┌──────────────────────────────────────────────────────────────────────┐
│                    Red-Green-Refactor Workflow                        │
└──────────────────────────────────────────────────────────────────────┘

  ┌─────────┐     ┌─────────┐     ┌─────────┐     ┌─────────┐
  │  SPEC   │────►│   RED   │────►│  GREEN  │────►│REFACTOR │
  │         │     │         │     │         │     │         │
  │ Design  │     │  Write  │     │  Make   │     │ Clean   │
  │ Review  │     │  Tests  │     │  Pass   │     │   Up    │
  └────┬────┘     └────┬────┘     └────┬────┘     └────┬────┘
       │               │               │               │
       ▼               ▼               ▼               ▼
  ┌─────────┐     ┌─────────┐     ┌─────────┐     ┌─────────┐
  │Architect│     │  Test   │     │  Coder  │     │Reviewer │
  │Review   │     │  Agent  │     │  Agent  │     │ Agent   │
  └─────────┘     └─────────┘     └─────────┘     └─────────┘
       │               │               │               │
       ▼               ▼               ▼               ▼
  ┌─────────┐     ┌─────────┐     ┌─────────┐     ┌─────────┐
  │ Design  │     │ Failing │     │ Passing │     │  Clean  │
  │   Doc   │     │  Tests  │     │  Tests  │     │  Code   │
  └─────────┘     └─────────┘     └─────────┘     └─────────┘
```

### Workflow Implementation

```rust
// dispatch-governance/src/workflows/tdd.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TddPhase {
    /// Initial design and specification
    Specification,
    /// Design review by architect/reviewer
    DesignReview,
    /// Write tests that should fail (RED)
    WriteTests,
    /// Review tests before implementation
    TestReview,
    /// Verify tests fail (validation)
    VerifyRed,
    /// Implement to make tests pass (GREEN)
    Implementation,
    /// Code review of implementation
    CodeReview,
    /// Verify tests pass (validation)
    VerifyGreen,
    /// Refactor and clean up
    Refactor,
    /// Final review
    FinalReview,
    /// Complete
    Complete,
}

impl TddPhase {
    pub fn next(&self) -> Self {
        match self {
            Self::Specification => Self::DesignReview,
            Self::DesignReview => Self::WriteTests,
            Self::WriteTests => Self::TestReview,
            Self::TestReview => Self::VerifyRed,
            Self::VerifyRed => Self::Implementation,
            Self::Implementation => Self::CodeReview,
            Self::CodeReview => Self::VerifyGreen,
            Self::VerifyGreen => Self::Refactor,
            Self::Refactor => Self::FinalReview,
            Self::FinalReview => Self::Complete,
            Self::Complete => Self::Complete,
        }
    }

    pub fn can_loop_back_to(&self) -> Option<Self> {
        match self {
            Self::DesignReview => Some(Self::Specification),
            Self::TestReview => Some(Self::WriteTests),
            Self::CodeReview => Some(Self::Implementation),
            Self::FinalReview => Some(Self::Refactor),
            Self::VerifyRed => Some(Self::WriteTests),  // If tests pass, need more tests
            Self::VerifyGreen => Some(Self::Implementation),  // If tests fail, fix impl
            _ => None,
        }
    }

    pub fn required_agent(&self) -> AgentType {
        match self {
            Self::Specification => AgentType::Architect,
            Self::DesignReview => AgentType::Reviewer,
            Self::WriteTests => AgentType::Test,
            Self::TestReview => AgentType::Reviewer,
            Self::VerifyRed => AgentType::Test,
            Self::Implementation => AgentType::Coder,
            Self::CodeReview => AgentType::Reviewer,
            Self::VerifyGreen => AgentType::Test,
            Self::Refactor => AgentType::Coder,
            Self::FinalReview => AgentType::Reviewer,
            Self::Complete => AgentType::Pm,
        }
    }
}

pub struct TddWorkflow {
    pub id: WorkflowId,
    pub issue_id: IssueId,
    pub phase: TddPhase,
    pub coordinator_id: AgentId,

    // Artifacts
    pub spec_document: Option<String>,
    pub test_files: Vec<PathBuf>,
    pub implementation_files: Vec<PathBuf>,

    // Review tracking
    pub design_reviews: Vec<ReviewResult>,
    pub test_reviews: Vec<ReviewResult>,
    pub code_reviews: Vec<ReviewResult>,
    pub final_reviews: Vec<ReviewResult>,

    // Iteration tracking
    pub iterations: HashMap<TddPhase, u32>,
    pub max_iterations: u32,

    // Test results
    pub red_phase_results: Option<TestResults>,
    pub green_phase_results: Option<TestResults>,

    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResult {
    pub reviewer_id: AgentId,
    pub reviewer_type: AgentType,
    pub approved: bool,
    pub feedback: Vec<FeedbackItem>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResults {
    pub total_tests: u32,
    pub passed: u32,
    pub failed: u32,
    pub skipped: u32,
    pub coverage_percent: Option<f64>,
    pub failure_details: Vec<TestFailure>,
}

impl TddWorkflow {
    /// Execute the current phase
    pub async fn execute_phase(&mut self, coordinator: &mut Coordinator) -> Result<PhaseResult> {
        match self.phase {
            TddPhase::Specification => self.execute_specification(coordinator).await,
            TddPhase::DesignReview => self.execute_design_review(coordinator).await,
            TddPhase::WriteTests => self.execute_write_tests(coordinator).await,
            TddPhase::TestReview => self.execute_test_review(coordinator).await,
            TddPhase::VerifyRed => self.execute_verify_red(coordinator).await,
            TddPhase::Implementation => self.execute_implementation(coordinator).await,
            TddPhase::CodeReview => self.execute_code_review(coordinator).await,
            TddPhase::VerifyGreen => self.execute_verify_green(coordinator).await,
            TddPhase::Refactor => self.execute_refactor(coordinator).await,
            TddPhase::FinalReview => self.execute_final_review(coordinator).await,
            TddPhase::Complete => Ok(PhaseResult::Complete),
        }
    }

    async fn execute_verify_red(&mut self, coordinator: &mut Coordinator) -> Result<PhaseResult> {
        // Run the tests - they SHOULD fail
        let results = self.run_tests().await?;
        self.red_phase_results = Some(results.clone());

        if results.failed == 0 {
            // Tests pass but shouldn't - need more/better tests
            coordinator.route_feedback(FeedbackItem {
                id: FeedbackId::new(),
                from_agent: coordinator.id.clone(),
                to_agent: self.get_test_agent()?,
                feedback_type: FeedbackType::TestFailure,
                content: "RED phase validation failed: Tests should fail but all pass. \
                          Write tests that actually verify the unimplemented functionality.".to_string(),
                severity: FeedbackSeverity::Blocker,
                requires_action: true,
                created_at: Utc::now(),
                resolved_at: None,
            }).await?;

            return Ok(PhaseResult::NeedsIteration(TddPhase::WriteTests));
        }

        // Tests fail as expected - proceed
        Ok(PhaseResult::Advance)
    }

    async fn execute_verify_green(&mut self, coordinator: &mut Coordinator) -> Result<PhaseResult> {
        // Run the tests - they SHOULD pass
        let results = self.run_tests().await?;
        self.green_phase_results = Some(results.clone());

        if results.failed > 0 {
            // Tests still failing - need to fix implementation
            coordinator.route_feedback(FeedbackItem {
                id: FeedbackId::new(),
                from_agent: coordinator.id.clone(),
                to_agent: self.get_coder_agent()?,
                feedback_type: FeedbackType::TestFailure,
                content: format!(
                    "GREEN phase validation failed: {} tests still failing.\n\nFailures:\n{}",
                    results.failed,
                    results.failure_details.iter()
                        .map(|f| format!("- {}: {}", f.test_name, f.message))
                        .collect::<Vec<_>>()
                        .join("\n")
                ),
                severity: FeedbackSeverity::Blocker,
                requires_action: true,
                created_at: Utc::now(),
                resolved_at: None,
            }).await?;

            return Ok(PhaseResult::NeedsIteration(TddPhase::Implementation));
        }

        // All tests pass - proceed to refactor
        Ok(PhaseResult::Advance)
    }
}

#[derive(Debug)]
pub enum PhaseResult {
    Advance,
    NeedsIteration(TddPhase),
    NeedsHumanIntervention(String),
    Complete,
}
```

---

## Design Review Workflow

Before implementation begins, designs must be reviewed by appropriate agents.

### Review Types

```rust
// dispatch-governance/src/workflows/review.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewType {
    /// Architecture and high-level design
    ArchitectureReview,
    /// API design and contracts
    ApiReview,
    /// Security implications
    SecurityReview,
    /// Code implementation
    CodeReview,
    /// Test coverage and quality
    TestReview,
    /// Documentation completeness
    DocumentationReview,
    /// Final approval before merge
    FinalReview,
}

impl ReviewType {
    pub fn required_reviewers(&self) -> Vec<AgentType> {
        match self {
            Self::ArchitectureReview => vec![AgentType::Architect, AgentType::Reviewer],
            Self::ApiReview => vec![AgentType::Architect, AgentType::Coder],
            Self::SecurityReview => vec![AgentType::Security],
            Self::CodeReview => vec![AgentType::Reviewer, AgentType::Coder],
            Self::TestReview => vec![AgentType::Test, AgentType::Reviewer],
            Self::DocumentationReview => vec![AgentType::Docs, AgentType::Reviewer],
            Self::FinalReview => vec![AgentType::Reviewer, AgentType::Architect],
        }
    }

    pub fn approval_threshold(&self) -> ConsensusThreshold {
        match self {
            Self::SecurityReview => ConsensusThreshold::Unanimous,
            Self::ArchitectureReview => ConsensusThreshold::SuperMajority,
            _ => ConsensusThreshold::SimpleMajority,
        }
    }
}

pub struct DesignReviewWorkflow {
    pub id: WorkflowId,
    pub issue_id: IssueId,
    pub design_document: String,
    pub review_type: ReviewType,

    // Review state
    pub reviews: Vec<AgentReview>,
    pub status: ReviewStatus,
    pub iteration: u32,

    // Feedback tracking
    pub feedback_items: Vec<FeedbackItem>,
    pub resolved_feedback: Vec<FeedbackId>,

    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentReview {
    pub reviewer_id: AgentId,
    pub reviewer_type: AgentType,
    pub decision: ReviewDecision,
    pub comments: Vec<ReviewComment>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewDecision {
    Approved,
    ApprovedWithComments,
    RequestChanges,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewComment {
    pub id: CommentId,
    pub location: Option<String>,  // File:line or section reference
    pub severity: FeedbackSeverity,
    pub category: CommentCategory,
    pub content: String,
    pub suggested_change: Option<String>,
    pub resolved: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommentCategory {
    Bug,
    Security,
    Performance,
    Maintainability,
    Style,
    Documentation,
    Testing,
    Architecture,
    Other,
}

impl DesignReviewWorkflow {
    /// Check if review is complete and approved
    pub fn is_approved(&self) -> bool {
        let threshold = self.review_type.approval_threshold();
        let required = self.review_type.required_reviewers();

        // Check all required reviewer types have reviewed
        let reviewed_types: HashSet<_> = self.reviews.iter()
            .map(|r| r.reviewer_type)
            .collect();

        let all_reviewed = required.iter().all(|t| reviewed_types.contains(t));
        if !all_reviewed {
            return false;
        }

        // Calculate approval ratio
        let approvals = self.reviews.iter()
            .filter(|r| matches!(r.decision, ReviewDecision::Approved | ReviewDecision::ApprovedWithComments))
            .count();

        let total = self.reviews.len();
        let ratio = approvals as f64 / total as f64;

        match threshold {
            ConsensusThreshold::Unanimous => approvals == total,
            ConsensusThreshold::SuperMajority => ratio >= 0.67,
            ConsensusThreshold::SimpleMajority => ratio > 0.5,
            ConsensusThreshold::SingleApproval => approvals >= 1,
        }
    }

    /// Get blocking feedback that must be resolved
    pub fn blocking_feedback(&self) -> Vec<&FeedbackItem> {
        self.feedback_items.iter()
            .filter(|f| {
                f.severity == FeedbackSeverity::Blocker
                && f.resolved_at.is_none()
            })
            .collect()
    }

    /// Request re-review after addressing feedback
    pub async fn request_rereview(&mut self, coordinator: &mut Coordinator) -> Result<()> {
        self.iteration += 1;
        self.status = ReviewStatus::PendingReview;

        // Clear old reviews
        self.reviews.clear();

        // Notify reviewers
        for reviewer_type in self.review_type.required_reviewers() {
            coordinator.request_review(
                &self.issue_id,
                self.review_type,
                reviewer_type,
            ).await?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewStatus {
    Draft,
    PendingReview,
    InReview,
    ChangesRequested,
    Approved,
    Rejected,
}
```

### Implementation Iteration Cycle

```rust
// dispatch-governance/src/workflows/iteration.rs

/// Manages multiple implementation passes with feedback
pub struct IterationCycle {
    pub id: WorkflowId,
    pub issue_id: IssueId,

    // Iteration tracking
    pub current_iteration: u32,
    pub max_iterations: u32,
    pub iterations: Vec<Iteration>,

    // Quality metrics
    pub test_coverage_target: f64,
    pub required_approvals: u32,

    pub status: IterationStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Iteration {
    pub number: u32,
    pub phase: IterationPhase,

    // Work done
    pub changes_made: Vec<String>,
    pub files_modified: Vec<PathBuf>,

    // Feedback received
    pub feedback: Vec<FeedbackItem>,
    pub feedback_addressed: Vec<FeedbackId>,

    // Review results
    pub review_results: Vec<ReviewResult>,

    // Test results
    pub test_results: Option<TestResults>,

    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IterationPhase {
    Implementation,
    Testing,
    Review,
    FeedbackResolution,
    Complete,
}

impl IterationCycle {
    /// Start a new iteration
    pub async fn start_iteration(&mut self, coordinator: &mut Coordinator) -> Result<()> {
        if self.current_iteration >= self.max_iterations {
            return Err(DispatchError::Validation(
                format!("Maximum iterations ({}) reached, escalating to human", self.max_iterations)
            ));
        }

        self.current_iteration += 1;

        let iteration = Iteration {
            number: self.current_iteration,
            phase: IterationPhase::Implementation,
            changes_made: vec![],
            files_modified: vec![],
            feedback: vec![],
            feedback_addressed: vec![],
            review_results: vec![],
            test_results: None,
            started_at: Utc::now(),
            completed_at: None,
        };

        self.iterations.push(iteration);

        // Gather unresolved feedback from previous iteration
        let previous_feedback: Vec<_> = if self.iterations.len() > 1 {
            self.iterations[self.iterations.len() - 2]
                .feedback.iter()
                .filter(|f| !f.resolved_at.is_some())
                .cloned()
                .collect()
        } else {
            vec![]
        };

        // Route unresolved feedback to implementer
        for feedback in previous_feedback {
            coordinator.route_feedback(feedback).await?;
        }

        Ok(())
    }

    /// Complete current iteration and check if more needed
    pub fn complete_iteration(&mut self) -> IterationResult {
        let current = self.iterations.last_mut().unwrap();
        current.phase = IterationPhase::Complete;
        current.completed_at = Some(Utc::now());

        // Check completion criteria
        let has_blocking_feedback = current.feedback.iter()
            .any(|f| f.severity == FeedbackSeverity::Blocker && f.resolved_at.is_none());

        let all_reviews_approved = current.review_results.iter()
            .all(|r| r.approved);

        let tests_passing = current.test_results
            .as_ref()
            .map(|t| t.failed == 0)
            .unwrap_or(false);

        let coverage_met = current.test_results
            .as_ref()
            .and_then(|t| t.coverage_percent)
            .map(|c| c >= self.test_coverage_target)
            .unwrap_or(false);

        if has_blocking_feedback {
            IterationResult::NeedsIteration("Blocking feedback not resolved".to_string())
        } else if !all_reviews_approved {
            IterationResult::NeedsIteration("Reviews not all approved".to_string())
        } else if !tests_passing {
            IterationResult::NeedsIteration("Tests failing".to_string())
        } else if !coverage_met {
            IterationResult::NeedsIteration(format!(
                "Coverage {:.1}% below target {:.1}%",
                current.test_results.as_ref().unwrap().coverage_percent.unwrap_or(0.0),
                self.test_coverage_target
            ))
        } else {
            self.status = IterationStatus::Complete;
            IterationResult::Complete
        }
    }
}

#[derive(Debug)]
pub enum IterationResult {
    NeedsIteration(String),
    Complete,
    EscalateToHuman(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IterationStatus {
    InProgress,
    Complete,
    EscalatedToHuman,
}
```

---

## Feedback Routing System

The Coordinator manages all feedback between agents.

```rust
// dispatch-governance/src/feedback.rs

pub struct FeedbackRouter {
    feedback_repo: FeedbackRepository,
    agent_repo: AgentRepository,
    events: broadcast::Sender<DispatchEvent>,
}

impl FeedbackRouter {
    /// Route feedback with smart targeting
    pub async fn route(&self, feedback: FeedbackItem) -> Result<()> {
        // Store feedback
        self.feedback_repo.create(&feedback).await?;

        // Determine routing strategy based on type
        match feedback.feedback_type {
            FeedbackType::SecurityConcern => {
                // Always escalate security to security agent AND human
                self.escalate_security_feedback(&feedback).await?;
            }
            FeedbackType::ArchitecturalIssue => {
                // Route to architect first
                self.route_to_agent_type(&feedback, AgentType::Architect).await?;
            }
            _ => {
                // Route to specified target
                self.route_to_agent(&feedback, &feedback.to_agent).await?;
            }
        }

        // Emit event
        self.events.send(DispatchEvent::FeedbackRouted {
            feedback_id: feedback.id.clone(),
            from_agent: feedback.from_agent.clone(),
            to_agent: feedback.to_agent.clone(),
        })?;

        Ok(())
    }

    /// Aggregate feedback for an agent
    pub async fn get_pending_feedback(&self, agent_id: &AgentId) -> Result<FeedbackSummary> {
        let all_feedback = self.feedback_repo.list_for_agent(agent_id).await?;

        let pending: Vec<_> = all_feedback.iter()
            .filter(|f| f.resolved_at.is_none())
            .collect();

        let blockers: Vec<_> = pending.iter()
            .filter(|f| f.severity == FeedbackSeverity::Blocker)
            .cloned()
            .cloned()
            .collect();

        let major: Vec<_> = pending.iter()
            .filter(|f| f.severity == FeedbackSeverity::Major)
            .cloned()
            .cloned()
            .collect();

        let minor: Vec<_> = pending.iter()
            .filter(|f| f.severity == FeedbackSeverity::Minor)
            .cloned()
            .cloned()
            .collect();

        Ok(FeedbackSummary {
            total_pending: pending.len(),
            blockers,
            major,
            minor,
            by_type: self.group_by_type(&pending),
        })
    }

    /// Format feedback for agent consumption
    pub fn format_for_agent(&self, feedback: &[FeedbackItem]) -> String {
        let mut output = String::new();

        // Group by severity
        let blockers: Vec<_> = feedback.iter()
            .filter(|f| f.severity == FeedbackSeverity::Blocker)
            .collect();
        let major: Vec<_> = feedback.iter()
            .filter(|f| f.severity == FeedbackSeverity::Major)
            .collect();
        let minor: Vec<_> = feedback.iter()
            .filter(|f| f.severity == FeedbackSeverity::Minor)
            .collect();

        if !blockers.is_empty() {
            output.push_str("## 🚫 BLOCKING Issues (Must Fix)\n\n");
            for f in blockers {
                output.push_str(&format!("- **{}**: {}\n", f.feedback_type.as_str(), f.content));
            }
            output.push('\n');
        }

        if !major.is_empty() {
            output.push_str("## ⚠️ Major Issues (Should Fix)\n\n");
            for f in major {
                output.push_str(&format!("- **{}**: {}\n", f.feedback_type.as_str(), f.content));
            }
            output.push('\n');
        }

        if !minor.is_empty() {
            output.push_str("## 💡 Minor Issues (Nice to Fix)\n\n");
            for f in minor {
                output.push_str(&format!("- **{}**: {}\n", f.feedback_type.as_str(), f.content));
            }
        }

        output
    }
}

#[derive(Debug, Clone)]
pub struct FeedbackSummary {
    pub total_pending: usize,
    pub blockers: Vec<FeedbackItem>,
    pub major: Vec<FeedbackItem>,
    pub minor: Vec<FeedbackItem>,
    pub by_type: HashMap<FeedbackType, Vec<FeedbackItem>>,
}
```

---

## Database Schema Additions

```sql
-- Add to migrations/004_add_workflows.sql

-- Workflows table (TDD, Review, etc.)
CREATE TABLE workflows (
    id TEXT PRIMARY KEY,
    workflow_type TEXT NOT NULL,        -- tdd, design_review, iteration
    issue_id TEXT NOT NULL,
    coordinator_id TEXT,                -- FK to agents

    status TEXT NOT NULL DEFAULT 'pending',
    current_phase TEXT NOT NULL,
    iteration_count INTEGER DEFAULT 0,
    max_iterations INTEGER DEFAULT 3,

    config TEXT,                        -- JSON workflow config
    state TEXT,                         -- JSON workflow state

    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    completed_at TEXT,

    FOREIGN KEY (issue_id) REFERENCES issues(id) ON DELETE CASCADE,
    FOREIGN KEY (coordinator_id) REFERENCES agents(id) ON DELETE SET NULL
);

CREATE INDEX idx_workflows_issue ON workflows(issue_id);
CREATE INDEX idx_workflows_status ON workflows(status);

-- Feedback items
CREATE TABLE feedback (
    id TEXT PRIMARY KEY,
    workflow_id TEXT,                   -- FK to workflows
    from_agent_id TEXT NOT NULL,
    to_agent_id TEXT NOT NULL,

    feedback_type TEXT NOT NULL,
    severity TEXT NOT NULL,
    content TEXT NOT NULL,
    requires_action INTEGER DEFAULT 1,

    created_at TEXT NOT NULL,
    resolved_at TEXT,
    resolved_by TEXT,                   -- FK to agents

    FOREIGN KEY (workflow_id) REFERENCES workflows(id) ON DELETE CASCADE,
    FOREIGN KEY (from_agent_id) REFERENCES agents(id),
    FOREIGN KEY (to_agent_id) REFERENCES agents(id),
    FOREIGN KEY (resolved_by) REFERENCES agents(id)
);

CREATE INDEX idx_feedback_workflow ON feedback(workflow_id);
CREATE INDEX idx_feedback_to ON feedback(to_agent_id);
CREATE INDEX idx_feedback_resolved ON feedback(resolved_at);

-- Reviews
CREATE TABLE reviews (
    id TEXT PRIMARY KEY,
    workflow_id TEXT NOT NULL,
    review_type TEXT NOT NULL,

    reviewer_id TEXT NOT NULL,
    reviewer_type TEXT NOT NULL,

    decision TEXT NOT NULL,
    comments TEXT,                      -- JSON array of comments

    created_at TEXT NOT NULL,

    FOREIGN KEY (workflow_id) REFERENCES workflows(id) ON DELETE CASCADE,
    FOREIGN KEY (reviewer_id) REFERENCES agents(id)
);

CREATE INDEX idx_reviews_workflow ON reviews(workflow_id);
CREATE INDEX idx_reviews_reviewer ON reviews(reviewer_id);

-- Test results
CREATE TABLE test_results (
    id TEXT PRIMARY KEY,
    workflow_id TEXT NOT NULL,
    iteration INTEGER NOT NULL,
    phase TEXT NOT NULL,                -- red, green

    total_tests INTEGER NOT NULL,
    passed INTEGER NOT NULL,
    failed INTEGER NOT NULL,
    skipped INTEGER NOT NULL,
    coverage_percent REAL,

    failure_details TEXT,               -- JSON array

    created_at TEXT NOT NULL,

    FOREIGN KEY (workflow_id) REFERENCES workflows(id) ON DELETE CASCADE
);

CREATE INDEX idx_test_results_workflow ON test_results(workflow_id);
```

---

## Integration with Agents

Agents participate in governance through their system prompts:

```markdown
<!-- prompts/base.md (addition) -->

## Governance Participation

You are part of the Sangha - the collective of AI agents working on this project.

### Proposals

When you encounter a decision that affects the broader system (not just your current issue), you should create a proposal:

1. **When to propose:**
   - Multiple valid implementation approaches
   - Technology/library choices
   - Architecture decisions
   - Improvements to workflows or prompts

2. **How to propose:**
   Use the `dispatch_propose` tool:
   ```
   dispatch_propose(
     type: "implementation_approach",
     title: "Use async/await pattern for API calls",
     description: "...",
     rationale: "...",
     options: [...]
   )
   ```

### Voting

When a proposal requires your vote, you'll receive a notification. Evaluate proposals thoughtfully:

1. **Consider:**
   - Technical merit
   - Consistency with existing code
   - Long-term maintainability
   - Security implications

2. **Vote honestly:**
   - Approve if you support the proposal
   - Reject if you have concerns
   - NeedMoreInfo if proposal is unclear
   - Abstain if outside your expertise

3. **Explain your reasoning:**
   Always provide clear rationale for your vote.
```

---

## Implementation PRs

| PR | Description | Files |
|----|-------------|-------|
| PR-045 | Proposal data model | `dispatch-core/src/types/proposal.rs`, `dispatch-governance/src/proposals.rs` |
| PR-046 | Voting mechanism | `dispatch-governance/src/voting.rs`, `dispatch-core/src/types/vote.rs` |
| PR-047 | Consensus calculation | `dispatch-governance/src/consensus.rs` |
| PR-048 | Agent-to-agent broadcast | `dispatch-governance/src/broadcast.rs` |
| PR-049 | Proposal execution | `dispatch-governance/src/execution.rs` |
| PR-050 | Human override: force | `dispatch-governance/src/overrides.rs` |
| PR-051 | Human override: veto | `dispatch-governance/src/overrides.rs` |
| PR-052 | Decision logging | `dispatch-db/src/repos/decision.rs` |
| PR-053 | CLI proposal commands | `dispatch-cli/src/commands/proposal.rs` |
| PR-054a | Coordinator agent | `dispatch-agents/src/coordinator.rs`, `prompts/coordinator.md` |
| PR-054b | Feedback routing system | `dispatch-governance/src/feedback.rs` |
| PR-054c | TDD workflow | `dispatch-governance/src/workflows/tdd.rs` |
| PR-054d | Design review workflow | `dispatch-governance/src/workflows/review.rs` |
| PR-054e | Iteration cycle management | `dispatch-governance/src/workflows/iteration.rs` |
| PR-054f | Workflow database schema | `migrations/004_add_workflows.sql` |
