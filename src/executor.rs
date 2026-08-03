use anyhow::{anyhow, Result};
use std::process::{Command, Stdio};
use colored::*;
use crate::env_detector::{EnvironmentContext, ShellType};

pub fn execute_command(command: &str, ctx: &EnvironmentContext) -> Result<()> {
    println!("{}", "⚡ Executing command...".bold().yellow());
    println!();

    let mut child_cmd = match ctx.shell {
        ShellType::PowerShell => {
            let mut c = Command::new("powershell");
            c.arg("-NoProfile").arg("-Command").arg(command);
            c
        }
        ShellType::Cmd => {
            let mut c = Command::new("cmd");
            c.arg("/C").arg(command);
            c
        }
        ShellType::Zsh => {
            let mut c = Command::new("zsh");
            c.arg("-c").arg(command);
            c
        }
        ShellType::Bash => {
            let mut c = Command::new("bash");
            c.arg("-c").arg(command);
            c
        }
    };

    // Capture stdout and stderr for better UX and error diagnosis
    let output = child_cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| anyhow!("Failed to spawn shell process: {}", e))?;

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let stderr_str = String::from_utf8_lossy(&output.stderr);

    // Print standard output if available
    if !stdout_str.trim().is_empty() {
        println!("{}", stdout_str);
    }

    // Print standard error if available
    if !stderr_str.trim().is_empty() {
        println!("{}", stderr_str.red());
    }

    // Friendly status handling
    if output.status.success() {
        if stdout_str.trim().is_empty() {
            println!("{}", "ℹ️ Command completed successfully (no output returned).".dimmed());
        } else {
            println!("{}", "✨ Command completed successfully!".bold().green());
        }
    } else {
        let code = output.status.code().unwrap_or(1);
        if stdout_str.trim().is_empty() && stderr_str.trim().is_empty() {
            println!(
                "{}",
                format!("ℹ️ Command finished (Exit Code {}): No matching process, file, or connection found.", code).bold().yellow()
            );
        } else {
            println!(
                "{}",
                format!("⚠️ Command exited with status code: {}", code).bold().red()
            );
        }
    }

    Ok(())
}
