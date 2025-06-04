use crate::actions::package::PackageManager;
use crate::atoms::RunCommand;
use crate::platform::PlatformInfo;
use crate::{Atom, Result};
use std::process::Command;

/// NPM package manager for Node.js
#[derive(Debug)]
pub struct Npm;

impl PackageManager for Npm {
    fn name(&self) -> &'static str {
        "npm"
    }

    fn supports(&self, _platform: &PlatformInfo) -> bool {
        // npm works on all platforms where Node.js is available
        true
    }

    fn is_available(&self) -> bool {
        Command::new("npm")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn is_installed(&self, package: &str) -> bool {
        // Check globally installed packages
        Command::new("npm")
            .args(&["list", "-g", "--depth=0", package])
            .output()
            .map(|output| {
                output.status.success() && String::from_utf8_lossy(&output.stdout).contains(package)
            })
            .unwrap_or(false)
    }

    fn bootstrap(&self) -> Result<Vec<Box<dyn Atom>>> {
        // npm is installed with Node.js, so no bootstrap needed
        Ok(vec![])
    }

    fn install(&self, packages: Vec<String>) -> Result<Box<dyn Atom>> {
        let mut args = vec!["install".to_string(), "-g".to_string()];
        args.extend(packages);

        Ok(Box::new(RunCommand {
            command: "npm".to_string(),
            args: Some(args),
            cwd: None,
            env: None,
            shell: None,
        }))
    }

    fn needs_privilege_escalation(&self) -> bool {
        // Global npm installs typically don't need privilege escalation
        // if npm is properly configured
        false
    }

    fn priority(&self) -> i32 {
        5 // Lower priority than system package managers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_npm_name() {
        assert_eq!(Npm.name(), "npm");
    }

    #[test]
    fn test_npm_needs_privilege_escalation() {
        assert!(!Npm.needs_privilege_escalation());
    }
}
