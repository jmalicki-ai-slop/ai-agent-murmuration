//! Metadata parsing from issue bodies

use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

/// Murmuration metadata embedded in issue body
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IssueMetadata {
    /// Phase number (e.g., 3 for Phase 3)
    #[serde(default)]
    pub phase: Option<u32>,

    /// PR identifier (e.g., "013")
    #[serde(default)]
    pub pr: Option<String>,

    /// Issue numbers this depends on
    #[serde(default)]
    pub depends_on: Option<Vec<u64>>,

    /// Status (e.g., "blocked", "ready", "in_progress")
    #[serde(default)]
    pub status: Option<String>,

    /// Type of issue (e.g., "epic", "pr", "task")
    #[serde(default, rename = "type")]
    pub issue_type: Option<String>,

    /// Parent epic issue number
    #[serde(default)]
    pub parent: Option<u64>,
}

impl IssueMetadata {
    /// Parse metadata from issue body text
    ///
    /// Looks for HTML comment blocks in the format:
    /// ```markdown
    /// <!-- murmur:metadata
    /// {
    ///   "phase": 3,
    ///   "pr": "013",
    ///   "depends_on": [15],
    ///   "status": "blocked"
    /// }
    /// -->
    /// ```
    pub fn parse(body: &str) -> Option<Self> {
        Self::parse_all(body).into_iter().next()
    }

    /// Parse all metadata blocks from issue body
    ///
    /// Returns all valid metadata blocks found. Invalid JSON is logged but skipped.
    pub fn parse_all(body: &str) -> Vec<Self> {
        let mut results = Vec::new();

        for block in extract_metadata_blocks(body) {
            match serde_json::from_str::<Self>(&block) {
                Ok(metadata) => {
                    debug!(?metadata, "Parsed metadata block");
                    results.push(metadata);
                }
                Err(e) => {
                    warn!(?e, block = %block, "Failed to parse metadata block");
                }
            }
        }

        results
    }

    /// Check if this issue has dependencies
    pub fn has_dependencies(&self) -> bool {
        self.depends_on
            .as_ref()
            .is_some_and(|deps| !deps.is_empty())
    }

    /// Get dependency issue numbers
    pub fn dependencies(&self) -> &[u64] {
        self.depends_on.as_deref().unwrap_or(&[])
    }

    /// Check if this is an epic
    pub fn is_epic(&self) -> bool {
        self.issue_type
            .as_ref()
            .is_some_and(|t| t.eq_ignore_ascii_case("epic"))
    }

    /// Check if status indicates blocked
    pub fn is_blocked(&self) -> bool {
        self.status
            .as_ref()
            .is_some_and(|s| s.eq_ignore_ascii_case("blocked"))
    }
}

/// Extract JSON content from murmur:metadata HTML comment blocks
fn extract_metadata_blocks(body: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let start_marker = "<!-- murmur:metadata";
    let end_marker = "-->";

    let mut search_pos = 0;
    while let Some(start) = body[search_pos..].find(start_marker) {
        let absolute_start = search_pos + start + start_marker.len();

        if let Some(end) = body[absolute_start..].find(end_marker) {
            let json_content = body[absolute_start..absolute_start + end].trim();
            if !json_content.is_empty() {
                blocks.push(json_content.to_string());
            }
            search_pos = absolute_start + end + end_marker.len();
        } else {
            // No closing marker found, stop searching
            break;
        }
    }

    blocks
}

/// Parse "Depends on #X" style dependencies from body text
pub fn parse_depends_on_links(body: &str) -> Vec<u64> {
    let mut deps = Vec::new();

    // Pattern: "Depends on #123" or "depends on #123"
    let patterns = ["Depends on #", "depends on #", "Blocked by #", "blocked by #"];

    for pattern in patterns {
        for part in body.split(pattern).skip(1) {
            // Extract the number after #
            let num_str: String = part.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(num) = num_str.parse::<u64>() {
                if !deps.contains(&num) {
                    deps.push(num);
                }
            }
        }
    }

    deps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_metadata_block() {
        let body = r#"
## Description

Some description here.

<!-- murmur:metadata
{
  "phase": 3,
  "pr": "013",
  "depends_on": [15, 16],
  "status": "blocked"
}
-->
"#;

        let metadata = IssueMetadata::parse(body).unwrap();
        assert_eq!(metadata.phase, Some(3));
        assert_eq!(metadata.pr, Some("013".to_string()));
        assert_eq!(metadata.depends_on, Some(vec![15, 16]));
        assert_eq!(metadata.status, Some("blocked".to_string()));
    }

    #[test]
    fn test_parse_metadata_with_type() {
        let body = r#"<!-- murmur:metadata
{
  "type": "epic",
  "phase": 3
}
-->"#;

        let metadata = IssueMetadata::parse(body).unwrap();
        assert_eq!(metadata.issue_type, Some("epic".to_string()));
        assert!(metadata.is_epic());
    }

    #[test]
    fn test_parse_no_metadata() {
        let body = "Just a regular issue with no metadata.";
        assert!(IssueMetadata::parse(body).is_none());
    }

    #[test]
    fn test_parse_multiple_metadata_blocks() {
        let body = r#"
<!-- murmur:metadata
{ "phase": 1 }
-->

Some text

<!-- murmur:metadata
{ "phase": 2 }
-->
"#;

        let blocks = IssueMetadata::parse_all(body);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].phase, Some(1));
        assert_eq!(blocks[1].phase, Some(2));
    }

    #[test]
    fn test_parse_malformed_json() {
        let body = r#"<!-- murmur:metadata
{ invalid json }
-->"#;

        // Should return None but not panic
        assert!(IssueMetadata::parse(body).is_none());
    }

    #[test]
    fn test_has_dependencies() {
        let mut metadata = IssueMetadata::default();
        assert!(!metadata.has_dependencies());

        metadata.depends_on = Some(vec![]);
        assert!(!metadata.has_dependencies());

        metadata.depends_on = Some(vec![1, 2]);
        assert!(metadata.has_dependencies());
    }

    #[test]
    fn test_is_blocked() {
        let mut metadata = IssueMetadata::default();
        assert!(!metadata.is_blocked());

        metadata.status = Some("ready".to_string());
        assert!(!metadata.is_blocked());

        metadata.status = Some("blocked".to_string());
        assert!(metadata.is_blocked());

        metadata.status = Some("BLOCKED".to_string());
        assert!(metadata.is_blocked());
    }

    #[test]
    fn test_parse_depends_on_links() {
        let body = "Depends on #15\nAlso depends on #16\nBlocked by #17";
        let deps = parse_depends_on_links(body);
        assert_eq!(deps, vec![15, 16, 17]);
    }

    #[test]
    fn test_parse_depends_on_links_case_insensitive() {
        let body = "depends on #15\nblocked by #16";
        let deps = parse_depends_on_links(body);
        assert_eq!(deps, vec![15, 16]);
    }

    #[test]
    fn test_parse_depends_on_deduplicates() {
        let body = "Depends on #15\ndepends on #15";
        let deps = parse_depends_on_links(body);
        assert_eq!(deps, vec![15]);
    }
}
