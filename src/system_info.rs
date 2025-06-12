use dhd_macros::typescript_type;
use serde::Serialize;
use sysinfo::System;

/// System information available to conditions
#[derive(Default, Serialize)]
#[typescript_type]
pub struct SystemInfo {
    pub os: OsInfo,
    pub hardware: HardwareInfo,
    pub auth: AuthInfo,
    pub user: UserInfo,
}

#[derive(Default, Serialize)]
#[typescript_type]
pub struct OsInfo {
    pub family: String,      // "debian", "fedora", "arch", etc.
    pub distro: String,      // "ubuntu", "fedora", etc.
    pub version: String,     // "22.04", "39", etc.
    pub codename: String,    // "jammy", etc.
}

#[derive(Default, Serialize)]
#[typescript_type]
pub struct HardwareInfo {
    pub fingerprint: bool,   // Has fingerprint reader
    pub tpm: bool,          // Has TPM
    pub gpu_vendor: String, // "nvidia", "amd", "intel"
}

#[derive(Default, Serialize)]
#[typescript_type]
pub struct AuthInfo {
    pub auth_type: String,  // "local", "central", "ldap"
    pub method: String,     // "password", "biometric", "smartcard"
}

#[derive(Default, Serialize)]
#[typescript_type]
pub struct UserInfo {
    pub name: String,
    pub shell: String,
    pub home: String,
}

/// Get current system information
pub fn get_system_info() -> SystemInfo {
    let mut info = SystemInfo::default();
    
    // Use os_info for better OS detection
    let os = os_info::get();
    
    // Set OS information
    info.os.distro = match os.os_type() {
        os_info::Type::Ubuntu => "ubuntu".to_string(),
        os_info::Type::Debian => "debian".to_string(),
        os_info::Type::Arch => "arch".to_string(),
        os_info::Type::Fedora => "fedora".to_string(),
        os_info::Type::CentOS => "centos".to_string(),
        os_info::Type::Redhat => "rhel".to_string(),
        os_info::Type::openSUSE => "opensuse".to_string(),
        os_info::Type::Manjaro => "manjaro".to_string(),
        os_info::Type::Mint => "mint".to_string(),
        os_info::Type::Pop => "pop".to_string(),
        os_info::Type::NixOS => "nixos".to_string(),
        os_info::Type::Gentoo => "gentoo".to_string(),
        os_info::Type::Alpine => "alpine".to_string(),
        _ => os.os_type().to_string().to_lowercase(),
    };
    
    // Determine OS family based on distro
    info.os.family = match info.os.distro.as_str() {
        "ubuntu" | "debian" | "mint" | "pop" => "debian".to_string(),
        "fedora" | "centos" | "rhel" => "fedora".to_string(),
        "arch" | "manjaro" => "arch".to_string(),
        "opensuse" => "suse".to_string(),
        _ => info.os.distro.clone(),
    };
    
    info.os.version = os.version().to_string();
    info.os.codename = os.codename().unwrap_or_default().to_string();
    
    // Detect hardware
    // Check for TPM
    info.hardware.tpm = std::path::Path::new("/dev/tpm0").exists() 
        || std::path::Path::new("/dev/tpmrm0").exists()
        || std::path::Path::new("/sys/class/tpm/tpm0").exists();
    
    // Check for fingerprint reader
    if let Ok(output) = std::process::Command::new("sh")
        .arg("-c")
        .arg("lsusb 2>/dev/null | grep -qi fingerprint")
        .output()
    {
        info.hardware.fingerprint = output.status.success();
    }
    
    // Detect GPU vendor
    if let Ok(output) = std::process::Command::new("lspci")
        .output()
    {
        if let Ok(lspci_str) = String::from_utf8(output.stdout) {
            for line in lspci_str.lines() {
                if line.contains("VGA") || line.contains("3D") || line.contains("Display") {
                    let line_lower = line.to_lowercase();
                    if line_lower.contains("nvidia") {
                        info.hardware.gpu_vendor = "nvidia".to_string();
                        break;
                    } else if line_lower.contains("amd") || line_lower.contains("radeon") || line_lower.contains("advanced micro devices") {
                        info.hardware.gpu_vendor = "amd".to_string();
                        break;
                    } else if line_lower.contains("intel") {
                        info.hardware.gpu_vendor = "intel".to_string();
                        break;
                    }
                }
            }
        }
    }
    
    // Get user info from environment
    info.user.name = std::env::var("USER").unwrap_or_default();
    info.user.home = std::env::var("HOME").unwrap_or_default();
    info.user.shell = std::env::var("SHELL").unwrap_or_default();
    
    // Use sysinfo for additional system information if needed in the future
    let mut _sys = System::new();
    
    info
}