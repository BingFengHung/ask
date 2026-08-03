use colored::*;
use anyhow::Result;
use inquire::{Select, Text};
use crate::safety_auditor::RiskLevel;

#[derive(Debug, PartialEq, Eq)]
pub enum UserChoice {
    Execute,
    Cancel,
    Edit(String),
}

pub fn render_header(query: &str, provider: &str, os_name: &str, shell_name: &str) {
    println!();
    println!(
        "{} {}",
        "🤖 [ask Agent]".bold().cyan(),
        format!("(via {} | {} / {})", provider, os_name, shell_name).dimmed()
    );
    println!("{} {}", "❓ Query:".bold().yellow(), query);
    println!();
}

pub fn render_command_box(command: &str, risk: &RiskLevel) {
    if let RiskLevel::HighRisk { reason } = risk {
        println!("{}", "🚨 [DANGER WARNING] High-Risk Command Detected!".bold().on_red().white());
        println!("{} {}", "⚠️ Risk Analysis:".bold().red(), reason.yellow());
        println!();
    }

    let border = "─".repeat(60);
    if matches!(risk, RiskLevel::HighRisk { .. }) {
        println!("{}", border.red());
        println!("{}", command.bold().red());
        println!("{}", border.red());
    } else {
        println!("{}", border.dimmed());
        println!("{}", command.bold().green());
        println!("{}", border.dimmed());
    }
    println!();
}

pub fn prompt_user_action(command: &str, risk: &RiskLevel) -> Result<UserChoice> {
    if let RiskLevel::HighRisk { .. } = risk {
        println!("{}", "🛡️ High-Risk Safety Guardrail Activated!".bold().red());
        let confirmation = Text::new("Type 'YES' (in capital letters) to confirm execution of this dangerous command:")
            .prompt()?;

        if confirmation.trim() == "YES" {
            Ok(UserChoice::Execute)
        } else {
            println!("{}", "🚫 Execution aborted: Safety confirmation 'YES' was not entered.".yellow());
            Ok(UserChoice::Cancel)
        }
    } else {
        let options = vec![
            "🚀 Execute command immediately",
            "✏️  Edit command before executing",
            "❌ Cancel",
        ];

        let ans = Select::new("What would you like to do?", options).prompt();

        match ans {
            Ok("🚀 Execute command immediately") => Ok(UserChoice::Execute),
            Ok("✏️  Edit command before executing") => {
                let edited = Text::new("Edit command:")
                    .with_initial_value(command)
                    .prompt()?;
                Ok(UserChoice::Edit(edited))
            }
            _ => Ok(UserChoice::Cancel),
        }
    }
}
