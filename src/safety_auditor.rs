#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RiskLevel {
    Safe,
    HighRisk { reason: String },
}

pub fn audit_command(command: &str) -> RiskLevel {
    let cmd_lower = command.to_lowercase();

    // 1. Recursive file deletion patterns
    if cmd_lower.contains("rm -rf")
        || cmd_lower.contains("rm -r -f")
        || cmd_lower.contains("rmdir /s")
        || (cmd_lower.contains("remove-item") && cmd_lower.contains("-recurse") && cmd_lower.contains("-force"))
        || cmd_lower.contains("del /f /s")
    {
        return RiskLevel::HighRisk {
            reason: "Recursive file deletion can permanently destroy files without trash recovery.".to_string(),
        };
    }

    // 2. System formatting or disk wipe
    if cmd_lower.contains("format ") || cmd_lower.contains("mkfs") || cmd_lower.contains("diskpart") {
        return RiskLevel::HighRisk {
            reason: "Disk formatting or partition operation will wipe hard drive data.".to_string(),
        };
    }

    // 3. Database drop or truncate
    if cmd_lower.contains("drop database")
        || cmd_lower.contains("drop table")
        || cmd_lower.contains("truncate table")
    {
        return RiskLevel::HighRisk {
            reason: "Database DROP / TRUNCATE operations permanently delete database tables.".to_string(),
        };
    }

    // 4. Git destructive actions
    if cmd_lower.contains("git reset --hard") || cmd_lower.contains("git clean -fd") || cmd_lower.contains("git clean -f -d") {
        return RiskLevel::HighRisk {
            reason: "Destructive git operation will permanently discard uncommitted changes.".to_string(),
        };
    }

    // 5. System shutdown or reboot
    if cmd_lower.contains("shutdown") || cmd_lower.contains("init 0") || cmd_lower.contains("reboot") {
        return RiskLevel::HighRisk {
            reason: "System shutdown or reboot will terminate all running applications.".to_string(),
        };
    }

    RiskLevel::Safe
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_safe_commands() {
        assert_eq!(audit_command("git status"), RiskLevel::Safe);
        assert_eq!(audit_command("Get-Process | Select-Object -First 5"), RiskLevel::Safe);
    }

    #[test]
    fn test_audit_dangerous_rm_rf() {
        let res = audit_command("rm -rf /tmp/myfiles");
        assert!(matches!(res, RiskLevel::HighRisk { .. }));
    }

    #[test]
    fn test_audit_dangerous_powershell_remove_item() {
        let res = audit_command("Remove-Item -Path C:\\temp -Recurse -Force");
        assert!(matches!(res, RiskLevel::HighRisk { .. }));
    }

    #[test]
    fn test_audit_dangerous_git_reset() {
        let res = audit_command("git reset --hard HEAD~1");
        assert!(matches!(res, RiskLevel::HighRisk { .. }));
    }
}
