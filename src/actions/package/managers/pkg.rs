use crate::actions::package::PackageManager;
use crate::atoms::RunCommand;
use crate::platform::PlatformInfo;
use crate::utils::detect_privilege_escalation_command;
use crate::{Atom, DhdError, Result};
use std::process::Command;

/// pkg package manager for FreeBSD
#[derive(Debug)]
pub struct Pkg;

impl PackageManager for Pkg {
    fn name(&self) -> &'static str {
        "pkg"
    }

    fn supports(&self, platform: &PlatformInfo) -> bool {
        // pkg is for FreeBSD and other BSDs
        platform.is_bsd()
    }

    fn is_available(&self) -> bool {
        Command::new("pkg")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn is_installed(&self, package: &str) -> bool {
        Command::new("pkg")
            .args(["info", package])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn bootstrap(&self) -> Result<Vec<Box<dyn Atom>>> {
        // pkg might need bootstrapping on fresh FreeBSD installs
        if self.is_available() {
            return Ok(vec![]);
        }

        let priv_cmd = detect_privilege_escalation_command().ok_or_else(|| {
            DhdError::AtomExecution(
                "No privilege escalation command found (sudo, doas, run0)".to_string(),
            )
        })?;

        // Bootstrap pkg
        Ok(vec![Box::new(RunCommand {
            command: priv_cmd,
            args: Some(vec![
                "env".to_string(),
                "ASSUME_ALWAYS_YES=YES".to_string(),
                "pkg".to_string(),
                "bootstrap".to_string(),
            ]),
            cwd: None,
            env: None,
            shell: None,
        })])
    }

    fn install(&self, packages: Vec<String>) -> Result<Box<dyn Atom>> {
        let priv_cmd = detect_privilege_escalation_command().ok_or_else(|| {
            DhdError::AtomExecution(
                "No privilege escalation command found (sudo, doas, run0)".to_string(),
            )
        })?;

        let mut args = vec!["pkg".to_string(), "install".to_string(), "-y".to_string()];
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
    fn test_pkg_name() {
        assert_eq!(Pkg.name(), "pkg");
    }

    #[test]
    fn test_pkg_needs_privilege_escalation() {
        assert!(Pkg.needs_privilege_escalation());
    }
}
