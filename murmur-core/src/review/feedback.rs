//! Review feedback parsing module
//!
//! This module provides functionality to parse structured feedback from reviewer agent output.
//! It extracts actionable feedback items, severity levels, and decision outcomes from
//! the reviewer's response.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Severity level of a feedback item
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum FeedbackSeverity {
    /// Critical issues that must be fixed
    Critical,
    /// Major issues that should be fixed
    Major,
    /// Minor issues that could be improved
    #[default]
    Minor,
    /// Suggestions for improvement (optional)
    Suggestion,
    /// Positive feedback (things done well)
    Praise,
}

impl fmt::Display for FeedbackSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Critical => write!(f, "critical"),
            Self::Major => write!(f, "major"),
            Self::Minor => write!(f, "minor"),
            Self::Suggestion => write!(f, "suggestion"),
            Self::Praise => write!(f, "praise"),
        }
    }
}

/// A single feedback item from the review
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeedbackItem {
    /// Severity of the issue
    pub severity: FeedbackSeverity,
    /// Category of the feedback (e.g., "security", "performance", "style")
    pub category: Option<String>,
    /// The feedback message
    pub message: String,
    /// File path this feedback relates to (if applicable)
    pub file: Option<String>,
    /// Line number (if applicable)
    pub line: Option<u32>,
    /// Suggested fix (if provided)
    pub suggestion: Option<String>,
}

impl FeedbackItem {
    /// Create a new feedback item with just a message
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            severity: FeedbackSeverity::default(),
            category: None,
            message: message.into(),
            file: None,
            line: None,
            suggestion: None,
        }
    }

    /// Set the severity level
    pub fn with_severity(mut self, severity: FeedbackSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Set the category
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    /// Set the file path
    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        self.file = Some(file.into());
        self
    }

    /// Set the line number
    pub fn with_line(mut self, line: u32) -> Self {
        self.line = Some(line);
        self
    }

    /// Set the suggested fix
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Check if this is a blocking issue (critical or major)
    pub fn is_blocking(&self) -> bool {
        matches!(
            self.severity,
            FeedbackSeverity::Critical | FeedbackSeverity::Major
        )
    }
}

/// The decision from a review
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ReviewDecision {
    /// Approved - can proceed to next phase
    Approved,
    /// Approved with minor suggestions
    ApprovedWithSuggestions,
    /// Changes requested - must address feedback
    #[default]
    ChangesRequested,
    /// Rejected - fundamental issues
    Rejected,
}

impl ReviewDecision {
    /// Check if this decision allows proceeding
    pub fn can_proceed(&self) -> bool {
        matches!(self, Self::Approved | Self::ApprovedWithSuggestions)
    }
}

impl fmt::Display for ReviewDecision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Approved => write!(f, "approved"),
            Self::ApprovedWithSuggestions => write!(f, "approved with suggestions"),
            Self::ChangesRequested => write!(f, "changes requested"),
            Self::Rejected => write!(f, "rejected"),
        }
    }
}

/// Parsed feedback from a review
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReviewFeedback {
    /// The overall decision
    pub decision: ReviewDecision,
    /// Summary of the review
    pub summary: String,
    /// Individual feedback items
    pub items: Vec<FeedbackItem>,
    /// Raw output from the reviewer (for reference)
    pub raw_output: String,
}

impl ReviewFeedback {
    /// Create a new empty feedback
    pub fn new() -> Self {
        Self::default()
    }

    /// Create feedback with a decision and summary
    pub fn with_decision(decision: ReviewDecision, summary: impl Into<String>) -> Self {
        Self {
            decision,
            summary: summary.into(),
            items: Vec::new(),
            raw_output: String::new(),
        }
    }

    /// Add a feedback item
    pub fn add_item(&mut self, item: FeedbackItem) {
        self.items.push(item);
    }

    /// Add multiple feedback items
    pub fn add_items(&mut self, items: impl IntoIterator<Item = FeedbackItem>) {
        self.items.extend(items);
    }

    /// Set the raw output
    pub fn with_raw_output(mut self, output: impl Into<String>) -> Self {
        self.raw_output = output.into();
        self
    }

    /// Get all blocking items (critical or major)
    pub fn blocking_items(&self) -> Vec<&FeedbackItem> {
        self.items.iter().filter(|i| i.is_blocking()).collect()
    }

    /// Check if there are any blocking issues
    pub fn has_blocking_issues(&self) -> bool {
        self.items.iter().any(|i| i.is_blocking())
    }

    /// Get items by severity
    pub fn items_by_severity(&self, severity: FeedbackSeverity) -> Vec<&FeedbackItem> {
        self.items
            .iter()
            .filter(|i| i.severity == severity)
            .collect()
    }

    /// Get the count of items by severity
    pub fn count_by_severity(&self) -> std::collections::HashMap<FeedbackSeverity, usize> {
        let mut counts = std::collections::HashMap::new();
        for item in &self.items {
            *counts.entry(item.severity).or_insert(0) += 1;
        }
        counts
    }

    /// Check if the review allows proceeding to the next phase
    pub fn can_proceed(&self) -> bool {
        self.decision.can_proceed() && !self.has_blocking_issues()
    }

    /// Format the feedback as a string for display or passing back to agents
    pub fn to_feedback_string(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("## Review Decision: {}\n\n", self.decision));

        if !self.summary.is_empty() {
            output.push_str(&format!("{}\n\n", self.summary));
        }

        if !self.items.is_empty() {
            output.push_str("### Feedback Items\n\n");
            for (i, item) in self.items.iter().enumerate() {
                output.push_str(&format!("{}. **[{}]** ", i + 1, item.severity));

                if let Some(ref cat) = item.category {
                    output.push_str(&format!("({}): ", cat));
                }

                output.push_str(&item.message);

                if let Some(ref file) = item.file {
                    if let Some(line) = item.line {
                        output.push_str(&format!(" ({}:{})", file, line));
                    } else {
                        output.push_str(&format!(" ({})", file));
                    }
                }

                output.push('\n');

                if let Some(ref suggestion) = item.suggestion {
                    output.push_str(&format!("   Suggestion: {}\n", suggestion));
                }
            }
        }

        output
    }
}

/// Parser for extracting structured feedback from reviewer output
pub struct FeedbackParser {
    /// Whether to be lenient with parsing (try to extract what we can)
    lenient: bool,
}

impl Default for FeedbackParser {
    fn default() -> Self {
        Self::new()
    }
}

impl FeedbackParser {
    /// Create a new parser with default settings
    pub fn new() -> Self {
        Self { lenient: true }
    }

    /// Set lenient mode
    pub fn with_lenient(mut self, lenient: bool) -> Self {
        self.lenient = lenient;
        self
    }

    /// Parse feedback from raw reviewer output
    pub fn parse(&self, output: &str) -> ReviewFeedback {
        let mut feedback = ReviewFeedback::new();
        feedback.raw_output = output.to_string();

        // Try to detect the decision from keywords
        feedback.decision = self.detect_decision(output);

        // Extract summary (first paragraph or section)
        feedback.summary = self.extract_summary(output);

        // Extract individual feedback items
        feedback.items = self.extract_items(output);

        feedback
    }

    /// Detect the review decision from the output
    fn detect_decision(&self, output: &str) -> ReviewDecision {
        let lower = output.to_lowercase();

        // Look for explicit decision markers
        if lower.contains("approved") || lower.contains("lgtm") || lower.contains("looks good") {
            if lower.contains("with suggestions")
                || lower.contains("minor")
                || lower.contains("nit")
            {
                ReviewDecision::ApprovedWithSuggestions
            } else {
                ReviewDecision::Approved
            }
        } else if lower.contains("rejected") || lower.contains("do not merge") {
            ReviewDecision::Rejected
        } else if lower.contains("changes requested")
            || lower.contains("needs work")
            || lower.contains("please fix")
            || lower.contains("must be")
            || lower.contains("should be")
        {
            ReviewDecision::ChangesRequested
        } else {
            // Default to changes requested if we can't determine
            ReviewDecision::ChangesRequested
        }
    }

    /// Extract a summary from the output
    fn extract_summary(&self, output: &str) -> String {
        // Look for a summary section
        let lines: Vec<&str> = output.lines().collect();

        // Try to find a summary header
        for (i, line) in lines.iter().enumerate() {
            let lower = line.to_lowercase();
            if lower.contains("## summary")
                || lower.contains("## overview")
                || lower.contains("### summary")
            {
                // Collect lines until the next header
                let mut summary = String::new();
                for line in lines.iter().skip(i + 1) {
                    if line.starts_with('#') {
                        break;
                    }
                    if !summary.is_empty() {
                        summary.push('\n');
                    }
                    summary.push_str(line);
                }
                return summary.trim().to_string();
            }
        }

        // If no summary section, take the first paragraph
        let mut summary = String::new();
        for line in lines {
            if line.trim().is_empty() && !summary.is_empty() {
                break;
            }
            if !line.starts_with('#') && !line.trim().is_empty() {
                if !summary.is_empty() {
                    summary.push(' ');
                }
                summary.push_str(line.trim());
            }
        }

        // Truncate if too long
        if summary.len() > 500 {
            summary.truncate(500);
            summary.push_str("...");
        }

        summary
    }

    /// Extract feedback items from the output
    fn extract_items(&self, output: &str) -> Vec<FeedbackItem> {
        let mut items = Vec::new();

        for line in output.lines() {
            // Look for bullet points with feedback
            let trimmed = line.trim();

            if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("• ")
            {
                let content = trimmed[2..].trim();
                if !content.is_empty() {
                    let item = self.parse_feedback_line(content);
                    items.push(item);
                }
            }
            // Look for numbered lists
            else if let Some(rest) = self.strip_numbered_prefix(trimmed) {
                if !rest.is_empty() {
                    let item = self.parse_feedback_line(rest);
                    items.push(item);
                }
            }
        }

        items
    }

    /// Strip a numbered list prefix (e.g., "1. ", "1) ")
    fn strip_numbered_prefix<'a>(&self, line: &'a str) -> Option<&'a str> {
        let mut chars = line.char_indices();

        // Skip digits
        loop {
            match chars.next() {
                Some((_, c)) if c.is_ascii_digit() => continue,
                Some((i, '.')) | Some((i, ')')) => {
                    let rest = &line[i + 1..].trim_start();
                    return Some(rest);
                }
                _ => return None,
            }
        }
    }

    /// Parse a single feedback line into a FeedbackItem
    fn parse_feedback_line(&self, content: &str) -> FeedbackItem {
        let mut item = FeedbackItem::new(content);

        // Detect severity from keywords
        let lower = content.to_lowercase();

        if lower.contains("[critical]")
            || lower.contains("critical:")
            || lower.contains("security vulnerability")
            || lower.contains("must fix")
        {
            item.severity = FeedbackSeverity::Critical;
        } else if lower.contains("[major]")
            || lower.contains("major:")
            || lower.contains("should fix")
            || lower.contains("important:")
        {
            item.severity = FeedbackSeverity::Major;
        } else if lower.contains("[minor]")
            || lower.contains("minor:")
            || lower.contains("nit:")
            || lower.contains("nitpick:")
        {
            item.severity = FeedbackSeverity::Minor;
        } else if lower.contains("[suggestion]")
            || lower.contains("suggestion:")
            || lower.contains("consider:")
            || lower.contains("could")
        {
            item.severity = FeedbackSeverity::Suggestion;
        } else if lower.contains("good")
            || lower.contains("well done")
            || lower.contains("nice")
            || lower.contains("excellent")
        {
            item.severity = FeedbackSeverity::Praise;
        }

        // Try to detect category
        if lower.contains("security") || lower.contains("vulnerability") {
            item.category = Some("security".to_string());
        } else if lower.contains("performance")
            || lower.contains("slow")
            || lower.contains("optimize")
        {
            item.category = Some("performance".to_string());
        } else if lower.contains("style")
            || lower.contains("formatting")
            || lower.contains("naming")
        {
            item.category = Some("style".to_string());
        } else if lower.contains("test") || lower.contains("coverage") {
            item.category = Some("testing".to_string());
        } else if lower.contains("doc") || lower.contains("comment") {
            item.category = Some("documentation".to_string());
        } else if lower.contains("error") || lower.contains("exception") || lower.contains("panic")
        {
            item.category = Some("error-handling".to_string());
        }

        // Try to extract file path (pattern: `path/to/file.rs` or path/to/file.rs:123)
        if let Some(file_match) = self.extract_file_reference(content) {
            item.file = Some(file_match.0);
            item.line = file_match.1;
        }

        item
    }

    /// Extract a file reference from text
    fn extract_file_reference(&self, text: &str) -> Option<(String, Option<u32>)> {
        // Look for backtick-quoted paths or paths with line numbers
        let patterns = [
            // `path/to/file.rs:123`
            r"`([^`]+\.[a-z]+):(\d+)`",
            // `path/to/file.rs`
            r"`([^`]+\.[a-z]+)`",
            // path/to/file.rs:123
            r"([a-zA-Z0-9_/.-]+\.[a-z]+):(\d+)",
        ];

        for pattern in patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if let Some(caps) = re.captures(text) {
                    let file = caps.get(1)?.as_str().to_string();
                    let line = caps.get(2).and_then(|m| m.as_str().parse().ok());
                    return Some((file, line));
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feedback_severity_display() {
        assert_eq!(FeedbackSeverity::Critical.to_string(), "critical");
        assert_eq!(FeedbackSeverity::Major.to_string(), "major");
        assert_eq!(FeedbackSeverity::Minor.to_string(), "minor");
        assert_eq!(FeedbackSeverity::Suggestion.to_string(), "suggestion");
        assert_eq!(FeedbackSeverity::Praise.to_string(), "praise");
    }

    #[test]
    fn test_feedback_item_builder() {
        let item = FeedbackItem::new("Fix this bug")
            .with_severity(FeedbackSeverity::Critical)
            .with_category("security")
            .with_file("src/auth.rs")
            .with_line(42)
            .with_suggestion("Use constant-time comparison");

        assert_eq!(item.message, "Fix this bug");
        assert_eq!(item.severity, FeedbackSeverity::Critical);
        assert_eq!(item.category, Some("security".to_string()));
        assert_eq!(item.file, Some("src/auth.rs".to_string()));
        assert_eq!(item.line, Some(42));
        assert!(item.is_blocking());
    }

    #[test]
    fn test_feedback_item_is_blocking() {
        assert!(FeedbackItem::new("test")
            .with_severity(FeedbackSeverity::Critical)
            .is_blocking());
        assert!(FeedbackItem::new("test")
            .with_severity(FeedbackSeverity::Major)
            .is_blocking());
        assert!(!FeedbackItem::new("test")
            .with_severity(FeedbackSeverity::Minor)
            .is_blocking());
        assert!(!FeedbackItem::new("test")
            .with_severity(FeedbackSeverity::Suggestion)
            .is_blocking());
        assert!(!FeedbackItem::new("test")
            .with_severity(FeedbackSeverity::Praise)
            .is_blocking());
    }

    #[test]
    fn test_review_decision_can_proceed() {
        assert!(ReviewDecision::Approved.can_proceed());
        assert!(ReviewDecision::ApprovedWithSuggestions.can_proceed());
        assert!(!ReviewDecision::ChangesRequested.can_proceed());
        assert!(!ReviewDecision::Rejected.can_proceed());
    }

    #[test]
    fn test_review_feedback_blocking_items() {
        let mut feedback = ReviewFeedback::new();
        feedback.add_item(
            FeedbackItem::new("Critical issue").with_severity(FeedbackSeverity::Critical),
        );
        feedback.add_item(FeedbackItem::new("Minor issue").with_severity(FeedbackSeverity::Minor));
        feedback.add_item(FeedbackItem::new("Major issue").with_severity(FeedbackSeverity::Major));

        let blocking = feedback.blocking_items();
        assert_eq!(blocking.len(), 2);
        assert!(feedback.has_blocking_issues());
    }

    #[test]
    fn test_review_feedback_can_proceed() {
        let mut approved = ReviewFeedback::with_decision(ReviewDecision::Approved, "LGTM");
        assert!(approved.can_proceed());

        // Add a blocking issue
        approved.add_item(FeedbackItem::new("Critical").with_severity(FeedbackSeverity::Critical));
        assert!(!approved.can_proceed());

        let changes = ReviewFeedback::with_decision(ReviewDecision::ChangesRequested, "Needs work");
        assert!(!changes.can_proceed());
    }

    #[test]
    fn test_review_feedback_count_by_severity() {
        let mut feedback = ReviewFeedback::new();
        feedback.add_item(FeedbackItem::new("1").with_severity(FeedbackSeverity::Critical));
        feedback.add_item(FeedbackItem::new("2").with_severity(FeedbackSeverity::Critical));
        feedback.add_item(FeedbackItem::new("3").with_severity(FeedbackSeverity::Minor));

        let counts = feedback.count_by_severity();
        assert_eq!(counts.get(&FeedbackSeverity::Critical), Some(&2));
        assert_eq!(counts.get(&FeedbackSeverity::Minor), Some(&1));
        assert_eq!(counts.get(&FeedbackSeverity::Major), None);
    }

    #[test]
    fn test_parser_detect_decision_approved() {
        let parser = FeedbackParser::new();

        assert_eq!(parser.detect_decision("LGTM!"), ReviewDecision::Approved);
        assert_eq!(parser.detect_decision("Approved"), ReviewDecision::Approved);
        assert_eq!(
            parser.detect_decision("Looks good to me"),
            ReviewDecision::Approved
        );
    }

    #[test]
    fn test_parser_detect_decision_approved_with_suggestions() {
        let parser = FeedbackParser::new();

        assert_eq!(
            parser.detect_decision("Approved with minor suggestions"),
            ReviewDecision::ApprovedWithSuggestions
        );
        assert_eq!(
            parser.detect_decision("LGTM with some nits"),
            ReviewDecision::ApprovedWithSuggestions
        );
    }

    #[test]
    fn test_parser_detect_decision_changes_requested() {
        let parser = FeedbackParser::new();

        assert_eq!(
            parser.detect_decision("Changes requested"),
            ReviewDecision::ChangesRequested
        );
        assert_eq!(
            parser.detect_decision("This needs work"),
            ReviewDecision::ChangesRequested
        );
        assert_eq!(
            parser.detect_decision("Please fix the issues below"),
            ReviewDecision::ChangesRequested
        );
    }

    #[test]
    fn test_parser_detect_decision_rejected() {
        let parser = FeedbackParser::new();

        assert_eq!(parser.detect_decision("Rejected"), ReviewDecision::Rejected);
        assert_eq!(
            parser.detect_decision("Do not merge this"),
            ReviewDecision::Rejected
        );
    }

    #[test]
    fn test_parser_extract_summary() {
        let parser = FeedbackParser::new();

        let output = "## Summary\nThis is a good implementation.\n\n## Issues\n- Fix bug";
        assert_eq!(
            parser.extract_summary(output),
            "This is a good implementation."
        );

        let output2 = "First line summary.\n\nMore details here.";
        assert_eq!(parser.extract_summary(output2), "First line summary.");
    }

    #[test]
    fn test_parser_extract_items() {
        let parser = FeedbackParser::new();

        let output = "## Issues\n- Fix the security vulnerability\n- Minor: Add documentation\n* Consider using a different algorithm";
        let items = parser.extract_items(output);

        assert_eq!(items.len(), 3);
        assert_eq!(items[0].message, "Fix the security vulnerability");
        assert_eq!(items[1].severity, FeedbackSeverity::Minor);
    }

    #[test]
    fn test_parser_extract_numbered_items() {
        let parser = FeedbackParser::new();

        let output = "## Issues\n1. First issue\n2. Second issue\n3) Third issue";
        let items = parser.extract_items(output);

        assert_eq!(items.len(), 3);
        assert_eq!(items[0].message, "First issue");
        assert_eq!(items[1].message, "Second issue");
        assert_eq!(items[2].message, "Third issue");
    }

    #[test]
    fn test_parser_detect_severity_from_content() {
        let parser = FeedbackParser::new();

        let items = parser
            .extract_items("- [Critical] Security issue\n- nit: formatting\n- This looks good");

        assert_eq!(items[0].severity, FeedbackSeverity::Critical);
        assert_eq!(items[1].severity, FeedbackSeverity::Minor);
        assert_eq!(items[2].severity, FeedbackSeverity::Praise);
    }

    #[test]
    fn test_parser_detect_category() {
        let parser = FeedbackParser::new();

        let items = parser.extract_items(
            "- Security vulnerability in auth\n- Performance issue\n- Missing test coverage",
        );

        assert_eq!(items[0].category, Some("security".to_string()));
        assert_eq!(items[1].category, Some("performance".to_string()));
        assert_eq!(items[2].category, Some("testing".to_string()));
    }

    #[test]
    fn test_parser_full_parse() {
        let parser = FeedbackParser::new();

        let output = r#"## Summary
This PR implements the login feature. Overall it looks good with a few issues.

## Review Decision: Approved with suggestions

## Issues
- [Critical] SQL injection vulnerability in `src/auth.rs:42`
- Minor: Consider adding more tests
- Good use of error handling throughout

## Conclusion
LGTM with the critical fix.
"#;

        let feedback = parser.parse(output);

        assert_eq!(feedback.decision, ReviewDecision::ApprovedWithSuggestions);
        assert!(feedback.summary.contains("login feature"));
        assert_eq!(feedback.items.len(), 3);
        assert!(feedback.has_blocking_issues());
        assert!(!feedback.can_proceed()); // Has blocking issues
    }

    #[test]
    fn test_feedback_to_string() {
        let mut feedback =
            ReviewFeedback::with_decision(ReviewDecision::ChangesRequested, "Need fixes");
        feedback.add_item(
            FeedbackItem::new("Fix the bug")
                .with_severity(FeedbackSeverity::Critical)
                .with_file("src/main.rs")
                .with_line(10),
        );
        feedback.add_item(
            FeedbackItem::new("Add tests")
                .with_severity(FeedbackSeverity::Minor)
                .with_category("testing"),
        );

        let output = feedback.to_feedback_string();
        assert!(output.contains("changes requested"));
        assert!(output.contains("[critical]"));
        assert!(output.contains("src/main.rs:10"));
        assert!(output.contains("(testing)"));
    }
}
