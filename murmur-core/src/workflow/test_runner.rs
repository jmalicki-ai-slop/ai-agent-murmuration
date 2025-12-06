//! Test runner integration for TDD workflow
//!
//! This module provides framework detection and test execution capabilities
//! to validate VerifyRed and VerifyGreen phases.

use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Supported test frameworks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestFramework {
    /// Rust/Cargo test runner
    Cargo,
    /// Python pytest
    Pytest,
    /// Python unittest
    PythonUnittest,
    /// JavaScript Jest
    Jest,
    /// JavaScript Mocha
    Mocha,
    /// JavaScript Vitest
    Vitest,
    /// Go test
    Go,
}

impl TestFramework {
    /// Detect the test framework from project files in the given directory
    pub fn detect(path: &Path) -> Option<Self> {
        // Check for Rust/Cargo
        if path.join("Cargo.toml").exists() {
            return Some(Self::Cargo);
        }

        // Check for Go
        if path.join("go.mod").exists() {
            return Some(Self::Go);
        }

        // Check for Python (pytest or unittest)
        if path.join("pytest.ini").exists()
            || path.join("pyproject.toml").exists() && Self::has_pytest_config(path)
            || path.join("setup.cfg").exists() && Self::has_pytest_in_setup_cfg(path)
            || path.join("conftest.py").exists()
        {
            return Some(Self::Pytest);
        }

        // Fallback to Python unittest if Python files exist
        if path.join("setup.py").exists()
            || path.join("pyproject.toml").exists()
            || path.join("requirements.txt").exists()
        {
            return Some(Self::PythonUnittest);
        }

        // Check for JavaScript/TypeScript
        if path.join("package.json").exists() {
            return Self::detect_js_framework(path);
        }

        None
    }

    /// Check if pyproject.toml has pytest config
    fn has_pytest_config(path: &Path) -> bool {
        let pyproject = path.join("pyproject.toml");
        if let Ok(content) = std::fs::read_to_string(pyproject) {
            content.contains("[tool.pytest")
        } else {
            false
        }
    }

    /// Check if setup.cfg has pytest config
    fn has_pytest_in_setup_cfg(path: &Path) -> bool {
        let setup_cfg = path.join("setup.cfg");
        if let Ok(content) = std::fs::read_to_string(setup_cfg) {
            content.contains("[tool:pytest]")
        } else {
            false
        }
    }

    /// Detect JS test framework from package.json
    fn detect_js_framework(path: &Path) -> Option<Self> {
        let package_json = path.join("package.json");
        let content = std::fs::read_to_string(package_json).ok()?;

        // Check devDependencies and dependencies for test frameworks
        if content.contains("\"vitest\"") {
            return Some(Self::Vitest);
        }
        if content.contains("\"jest\"") {
            return Some(Self::Jest);
        }
        if content.contains("\"mocha\"") {
            return Some(Self::Mocha);
        }

        // Check scripts for test command hints
        if content.contains("vitest") {
            return Some(Self::Vitest);
        }
        if content.contains("jest") {
            return Some(Self::Jest);
        }
        if content.contains("mocha") {
            return Some(Self::Mocha);
        }

        // Default to Jest if none found but package.json exists
        Some(Self::Jest)
    }

    /// Get the command to run tests
    pub fn run_command(&self) -> Command {
        match self {
            Self::Cargo => {
                let mut cmd = Command::new("cargo");
                cmd.args(["test", "--no-fail-fast"]);
                cmd
            }
            Self::Pytest => {
                let mut cmd = Command::new("pytest");
                cmd.args(["--tb=short", "-v"]);
                cmd
            }
            Self::PythonUnittest => {
                let mut cmd = Command::new("python");
                cmd.args(["-m", "unittest", "discover", "-v"]);
                cmd
            }
            Self::Jest => {
                let mut cmd = Command::new("npx");
                cmd.args(["jest", "--passWithNoTests"]);
                cmd
            }
            Self::Mocha => {
                let mut cmd = Command::new("npx");
                cmd.args(["mocha"]);
                cmd
            }
            Self::Vitest => {
                let mut cmd = Command::new("npx");
                cmd.args(["vitest", "run"]);
                cmd
            }
            Self::Go => {
                let mut cmd = Command::new("go");
                cmd.args(["test", "-v", "./..."]);
                cmd
            }
        }
    }

    /// Get the name of the framework
    pub fn name(&self) -> &'static str {
        match self {
            Self::Cargo => "cargo test",
            Self::Pytest => "pytest",
            Self::PythonUnittest => "unittest",
            Self::Jest => "jest",
            Self::Mocha => "mocha",
            Self::Vitest => "vitest",
            Self::Go => "go test",
        }
    }
}

/// Results of a test run
#[derive(Debug, Clone)]
pub struct TestResults {
    /// Number of tests that passed
    pub passed: u32,
    /// Number of tests that failed
    pub failed: u32,
    /// Number of tests that were skipped
    pub skipped: u32,
    /// Duration of the test run in milliseconds
    pub duration_ms: u64,
    /// Raw output from the test run
    pub output: String,
    /// Error if the test command itself failed to execute
    pub execution_error: Option<String>,
}

impl TestResults {
    /// Create empty results (for when no tests exist)
    pub fn empty() -> Self {
        Self {
            passed: 0,
            failed: 0,
            skipped: 0,
            duration_ms: 0,
            output: String::new(),
            execution_error: None,
        }
    }

    /// Create results indicating an execution error
    pub fn with_error(error: String) -> Self {
        Self {
            passed: 0,
            failed: 0,
            skipped: 0,
            duration_ms: 0,
            output: String::new(),
            execution_error: Some(error),
        }
    }

    /// Check if tests are in "red" state (at least one failing)
    ///
    /// This is used by the VerifyRed phase to ensure tests fail before implementation.
    pub fn is_red(&self) -> bool {
        self.execution_error.is_none() && self.failed > 0
    }

    /// Check if tests are in "green" state (all passing, none failing)
    ///
    /// This is used by the VerifyGreen phase to ensure implementation is correct.
    pub fn is_green(&self) -> bool {
        self.execution_error.is_none() && self.failed == 0 && self.passed > 0
    }

    /// Check if no tests were found
    pub fn no_tests_found(&self) -> bool {
        self.execution_error.is_none() && self.passed == 0 && self.failed == 0 && self.skipped == 0
    }

    /// Get a summary string
    pub fn summary(&self) -> String {
        if let Some(ref error) = self.execution_error {
            return format!("Execution error: {}", error);
        }
        if self.no_tests_found() {
            return "No tests found".to_string();
        }
        format!(
            "{} passed, {} failed, {} skipped ({}ms)",
            self.passed, self.failed, self.skipped, self.duration_ms
        )
    }

    /// Get total number of tests
    pub fn total(&self) -> u32 {
        self.passed + self.failed + self.skipped
    }
}

/// Test runner that executes tests and parses results
pub struct TestRunner {
    workdir: std::path::PathBuf,
    framework: Option<TestFramework>,
    filter: Option<String>,
    timeout: Duration,
}

impl TestRunner {
    /// Create a new test runner for the given directory
    ///
    /// Automatically detects the test framework from project files.
    pub fn new(workdir: impl Into<std::path::PathBuf>) -> Self {
        let workdir = workdir.into();
        let framework = TestFramework::detect(&workdir);
        Self {
            workdir,
            framework,
            filter: None,
            timeout: Duration::from_secs(300), // 5 minute default timeout
        }
    }

    /// Set a test filter (e.g., test name pattern)
    pub fn with_filter(mut self, filter: impl Into<String>) -> Self {
        self.filter = Some(filter.into());
        self
    }

    /// Set the timeout for test execution
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Explicitly set the test framework
    pub fn with_framework(mut self, framework: TestFramework) -> Self {
        self.framework = Some(framework);
        self
    }

    /// Get the detected framework
    pub fn framework(&self) -> Option<TestFramework> {
        self.framework
    }

    /// Run tests and return results
    pub fn run(&self) -> TestResults {
        let Some(framework) = self.framework else {
            return TestResults::with_error("No test framework detected".to_string());
        };

        let mut cmd = framework.run_command();
        cmd.current_dir(&self.workdir);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // Add filter if specified
        if let Some(ref filter) = self.filter {
            match framework {
                TestFramework::Cargo => {
                    cmd.arg(filter);
                }
                TestFramework::Pytest => {
                    cmd.args(["-k", filter]);
                }
                TestFramework::Jest | TestFramework::Vitest => {
                    cmd.args(["-t", filter]);
                }
                TestFramework::Go => {
                    cmd.args(["-run", filter]);
                }
                TestFramework::PythonUnittest => {
                    cmd.args(["-k", filter]);
                }
                TestFramework::Mocha => {
                    cmd.args(["--grep", filter]);
                }
            }
        }

        let start = Instant::now();
        let output = match cmd.output() {
            Ok(output) => output,
            Err(e) => return TestResults::with_error(format!("Failed to run tests: {}", e)),
        };
        let duration_ms = start.elapsed().as_millis() as u64;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let combined_output = format!("{}\n{}", stdout, stderr);

        // Parse results based on framework
        let mut results = match framework {
            TestFramework::Cargo => parse_cargo_output(&stdout, &stderr),
            TestFramework::Pytest => parse_pytest_output(&stdout),
            TestFramework::PythonUnittest => parse_unittest_output(&stdout, &stderr),
            TestFramework::Jest | TestFramework::Vitest => parse_jest_output(&stdout, &stderr),
            TestFramework::Mocha => parse_mocha_output(&stdout),
            TestFramework::Go => parse_go_output(&stdout, &stderr),
        };

        results.duration_ms = duration_ms;
        results.output = combined_output;

        // If output parsing found no results but exit code indicates failure,
        // mark as at least one failure
        if results.no_tests_found() && !output.status.success() {
            results.failed = 1;
        }

        results
    }
}

/// Parse Cargo test output
fn parse_cargo_output(stdout: &str, stderr: &str) -> TestResults {
    let mut passed = 0u32;
    let mut failed = 0u32;
    let mut ignored = 0u32;

    // Look for the summary line: "test result: ok. X passed; Y failed; Z ignored"
    for line in stdout.lines().chain(stderr.lines()) {
        if line.starts_with("test result:") {
            // Parse "test result: ok. 5 passed; 0 failed; 1 ignored"
            // Extract everything after "test result:" and split by ';'
            if let Some(rest) = line.strip_prefix("test result:") {
                for part in rest.split(';') {
                    let part = part.trim();
                    // Extract the number before "passed", "failed", or "ignored"
                    let words: Vec<&str> = part.split_whitespace().collect();
                    for (i, word) in words.iter().enumerate() {
                        if *word == "passed" && i > 0 {
                            passed += words[i - 1].parse::<u32>().unwrap_or(0);
                        } else if *word == "failed" && i > 0 {
                            failed += words[i - 1].parse::<u32>().unwrap_or(0);
                        } else if *word == "ignored" && i > 0 {
                            ignored += words[i - 1].parse::<u32>().unwrap_or(0);
                        }
                    }
                }
            }
        }
    }

    TestResults {
        passed,
        failed,
        skipped: ignored,
        duration_ms: 0,
        output: String::new(),
        execution_error: None,
    }
}

/// Parse pytest output
fn parse_pytest_output(stdout: &str) -> TestResults {
    let mut passed = 0u32;
    let mut failed = 0u32;
    let mut skipped = 0u32;

    // Look for summary line: "=== 5 passed, 2 failed, 1 skipped ==="
    for line in stdout.lines() {
        // Check for the summary line with === markers and result keywords
        if (line.contains("passed") || line.contains("failed") || line.contains("skipped"))
            && line.contains("===")
        {
            // Parse "=== 5 passed, 2 failed, 1 skipped ===" or "========================= 2 passed, 2 failed, 1 skipped ========================"
            // Remove leading/trailing = signs and split by comma
            let trimmed = line.trim().trim_matches('=').trim();
            for part in trimmed.split(',') {
                let part = part.trim();
                // Find the number before the keyword
                let words: Vec<&str> = part.split_whitespace().collect();
                for (i, word) in words.iter().enumerate() {
                    if *word == "passed" && i > 0 {
                        passed += words[i - 1].parse::<u32>().unwrap_or(0);
                    } else if *word == "failed" && i > 0 {
                        failed += words[i - 1].parse::<u32>().unwrap_or(0);
                    } else if *word == "skipped" && i > 0 {
                        skipped += words[i - 1].parse::<u32>().unwrap_or(0);
                    }
                }
            }
        }
    }

    TestResults {
        passed,
        failed,
        skipped,
        duration_ms: 0,
        output: String::new(),
        execution_error: None,
    }
}

/// Parse Python unittest output
fn parse_unittest_output(stdout: &str, stderr: &str) -> TestResults {
    let mut passed = 0u32;
    let mut failed = 0u32;
    let mut errors = 0u32;
    let mut skipped = 0u32;

    let combined = format!("{}\n{}", stdout, stderr);

    // Look for "Ran X tests" and "OK" or "FAILED"
    for line in combined.lines() {
        if line.starts_with("Ran ") && line.contains(" test") {
            // "Ran 5 tests in 0.001s"
            if let Some(num) = line
                .strip_prefix("Ran ")
                .and_then(|s| s.split_whitespace().next())
            {
                let total: u32 = num.parse().unwrap_or(0);
                if combined.contains("OK") && !combined.contains("FAILED") {
                    passed = total;
                }
            }
        }
        if line.contains("FAILED") {
            // "FAILED (failures=2, errors=1)"
            if let Some(start) = line.find('(') {
                if let Some(end) = line.find(')') {
                    let inner = &line[start + 1..end];
                    for part in inner.split(',') {
                        let part = part.trim();
                        if part.starts_with("failures=") {
                            if let Some(num) = part.strip_prefix("failures=") {
                                failed = num.parse().unwrap_or(0);
                            }
                        } else if part.starts_with("errors=") {
                            if let Some(num) = part.strip_prefix("errors=") {
                                errors = num.parse().unwrap_or(0);
                            }
                        } else if part.starts_with("skipped=") {
                            if let Some(num) = part.strip_prefix("skipped=") {
                                skipped = num.parse().unwrap_or(0);
                            }
                        }
                    }
                }
            }
        }
    }

    TestResults {
        passed,
        failed: failed + errors,
        skipped,
        duration_ms: 0,
        output: String::new(),
        execution_error: None,
    }
}

/// Parse Jest/Vitest output
fn parse_jest_output(stdout: &str, stderr: &str) -> TestResults {
    let mut passed = 0u32;
    let mut failed = 0u32;
    let mut skipped = 0u32;

    let combined = format!("{}\n{}", stdout, stderr);

    // Look for "Tests:       2 passed, 1 skipped, 3 total"
    for line in combined.lines() {
        if line.contains("Tests:") {
            // Strip "Tests:" prefix and parse the rest
            if let Some(rest) = line.split("Tests:").nth(1) {
                for part in rest.split(',') {
                    let part = part.trim();
                    // Find the number before the keyword
                    let words: Vec<&str> = part.split_whitespace().collect();
                    for (i, word) in words.iter().enumerate() {
                        if *word == "passed" && i > 0 {
                            passed += words[i - 1].parse::<u32>().unwrap_or(0);
                        } else if *word == "failed" && i > 0 {
                            failed += words[i - 1].parse::<u32>().unwrap_or(0);
                        } else if (*word == "skipped" || *word == "pending") && i > 0 {
                            skipped += words[i - 1].parse::<u32>().unwrap_or(0);
                        }
                    }
                }
            }
        }
    }

    TestResults {
        passed,
        failed,
        skipped,
        duration_ms: 0,
        output: String::new(),
        execution_error: None,
    }
}

/// Parse Mocha output
fn parse_mocha_output(stdout: &str) -> TestResults {
    let mut passed = 0u32;
    let mut failed = 0u32;
    let mut pending = 0u32;

    // Mocha output: "  5 passing (10ms)"
    //               "  2 failing"
    //               "  1 pending"
    for line in stdout.lines() {
        let line = line.trim();
        if line.contains("passing") {
            if let Some(num) = line.split_whitespace().next() {
                passed = num.parse().unwrap_or(0);
            }
        } else if line.contains("failing") {
            if let Some(num) = line.split_whitespace().next() {
                failed = num.parse().unwrap_or(0);
            }
        } else if line.contains("pending") {
            if let Some(num) = line.split_whitespace().next() {
                pending = num.parse().unwrap_or(0);
            }
        }
    }

    TestResults {
        passed,
        failed,
        skipped: pending,
        duration_ms: 0,
        output: String::new(),
        execution_error: None,
    }
}

/// Parse Go test output
fn parse_go_output(stdout: &str, stderr: &str) -> TestResults {
    let mut passed = 0u32;
    let mut failed = 0u32;
    let mut skipped = 0u32;

    let combined = format!("{}\n{}", stdout, stderr);

    // Go test output has individual lines like:
    // "--- PASS: TestFoo (0.00s)"
    // "--- FAIL: TestBar (0.01s)"
    // "--- SKIP: TestBaz (0.00s)"
    for line in combined.lines() {
        if line.contains("--- PASS:") {
            passed += 1;
        } else if line.contains("--- FAIL:") {
            failed += 1;
        } else if line.contains("--- SKIP:") {
            skipped += 1;
        }
    }

    // Also check for "ok" and "FAIL" package summaries
    // "ok      package/name    0.005s"
    // "FAIL    package/name    0.005s"
    if passed == 0 && failed == 0 {
        for line in combined.lines() {
            if line.starts_with("ok") || line.starts_with("FAIL") {
                // If we see package summaries but no individual test results,
                // at least count the packages
                if line.starts_with("ok") {
                    passed += 1;
                } else {
                    failed += 1;
                }
            }
        }
    }

    TestResults {
        passed,
        failed,
        skipped,
        duration_ms: 0,
        output: String::new(),
        execution_error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_detect_cargo_project() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[package]").unwrap();
        assert_eq!(
            TestFramework::detect(dir.path()),
            Some(TestFramework::Cargo)
        );
    }

    #[test]
    fn test_detect_go_project() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("go.mod"), "module test").unwrap();
        assert_eq!(TestFramework::detect(dir.path()), Some(TestFramework::Go));
    }

    #[test]
    fn test_detect_pytest_with_pytest_ini() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("pytest.ini"), "[pytest]").unwrap();
        assert_eq!(
            TestFramework::detect(dir.path()),
            Some(TestFramework::Pytest)
        );
    }

    #[test]
    fn test_detect_pytest_with_conftest() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("conftest.py"), "# conftest").unwrap();
        assert_eq!(
            TestFramework::detect(dir.path()),
            Some(TestFramework::Pytest)
        );
    }

    #[test]
    fn test_detect_jest_from_package_json() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("package.json"),
            r#"{"devDependencies": {"jest": "^29.0.0"}}"#,
        )
        .unwrap();
        assert_eq!(TestFramework::detect(dir.path()), Some(TestFramework::Jest));
    }

    #[test]
    fn test_detect_vitest_from_package_json() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("package.json"),
            r#"{"devDependencies": {"vitest": "^1.0.0"}}"#,
        )
        .unwrap();
        assert_eq!(
            TestFramework::detect(dir.path()),
            Some(TestFramework::Vitest)
        );
    }

    #[test]
    fn test_detect_mocha_from_package_json() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("package.json"),
            r#"{"devDependencies": {"mocha": "^10.0.0"}}"#,
        )
        .unwrap();
        assert_eq!(
            TestFramework::detect(dir.path()),
            Some(TestFramework::Mocha)
        );
    }

    #[test]
    fn test_detect_no_framework() {
        let dir = TempDir::new().unwrap();
        assert_eq!(TestFramework::detect(dir.path()), None);
    }

    #[test]
    fn test_parse_cargo_output_success() {
        let stdout = r#"
running 5 tests
test test_one ... ok
test test_two ... ok
test test_three ... ok
test test_four ... ok
test test_five ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
"#;
        let results = parse_cargo_output(stdout, "");
        assert_eq!(results.passed, 5);
        assert_eq!(results.failed, 0);
        assert!(results.is_green());
        assert!(!results.is_red());
    }

    #[test]
    fn test_parse_cargo_output_failure() {
        let stdout = r#"
running 5 tests
test test_one ... ok
test test_two ... FAILED
test test_three ... ok
test test_four ... FAILED
test test_five ... ok

test result: FAILED. 3 passed; 2 failed; 0 ignored
"#;
        let results = parse_cargo_output(stdout, "");
        assert_eq!(results.passed, 3);
        assert_eq!(results.failed, 2);
        assert!(results.is_red());
        assert!(!results.is_green());
    }

    #[test]
    fn test_parse_cargo_output_with_ignored() {
        let stdout = r#"
test result: ok. 10 passed; 0 failed; 3 ignored
"#;
        let results = parse_cargo_output(stdout, "");
        assert_eq!(results.passed, 10);
        assert_eq!(results.failed, 0);
        assert_eq!(results.skipped, 3);
    }

    #[test]
    fn test_parse_pytest_output_success() {
        let stdout = r#"
============================= test session starts ==============================
collected 5 items

test_example.py::test_one PASSED                                           [ 20%]
test_example.py::test_two PASSED                                           [ 40%]
test_example.py::test_three PASSED                                         [ 60%]
test_example.py::test_four PASSED                                          [ 80%]
test_example.py::test_five PASSED                                          [100%]

============================== 5 passed in 0.02s ===============================
"#;
        let results = parse_pytest_output(stdout);
        assert_eq!(results.passed, 5);
        assert_eq!(results.failed, 0);
        assert!(results.is_green());
    }

    #[test]
    fn test_parse_pytest_output_failure() {
        let stdout = r#"
============================= test session starts ==============================
collected 5 items

test_example.py::test_one PASSED                                           [ 20%]
test_example.py::test_two FAILED                                           [ 40%]
test_example.py::test_three PASSED                                         [ 60%]
test_example.py::test_four FAILED                                          [ 80%]
test_example.py::test_five SKIPPED                                         [100%]

========================= 2 passed, 2 failed, 1 skipped ========================
"#;
        let results = parse_pytest_output(stdout);
        assert_eq!(results.passed, 2);
        assert_eq!(results.failed, 2);
        assert_eq!(results.skipped, 1);
        assert!(results.is_red());
    }

    #[test]
    fn test_parse_jest_output() {
        let stdout = r#"
 PASS  tests/example.test.js
  ✓ test one (5 ms)
  ✓ test two (3 ms)
  ○ skipped test three

Tests:       2 passed, 1 skipped, 3 total
Snapshots:   0 total
Time:        1.234 s
"#;
        let results = parse_jest_output(stdout, "");
        assert_eq!(results.passed, 2);
        assert_eq!(results.skipped, 1);
    }

    #[test]
    fn test_parse_go_output() {
        let stdout = r#"
=== RUN   TestOne
--- PASS: TestOne (0.00s)
=== RUN   TestTwo
--- PASS: TestTwo (0.00s)
=== RUN   TestThree
--- FAIL: TestThree (0.01s)
=== RUN   TestFour
--- SKIP: TestFour (0.00s)
FAIL
"#;
        let results = parse_go_output(stdout, "");
        assert_eq!(results.passed, 2);
        assert_eq!(results.failed, 1);
        assert_eq!(results.skipped, 1);
        assert!(results.is_red());
    }

    #[test]
    fn test_parse_mocha_output() {
        let stdout = r#"
  Example Tests
    ✓ should pass test one
    ✓ should pass test two
    1) should fail test three
    - should skip test four

  2 passing (15ms)
  1 failing
  1 pending
"#;
        let results = parse_mocha_output(stdout);
        assert_eq!(results.passed, 2);
        assert_eq!(results.failed, 1);
        assert_eq!(results.skipped, 1);
    }

    #[test]
    fn test_results_is_red() {
        let mut results = TestResults::empty();
        assert!(!results.is_red());

        results.failed = 1;
        assert!(results.is_red());

        results.execution_error = Some("error".to_string());
        assert!(!results.is_red()); // Error means we can't trust the results
    }

    #[test]
    fn test_results_is_green() {
        let mut results = TestResults::empty();
        assert!(!results.is_green()); // No tests means not green

        results.passed = 5;
        assert!(results.is_green());

        results.failed = 1;
        assert!(!results.is_green());
    }

    #[test]
    fn test_results_summary() {
        let results = TestResults {
            passed: 10,
            failed: 2,
            skipped: 1,
            duration_ms: 150,
            output: String::new(),
            execution_error: None,
        };
        assert_eq!(results.summary(), "10 passed, 2 failed, 1 skipped (150ms)");
    }

    #[test]
    fn test_results_summary_with_error() {
        let results = TestResults::with_error("Test command not found".to_string());
        assert!(results.summary().contains("Execution error"));
    }

    #[test]
    fn test_results_no_tests_found() {
        let results = TestResults::empty();
        assert!(results.no_tests_found());
        assert_eq!(results.summary(), "No tests found");
    }

    #[test]
    fn test_runner_with_framework() {
        let dir = TempDir::new().unwrap();
        let runner = TestRunner::new(dir.path()).with_framework(TestFramework::Cargo);
        assert_eq!(runner.framework(), Some(TestFramework::Cargo));
    }

    #[test]
    fn test_runner_with_timeout() {
        let dir = TempDir::new().unwrap();
        let runner = TestRunner::new(dir.path()).with_timeout(Duration::from_secs(60));
        assert_eq!(runner.timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_runner_no_framework_detected() {
        let dir = TempDir::new().unwrap();
        let runner = TestRunner::new(dir.path());
        assert!(runner.framework().is_none());
        let results = runner.run();
        assert!(results.execution_error.is_some());
    }
}
