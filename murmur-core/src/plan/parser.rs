//! PLAN.md parser

use crate::Result;
use serde::{Deserialize, Serialize};

/// A parsed plan from PLAN.md
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Plan {
    /// Title of the plan
    pub title: String,
    /// Description/overview
    pub description: String,
    /// Phases in the plan
    pub phases: Vec<Phase>,
}

/// A phase in the plan (corresponds to an epic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phase {
    /// Phase identifier (e.g., "1", "3b")
    pub id: String,
    /// Phase name
    pub name: String,
    /// Goal description
    pub goal: String,
    /// PRs in this phase
    pub prs: Vec<PlannedPR>,
    /// Phase dependencies (other phase IDs)
    pub depends_on: Vec<String>,
    /// Checkpoint description
    pub checkpoint: Option<String>,
}

/// A planned PR (corresponds to an issue)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedPR {
    /// PR identifier (e.g., "PR-001", "PR-001a")
    pub id: String,
    /// PR description
    pub description: String,
    /// Files to be modified
    pub files: Vec<String>,
    /// PR dependencies (other PR IDs, inferred from ordering)
    pub depends_on: Vec<String>,
    /// Whether this is a sub-PR (has letter suffix like PR-001a)
    pub is_sub_pr: bool,
    /// Parent PR for sub-PRs
    pub parent_pr: Option<String>,
}

impl PlannedPR {
    /// Get the numeric part of the PR ID (e.g., "001" from "PR-001a")
    pub fn number(&self) -> Option<&str> {
        self.id
            .strip_prefix("PR-")
            .map(|s| s.trim_end_matches(|c: char| c.is_alphabetic()))
    }
}

/// Parse a PLAN.md file
pub fn parse_plan(content: &str) -> Result<Plan> {
    let mut plan = Plan::default();
    let mut current_phase: Option<Phase> = None;
    let mut in_table = false;
    let mut prev_pr_id: Option<String> = None;

    for line in content.lines() {
        let line = line.trim();

        // Extract title (first # header)
        if line.starts_with("# ") && plan.title.is_empty() {
            plan.title = line[2..].trim().to_string();
            continue;
        }

        // Detect phase headers: ### Phase N: Name
        if let Some(rest) = line.strip_prefix("### Phase ") {
            // Save previous phase
            if let Some(phase) = current_phase.take() {
                plan.phases.push(phase);
            }

            // Parse phase header
            if let Some((id, name)) = parse_phase_header(rest) {
                current_phase = Some(Phase {
                    id,
                    name,
                    goal: String::new(),
                    prs: Vec::new(),
                    depends_on: Vec::new(),
                    checkpoint: None,
                });
                in_table = false;
                prev_pr_id = None;
            }
            continue;
        }

        // Handle phase content
        if let Some(ref mut phase) = current_phase {
            // Goal line: *Goal: ...*
            if line.starts_with("*Goal:") && line.ends_with('*') {
                phase.goal = line[6..line.len() - 1].trim().to_string();
                continue;
            }

            // Checkpoint line: **Checkpoint:** ...
            if let Some(rest) = line.strip_prefix("**Checkpoint:**") {
                phase.checkpoint = Some(rest.trim().to_string());
                continue;
            }

            // Table header detection
            if line.starts_with("| PR |") {
                in_table = true;
                continue;
            }

            // Skip separator line
            if line.starts_with("|---") || line.starts_with("| ---") {
                continue;
            }

            // Table row
            if in_table && line.starts_with('|') && line.ends_with('|') {
                if let Some(pr) = parse_table_row(line, &prev_pr_id) {
                    prev_pr_id = Some(pr.id.clone());
                    phase.prs.push(pr);
                }
                continue;
            }

            // Horizontal rule ends table
            if line == "---" {
                in_table = false;
            }
        }
    }

    // Save last phase
    if let Some(phase) = current_phase {
        plan.phases.push(phase);
    }

    // Infer dependencies between phases
    infer_phase_dependencies(&mut plan);

    Ok(plan)
}

/// Parse "N: Name" or "Nb: Name" from phase header
fn parse_phase_header(rest: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = rest.splitn(2, ':').collect();
    if parts.len() == 2 {
        let id = parts[0].trim().to_string();
        let name = parts[1].trim().to_string();
        Some((id, name))
    } else {
        None
    }
}

/// Parse a PR table row
fn parse_table_row(line: &str, prev_pr_id: &Option<String>) -> Option<PlannedPR> {
    let cells: Vec<&str> = line
        .split('|')
        .filter(|s| !s.is_empty())
        .map(|s| s.trim())
        .collect();

    if cells.len() < 2 {
        return None;
    }

    let id = cells[0].to_string();
    let description = cells.get(1).map(|s| s.to_string()).unwrap_or_default();
    let files = cells
        .get(2)
        .map(|s| {
            s.split(',')
                .map(|f| f.trim().trim_matches('`').to_string())
                .filter(|f| !f.is_empty())
                .collect()
        })
        .unwrap_or_default();

    // Detect sub-PRs (PR-001a, PR-001b, etc.)
    let is_sub_pr = id
        .strip_prefix("PR-")
        .map(|s| s.chars().last().map(|c| c.is_alphabetic()).unwrap_or(false))
        .unwrap_or(false);

    let parent_pr = if is_sub_pr {
        id.strip_prefix("PR-")
            .map(|s| format!("PR-{}", s.trim_end_matches(|c: char| c.is_alphabetic())))
    } else {
        None
    };

    // Infer dependency from previous PR (except first PR in phase)
    let depends_on = if let Some(prev) = prev_pr_id {
        // Sub-PRs depend on their parent, not previous
        if is_sub_pr {
            parent_pr
                .as_ref()
                .map(|p| vec![p.clone()])
                .unwrap_or_default()
        } else {
            vec![prev.clone()]
        }
    } else {
        Vec::new()
    };

    Some(PlannedPR {
        id,
        description,
        files,
        depends_on,
        is_sub_pr,
        parent_pr,
    })
}

/// Infer dependencies between phases based on ordering
fn infer_phase_dependencies(plan: &mut Plan) {
    // Each phase depends on the previous phase
    let phase_ids: Vec<String> = plan.phases.iter().map(|p| p.id.clone()).collect();

    for (i, phase) in plan.phases.iter_mut().enumerate() {
        if i > 0 {
            phase.depends_on.push(phase_ids[i - 1].clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_PLAN: &str = r#"# Test Plan

## Overview

Test description.

---

### Phase 1: First Phase
*Goal: Test the first goal*

| PR | Description | Files |
|----|-------------|-------|
| PR-001 | First PR | `file1.rs` |
| PR-002 | Second PR | `file2.rs`, `file3.rs` |

**Checkpoint:** First checkpoint.

---

### Phase 2: Second Phase
*Goal: Test the second goal*

| PR | Description | Files |
|----|-------------|-------|
| PR-003 | Third PR | `file4.rs` |
| PR-003a | Sub PR A | |
| PR-003b | Sub PR B | |
| PR-004 | Fourth PR | `file5.rs` |

**Checkpoint:** Second checkpoint.
"#;

    #[test]
    fn test_parse_plan_title() {
        let plan = parse_plan(SAMPLE_PLAN).unwrap();
        assert_eq!(plan.title, "Test Plan");
    }

    #[test]
    fn test_parse_phases() {
        let plan = parse_plan(SAMPLE_PLAN).unwrap();
        assert_eq!(plan.phases.len(), 2);
        assert_eq!(plan.phases[0].id, "1");
        assert_eq!(plan.phases[0].name, "First Phase");
        assert_eq!(plan.phases[1].id, "2");
        assert_eq!(plan.phases[1].name, "Second Phase");
    }

    #[test]
    fn test_parse_phase_goal() {
        let plan = parse_plan(SAMPLE_PLAN).unwrap();
        assert_eq!(plan.phases[0].goal, "Test the first goal");
    }

    #[test]
    fn test_parse_checkpoint() {
        let plan = parse_plan(SAMPLE_PLAN).unwrap();
        assert_eq!(
            plan.phases[0].checkpoint,
            Some("First checkpoint.".to_string())
        );
    }

    #[test]
    fn test_parse_prs() {
        let plan = parse_plan(SAMPLE_PLAN).unwrap();
        assert_eq!(plan.phases[0].prs.len(), 2);
        assert_eq!(plan.phases[0].prs[0].id, "PR-001");
        assert_eq!(plan.phases[0].prs[0].description, "First PR");
        assert_eq!(plan.phases[0].prs[0].files, vec!["file1.rs"]);
    }

    #[test]
    fn test_parse_multiple_files() {
        let plan = parse_plan(SAMPLE_PLAN).unwrap();
        assert_eq!(plan.phases[0].prs[1].files, vec!["file2.rs", "file3.rs"]);
    }

    #[test]
    fn test_infer_pr_dependencies() {
        let plan = parse_plan(SAMPLE_PLAN).unwrap();
        // First PR has no dependencies
        assert!(plan.phases[0].prs[0].depends_on.is_empty());
        // Second PR depends on first
        assert_eq!(plan.phases[0].prs[1].depends_on, vec!["PR-001"]);
    }

    #[test]
    fn test_sub_pr_detection() {
        let plan = parse_plan(SAMPLE_PLAN).unwrap();
        let phase2 = &plan.phases[1];

        // PR-003 is not a sub-PR
        assert!(!phase2.prs[0].is_sub_pr);

        // PR-003a is a sub-PR
        assert!(phase2.prs[1].is_sub_pr);
        assert_eq!(phase2.prs[1].parent_pr, Some("PR-003".to_string()));

        // PR-003b is a sub-PR
        assert!(phase2.prs[2].is_sub_pr);
        assert_eq!(phase2.prs[2].parent_pr, Some("PR-003".to_string()));
    }

    #[test]
    fn test_sub_pr_dependencies() {
        let plan = parse_plan(SAMPLE_PLAN).unwrap();
        let phase2 = &plan.phases[1];

        // Sub-PRs depend on parent, not previous
        assert_eq!(phase2.prs[1].depends_on, vec!["PR-003"]);
        assert_eq!(phase2.prs[2].depends_on, vec!["PR-003"]);
    }

    #[test]
    fn test_infer_phase_dependencies() {
        let plan = parse_plan(SAMPLE_PLAN).unwrap();
        // First phase has no dependencies
        assert!(plan.phases[0].depends_on.is_empty());
        // Second phase depends on first
        assert_eq!(plan.phases[1].depends_on, vec!["1"]);
    }

    #[test]
    fn test_pr_number() {
        let pr = PlannedPR {
            id: "PR-001a".to_string(),
            description: "Test".to_string(),
            files: vec![],
            depends_on: vec![],
            is_sub_pr: true,
            parent_pr: Some("PR-001".to_string()),
        };
        assert_eq!(pr.number(), Some("001"));
    }
}
