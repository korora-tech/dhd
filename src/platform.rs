use once_cell::sync::Lazy;
use std::env;

#[derive(Debug, Clone, PartialEq)]
pub enum Platform {
    Linux(LinuxDistro),
    MacOS,
    Windows,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LinuxDistro {
    Ubuntu,
    Debian,
    Fedora,
    Arch,
    NixOS,
    Other,
}

pub static CURRENT_PLATFORM: Lazy<Platform> = Lazy::new(detect_platform);

pub fn current_platform() -> Platform {
    CURRENT_PLATFORM.clone()
}

fn detect_platform() -> Platform {
    match env::consts::OS {
        "linux" => Platform::Linux(detect_linux_distro()),
        "macos" => Platform::MacOS,
        "windows" => Platform::Windows,
        _ => Platform::Unknown,
    }
}

fn detect_linux_distro() -> LinuxDistro {
    // Try to read /etc/os-release
    if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
        let id = content
            .lines()
            .find(|line| line.starts_with("ID="))
            .and_then(|line| line.split('=').nth(1))
            .map(|s| s.trim_matches('"'));
        
        match id {
            Some("ubuntu") => LinuxDistro::Ubuntu,
            Some("debian") => LinuxDistro::Debian,
            Some("fedora") => LinuxDistro::Fedora,
            Some("arch") => LinuxDistro::Arch,
            Some("nixos") => LinuxDistro::NixOS,
            _ => LinuxDistro::Other,
        }
    } else {
        LinuxDistro::Other
    }
}