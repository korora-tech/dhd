use crate::actions::package::PackageManager;
use crate::atoms::RunCommand;
use crate::platform::PlatformInfo;
use crate::utils::detect_privilege_escalation_command;
use crate::{Atom, DhdError, Result};
use std::process::Command;

/// Pacman package manager for Arch Linux
#[derive(Debug)]
pub struct Pacman;

impl PackageManager for Pacman {
    fn name(&self) -> &'static str {
        "pacman"
    }

    fn supports(&self, platform: &PlatformInfo) -> bool {
        platform.is_arch_based()
    }

    fn is_available(&self) -> bool {
        Command::new("pacman")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn is_installed(&self, package: &str) -> bool {
        Command::new("pacman")
            .args(&["-Q", package])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn bootstrap(&self) -> Result<Vec<Box<dyn Atom>>> {
        // Pacman doesn't need bootstrapping
        Ok(vec![])
    }

    fn install(&self, packages: Vec<String>) -> Result<Box<dyn Atom>> {
        let priv_cmd = detect_privilege_escalation_command().ok_or_else(|| {
            DhdError::AtomExecution(
                "No privilege escalation command found (sudo, doas, run0)".to_string(),
            )
        })?;

        let mut args = vec![
            "pacman".to_string(),
            "-S".to_string(),
            "--noconfirm".to_string(),
        ];
        args.extend(packages);

        Ok(Box::new(RunCommand {
            command: priv_cmd,
            args: Some(args),
            cwd: None,
            env: None,
        }))
    }

    fn needs_privilege_escalation(&self) -> bool {
        true
    }

    fn priority(&self) -> i32 {
        10 // Base priority for pacman
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pacman_name() {
        assert_eq!(Pacman.name(), "pacman");
    }

    #[test]
    fn test_pacman_needs_privilege_escalation() {
        assert!(Pacman.needs_privilege_escalation());
    }
}
