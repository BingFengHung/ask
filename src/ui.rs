use colored::*;
use anyhow::Result;
use inquire::{Select, Text};

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

pub fn render_command_box(command: &str) {
    let border = "─".repeat(60);
    println!("{}", border.dimmed());
    println!("{}", command.bold().green());
    println!("{}", border.dimmed());
    println!();
}

pub fn prompt_user_action(command: &str) -> Result<UserChoice> {
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
