pub mod package;

use crate::{Atom, Result};
use std::fmt::Debug;

/// An Action is a high-level operation that can be planned into one or more Atoms
pub trait Action: Debug + Send + Sync {
    /// Plan this action into a list of atoms to be executed
    fn plan(&self) -> Result<Vec<Box<dyn Atom>>>;

    /// Get a description of what this action will do
    fn describe(&self) -> String;
}

/// PackageInstall action that implements the Action trait
#[derive(Debug)]
pub struct PackageInstall {
    pub packages: Vec<String>,
    pub manager: Option<String>,
}

impl PackageInstall {
    pub fn new(packages: Vec<String>, manager: Option<String>) -> Self {
        Self { packages, manager }
    }
}

impl Action for PackageInstall {
    fn plan(&self) -> Result<Vec<Box<dyn Atom>>> {
        if self.packages.is_empty() {
            return Ok(vec![]);
        }

        // Get the package manager
        let manager = if let Some(ref manager_name) = self.manager {
            package::get_package_manager(manager_name)?
        } else {
            package::detect_system_package_manager().ok_or_else(|| {
                crate::DhdError::AtomExecution(
                    "Could not detect system package manager".to_string(),
                )
            })?
        };

        let mut atoms: Vec<Box<dyn Atom>> = vec![];

        // First, add bootstrap atoms if needed
        atoms.extend(manager.bootstrap()?);

        // Then add the install atom
        atoms.push(manager.install(self.packages.clone())?);

        Ok(atoms)
    }

    fn describe(&self) -> String {
        let manager_str = self
            .manager
            .as_ref()
            .map(|m| format!(" using {}", m))
            .unwrap_or_else(|| " using system package manager".to_string());

        format!(
            "Install packages{}: {}",
            manager_str,
            self.packages.join(", ")
        )
    }
}

/// LinkDotfile action
#[derive(Debug)]
pub struct LinkDotfile {
    pub source: String,
    pub target: Option<String>,
    pub backup: bool,
    pub force: bool,
}

impl LinkDotfile {
    pub fn new(source: String, target: Option<String>, backup: bool, force: bool) -> Self {
        Self {
            source,
            target,
            backup,
            force,
        }
    }
}

impl Action for LinkDotfile {
    fn plan(&self) -> Result<Vec<Box<dyn Atom>>> {
        // Create a LinkDotfile atom
        let atom = crate::atoms::LinkDotfile::new(
            self.source.clone(),
            self.target.clone(),
            self.backup,
            self.force,
        );

        Ok(vec![Box::new(atom)])
    }

    fn describe(&self) -> String {
        format!(
            "Link dotfile: {} -> {}",
            self.source,
            self.target.as_ref().unwrap_or(&"(auto)".to_string())
        )
    }
}

/// ExecuteCommand action
#[derive(Debug)]
pub struct ExecuteCommand {
    pub command: String,
    pub args: Vec<String>,
    pub cwd: Option<String>,
    pub env: Option<std::collections::HashMap<String, String>>,
    pub shell: Option<String>,
}

impl ExecuteCommand {
    pub fn new(
        command: String,
        args: Vec<String>,
        cwd: Option<String>,
        env: Option<std::collections::HashMap<String, String>>,
        shell: Option<String>,
    ) -> Self {
        Self {
            command,
            args,
            cwd,
            env,
            shell,
        }
    }
}

impl Action for ExecuteCommand {
    fn plan(&self) -> Result<Vec<Box<dyn Atom>>> {
        // For now, create a simple RunCommand atom
        // In the future, we might want to check if the command exists
        // and potentially install it first
        let atom = crate::atoms::RunCommand {
            command: self.command.clone(),
            args: Some(self.args.clone()),
            cwd: self.cwd.clone(),
            env: self.env.clone(),
            shell: self.shell.clone(),
        };

        Ok(vec![Box::new(atom)])
    }

    fn describe(&self) -> String {
        // Quote arguments that contain spaces or special characters
        let quoted_args: Vec<String> = self
            .args
            .iter()
            .map(|arg| {
                if arg.contains(' ') || arg.contains('"') || arg.contains('\'') {
                    format!("\"{}\"", arg.replace('"', "\\\""))
                } else {
                    arg.to_string()
                }
            })
            .collect();
        format!("Execute: {} {}", self.command, quoted_args.join(" "))
    }
}
