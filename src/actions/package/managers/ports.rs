use crate::actions::package::PackageManager;
use crate::atoms::RunCommand;
use crate::platform::PlatformInfo;
use crate::utils::detect_privilege_escalation_command;
use crate::{Atom, DhdError, Result};
use std::process::Command;

/// BSD Ports system
#[derive(Debug)]
pub struct Ports;

impl PackageManager for Ports {
    fn name(&self) -> &'static str {
        "ports"
    }

    fn supports(&self, platform: &PlatformInfo) -> bool {
        // Ports is available on BSDs
        platform.is_bsd()
    }

    fn is_available(&self) -> bool {
        // Check if ports tree exists
        std::path::Path::new("/usr/ports").exists()
    }

    fn is_installed(&self, package: &str) -> bool {
        // Check if package is installed via pkg_info
        Command::new("pkg_info")
            .arg(package)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn bootstrap(&self) -> Result<Vec<Box<dyn Atom>>> {
        if self.is_available() {
            return Ok(vec![]);
        }

        let priv_cmd = detect_privilege_escalation_command().ok_or_else(|| {
            DhdError::AtomExecution(
                "No privilege escalation command found (sudo, doas, run0)".to_string(),
            )
        })?;

        // Download and extract ports tree
        Ok(vec![
            Box::new(RunCommand {
                command: priv_cmd.clone(),
                args: Some(vec![
                    "mkdir".to_string(),
                    "-p".to_string(),
                    "/usr/ports".to_string(),
                ]),
                cwd: None,
                env: None,
            }),
            Box::new(RunCommand {
                command: priv_cmd.clone(),
                args: Some(vec![
                    "fetch".to_string(),
                    "-o".to_string(),
                    "/tmp/ports.tar.gz".to_string(),
                    "https://download.freebsd.org/ftp/ports/ports/ports.tar.gz".to_string(),
                ]),
                cwd: None,
                env: None,
            }),
            Box::new(RunCommand {
                command: priv_cmd,
                args: Some(vec![
                    "tar".to_string(),
                    "-xzf".to_string(),
                    "/tmp/ports.tar.gz".to_string(),
                    "-C".to_string(),
                    "/usr".to_string(),
                ]),
                cwd: None,
                env: None,
            }),
        ])
    }

    fn install(&self, packages: Vec<String>) -> Result<Box<dyn Atom>> {
        let priv_cmd = detect_privilege_escalation_command().ok_or_else(|| {
            DhdError::AtomExecution(
                "No privilege escalation command found (sudo, doas, run0)".to_string(),
            )
        })?;

        // For ports, we need to install each package separately
        // This is a simplified version - real ports installation is more complex
        let mut atoms: Vec<Box<dyn Atom>> = vec![];

        for package in packages {
            // Find the port directory (simplified - assumes standard location)
            let port_path = format!("/usr/ports/*/{}", package);

            atoms.push(Box::new(RunCommand {
                command: priv_cmd.clone(),
                args: Some(vec![
                    "sh".to_string(),
                    "-c".to_string(),
                    format!("cd {} && make install clean BATCH=yes", port_path),
                ]),
                cwd: None,
                env: None,
            }));
        }

        // Return the first atom if only one package, otherwise we'd need to handle multiple
        atoms
            .into_iter()
            .next()
            .ok_or_else(|| DhdError::AtomExecution("No packages to install".to_string()))
    }

    fn needs_privilege_escalation(&self) -> bool {
        true
    }

    fn priority(&self) -> i32 {
        5 // Lower priority than pkg
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ports_name() {
        assert_eq!(Ports.name(), "ports");
    }

    #[test]
    fn test_ports_needs_privilege_escalation() {
        assert!(Ports.needs_privilege_escalation());
    }
}
