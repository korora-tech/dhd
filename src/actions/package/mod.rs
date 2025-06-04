pub mod managers;

use crate::{Atom, Result};
use std::fmt::Debug;

use os_info::{Type as OsType};

/// Platform information for checking package manager support
#[derive(Debug, Clone)]
pub struct PlatformInfo {
    pub os_type: OsType,
    pub os_name: String,
    pub arch: String,
    pub info: os_info::Info,
}

impl PlatformInfo {
    pub fn current() -> Self {
        let info = os_info::get();
        let arch = std::env::consts::ARCH.to_string();
        
        Self {
            os_type: info.os_type(),
            os_name: info.os_type().to_string().to_lowercase(),
            arch,
            info,
        }
    }
    
    /// Check if the platform matches a specific OS type
    pub fn is_os(&self, os_type: OsType) -> bool {
        self.os_type == os_type
    }
    
    /// Check if running on any Linux distribution
    pub fn is_linux(&self) -> bool {
        matches!(self.os_type, 
            OsType::AlmaLinux | OsType::Alpine | OsType::Amazon | 
            OsType::Arch | OsType::CentOS | OsType::Debian | 
            OsType::EndeavourOS | OsType::Fedora | OsType::Gentoo |
            OsType::Manjaro | OsType::Mint | OsType::NixOS |
            OsType::openSUSE | OsType::OracleLinux | OsType::Pop |
            OsType::Raspbian | OsType::Redhat | OsType::RedHatEnterprise |
            OsType::Rocky | OsType::SUSE | OsType::Ubuntu | OsType::Linux
        )
    }
    
    /// Check if running on any BSD variant
    pub fn is_bsd(&self) -> bool {
        matches!(self.os_type,
            OsType::FreeBSD | OsType::NetBSD | OsType::OpenBSD | OsType::DragonFly
        )
    }
    
    /// Check if the platform is Debian-based (Debian, Ubuntu, Mint, Pop, etc.)
    pub fn is_debian_based(&self) -> bool {
        matches!(self.os_type,
            OsType::Debian | OsType::Ubuntu | OsType::Mint | 
            OsType::Pop | OsType::Raspbian
        )
    }
    
    /// Check if the platform is Arch-based (Arch, Manjaro, EndeavourOS)
    pub fn is_arch_based(&self) -> bool {
        matches!(self.os_type,
            OsType::Arch | OsType::Manjaro | OsType::EndeavourOS
        )
    }
    
    /// Check if the platform is RedHat-based
    pub fn is_redhat_based(&self) -> bool {
        matches!(self.os_type,
            OsType::Redhat | OsType::RedHatEnterprise | OsType::CentOS |
            OsType::Fedora | OsType::AlmaLinux | OsType::Rocky | OsType::OracleLinux
        )
    }
}

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
    let manager = managers::get_manager_by_name(name)
        .ok_or_else(|| crate::DhdError::AtomExecution(
            format!("Unknown package manager: {}", name)
        ))?;
    
    let platform = PlatformInfo::current();
    if !manager.supports(&platform) {
        return Err(crate::DhdError::AtomExecution(
            format!("Package manager '{}' is not supported on {} {}", 
                name, 
                platform.os_name,
                platform.os_type
            )
        ));
    }
    
    Ok(manager)
}