use crate::actions::package::PackageManager;
use crate::atoms::RunCommand;
use crate::platform::PlatformInfo;
use crate::{Atom, DhdError, Result};
use std::process::Command;

use super::arch::Pacman;

/// Paru AUR helper for Arch Linux
#[derive(Debug)]
pub struct Paru;

impl PackageManager for Paru {
    fn name(&self) -> &'static str {
        "paru"
    }

    fn supports(&self, platform: &PlatformInfo) -> bool {
        platform.is_arch_based()
    }

    fn is_available(&self) -> bool {
        Command::new("paru")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn is_installed(&self, package: &str) -> bool {
        Command::new("paru")
            .args(&["-Q", package])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn bootstrap(&self) -> Result<Vec<Box<dyn Atom>>> {
        if self.is_available() {
            return Ok(vec![]);
        }

        // Check if pacman is available
        if !Pacman.is_available() {
            return Err(DhdError::AtomExecution(
                "Pacman is required to install paru".to_string(),
            ));
        }

        let mut atoms: Vec<Box<dyn Atom>> = vec![];

        // Install base-devel if not already installed
        if !Pacman.is_installed("base-devel") {
            atoms.push(Pacman.install(vec!["base-devel".to_string()])?);
        }

        // Install git if not already installed
        if !Pacman.is_installed("git") {
            atoms.push(Pacman.install(vec!["git".to_string()])?);
        }

        // Clone and build paru
        atoms.push(Box::new(RunCommand {
            command: "git".to_string(),
            args: Some(vec![
                "clone".to_string(),
                "https://aur.archlinux.org/paru.git".to_string(),
                "/tmp/paru".to_string(),
            ]),
            cwd: None,
            env: None,
        }));

        atoms.push(Box::new(RunCommand {
            command: "makepkg".to_string(),
            args: Some(vec!["-si".to_string(), "--noconfirm".to_string()]),
            cwd: Some("/tmp/paru".to_string()),
            env: None,
        }));

        atoms.push(Box::new(RunCommand {
            command: "rm".to_string(),
            args: Some(vec!["-rf".to_string(), "/tmp/paru".to_string()]),
            cwd: None,
            env: None,
        }));

        Ok(atoms)
    }

    fn install(&self, packages: Vec<String>) -> Result<Box<dyn Atom>> {
        let mut args = vec!["-S".to_string(), "--noconfirm".to_string()];
        args.extend(packages);

        Ok(Box::new(RunCommand {
            command: "paru".to_string(),
            args: Some(args),
            cwd: None,
            env: None,
        }))
    }

    fn needs_privilege_escalation(&self) -> bool {
        false // paru handles privilege escalation internally
    }

    fn priority(&self) -> i32 {
        20 // Higher priority than pacman for AUR support
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paru_name() {
        assert_eq!(Paru.name(), "paru");
    }

    #[test]
    fn test_paru_needs_privilege_escalation() {
        assert!(!Paru.needs_privilege_escalation());
    }
}
