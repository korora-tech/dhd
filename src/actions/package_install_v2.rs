use crate::{
    action::{Action, PlatformSelect},
    atom::Atom,
    platform::{LinuxDistro, Platform, current_platform},
};
use dhd_macros::{typescript_fn, typescript_type};
use serde::{Deserialize, Serialize};
use std::any::Any;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PackageManagerType {
    Auto,
    Apt,
    Brew,
    Bun,
    Cargo,
    Dnf,
    Flatpak,
    GitHub,
    Go,
    Npm,
    Pacman,
    Paru,
    Pip,
    Snap,
    Winget,
    Yum,
}

#[typescript_type]
pub struct PackageInstallConfig {
    pub names: PlatformSelect<Vec<String>>,
    pub manager: Option<PackageManagerType>,
    pub module: Option<String>,
}

#[derive(Clone)]
pub struct PackageInstallAction {
    config: PackageInstallConfig,
    module: String,
}

#[typescript_fn]
pub fn package_install(config: PackageInstallConfig) -> Box<dyn Action> {
    Box::new(PackageInstallAction {
        config,
        module: "unknown".to_string(),
    })
}

impl Action for PackageInstallAction {
    fn plan(&self) -> anyhow::Result<Vec<Box<dyn Atom>>> {
        // Resolve platform-specific package names
        let packages = self
            .config
            .names
            .resolve()
            .ok_or_else(|| anyhow::anyhow!("No packages specified for current platform"))?;

        if packages.is_empty() {
            return Ok(vec![]);
        }

        // Detect appropriate package manager
        let manager = match &self.config.manager {
            Some(PackageManagerType::Auto) | None => detect_package_manager()?,
            Some(manager) => manager.clone(),
        };

        Ok(vec![Box::new(
            crate::atoms::install_packages_v2::InstallPackagesAtom {
                packages,
                manager,
                module: self.module.clone(),
            },
        )])
    }

    fn describe(&self) -> String {
        match self.config.names.resolve() {
            Some(packages) => format!("Install packages: {}", packages.join(", ")),
            None => "Install packages (platform-specific)".to_string(),
        }
    }

    fn module(&self) -> &str {
        &self.module
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn detect_package_manager() -> anyhow::Result<PackageManagerType> {
    use which::which;

    match current_platform() {
        Platform::Linux(distro) => match distro {
            LinuxDistro::Ubuntu | LinuxDistro::Debian => {
                if which("apt").is_ok() {
                    Ok(PackageManagerType::Apt)
                } else {
                    Err(anyhow::anyhow!("apt not found"))
                }
            }
            LinuxDistro::Fedora => {
                if which("dnf").is_ok() {
                    Ok(PackageManagerType::Dnf)
                } else if which("yum").is_ok() {
                    Ok(PackageManagerType::Yum)
                } else {
                    Err(anyhow::anyhow!("dnf/yum not found"))
                }
            }
            LinuxDistro::Arch => {
                if which("paru").is_ok() {
                    Ok(PackageManagerType::Paru)
                } else if which("pacman").is_ok() {
                    Ok(PackageManagerType::Pacman)
                } else {
                    Err(anyhow::anyhow!("pacman not found"))
                }
            }
            _ => Err(anyhow::anyhow!("Unknown Linux distribution")),
        },
        Platform::MacOS => {
            if which("brew").is_ok() {
                Ok(PackageManagerType::Brew)
            } else {
                Err(anyhow::anyhow!("brew not found"))
            }
        }
        Platform::Windows => {
            if which("winget").is_ok() {
                Ok(PackageManagerType::Winget)
            } else {
                Err(anyhow::anyhow!("winget not found"))
            }
        }
        Platform::Unknown => Err(anyhow::anyhow!("Unknown platform")),
    }
}
