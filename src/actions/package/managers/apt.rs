use crate::actions::package::PackageManager;
use crate::atoms::RunCommand;
use crate::platform::PlatformInfo;
use crate::utils::detect_privilege_escalation_command;
use crate::{Atom, DhdError, Result};
use std::process::Command;

/// APT package manager for Debian/Ubuntu
#[derive(Debug)]
pub struct Apt;

impl PackageManager for Apt {
    fn name(&self) -> &'static str {
        "apt"
    }

    fn supports(&self, platform: &PlatformInfo) -> bool {
        platform.is_debian_based()
    }

    fn is_available(&self) -> bool {
        Command::new("apt")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn is_installed(&self, package: &str) -> bool {
        Command::new("dpkg")
            .args(&["-l", package])
            .output()
            .map(|output| {
                output.status.success() && String::from_utf8_lossy(&output.stdout).contains("ii ")
            })
            .unwrap_or(false)
    }

    fn bootstrap(&self) -> Result<Vec<Box<dyn Atom>>> {
        // APT doesn't need bootstrapping
        Ok(vec![])
    }

    fn install(&self, packages: Vec<String>) -> Result<Box<dyn Atom>> {
        let priv_cmd = detect_privilege_escalation_command().ok_or_else(|| {
            DhdError::AtomExecution(
                "No privilege escalation command found (sudo, doas, run0)".to_string(),
            )
        })?;

        let mut args = vec!["apt".to_string(), "install".to_string(), "-y".to_string()];
        args.extend(packages);

        Ok(Box::new(RunCommand {
            command: priv_cmd,
            args: Some(args),
            cwd: None,
            env: None,
            shell: None,
        }))
    }

    fn needs_privilege_escalation(&self) -> bool {
        true
    }

    fn priority(&self) -> i32 {
        10
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apt_name() {
        assert_eq!(Apt.name(), "apt");
    }

    #[test]
    fn test_apt_needs_privilege_escalation() {
        assert!(Apt.needs_privilege_escalation());
    }
}
