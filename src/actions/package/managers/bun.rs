use crate::actions::package::PackageManager;
use crate::atoms::RunCommand;
use crate::platform::PlatformInfo;
use crate::{Atom, Result};
use std::process::Command;

/// Bun package manager and runtime
#[derive(Debug)]
pub struct Bun;

impl PackageManager for Bun {
    fn name(&self) -> &'static str {
        "bun"
    }

    fn supports(&self, platform: &PlatformInfo) -> bool {
        // Bun supports macOS, Linux, and WSL (not native Windows yet)
        platform.is_os(os_info::Type::Macos) || platform.is_linux()
    }

    fn is_available(&self) -> bool {
        Command::new("bun")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn is_installed(&self, package: &str) -> bool {
        // Check globally installed packages
        Command::new("bun")
            .args(&["pm", "ls", "-g"])
            .output()
            .map(|output| {
                output.status.success() && String::from_utf8_lossy(&output.stdout).contains(package)
            })
            .unwrap_or(false)
    }

    fn bootstrap(&self) -> Result<Vec<Box<dyn Atom>>> {
        if self.is_available() {
            return Ok(vec![]);
        }

        // Install bun using the official install script
        Ok(vec![Box::new(RunCommand {
            command: "/bin/bash".to_string(),
            args: Some(vec![
                "-c".to_string(),
                "curl -fsSL https://bun.sh/install | bash".to_string(),
            ]),
            cwd: None,
            env: None,
            shell: None,
        })])
    }

    fn install(&self, packages: Vec<String>) -> Result<Box<dyn Atom>> {
        let mut args = vec!["add".to_string(), "-g".to_string()];
        args.extend(packages);

        Ok(Box::new(RunCommand {
            command: "bun".to_string(),
            args: Some(args),
            cwd: None,
            env: None,
            shell: None,
        }))
    }

    fn needs_privilege_escalation(&self) -> bool {
        false
    }

    fn priority(&self) -> i32 {
        8 // Higher priority than npm for performance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bun_name() {
        assert_eq!(Bun.name(), "bun");
    }

    #[test]
    fn test_bun_needs_privilege_escalation() {
        assert!(!Bun.needs_privilege_escalation());
    }
}
