//! Issue creation from plan

use crate::{GitHubClient, IssueFilter, IssueState, Result};
use murmur_core::{Phase, Plan, PlannedPR};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Result of importing a plan to GitHub
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImportResult {
    /// Mapping of phase IDs to epic issue numbers
    pub epics: HashMap<String, u64>,
    /// Mapping of PR IDs to issue numbers
    pub prs: HashMap<String, u64>,
    /// Number of issues created
    pub created: usize,
    /// Number of issues skipped (already existed)
    pub skipped: usize,
    /// Any errors encountered
    pub errors: Vec<String>,
}

/// Options for importing a plan
#[derive(Debug, Clone, Default)]
pub struct ImportOptions {
    /// Labels to add to all created issues
    pub labels: Vec<String>,
    /// Dry run - don't actually create issues
    pub dry_run: bool,
    /// Skip issues that already exist
    pub skip_existing: bool,
}

impl GitHubClient {
    /// Import a plan to GitHub as issues
    pub async fn import_plan(
        &self,
        plan: &Plan,
        options: &ImportOptions,
    ) -> Result<ImportResult> {
        let mut result = ImportResult::default();

        // Get existing issues to check for duplicates
        let existing = self
            .list_issues(&IssueFilter {
                state: Some(IssueState::Open),
                ..Default::default()
            })
            .await?;

        let existing_titles: HashMap<String, u64> = existing
            .iter()
            .map(|i| (i.title.clone(), i.number))
            .collect();

        // Create epics first
        for phase in &plan.phases {
            let epic_title = format!("Phase {}: {}", phase.id, phase.name);

            if let Some(&existing_num) = existing_titles.get(&epic_title) {
                info!(title = %epic_title, number = existing_num, "Epic already exists, skipping");
                result.epics.insert(phase.id.clone(), existing_num);
                result.skipped += 1;
                continue;
            }

            if options.dry_run {
                info!(title = %epic_title, "[DRY RUN] Would create epic");
                continue;
            }

            let body = build_epic_body(phase);
            let mut labels = vec!["epic".to_string(), format!("phase-{}", phase.id)];
            labels.extend(options.labels.clone());

            match self.create_issue(&epic_title, &body, &labels).await {
                Ok(issue) => {
                    info!(title = %epic_title, number = issue.number, "Created epic");
                    result.epics.insert(phase.id.clone(), issue.number);
                    result.created += 1;
                }
                Err(e) => {
                    warn!(title = %epic_title, error = %e, "Failed to create epic");
                    result.errors.push(format!("Failed to create epic '{}': {}", epic_title, e));
                }
            }
        }

        // Create PR issues
        for phase in &plan.phases {
            let epic_number = result.epics.get(&phase.id).copied();
            let phase_label = format!("phase-{}", phase.id);

            for pr in &phase.prs {
                let pr_title = format!("{}: {}", pr.id, pr.description);

                if let Some(&existing_num) = existing_titles.get(&pr_title) {
                    info!(title = %pr_title, number = existing_num, "PR already exists, skipping");
                    result.prs.insert(pr.id.clone(), existing_num);
                    result.skipped += 1;
                    continue;
                }

                if options.dry_run {
                    info!(title = %pr_title, "[DRY RUN] Would create PR issue");
                    continue;
                }

                // Calculate actual dependency issue numbers
                let dep_numbers: Vec<u64> = pr
                    .depends_on
                    .iter()
                    .filter_map(|dep_id| result.prs.get(dep_id).copied())
                    .collect();

                let body = build_pr_body(pr, epic_number, &dep_numbers, &phase.id);
                let mut labels = vec![phase_label.clone()];
                labels.extend(options.labels.clone());

                match self.create_issue(&pr_title, &body, &labels).await {
                    Ok(issue) => {
                        info!(title = %pr_title, number = issue.number, "Created PR issue");
                        result.prs.insert(pr.id.clone(), issue.number);
                        result.created += 1;
                    }
                    Err(e) => {
                        warn!(title = %pr_title, error = %e, "Failed to create PR issue");
                        result
                            .errors
                            .push(format!("Failed to create PR '{}': {}", pr_title, e));
                    }
                }
            }
        }

        Ok(result)
    }

    /// Create a single issue
    async fn create_issue(
        &self,
        title: &str,
        body: &str,
        labels: &[String],
    ) -> Result<crate::Issue> {
        debug!(title, "Creating issue");

        let issue = self
            .client()
            .issues(self.owner(), self.repo())
            .create(title)
            .body(body)
            .labels(labels.to_vec())
            .send()
            .await
            .map_err(crate::Error::Api)?;

        Ok(issue.into())
    }
}

fn build_epic_body(phase: &Phase) -> String {
    let mut body = String::new();

    body.push_str("## Overview\n\n");
    if !phase.goal.is_empty() {
        body.push_str(&format!("**Goal:** {}\n\n", phase.goal));
    }

    // PR checklist
    if !phase.prs.is_empty() {
        body.push_str("## Child Issues\n\n");
        for pr in &phase.prs {
            body.push_str(&format!("- [ ] {}: {}\n", pr.id, pr.description));
        }
        body.push('\n');
    }

    // Checkpoint
    if let Some(ref checkpoint) = phase.checkpoint {
        body.push_str("## Acceptance Criteria\n\n");
        body.push_str(checkpoint);
        body.push_str("\n\n");
    }

    // Dependencies
    if !phase.depends_on.is_empty() {
        body.push_str("## Dependencies\n\n");
        for dep in &phase.depends_on {
            body.push_str(&format!("- Phase {}\n", dep));
        }
        body.push('\n');
    }

    // Metadata
    body.push_str(&format!(
        r#"<!-- murmur:metadata
{{
  "type": "epic",
  "phase": "{}"
}}
-->"#,
        phase.id
    ));

    body
}

fn build_pr_body(
    pr: &PlannedPR,
    epic_number: Option<u64>,
    dep_numbers: &[u64],
    phase_id: &str,
) -> String {
    let mut body = String::new();

    body.push_str("## Description\n\n");
    body.push_str(&pr.description);
    body.push_str("\n\n");

    // Parent epic
    if let Some(epic) = epic_number {
        body.push_str("## Parent\n\n");
        body.push_str(&format!("Parent: #{}\n\n", epic));
    }

    // Dependencies
    if !dep_numbers.is_empty() {
        body.push_str("## Dependencies\n\n");
        for dep in dep_numbers {
            body.push_str(&format!("Depends on #{}\n", dep));
        }
        body.push('\n');
    }

    // Files
    if !pr.files.is_empty() {
        body.push_str("## Files\n\n");
        for file in &pr.files {
            body.push_str(&format!("- `{}`\n", file));
        }
        body.push('\n');
    }

    // Metadata
    let deps_json = if dep_numbers.is_empty() {
        "[]".to_string()
    } else {
        format!(
            "[{}]",
            dep_numbers
                .iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    };

    body.push_str(&format!(
        r#"<!-- murmur:metadata
{{
  "phase": "{}",
  "pr": "{}",
  "depends_on": {},
  "status": "pending"
}}
-->"#,
        phase_id,
        pr.id.strip_prefix("PR-").unwrap_or(&pr.id),
        deps_json
    ));

    body
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_epic_body() {
        let phase = Phase {
            id: "1".to_string(),
            name: "Test Phase".to_string(),
            goal: "Test goal".to_string(),
            prs: vec![PlannedPR {
                id: "PR-001".to_string(),
                description: "First PR".to_string(),
                files: vec![],
                depends_on: vec![],
                is_sub_pr: false,
                parent_pr: None,
            }],
            depends_on: vec![],
            checkpoint: Some("Test checkpoint".to_string()),
        };

        let body = build_epic_body(&phase);
        assert!(body.contains("**Goal:** Test goal"));
        assert!(body.contains("PR-001: First PR"));
        assert!(body.contains("Test checkpoint"));
        assert!(body.contains("murmur:metadata"));
    }

    #[test]
    fn test_build_pr_body() {
        let pr = PlannedPR {
            id: "PR-001".to_string(),
            description: "Test description".to_string(),
            files: vec!["file1.rs".to_string()],
            depends_on: vec![],
            is_sub_pr: false,
            parent_pr: None,
        };

        let body = build_pr_body(&pr, Some(1), &[2, 3], "1");
        assert!(body.contains("Test description"));
        assert!(body.contains("Parent: #1"));
        assert!(body.contains("Depends on #2"));
        assert!(body.contains("Depends on #3"));
        assert!(body.contains("`file1.rs`"));
        assert!(body.contains("murmur:metadata"));
    }

    #[test]
    fn test_import_result_default() {
        let result = ImportResult::default();
        assert!(result.epics.is_empty());
        assert!(result.prs.is_empty());
        assert_eq!(result.created, 0);
        assert_eq!(result.skipped, 0);
    }
}
