use crate::atom::Atom;
use serde::{Deserialize, Serialize};
use std::any::Any;

/// High-level action that can be planned into one or more atoms
pub trait Action: Send + Sync {
    /// Plan this action into a series of atoms to execute
    fn plan(&self) -> anyhow::Result<Vec<Box<dyn Atom>>>;

    /// Get a human-readable description of this action
    fn describe(&self) -> String;

    /// Get the module this action belongs to
    fn module(&self) -> &str;

    /// Convert to Any for downcasting
    fn as_any(&self) -> &dyn Any;
}

/// Platform selection for conditional logic
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum PlatformSelect<T> {
    /// Same value for all platforms
    All(T),
    /// Platform-specific values
    Platform {
        #[serde(skip_serializing_if = "Option::is_none")]
        linux: Option<LinuxSelect<T>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        mac: Option<T>,
        #[serde(skip_serializing_if = "Option::is_none")]
        windows: Option<T>,
    },
}

/// Linux distribution-specific selection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum LinuxSelect<T> {
    /// Same for all Linux distros
    All(T),
    /// Distro-specific values
    Distro {
        #[serde(skip_serializing_if = "Option::is_none")]
        ubuntu: Option<T>,
        #[serde(skip_serializing_if = "Option::is_none")]
        debian: Option<T>,
        #[serde(skip_serializing_if = "Option::is_none")]
        fedora: Option<T>,
        #[serde(skip_serializing_if = "Option::is_none")]
        arch: Option<T>,
        #[serde(skip_serializing_if = "Option::is_none")]
        nixos: Option<T>,
        #[serde(skip_serializing_if = "Option::is_none")]
        other: Option<T>,
    },
}

impl<T: Clone> PlatformSelect<T> {
    /// Resolve the platform-specific value for the current platform
    pub fn resolve(&self) -> Option<T> {
        use crate::platform::{LinuxDistro, Platform, current_platform};

        match self {
            PlatformSelect::All(value) => Some(value.clone()),
            PlatformSelect::Platform {
                linux,
                mac,
                windows,
            } => match current_platform() {
                Platform::Linux(distro) => linux.as_ref().and_then(|l| match l {
                    LinuxSelect::All(value) => Some(value.clone()),
                    LinuxSelect::Distro {
                        ubuntu,
                        debian,
                        fedora,
                        arch,
                        nixos,
                        other,
                    } => match distro {
                        LinuxDistro::Ubuntu => ubuntu.clone(),
                        LinuxDistro::Debian => debian.clone(),
                        LinuxDistro::Fedora => fedora.clone(),
                        LinuxDistro::Arch => arch.clone(),
                        LinuxDistro::NixOS => nixos.clone(),
                        LinuxDistro::Other => other.clone(),
                    },
                }),
                Platform::MacOS => mac.clone(),
                Platform::Windows => windows.clone(),
                Platform::Unknown => None,
            },
        }
    }
}
