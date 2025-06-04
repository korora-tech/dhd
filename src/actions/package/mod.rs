pub mod managers;

use crate::platform::PlatformInfo;
use crate::{Atom, Result};
use std::fmt::Debug;

/// Trait that all package managers must implement
pub trait PackageManager: Debug + Send + Sync {
    /// Get the name of the package manager
    fn name(&self) -> &'static str;

    /// Check if this package manager supports the given platform
    fn supports(&self, platform: &PlatformInfo) -> bool;

    /// Check if this package manager is available on the system
    fn is_available(&self) -> bool;

    /// Check if a package is installed
    fn is_installed(&self, package: &str) -> bool;

    /// Get the atoms needed to bootstrap this package manager
    /// For example, installing paru for AUR, or installing bun via npm
    fn bootstrap(&self) -> Result<Vec<Box<dyn Atom>>>;

    /// Create an atom to install packages with this manager
    fn install(&self, packages: Vec<String>) -> Result<Box<dyn Atom>>;

    /// Check if this manager needs privilege escalation for installation
    fn needs_privilege_escalation(&self) -> bool {
        false
    }

    /// Priority for auto-detection (higher = preferred)
    fn priority(&self) -> i32 {
        0
    }
}

/// Detect the best available package manager for the current system
pub fn detect_system_package_manager() -> Option<Box<dyn PackageManager>> {
    let platform = PlatformInfo::current();
    let managers = managers::all_system_managers();

    managers
        .into_iter()
        .filter(|m| m.supports(&platform) && m.is_available())
        .max_by_key(|m| m.priority())
}

/// Get a package manager by name
pub fn get_package_manager(name: &str) -> Result<Box<dyn PackageManager>> {
    let manager = managers::get_manager_by_name(name).ok_or_else(|| {
        crate::DhdError::AtomExecution(format!("Unknown package manager: {}", name))
    })?;

    let platform = PlatformInfo::current();
    if !manager.supports(&platform) {
        return Err(crate::DhdError::AtomExecution(format!(
            "Package manager '{}' is not supported on {} {}",
            name, platform.os_name, platform.os_type
        )));
    }

    Ok(manager)
}
