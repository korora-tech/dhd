use os_info::Type as OsType;

/// Platform information for checking system compatibility
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
        matches!(
            self.os_type,
            OsType::AlmaLinux
                | OsType::Alpine
                | OsType::Amazon
                | OsType::Arch
                | OsType::CentOS
                | OsType::Debian
                | OsType::EndeavourOS
                | OsType::Fedora
                | OsType::Gentoo
                | OsType::Manjaro
                | OsType::Mint
                | OsType::NixOS
                | OsType::openSUSE
                | OsType::OracleLinux
                | OsType::Pop
                | OsType::Raspbian
                | OsType::Redhat
                | OsType::RedHatEnterprise
                | OsType::SUSE
                | OsType::Ubuntu
                | OsType::Linux
        )
    }

    /// Check if running on any BSD variant
    pub fn is_bsd(&self) -> bool {
        matches!(
            self.os_type,
            OsType::FreeBSD | OsType::NetBSD | OsType::OpenBSD | OsType::DragonFly
        )
    }

    /// Check if the platform is Debian-based (Debian, Ubuntu, Mint, Pop, etc.)
    pub fn is_debian_based(&self) -> bool {
        matches!(
            self.os_type,
            OsType::Debian | OsType::Ubuntu | OsType::Mint | OsType::Pop | OsType::Raspbian
        )
    }

    /// Check if the platform is Arch-based (Arch, Manjaro, EndeavourOS)
    pub fn is_arch_based(&self) -> bool {
        matches!(
            self.os_type,
            OsType::Arch | OsType::Manjaro | OsType::EndeavourOS
        )
    }

    /// Check if the platform is RedHat-based
    pub fn is_redhat_based(&self) -> bool {
        matches!(
            self.os_type,
            OsType::Redhat
                | OsType::RedHatEnterprise
                | OsType::CentOS
                | OsType::Fedora
                | OsType::AlmaLinux
                | OsType::OracleLinux
        )
    }

    /// Check if the platform is SUSE-based
    pub fn is_suse_based(&self) -> bool {
        matches!(self.os_type, OsType::SUSE | OsType::openSUSE)
    }

    /// Check if running on macOS
    pub fn is_macos(&self) -> bool {
        self.os_type == OsType::Macos
    }

    /// Check if running on Windows
    pub fn is_windows(&self) -> bool {
        self.os_type == OsType::Windows
    }

    /// Check if the platform supports systemd
    pub fn has_systemd(&self) -> bool {
        // Most modern Linux distributions use systemd
        // BSD systems typically don't use systemd
        self.is_linux()
            && !matches!(
                self.os_type,
                OsType::Alpine | // Alpine uses OpenRC
            OsType::Gentoo // Gentoo can use various init systems
            )
    }

    /// Check if the platform supports dconf (GNOME configuration system)
    pub fn has_dconf(&self) -> bool {
        // dconf is primarily used on Linux systems with GNOME
        self.is_linux()
    }

    /// Get a human-readable platform description
    pub fn description(&self) -> String {
        format!(
            "{} {} ({})",
            self.info.os_type(),
            self.info.version(),
            self.arch
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_info_current() {
        let platform = PlatformInfo::current();
        assert!(!platform.os_name.is_empty());
        assert!(!platform.arch.is_empty());
    }

    #[test]
    fn test_platform_detection() {
        let platform = PlatformInfo::current();

        // At least one of these should be true
        let is_some_platform = platform.is_linux()
            || platform.is_macos()
            || platform.is_windows()
            || platform.is_bsd();

        assert!(is_some_platform);
    }
}
