mod env_detector;
mod executor;
mod llm_provider;
mod safety_auditor;
mod ui;

use anyhow::Result;
use clap::Parser;
use colored::*;
use env_detector::EnvironmentContext;
use executor::execute_command;
use llm_provider::query_best_available_provider;
use safety_auditor::audit_command;
use ui::{prompt_user_action, render_command_box, render_header, UserChoice};

#[derive(Parser, Debug)]
#[command(
    name = "ask",
    author = "Your Name <your.email@example.com>",
    version = "0.1.0",
    about = "Natural language shell CLI assistant powered by AI Agents."
)]
struct Cli {
    /// Natural language prompt describing the command you want to run
    #[arg(required = true, value_name = "QUERY")]
    query: Vec<String>,

    /// Dry run mode: print the generated command without executing it
    #[arg(short, long)]
    dry_run: bool,

    /// Force automatic execution without prompting (use with caution)
    #[arg(short = 'y', long = "yes")]
    auto_approve: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let query_str = cli.query.join(" ");

    if query_str.trim().is_empty() {
        println!("{}", "Please provide a query. Example: ask \"kill process on port 8080\"".red());
        return Ok(());
    }

    // 1. Gather Environment Context
    let ctx = EnvironmentContext::gather();

    // 2. Query LLM Provider
    println!("{}", "🔍 Analyzing query & generating shell command...".dimmed());
    let (generated_cmd, provider_name) = match query_best_available_provider(&query_str, &ctx).await {
        Ok(res) => res,
        Err(e) => {
            eprintln!("{} {}", "❌ Error:".bold().red(), e);
            std::process::exit(1);
        }
    };

    // 3. Safety Audit
    let risk = audit_command(&generated_cmd);

    // 4. Render UI
    render_header(&query_str, provider_name, ctx.os.name(), ctx.shell.name());
    render_command_box(&generated_cmd, &risk);

    // 5. Handle Execution or Dry-Run
    if cli.dry_run {
        println!("{}", "ℹ️ Dry-run mode enabled. Command was not executed.".dimmed());
        return Ok(());
    }

    if cli.auto_approve && matches!(risk, safety_auditor::RiskLevel::Safe) {
        execute_command(&generated_cmd, &ctx)?;
        return Ok(());
    }

    // 6. Interactive User Choice
    match prompt_user_action(&generated_cmd, &risk)? {
        UserChoice::Execute => {
            execute_command(&generated_cmd, &ctx)?;
        }
        UserChoice::Edit(edited_cmd) => {
            let edited_risk = audit_command(&edited_cmd);
            println!();
            println!("{}", "Modified command:".bold().cyan());
            render_command_box(&edited_cmd, &edited_risk);
            execute_command(&edited_cmd, &ctx)?;
        }
        UserChoice::Cancel => {
            println!("{}", "🚫 Execution cancelled.".yellow());
        }
    }

    Ok(())
}
