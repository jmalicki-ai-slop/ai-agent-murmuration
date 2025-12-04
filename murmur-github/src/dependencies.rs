//! Dependency parsing and graph building

use crate::{Issue, IssueMetadata};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Reference to an issue, possibly in another repository
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IssueRef {
    /// Repository owner (None for same repo)
    pub owner: Option<String>,
    /// Repository name (None for same repo)
    pub repo: Option<String>,
    /// Issue number
    pub number: u64,
}

impl IssueRef {
    /// Create a reference to an issue in the current repository
    pub fn local(number: u64) -> Self {
        Self {
            owner: None,
            repo: None,
            number,
        }
    }

    /// Create a reference to an issue in another repository
    pub fn external(owner: impl Into<String>, repo: impl Into<String>, number: u64) -> Self {
        Self {
            owner: Some(owner.into()),
            repo: Some(repo.into()),
            number,
        }
    }

    /// Check if this is a local (same repo) reference
    pub fn is_local(&self) -> bool {
        self.owner.is_none() && self.repo.is_none()
    }
}

impl std::fmt::Display for IssueRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (&self.owner, &self.repo) {
            (Some(owner), Some(repo)) => write!(f, "{}/{}#{}", owner, repo, self.number),
            _ => write!(f, "#{}", self.number),
        }
    }
}

/// Parsed dependencies from an issue
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IssueDependencies {
    /// Issues this depends on
    pub depends_on: Vec<IssueRef>,
    /// Issues that block this
    pub blocked_by: Vec<IssueRef>,
    /// Parent epic (if any)
    pub parent: Option<IssueRef>,
}

impl IssueDependencies {
    /// Parse dependencies from issue body text
    pub fn parse(body: &str) -> Self {
        let mut deps = Self::default();

        // Parse "Depends on" patterns
        deps.depends_on.extend(parse_depends_pattern(body));

        // Parse "Blocked by" patterns
        deps.blocked_by.extend(parse_blocked_pattern(body));

        // Parse "Parent:" pattern
        deps.parent = parse_parent_pattern(body);

        // Also check metadata block
        if let Some(metadata) = IssueMetadata::parse(body) {
            // Add dependencies from metadata
            if let Some(meta_deps) = metadata.depends_on {
                for num in meta_deps {
                    let r = IssueRef::local(num);
                    if !deps.depends_on.contains(&r) {
                        deps.depends_on.push(r);
                    }
                }
            }

            // Add parent from metadata
            if deps.parent.is_none() {
                if let Some(parent_num) = metadata.parent {
                    deps.parent = Some(IssueRef::local(parent_num));
                }
            }
        }

        deps
    }

    /// Check if this issue has any dependencies
    pub fn has_dependencies(&self) -> bool {
        !self.depends_on.is_empty() || !self.blocked_by.is_empty()
    }

    /// Get all dependency issue numbers (local only)
    pub fn all_local_deps(&self) -> Vec<u64> {
        let mut nums = Vec::new();
        for r in &self.depends_on {
            if r.is_local() {
                nums.push(r.number);
            }
        }
        for r in &self.blocked_by {
            if r.is_local() && !nums.contains(&r.number) {
                nums.push(r.number);
            }
        }
        nums
    }
}

/// Dependency graph for a set of issues
#[derive(Debug, Clone, Default)]
pub struct DependencyGraph {
    /// Map from issue number to issues it depends on
    pub dependencies: HashMap<u64, Vec<u64>>,
    /// Map from issue number to issues that depend on it
    pub dependents: HashMap<u64, Vec<u64>>,
    /// Map from issue number to parent epic
    pub parents: HashMap<u64, u64>,
    /// Issues that have no unmet dependencies
    pub ready: HashSet<u64>,
    /// Issues with unmet dependencies
    pub blocked: HashSet<u64>,
}

impl DependencyGraph {
    /// Build a dependency graph from a list of issues
    pub fn from_issues(issues: &[Issue]) -> Self {
        let mut graph = Self::default();
        let issue_nums: HashSet<u64> = issues.iter().map(|i| i.number).collect();

        for issue in issues {
            let deps = IssueDependencies::parse(&issue.body);

            // Record dependencies
            let local_deps: Vec<u64> = deps.all_local_deps();
            if !local_deps.is_empty() {
                graph.dependencies.insert(issue.number, local_deps.clone());

                // Also record reverse mapping
                for dep_num in &local_deps {
                    graph
                        .dependents
                        .entry(*dep_num)
                        .or_default()
                        .push(issue.number);
                }
            }

            // Record parent
            if let Some(parent_ref) = deps.parent {
                if parent_ref.is_local() {
                    graph.parents.insert(issue.number, parent_ref.number);
                }
            }

            // Determine if ready or blocked
            let unmet_deps: Vec<u64> = local_deps
                .iter()
                .filter(|d| issue_nums.contains(d))
                .copied()
                .collect();

            if unmet_deps.is_empty() {
                graph.ready.insert(issue.number);
            } else {
                graph.blocked.insert(issue.number);
            }
        }

        graph
    }

    /// Get issues that are ready to work on (no unmet dependencies)
    pub fn ready_issues(&self) -> Vec<u64> {
        self.ready.iter().copied().collect()
    }

    /// Get issues that are blocked
    pub fn blocked_issues(&self) -> Vec<u64> {
        self.blocked.iter().copied().collect()
    }

    /// Check for circular dependencies
    pub fn find_cycles(&self) -> Vec<Vec<u64>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for &node in self.dependencies.keys() {
            if !visited.contains(&node) {
                let mut path = Vec::new();
                if let Some(cycle) = self.dfs_cycle(node, &mut visited, &mut rec_stack, &mut path) {
                    cycles.push(cycle);
                }
            }
        }

        cycles
    }

    fn dfs_cycle(
        &self,
        node: u64,
        visited: &mut HashSet<u64>,
        rec_stack: &mut HashSet<u64>,
        path: &mut Vec<u64>,
    ) -> Option<Vec<u64>> {
        visited.insert(node);
        rec_stack.insert(node);
        path.push(node);

        if let Some(deps) = self.dependencies.get(&node) {
            for &dep in deps {
                if !visited.contains(&dep) {
                    if let Some(cycle) = self.dfs_cycle(dep, visited, rec_stack, path) {
                        return Some(cycle);
                    }
                } else if rec_stack.contains(&dep) {
                    // Found a cycle - extract it
                    if let Some(start) = path.iter().position(|&n| n == dep) {
                        let cycle = path[start..].to_vec();
                        return Some(cycle);
                    }
                }
            }
        }

        path.pop();
        rec_stack.remove(&node);
        None
    }

    /// Get topological order of issues (respecting dependencies)
    ///
    /// Returns issues in order where dependencies come before dependents.
    /// For example, if issue 2 depends on issue 1, the order will be [1, 2].
    pub fn topological_order(&self) -> Option<Vec<u64>> {
        let mut order = Vec::new();
        let mut visited = HashSet::new();
        let mut temp_visited = HashSet::new();

        let all_nodes: HashSet<u64> = self
            .dependencies
            .keys()
            .chain(self.dependents.keys())
            .copied()
            .collect();

        for &node in &all_nodes {
            if !visited.contains(&node)
                && !self.topo_visit(node, &mut visited, &mut temp_visited, &mut order)
            {
                return None; // Cycle detected
            }
        }

        // order is already in correct order (dependencies first)
        Some(order)
    }

    fn topo_visit(
        &self,
        node: u64,
        visited: &mut HashSet<u64>,
        temp_visited: &mut HashSet<u64>,
        order: &mut Vec<u64>,
    ) -> bool {
        if temp_visited.contains(&node) {
            return false; // Cycle
        }
        if visited.contains(&node) {
            return true;
        }

        temp_visited.insert(node);

        // Visit dependencies first (what this node depends on)
        if let Some(deps) = self.dependencies.get(&node) {
            for &dep in deps {
                if !self.topo_visit(dep, visited, temp_visited, order) {
                    return false;
                }
            }
        }

        temp_visited.remove(&node);
        visited.insert(node);
        // Add node after its dependencies are added
        order.push(node);
        true
    }
}

/// Parse "Depends on #X" and "Depends on owner/repo#X" patterns
fn parse_depends_pattern(body: &str) -> Vec<IssueRef> {
    parse_issue_refs(
        body,
        &["Depends on", "depends on", "Depend on", "depend on"],
    )
}

/// Parse "Blocked by #X" patterns
fn parse_blocked_pattern(body: &str) -> Vec<IssueRef> {
    parse_issue_refs(body, &["Blocked by", "blocked by"])
}

/// Parse "Parent: #X" pattern
fn parse_parent_pattern(body: &str) -> Option<IssueRef> {
    let patterns = ["Parent:", "parent:", "Parent :", "parent :"];

    for pattern in patterns {
        for part in body.split(pattern).skip(1) {
            let trimmed = part.trim_start();
            if let Some(r) = parse_single_issue_ref(trimmed) {
                return Some(r);
            }
        }
    }

    None
}

/// Parse issue references after a pattern
fn parse_issue_refs(body: &str, patterns: &[&str]) -> Vec<IssueRef> {
    let mut refs = Vec::new();

    for pattern in patterns {
        for part in body.split(pattern).skip(1) {
            // Get the rest of the line
            let line_end = part.find('\n').unwrap_or(part.len());
            let line = &part[..line_end];

            // Parse comma-separated refs: "#123, #456" or "owner/repo#123"
            for segment in line.split(',') {
                let segment = segment.trim();
                if let Some(r) = parse_single_issue_ref(segment) {
                    if !refs.contains(&r) {
                        refs.push(r);
                    }
                }
            }
        }
    }

    refs
}

/// Parse a single issue reference like "#123" or "owner/repo#123"
fn parse_single_issue_ref(s: &str) -> Option<IssueRef> {
    let s = s.trim();

    // Check for cross-repo reference: owner/repo#123
    if let Some(hash_pos) = s.find('#') {
        let before_hash = &s[..hash_pos];
        let after_hash = &s[hash_pos + 1..];

        // Parse the number
        let num_str: String = after_hash
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        let number = num_str.parse::<u64>().ok()?;

        if before_hash.is_empty() {
            // Local reference: #123
            return Some(IssueRef::local(number));
        }

        // Cross-repo reference: owner/repo#123
        if let Some(slash_pos) = before_hash.find('/') {
            let owner = before_hash[..slash_pos].trim();
            let repo = before_hash[slash_pos + 1..].trim();
            if !owner.is_empty() && !repo.is_empty() {
                return Some(IssueRef::external(owner, repo, number));
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_local_ref() {
        let r = parse_single_issue_ref("#123").unwrap();
        assert!(r.is_local());
        assert_eq!(r.number, 123);
    }

    #[test]
    fn test_parse_cross_repo_ref() {
        let r = parse_single_issue_ref("owner/repo#456").unwrap();
        assert!(!r.is_local());
        assert_eq!(r.owner, Some("owner".to_string()));
        assert_eq!(r.repo, Some("repo".to_string()));
        assert_eq!(r.number, 456);
    }

    #[test]
    fn test_issue_ref_display() {
        assert_eq!(IssueRef::local(123).to_string(), "#123");
        assert_eq!(
            IssueRef::external("owner", "repo", 456).to_string(),
            "owner/repo#456"
        );
    }

    #[test]
    fn test_parse_depends_on() {
        let body = "Depends on #15\nAlso depends on #16";
        let deps = IssueDependencies::parse(body);
        assert_eq!(deps.depends_on.len(), 2);
        assert_eq!(deps.depends_on[0].number, 15);
        assert_eq!(deps.depends_on[1].number, 16);
    }

    #[test]
    fn test_parse_depends_on_comma_separated() {
        let body = "Depends on #15, #16, #17";
        let deps = IssueDependencies::parse(body);
        assert_eq!(deps.depends_on.len(), 3);
    }

    #[test]
    fn test_parse_blocked_by() {
        let body = "Blocked by #15";
        let deps = IssueDependencies::parse(body);
        assert_eq!(deps.blocked_by.len(), 1);
        assert_eq!(deps.blocked_by[0].number, 15);
    }

    #[test]
    fn test_parse_parent() {
        let body = "Parent: #1\nSome description";
        let deps = IssueDependencies::parse(body);
        assert!(deps.parent.is_some());
        assert_eq!(deps.parent.unwrap().number, 1);
    }

    #[test]
    fn test_parse_cross_repo_dependency() {
        let body = "Depends on other/repo#123";
        let deps = IssueDependencies::parse(body);
        assert_eq!(deps.depends_on.len(), 1);
        assert!(!deps.depends_on[0].is_local());
        assert_eq!(deps.depends_on[0].owner, Some("other".to_string()));
    }

    #[test]
    fn test_dependency_graph_ready() {
        let issues = vec![
            make_test_issue(1, "First issue"),
            make_test_issue(2, "Depends on #1"),
            make_test_issue(3, "No deps"),
        ];

        let graph = DependencyGraph::from_issues(&issues);

        assert!(graph.ready.contains(&1));
        assert!(graph.ready.contains(&3));
        assert!(graph.blocked.contains(&2));
    }

    #[test]
    fn test_dependency_graph_cycle_detection() {
        let issues = vec![
            make_test_issue(1, "Depends on #2"),
            make_test_issue(2, "Depends on #1"),
        ];

        let graph = DependencyGraph::from_issues(&issues);
        let cycles = graph.find_cycles();

        assert!(!cycles.is_empty());
    }

    #[test]
    fn test_topological_order() {
        let issues = vec![
            make_test_issue(1, "First"),
            make_test_issue(2, "Depends on #1"),
            make_test_issue(3, "Depends on #2"),
        ];

        let graph = DependencyGraph::from_issues(&issues);
        let order = graph.topological_order().unwrap();

        // 1 should come before 2, and 2 before 3
        let pos_1 = order.iter().position(|&n| n == 1).unwrap();
        let pos_2 = order.iter().position(|&n| n == 2).unwrap();
        let pos_3 = order.iter().position(|&n| n == 3).unwrap();

        assert!(pos_1 < pos_2);
        assert!(pos_2 < pos_3);
    }

    fn make_test_issue(number: u64, body: &str) -> Issue {
        Issue {
            number,
            title: format!("Test issue {}", number),
            body: body.to_string(),
            state: crate::IssueState::Open,
            labels: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            pull_request_url: None,
        }
    }
}
