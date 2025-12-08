//! TDD Workflow Executor
//!
//! This module provides a shared executor for running TDD workflows.
//! It extracts the core TDD loop logic that was duplicated between
//! `murmur work --tdd` and `murmur tdd` commands.
//!
//! The executor handles:
//! - Phase iteration and transitions
//! - Test runner integration for VerifyRed/VerifyGreen phases
//! - Agent spawning for WriteSpec/WriteTests/Implement/Refactor phases
//! - Output streaming
//! - Progress reporting with emoji/ASCII support

use crate::agent::{AgentFactory, AgentType, OutputStreamer, StreamHandler};
use crate::config::AgentConfig;
use crate::error::{Error, Result};
use crate::secrets::Secrets;
use crate::workflow::tdd::{TddPhase, TddWorkflow};
use crate::workflow::test_runner::{TestFramework, TestRunner};
use std::path::{Path, PathBuf};

/// Configuration for TDD workflow execution
#[derive(Debug, Clone)]
pub struct TddExecutorConfig {
    /// Skip the WriteSpec phase
    pub skip_spec: bool,
    /// Skip the Refactor phase
    pub skip_refactor: bool,
    /// Maximum iterations for Implement->VerifyGreen loop
    pub max_iterations: u32,
    /// Whether to show verbose output
    pub verbose: bool,
    /// Whether to use ASCII instead of emoji
    pub no_emoji: bool,
    /// Agent configuration
    pub agent_config: AgentConfig,
}

impl Default for TddExecutorConfig {
    fn default() -> Self {
        Self {
            skip_spec: false,
            skip_refactor: false,
            max_iterations: 3,
            verbose: false,
            no_emoji: false,
            agent_config: AgentConfig::default(),
        }
    }
}

impl TddExecutorConfig {
    /// Create a new config with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set skip_spec option
    pub fn with_skip_spec(mut self, skip: bool) -> Self {
        self.skip_spec = skip;
        self
    }

    /// Set skip_refactor option
    pub fn with_skip_refactor(mut self, skip: bool) -> Self {
        self.skip_refactor = skip;
        self
    }

    /// Set max_iterations option
    pub fn with_max_iterations(mut self, max: u32) -> Self {
        self.max_iterations = max;
        self
    }

    /// Set verbose option
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Set no_emoji option
    pub fn with_no_emoji(mut self, no_emoji: bool) -> Self {
        self.no_emoji = no_emoji;
        self
    }

    /// Set agent configuration
    pub fn with_agent_config(mut self, config: AgentConfig) -> Self {
        self.agent_config = config;
        self
    }
}

/// Result of a single phase execution
#[derive(Debug, Clone)]
pub struct PhaseResult {
    /// The phase that was executed
    pub phase: TddPhase,
    /// Whether the phase succeeded
    pub success: bool,
    /// Optional message about the result
    pub message: Option<String>,
}

/// Callback trait for handling phase events during TDD execution
///
/// Implementors can use this to:
/// - Log phase starts and completions
/// - Track agent runs in a database
/// - Show progress to users
pub trait TddExecutorCallback: Send {
    /// Called when a phase starts
    fn on_phase_start(&mut self, phase: &TddPhase, phase_num: u32, total_phases: u32);

    /// Called when a phase completes
    fn on_phase_complete(&mut self, result: &PhaseResult);

    /// Called when an agent is about to be spawned for a phase
    /// Returns an optional run_id that can be used for tracking
    fn on_agent_start(&mut self, phase: &TddPhase, prompt: &str) -> Option<i64>;

    /// Called when an agent completes
    fn on_agent_complete(&mut self, run_id: Option<i64>, exit_code: i32);

    /// Called when test results are available
    fn on_test_results(&mut self, passed: u32, failed: u32, skipped: u32);

    /// Called when the workflow completes
    fn on_workflow_complete(&mut self, success: bool, iterations: u32);

    /// Get the stream handler for agent output
    /// This allows callbacks to provide custom stream handlers (e.g., for database logging)
    fn stream_handler(&mut self) -> Box<dyn StreamHandler + Send>;
}

/// A simple callback implementation that just prints to stdout
pub struct PrintCallback {
    verbose: bool,
    no_emoji: bool,
}

impl PrintCallback {
    /// Create a new print callback
    pub fn new(verbose: bool, no_emoji: bool) -> Self {
        Self { verbose, no_emoji }
    }
}

impl TddExecutorCallback for PrintCallback {
    fn on_phase_start(&mut self, phase: &TddPhase, phase_num: u32, total_phases: u32) {
        println!(
            "Phase {}/{}: {} {}",
            phase_num,
            total_phases,
            emoji(self.no_emoji, phase_emoji(phase), phase_ascii(phase)),
            phase.description()
        );
        println!();
    }

    fn on_phase_complete(&mut self, result: &PhaseResult) {
        if result.success {
            println!("{} Phase completed", emoji(self.no_emoji, "✅", "[OK]"));
        } else {
            let msg = result.message.as_deref().unwrap_or("Phase failed");
            println!("{} {}", emoji(self.no_emoji, "❌", "[FAIL]"), msg);
        }
    }

    fn on_agent_start(&mut self, _phase: &TddPhase, prompt: &str) -> Option<i64> {
        if self.verbose {
            println!("Prompt: {}", prompt);
            println!();
        }
        println!("Starting agent...");
        None
    }

    fn on_agent_complete(&mut self, _run_id: Option<i64>, _exit_code: i32) {
        // Nothing to do for simple print callback
    }

    fn on_test_results(&mut self, passed: u32, failed: u32, skipped: u32) {
        println!();
        println!(
            "Test results: {} passed, {} failed, {} skipped",
            passed, failed, skipped
        );
    }

    fn on_workflow_complete(&mut self, success: bool, iterations: u32) {
        println!("═══════════════════════════════════════");
        if success {
            println!(
                "{} TDD workflow completed successfully!",
                emoji(self.no_emoji, "🎉", "[DONE]")
            );
        } else {
            println!(
                "{} TDD workflow failed after {} iterations",
                emoji(self.no_emoji, "💥", "[FAIL]"),
                iterations
            );
        }
        println!("═══════════════════════════════════════");
    }

    fn stream_handler(&mut self) -> Box<dyn StreamHandler + Send> {
        Box::new(crate::agent::PrintHandler::new(self.verbose))
    }
}

/// TDD Workflow Executor
///
/// This is the core execution engine for TDD workflows. It handles:
/// - Phase transitions and iteration
/// - Test framework detection and execution
/// - Agent spawning and output streaming
/// - Progress callbacks
pub struct TddExecutor {
    /// The TDD workflow state
    workflow: TddWorkflow,
    /// Configuration options
    config: TddExecutorConfig,
    /// Working directory
    workdir: PathBuf,
    /// Test runner
    test_runner: TestRunner,
    /// Agent factory
    factory: AgentFactory,
}

impl TddExecutor {
    /// Create a new TDD executor
    pub fn new(behavior: impl Into<String>, workdir: impl Into<PathBuf>) -> Self {
        let workdir = workdir.into();
        let config = TddExecutorConfig::default();

        let workflow = TddWorkflow::new(behavior, &workdir);
        let test_runner = TestRunner::new(&workdir);
        let factory = AgentFactory::new();

        Self {
            workflow,
            config,
            workdir,
            test_runner,
            factory,
        }
    }

    /// Create a new TDD executor with configuration
    pub fn with_config(
        behavior: impl Into<String>,
        workdir: impl Into<PathBuf>,
        config: TddExecutorConfig,
    ) -> Self {
        let workdir = workdir.into();
        let behavior = behavior.into();

        let mut workflow = if config.skip_spec {
            TddWorkflow::new_without_spec(&behavior, &workdir)
        } else {
            TddWorkflow::with_config(&behavior, &workdir, config.agent_config.clone())
        };

        // Configure workflow state
        if config.skip_refactor {
            workflow.state_mut().skip_refactor = true;
        }
        workflow.state_mut().max_iterations = config.max_iterations;

        let test_runner = TestRunner::new(&workdir);
        let factory = AgentFactory::with_config(config.agent_config.clone());

        Self {
            workflow,
            config,
            workdir,
            test_runner,
            factory,
        }
    }

    /// Set the test framework explicitly
    pub fn with_framework(mut self, framework: TestFramework) -> Self {
        self.test_runner = self.test_runner.with_framework(framework);
        self
    }

    /// Get the detected test framework
    pub fn detected_framework(&self) -> Option<TestFramework> {
        self.test_runner.framework()
    }

    /// Get the current workflow state
    pub fn workflow(&self) -> &TddWorkflow {
        &self.workflow
    }

    /// Get the working directory
    pub fn workdir(&self) -> &Path {
        &self.workdir
    }

    /// Get the configuration
    pub fn config(&self) -> &TddExecutorConfig {
        &self.config
    }

    /// Get total number of phases based on configuration
    pub fn total_phases(&self) -> u32 {
        if self.config.skip_spec && self.config.skip_refactor {
            5 // WriteTests, VerifyRed, Implement, VerifyGreen, Complete
        } else if self.config.skip_spec || self.config.skip_refactor {
            6
        } else {
            7
        }
    }

    /// Execute the TDD workflow
    ///
    /// This runs the complete TDD cycle, calling the provided callback
    /// at each phase transition.
    ///
    /// Returns `true` if the workflow completed successfully, `false` otherwise.
    pub async fn execute(&mut self, callback: &mut dyn TddExecutorCallback) -> Result<bool> {
        let total_phases = self.total_phases();

        while !self.workflow.is_complete() && !self.workflow.should_give_up() {
            let phase = self.workflow.phase();
            let phase_num = phase_number(&phase, self.config.skip_spec);

            callback.on_phase_start(&phase, phase_num, total_phases);

            let result = match phase {
                TddPhase::WriteSpec
                | TddPhase::WriteTests
                | TddPhase::Implement
                | TddPhase::Refactor => self.execute_agent_phase(&phase, callback).await?,
                TddPhase::VerifyRed => self.execute_verify_red(callback),
                TddPhase::VerifyGreen => self.execute_verify_green(callback),
                TddPhase::Complete => {
                    // Should not reach here due to while condition
                    break;
                }
            };

            callback.on_phase_complete(&result);

            if !result.success {
                match phase {
                    TddPhase::WriteSpec
                    | TddPhase::WriteTests
                    | TddPhase::Implement
                    | TddPhase::Refactor => {
                        // Agent failure - return immediately
                        callback.on_workflow_complete(false, self.workflow.state().iterations);
                        return Ok(false);
                    }
                    TddPhase::VerifyRed | TddPhase::VerifyGreen | TddPhase::Complete => {
                        // Handled by retry logic in execute_verify_* methods
                    }
                }
            }

            println!();
        }

        // Final status
        let success = self.workflow.is_complete();
        callback.on_workflow_complete(success, self.workflow.state().iterations);

        Ok(success)
    }

    /// Execute an agent-driven phase
    async fn execute_agent_phase(
        &mut self,
        phase: &TddPhase,
        callback: &mut dyn TddExecutorCallback,
    ) -> Result<PhaseResult> {
        let prompt = self.workflow.current_prompt();
        let run_id = callback.on_agent_start(phase, &prompt);

        // Choose agent type based on phase
        let agent_type = match phase {
            TddPhase::WriteTests => AgentType::Test,
            _ => AgentType::Implement,
        };

        let mut typed_agent = self.factory.create(agent_type);

        // Pass GitHub token to agent via environment variable
        if let Ok(secrets) = Secrets::load() {
            if let Some(token) = secrets.github_token() {
                typed_agent = typed_agent.with_env("GITHUB_TOKEN", token);
            }
        }

        // Spawn and run agent
        let mut handle = typed_agent.spawn_with_task(&prompt, &self.workdir).await?;

        let stdout = handle
            .child_mut()
            .stdout
            .take()
            .ok_or_else(|| Error::Agent("Failed to capture agent stdout".to_string()))?;

        // Stream output
        let mut streamer = OutputStreamer::new(stdout);
        let mut handler = callback.stream_handler();
        if let Err(e) = streamer.stream(handler.as_mut()).await {
            eprintln!("Stream error: {}", e);
        }

        let status = handle.wait().await?;
        let exit_code = status.code().unwrap_or(-1);

        callback.on_agent_complete(run_id, exit_code);

        if status.success() {
            println!();
            self.workflow.advance(true, None);
            Ok(PhaseResult {
                phase: *phase,
                success: true,
                message: None,
            })
        } else {
            println!();
            Ok(PhaseResult {
                phase: *phase,
                success: false,
                message: Some(format!("Agent exited with status: {}", status)),
            })
        }
    }

    /// Execute the VerifyRed phase
    fn execute_verify_red(&mut self, callback: &mut dyn TddExecutorCallback) -> PhaseResult {
        println!("Running tests (expecting failures)...");
        let results = self.test_runner.run();

        callback.on_test_results(results.passed, results.failed, results.skipped);

        if results.is_red() {
            println!();
            println!(
                "{} Tests failed as expected (red phase)",
                emoji(self.config.no_emoji, "✅", "[OK]")
            );
            self.workflow.advance(true, None);
            PhaseResult {
                phase: TddPhase::VerifyRed,
                success: true,
                message: None,
            }
        } else if results.passed > 0 && results.failed == 0 {
            println!();
            println!(
                "{} Tests passed unexpectedly - tests may not be testing new behavior",
                emoji(self.config.no_emoji, "⚠️", "[WARN]")
            );
            println!("Going back to WriteTests phase...");
            self.workflow
                .retry_tests(Some("Tests passed unexpectedly".to_string()));
            PhaseResult {
                phase: TddPhase::VerifyRed,
                success: false,
                message: Some("Tests passed unexpectedly".to_string()),
            }
        } else {
            println!();
            println!(
                "{} No tests found or error running tests",
                emoji(self.config.no_emoji, "❌", "[FAIL]")
            );
            self.workflow
                .retry_tests(Some("No tests found".to_string()));
            PhaseResult {
                phase: TddPhase::VerifyRed,
                success: false,
                message: Some("No tests found".to_string()),
            }
        }
    }

    /// Execute the VerifyGreen phase
    fn execute_verify_green(&mut self, callback: &mut dyn TddExecutorCallback) -> PhaseResult {
        let iteration = self.workflow.state().iterations;
        println!(
            "Running tests (iteration {}/{})...",
            iteration + 1,
            self.config.max_iterations
        );
        let results = self.test_runner.run();

        callback.on_test_results(results.passed, results.failed, results.skipped);

        if results.is_green() {
            println!();
            println!(
                "{} All tests pass (green phase)",
                emoji(self.config.no_emoji, "✅", "[OK]")
            );
            self.workflow.advance(true, None);
            PhaseResult {
                phase: TddPhase::VerifyGreen,
                success: true,
                message: None,
            }
        } else {
            println!();
            println!(
                "{} {} tests still failing",
                emoji(self.config.no_emoji, "❌", "[FAIL]"),
                results.failed
            );

            if self.workflow.state().iterations + 1 >= self.config.max_iterations {
                println!();
                println!(
                    "{} Maximum iterations reached, giving up",
                    emoji(self.config.no_emoji, "🛑", "[STOP]")
                );
                // Don't retry, let the workflow fail
                PhaseResult {
                    phase: TddPhase::VerifyGreen,
                    success: false,
                    message: Some("Maximum iterations reached".to_string()),
                }
            } else {
                println!("Returning to Implement phase...");
                self.workflow
                    .retry_implement(Some(format!("{} tests failing", results.failed)));
                PhaseResult {
                    phase: TddPhase::VerifyGreen,
                    success: false,
                    message: Some(format!("{} tests still failing", results.failed)),
                }
            }
        }
    }
}

/// Get the phase number for display
///
/// When `skip_spec` is true, phase numbers are adjusted so WriteTests becomes phase 1.
pub fn phase_number(phase: &TddPhase, skip_spec: bool) -> u32 {
    let base = match phase {
        TddPhase::WriteSpec => 1,
        TddPhase::WriteTests => 2,
        TddPhase::VerifyRed => 3,
        TddPhase::Implement => 4,
        TddPhase::VerifyGreen => 5,
        TddPhase::Refactor => 6,
        TddPhase::Complete => 7,
    };
    if skip_spec && base > 1 {
        base - 1
    } else {
        base
    }
}

/// Get the emoji for a TDD phase
pub fn phase_emoji(phase: &TddPhase) -> &'static str {
    match phase {
        TddPhase::WriteSpec => "📝",
        TddPhase::WriteTests => "🧪",
        TddPhase::VerifyRed => "🔴",
        TddPhase::Implement => "🔨",
        TddPhase::VerifyGreen => "🟢",
        TddPhase::Refactor => "✨",
        TddPhase::Complete => "🎉",
    }
}

/// Get the ASCII alternative for a TDD phase
pub fn phase_ascii(phase: &TddPhase) -> &'static str {
    match phase {
        TddPhase::WriteSpec => "[SPEC]",
        TddPhase::WriteTests => "[TEST]",
        TddPhase::VerifyRed => "[RED]",
        TddPhase::Implement => "[IMPL]",
        TddPhase::VerifyGreen => "[GREEN]",
        TddPhase::Refactor => "[REFAC]",
        TddPhase::Complete => "[DONE]",
    }
}

/// Helper function for emoji/ASCII display
pub fn emoji<'a>(no_emoji: bool, emoji_str: &'a str, ascii_str: &'a str) -> &'a str {
    if no_emoji {
        ascii_str
    } else {
        emoji_str
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = TddExecutorConfig::default();
        assert!(!config.skip_spec);
        assert!(!config.skip_refactor);
        assert_eq!(config.max_iterations, 3);
        assert!(!config.verbose);
        assert!(!config.no_emoji);
    }

    #[test]
    fn test_config_builder() {
        let config = TddExecutorConfig::new()
            .with_skip_spec(true)
            .with_skip_refactor(true)
            .with_max_iterations(5)
            .with_verbose(true)
            .with_no_emoji(true);

        assert!(config.skip_spec);
        assert!(config.skip_refactor);
        assert_eq!(config.max_iterations, 5);
        assert!(config.verbose);
        assert!(config.no_emoji);
    }

    #[test]
    fn test_phase_number() {
        assert_eq!(phase_number(&TddPhase::WriteSpec, false), 1);
        assert_eq!(phase_number(&TddPhase::WriteTests, false), 2);
        assert_eq!(phase_number(&TddPhase::VerifyRed, false), 3);
        assert_eq!(phase_number(&TddPhase::Implement, false), 4);
        assert_eq!(phase_number(&TddPhase::VerifyGreen, false), 5);
        assert_eq!(phase_number(&TddPhase::Refactor, false), 6);
        assert_eq!(phase_number(&TddPhase::Complete, false), 7);
    }

    #[test]
    fn test_phase_number_skip_spec() {
        assert_eq!(phase_number(&TddPhase::WriteSpec, true), 1);
        assert_eq!(phase_number(&TddPhase::WriteTests, true), 1);
        assert_eq!(phase_number(&TddPhase::VerifyRed, true), 2);
        assert_eq!(phase_number(&TddPhase::Implement, true), 3);
        assert_eq!(phase_number(&TddPhase::VerifyGreen, true), 4);
        assert_eq!(phase_number(&TddPhase::Refactor, true), 5);
        assert_eq!(phase_number(&TddPhase::Complete, true), 6);
    }

    #[test]
    fn test_phase_emoji() {
        assert_eq!(phase_emoji(&TddPhase::WriteSpec), "📝");
        assert_eq!(phase_emoji(&TddPhase::WriteTests), "🧪");
        assert_eq!(phase_emoji(&TddPhase::VerifyRed), "🔴");
        assert_eq!(phase_emoji(&TddPhase::Implement), "🔨");
        assert_eq!(phase_emoji(&TddPhase::VerifyGreen), "🟢");
        assert_eq!(phase_emoji(&TddPhase::Refactor), "✨");
        assert_eq!(phase_emoji(&TddPhase::Complete), "🎉");
    }

    #[test]
    fn test_phase_ascii() {
        assert_eq!(phase_ascii(&TddPhase::WriteSpec), "[SPEC]");
        assert_eq!(phase_ascii(&TddPhase::WriteTests), "[TEST]");
        assert_eq!(phase_ascii(&TddPhase::VerifyRed), "[RED]");
        assert_eq!(phase_ascii(&TddPhase::Implement), "[IMPL]");
        assert_eq!(phase_ascii(&TddPhase::VerifyGreen), "[GREEN]");
        assert_eq!(phase_ascii(&TddPhase::Refactor), "[REFAC]");
        assert_eq!(phase_ascii(&TddPhase::Complete), "[DONE]");
    }

    #[test]
    fn test_emoji_function() {
        assert_eq!(emoji(false, "🔥", "[FIRE]"), "🔥");
        assert_eq!(emoji(true, "🔥", "[FIRE]"), "[FIRE]");
    }

    #[test]
    fn test_total_phases() {
        use tempfile::TempDir;
        let dir = TempDir::new().unwrap();

        // Default config - all 7 phases
        let executor = TddExecutor::new("test", dir.path());
        assert_eq!(executor.total_phases(), 7);

        // Skip spec only - 6 phases
        let config = TddExecutorConfig::new().with_skip_spec(true);
        let executor = TddExecutor::with_config("test", dir.path(), config);
        assert_eq!(executor.total_phases(), 6);

        // Skip refactor only - 6 phases
        let config = TddExecutorConfig::new().with_skip_refactor(true);
        let executor = TddExecutor::with_config("test", dir.path(), config);
        assert_eq!(executor.total_phases(), 6);

        // Skip both - 5 phases
        let config = TddExecutorConfig::new()
            .with_skip_spec(true)
            .with_skip_refactor(true);
        let executor = TddExecutor::with_config("test", dir.path(), config);
        assert_eq!(executor.total_phases(), 5);
    }

    #[test]
    fn test_executor_creation() {
        use tempfile::TempDir;
        let dir = TempDir::new().unwrap();

        let executor = TddExecutor::new("test behavior", dir.path());
        assert_eq!(executor.workflow().phase(), TddPhase::WriteSpec);
        assert_eq!(executor.workdir(), dir.path());
    }

    #[test]
    fn test_executor_with_config() {
        use tempfile::TempDir;
        let dir = TempDir::new().unwrap();

        let config = TddExecutorConfig::new()
            .with_skip_spec(true)
            .with_max_iterations(5);

        let executor = TddExecutor::with_config("test behavior", dir.path(), config);
        assert_eq!(executor.workflow().phase(), TddPhase::WriteTests);
        assert_eq!(executor.config().max_iterations, 5);
    }

    #[test]
    fn test_print_callback() {
        let callback = PrintCallback::new(true, false);
        assert!(callback.verbose);
        assert!(!callback.no_emoji);

        let callback = PrintCallback::new(false, true);
        assert!(!callback.verbose);
        assert!(callback.no_emoji);
    }
}
