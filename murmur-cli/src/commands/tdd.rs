//! TDD command - Run a Test-Driven Development workflow
//!
//! This command coordinates the TDD phases:
//! 1. WriteSpec: Write specification document
//! 2. WriteTests: Write tests based on spec
//! 3. VerifyRed: Verify tests fail
//! 4. Implement: Make tests pass
//! 5. VerifyGreen: Verify tests pass
//! 6. Refactor: Clean up code
//! 7. Complete: Done

use std::path::PathBuf;

use clap::Args;
use murmur_core::workflow::{TestFramework, TestRunner};
use murmur_core::{
    AgentFactory, AgentType, Config, OutputStreamer, PrintHandler, TddPhase, TddWorkflow,
};

/// Arguments for the tdd command
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
    /// Execute the TDD workflow
    pub async fn execute(
        &self,
        verbose: bool,
        no_emoji: bool,
        config: &Config,
    ) -> anyhow::Result<()> {
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

        // Create workflow
        let mut workflow = if self.skip_spec {
            TddWorkflow::new_without_spec(&self.behavior, &workdir)
        } else {
            TddWorkflow::with_config(&self.behavior, &workdir, config.agent.clone())
        };

        // Configure workflow
        if self.skip_refactor {
            workflow.state_mut().skip_refactor = true;
        }
        workflow.state_mut().max_iterations = self.max_iterations;

        // Helper macro for emoji/ASCII output
        macro_rules! emoji {
            ($e:expr, $ascii:expr) => {
                if no_emoji {
                    $ascii
                } else {
                    $e
                }
            };
        }

        println!("TDD Workflow");
        println!("============");
        println!();
        println!("Behavior: {}", self.behavior);
        println!("Working directory: {}", workdir.display());
        if let Some(ref model) = config.agent.model {
            println!("Model: {}", model);
        }
        println!();

        if self.dry_run {
            println!("[Dry run] Would execute TDD workflow with the following phases:");
            println!();
            self.show_planned_phases(&workflow, no_emoji);
            return Ok(());
        }

        // Detect test framework
        let framework = TestFramework::detect(&workdir).unwrap_or(TestFramework::Cargo);
        println!("Detected test framework: {}", framework.name());
        println!();

        // Create test runner for validation phases
        let test_runner = TestRunner::new(workdir.clone()).with_framework(framework);

        // Create agent factory
        let factory = AgentFactory::with_config(config.agent.clone());

        // Run the TDD cycle
        while !workflow.is_complete() && !workflow.should_give_up() {
            let phase = workflow.phase();
            let phase_num = phase_number(&phase);
            let total_phases = if self.skip_spec { 6 } else { 7 };

            println!(
                "Phase {}/{}: {} {}",
                phase_num,
                total_phases,
                emoji!(phase_emoji(&phase), phase_ascii(&phase)),
                phase.description()
            );
            println!();

            match phase {
                TddPhase::WriteSpec
                | TddPhase::WriteTests
                | TddPhase::Implement
                | TddPhase::Refactor => {
                    // Agent-driven phases
                    let prompt = workflow.current_prompt();

                    if verbose {
                        println!("Prompt: {}", prompt);
                        println!();
                    }

                    println!("Starting agent...");

                    // Choose agent type based on phase
                    let agent_type = match phase {
                        TddPhase::WriteTests => AgentType::Test,
                        _ => AgentType::Implement,
                    };
                    let typed_agent = factory.create(agent_type);

                    // Spawn and run agent
                    let mut handle = typed_agent.spawn_with_task(&prompt, &workdir).await?;

                    let stdout = handle
                        .child_mut()
                        .stdout
                        .take()
                        .ok_or_else(|| anyhow::anyhow!("Failed to capture agent stdout"))?;

                    // Stream output
                    let mut streamer = OutputStreamer::new(stdout);
                    let mut handler = PrintHandler::new(verbose);
                    streamer.stream(&mut handler).await?;

                    let status = handle.wait().await?;

                    if status.success() {
                        println!();
                        println!("{} Phase completed", emoji!("✅", "[OK]"));
                        workflow.advance(true, None);
                    } else {
                        println!();
                        println!(
                            "{} Agent exited with status: {}",
                            emoji!("❌", "[FAIL]"),
                            status
                        );
                        // Don't advance, let user decide what to do
                        return Err(anyhow::anyhow!(
                            "Agent failed in {} phase",
                            phase.description()
                        ));
                    }
                }
                TddPhase::VerifyRed => {
                    // Run tests and expect them to fail
                    println!("Running tests (expecting failures)...");
                    let results = test_runner.run();

                    println!();
                    println!(
                        "Test results: {} passed, {} failed, {} skipped",
                        results.passed, results.failed, results.skipped
                    );

                    if results.is_red() {
                        println!();
                        println!(
                            "{} Tests failed as expected (red phase)",
                            emoji!("✅", "[OK]")
                        );
                        workflow.advance(true, None);
                    } else if results.passed > 0 && results.failed == 0 {
                        println!();
                        println!(
                            "{} Tests passed unexpectedly - tests may not be testing new behavior",
                            emoji!("⚠️", "[WARN]")
                        );
                        println!("Going back to WriteTests phase...");
                        workflow.retry_tests(Some("Tests passed unexpectedly".to_string()));
                    } else {
                        println!();
                        println!(
                            "{} No tests found or error running tests",
                            emoji!("❌", "[FAIL]")
                        );
                        workflow.retry_tests(Some("No tests found".to_string()));
                    }
                }
                TddPhase::VerifyGreen => {
                    // Run tests and expect them to pass
                    let iteration = workflow.state().iterations;
                    println!(
                        "Running tests (iteration {}/{})...",
                        iteration + 1,
                        self.max_iterations
                    );
                    let results = test_runner.run();

                    println!();
                    println!(
                        "Test results: {} passed, {} failed, {} skipped",
                        results.passed, results.failed, results.skipped
                    );

                    if results.is_green() {
                        println!();
                        println!("{} All tests pass (green phase)", emoji!("✅", "[OK]"));
                        workflow.advance(true, None);
                    } else {
                        println!();
                        println!(
                            "{} {} tests still failing",
                            emoji!("❌", "[FAIL]"),
                            results.failed
                        );

                        if workflow.state().iterations + 1 >= self.max_iterations {
                            println!();
                            println!(
                                "{} Maximum iterations reached, giving up",
                                emoji!("🛑", "[STOP]")
                            );
                        } else {
                            println!("Returning to Implement phase...");
                            workflow
                                .retry_implement(Some(format!("{} tests failing", results.failed)));
                        }
                    }
                }
                TddPhase::Complete => {
                    // Should not reach here due to while condition
                    break;
                }
            }
            println!();
        }

        // Final status
        if workflow.is_complete() {
            println!("═══════════════════════════════════════");
            println!(
                "{} TDD workflow completed successfully!",
                emoji!("🎉", "[DONE]")
            );
            println!("═══════════════════════════════════════");
        } else if workflow.should_give_up() {
            println!("═══════════════════════════════════════");
            println!(
                "{} TDD workflow failed after {} iterations",
                emoji!("💥", "[FAIL]"),
                workflow.state().iterations
            );
            println!("═══════════════════════════════════════");
            return Err(anyhow::anyhow!("TDD workflow failed after max iterations"));
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

fn phase_number(phase: &TddPhase) -> u32 {
    match phase {
        TddPhase::WriteSpec => 1,
        TddPhase::WriteTests => 2,
        TddPhase::VerifyRed => 3,
        TddPhase::Implement => 4,
        TddPhase::VerifyGreen => 5,
        TddPhase::Refactor => 6,
        TddPhase::Complete => 7,
    }
}

fn phase_emoji(phase: &TddPhase) -> &'static str {
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

fn phase_ascii(phase: &TddPhase) -> &'static str {
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
