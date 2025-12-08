//! TDD command - Run a Test-Driven Development workflow
//!
//! NOTE: This command is deprecated in favor of `murmur work --tdd <issue>`.
//! For standalone TDD without an issue, this command runs the TDD workflow
//! in the current directory without GitHub integration.
//!
//! For new projects, prefer using `murmur work --tdd <issue>` to get:
//! - GitHub issue tracking
//! - Worktree isolation
//! - Auto-commit, push, and PR creation
//! - Dependency checking
//!
//! The TDD workflow phases:
//! 1. WriteSpec: Write specification document
//! 2. WriteTests: Write tests based on spec
//! 3. VerifyRed: Verify tests fail
//! 4. Implement: Make tests pass
//! 5. VerifyGreen: Verify tests pass
//! 6. Refactor: Clean up code
//! 7. Complete: Done

use std::path::PathBuf;

use clap::Args;
use murmur_core::workflow::TestFramework;
use murmur_core::{
    phase_ascii, phase_emoji, Config, PrintCallback, TddExecutor, TddExecutorConfig, TddPhase,
    TddWorkflow,
};

/// Arguments for the tdd command (standalone TDD without issue tracking)
///
/// DEPRECATED: Consider using `murmur work --tdd <issue>` for full integration
/// with GitHub issues, worktrees, and auto-PR creation.
#[derive(Args, Debug)]
pub struct TddArgs {
    /// The behavior to implement using TDD
    #[arg(required = true)]
    pub behavior: String,

    /// Working directory (defaults to current directory)
    #[arg(short = 'd', long, default_value = ".")]
    pub workdir: PathBuf,

    /// Skip the WriteSpec phase (start from WriteTests)
    #[arg(long)]
    pub skip_spec: bool,

    /// Skip the Refactor phase (go straight to Complete after VerifyGreen)
    #[arg(long)]
    pub skip_refactor: bool,

    /// Maximum iterations for Implement->VerifyGreen loop
    #[arg(long, default_value = "3")]
    pub max_iterations: u32,

    /// Dry run - show what would be executed without running agents
    #[arg(long)]
    pub dry_run: bool,
}

impl TddArgs {
    /// Execute the TDD workflow (standalone mode without issue tracking)
    pub async fn execute(
        &self,
        verbose: bool,
        no_emoji: bool,
        config: &Config,
    ) -> anyhow::Result<()> {
        // Show deprecation notice
        eprintln!("Note: `murmur tdd` runs TDD without issue tracking.");
        eprintln!("      Consider using `murmur work --tdd <issue>` for full GitHub integration.");
        eprintln!();

        // Resolve to absolute path
        let workdir = if self.workdir.is_absolute() {
            self.workdir.clone()
        } else {
            std::env::current_dir()?.join(&self.workdir)
        };

        if verbose {
            tracing::info!(
                behavior = %self.behavior,
                workdir = %workdir.display(),
                skip_spec = %self.skip_spec,
                skip_refactor = %self.skip_refactor,
                max_iterations = %self.max_iterations,
                "Starting TDD workflow"
            );
        }

        println!("TDD Workflow (Standalone)");
        println!("=========================");
        println!();
        println!("Behavior: {}", self.behavior);
        println!("Working directory: {}", workdir.display());
        if let Some(ref model) = config.agent.model {
            println!("Model: {}", model);
        }
        println!();

        if self.dry_run {
            // Create workflow for dry run display only
            let workflow = if self.skip_spec {
                TddWorkflow::new_without_spec(&self.behavior, &workdir)
            } else {
                TddWorkflow::with_config(&self.behavior, &workdir, config.agent.clone())
            };
            println!("[Dry run] Would execute TDD workflow with the following phases:");
            println!();
            self.show_planned_phases(&workflow, no_emoji);
            return Ok(());
        }

        // Detect test framework
        let framework = TestFramework::detect(&workdir).unwrap_or(TestFramework::Cargo);
        println!("Detected test framework: {}", framework.name());
        println!();

        // Create executor configuration
        let executor_config = TddExecutorConfig::new()
            .with_skip_spec(self.skip_spec)
            .with_skip_refactor(self.skip_refactor)
            .with_max_iterations(self.max_iterations)
            .with_verbose(verbose)
            .with_no_emoji(no_emoji)
            .with_agent_config(config.agent.clone());

        // Create executor
        let mut executor = TddExecutor::with_config(&self.behavior, &workdir, executor_config)
            .with_framework(framework);

        // Create simple print callback (no database tracking for standalone mode)
        let mut callback = PrintCallback::new(verbose, no_emoji);

        // Execute the workflow
        let success = executor
            .execute(&mut callback)
            .await
            .map_err(|e| anyhow::anyhow!("TDD workflow error: {}", e))?;

        if !success {
            return Err(anyhow::anyhow!("TDD workflow failed"));
        }

        Ok(())
    }

    fn show_planned_phases(&self, workflow: &TddWorkflow, no_emoji: bool) {
        let phases = if self.skip_spec {
            vec![
                TddPhase::WriteTests,
                TddPhase::VerifyRed,
                TddPhase::Implement,
                TddPhase::VerifyGreen,
            ]
        } else {
            vec![
                TddPhase::WriteSpec,
                TddPhase::WriteTests,
                TddPhase::VerifyRed,
                TddPhase::Implement,
                TddPhase::VerifyGreen,
            ]
        };

        let phases: Vec<_> = if self.skip_refactor {
            phases
                .into_iter()
                .chain(std::iter::once(TddPhase::Complete))
                .collect()
        } else {
            phases
                .into_iter()
                .chain(std::iter::once(TddPhase::Refactor))
                .chain(std::iter::once(TddPhase::Complete))
                .collect()
        };

        for (i, phase) in phases.iter().enumerate() {
            let agent_type = match phase {
                TddPhase::WriteSpec | TddPhase::Implement | TddPhase::Refactor => "implement agent",
                TddPhase::WriteTests => "test agent",
                TddPhase::VerifyRed | TddPhase::VerifyGreen => "test runner",
                TddPhase::Complete => "n/a",
            };
            let icon = if no_emoji {
                phase_ascii(phase)
            } else {
                phase_emoji(phase)
            };
            println!(
                "  {}. {} {} ({})",
                i + 1,
                icon,
                phase.description(),
                agent_type
            );
        }

        println!();
        println!("Current prompt for first phase:");
        println!("---");
        println!("{}", workflow.current_prompt());
        println!("---");
    }
}
