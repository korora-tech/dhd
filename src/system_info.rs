use std::process::Command;

/// System information available to conditions
#[derive(Default)]
pub struct SystemInfo {
    pub os: OsInfo,
    pub hardware: HardwareInfo,
    pub auth: AuthInfo,
    pub user: UserInfo,
}

#[derive(Default)]
pub struct OsInfo {
    pub family: String,      // "debian", "fedora", "arch", etc.
    pub distro: String,      // "ubuntu", "fedora", etc.
    pub version: String,     // "22.04", "39", etc.
    pub codename: String,    // "jammy", etc.
}

#[derive(Default)]
pub struct HardwareInfo {
    pub fingerprint: bool,   // Has fingerprint reader
    pub tpm: bool,          // Has TPM
    pub gpu_vendor: String, // "nvidia", "amd", "intel"
}

#[derive(Default)]
pub struct AuthInfo {
    pub auth_type: String,  // "local", "central", "ldap"
    pub method: String,     // "password", "biometric", "smartcard"
}

#[derive(Default)]
pub struct UserInfo {
    pub name: String,
    pub shell: String,
    pub home: String,
}

/// Get current system information
pub fn get_system_info() -> SystemInfo {
    let mut info = SystemInfo::default();
    
    // Detect OS information
    if let Ok(os_release) = std::fs::read_to_string("/etc/os-release") {
        for line in os_release.lines() {
            if let Some((key, value)) = line.split_once('=') {
                let value = value.trim_matches('"');
                match key {
                    "ID" => info.os.distro = value.to_string(),
                    "ID_LIKE" => info.os.family = value.split_whitespace().next().unwrap_or("").to_string(),
                    "VERSION_ID" => info.os.version = value.to_string(),
                    "VERSION_CODENAME" => info.os.codename = value.to_string(),
                    _ => {}
                }
            }
        }
    }
    
    // If family wasn't set, use distro as family
    if info.os.family.is_empty() {
        info.os.family = info.os.distro.clone();
    }
    
    // Detect hardware
    // Check for fingerprint reader
    if let Ok(output) = Command::new("sh")
        .arg("-c")
        .arg("lsusb 2>/dev/null | grep -qi fingerprint || lshw -C biometric 2>/dev/null | grep -qi fingerprint")
        .output()
    {
        info.hardware.fingerprint = output.status.success();
    }
    
    // Get user info
    info.user.name = std::env::var("USER").unwrap_or_default();
    info.user.home = std::env::var("HOME").unwrap_or_default();
    info.user.shell = std::env::var("SHELL").unwrap_or_default();
    
    info
}