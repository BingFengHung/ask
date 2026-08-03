use anyhow::{anyhow, Result};
use std::process::Command;
use colored::*;
use crate::env_detector::{EnvironmentContext, ShellType};

pub fn execute_command(command: &str, ctx: &EnvironmentContext) -> Result<()> {
    println!("{}", "⚡ Executing command...".bold().yellow());
    println!();

    let mut cmd = match ctx.shell {
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

    let status = cmd.status().map_err(|e| anyhow!("Failed to spawn shell process: {}", e))?;

    println!();
    if status.success() {
        println!("{}", "✨ Command completed successfully!".bold().green());
    } else {
        println!(
            "{}",
            format!("⚠️ Command exited with status code: {:?}", status.code()).bold().red()
        );
    }

    Ok(())
}
