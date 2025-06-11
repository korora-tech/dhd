use crate::{actions::package_install_v2::PackageManagerType, atom::Atom};
use std::any::Any;
use std::process::Command;

pub struct InstallPackagesAtom {
    pub packages: Vec<String>,
    pub manager: PackageManagerType,
    pub module: String,
}

impl Atom for InstallPackagesAtom {
    fn check(&self) -> anyhow::Result<bool> {
        // For now, always return true to execute
        // In a real implementation, we would check if packages are already installed
        Ok(true)
    }

    fn execute(&self) -> anyhow::Result<()> {
        let (cmd, args) = match &self.manager {
            PackageManagerType::Apt => ("sudo", vec!["apt", "install", "-y"]),
            PackageManagerType::Brew => ("brew", vec!["install"]),
            PackageManagerType::Bun => ("bun", vec!["add", "-g"]),
            PackageManagerType::Cargo => ("cargo", vec!["install"]),
            PackageManagerType::Dnf => ("sudo", vec!["dnf", "install", "-y"]),
            PackageManagerType::Flatpak => ("flatpak", vec!["install", "-y"]),
            PackageManagerType::Go => ("go", vec!["install"]),
            PackageManagerType::Npm => ("npm", vec!["install", "-g"]),
            PackageManagerType::Pacman => ("sudo", vec!["pacman", "-S", "--noconfirm"]),
            PackageManagerType::Paru => ("paru", vec!["-S", "--noconfirm"]),
            PackageManagerType::Pip => ("pip", vec!["install"]),
            PackageManagerType::Snap => ("sudo", vec!["snap", "install"]),
            PackageManagerType::Winget => ("winget", vec!["install", "-e", "--id"]),
            PackageManagerType::Yum => ("sudo", vec!["yum", "install", "-y"]),
            PackageManagerType::Auto => {
                return Err(anyhow::anyhow!(
                    "Auto package manager should have been resolved"
                ));
            }
        };

        let mut command = Command::new(cmd);
        for arg in args {
            command.arg(arg);
        }
        for package in &self.packages {
            command.arg(package);
        }

        let output = command.output()?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Failed to install packages: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(())
    }

    fn describe(&self) -> String {
        format!(
            "Install {} packages using {:?}",
            self.packages.len(),
            self.manager
        )
    }

    fn module(&self) -> &str {
        &self.module
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn id(&self) -> String {
        format!("{}::install_packages::{:?}", self.module, self.manager)
    }
}
