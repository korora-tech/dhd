use crate::actions::package::PackageManager;
use crate::atoms::RunCommand;
use crate::platform::PlatformInfo;
use crate::{Atom, Result};
use std::process::Command;

/// Deno runtime and package manager
#[derive(Debug)]
pub struct Deno;

impl PackageManager for Deno {
    fn name(&self) -> &'static str {
        "deno"
    }

    fn supports(&self, _platform: &PlatformInfo) -> bool {
        // Deno works on all major platforms
        true
    }

    fn is_available(&self) -> bool {
        Command::new("deno")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn is_installed(&self, package: &str) -> bool {
        // Deno doesn't have a traditional package install system
        // Scripts are run directly from URLs or local files
        // We'll check if a script exists in the install directory
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let install_path = format!("{}/.deno/bin/{}", home, package);
        std::path::Path::new(&install_path).exists()
    }

    fn bootstrap(&self) -> Result<Vec<Box<dyn Atom>>> {
        if self.is_available() {
            return Ok(vec![]);
        }

        // Install Deno using the official install script
        Ok(vec![Box::new(RunCommand {
            command: "/bin/bash".to_string(),
            args: Some(vec![
                "-c".to_string(),
                "curl -fsSL https://deno.land/install.sh | sh".to_string(),
            ]),
            cwd: None,
            env: None,
        })])
    }

    fn install(&self, packages: Vec<String>) -> Result<Box<dyn Atom>> {
        // Deno installs are actually script installations
        // The package should be a URL or a module name from deno.land/x
        let mut args = vec!["install".to_string(), "--allow-all".to_string()];

        for package in packages {
            // If it looks like a URL, use it directly
            if package.starts_with("http://") || package.starts_with("https://") {
                args.push(package);
            } else {
                // Otherwise, assume it's a module from deno.land/x
                args.push(format!("https://deno.land/x/{}/mod.ts", package));
            }
        }

        Ok(Box::new(RunCommand {
            command: "deno".to_string(),
            args: Some(args),
            cwd: None,
            env: None,
        }))
    }

    fn needs_privilege_escalation(&self) -> bool {
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
    fn test_deno_name() {
        assert_eq!(Deno.name(), "deno");
    }

    #[test]
    fn test_deno_needs_privilege_escalation() {
        assert!(!Deno.needs_privilege_escalation());
    }
}
