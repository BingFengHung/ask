use std::env;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OsType {
    Windows,
    MacOS,
    Linux,
}

impl OsType {
    pub fn current() -> Self {
        if cfg!(target_os = "windows") {
            OsType::Windows
        } else if cfg!(target_os = "macos") {
            OsType::MacOS
        } else {
            OsType::Linux
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            OsType::Windows => "Windows",
            OsType::MacOS => "macOS",
            OsType::Linux => "Linux",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShellType {
    PowerShell,
    Cmd,
    Zsh,
    Bash,
}

impl ShellType {
    pub fn detect(os: &OsType) -> Self {
        match os {
            OsType::Windows => {
                if env::var("PSModulePath").is_ok() || env::var("POWERSHELL_DISTRIBUTION_CHANNEL").is_ok() {
                    ShellType::PowerShell
                } else {
                    ShellType::PowerShell // Default to PowerShell on modern Windows
                }
            }
            _ => {
                if let Ok(shell_var) = env::var("SHELL") {
                    if shell_var.contains("zsh") {
                        ShellType::Zsh
                    } else {
                        ShellType::Bash
                    }
                } else {
                    ShellType::Bash
                }
            }
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            ShellType::PowerShell => "PowerShell",
            ShellType::Cmd => "cmd.exe",
            ShellType::Zsh => "zsh",
            ShellType::Bash => "bash",
        }
    }
}

#[derive(Debug, Clone)]
pub struct EnvironmentContext {
    pub os: OsType,
    pub shell: ShellType,
    pub cwd: String,
}

impl EnvironmentContext {
    pub fn gather() -> Self {
        let os = OsType::current();
        let shell = ShellType::detect(&os);
        let cwd = env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "Unknown".to_string());

        EnvironmentContext { os, shell, cwd }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_os_detection() {
        let os = OsType::current();
        assert!(os.name().len() > 0);
    }

    #[test]
    fn test_environment_context_gather() {
        let ctx = EnvironmentContext::gather();
        assert!(!ctx.cwd.is_empty());
    }
}
