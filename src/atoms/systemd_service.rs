use crate::atoms::systemd_unit_builder::{SystemdServiceContent, build_service_unit};
use crate::platform::PlatformInfo;
use crate::utils::execute_with_privilege_escalation;
use crate::{Atom, DhdError, Result};
use serde_json;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub struct SystemdService {
    name: String,
    content: String,
    user: bool, // true for user service, false for system service
    enable: bool,
    start: bool,
    reload: bool,
}

impl SystemdService {
    pub fn new(
        name: String,
        content: String,
        user: bool,
        enable: bool,
        start: bool,
        reload: bool,
    ) -> Self {
        Self {
            name,
            content,
            user,
            enable,
            start,
            reload,
        }
    }

    fn get_service_path(&self) -> PathBuf {
        if self.user {
            let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"));
            config_dir.join("systemd/user").join(&self.name)
        } else {
            PathBuf::from("/etc/systemd/system").join(&self.name)
        }
    }

    fn ensure_service_name(&self) -> String {
        let mut name = self.name.clone();
        if !name.ends_with(".service") {
            name.push_str(".service");
        }
        name
    }

    fn systemctl_command(&self) -> Command {
        let mut cmd = Command::new("systemctl");
        if self.user {
            cmd.arg("--user");
        }
        cmd
    }

    fn check_platform_support(&self) -> Result<()> {
        let platform = PlatformInfo::current();

        if !platform.has_systemd() {
            return Err(DhdError::AtomExecution(format!(
                "systemd is not supported on {}",
                platform.description()
            )));
        }

        Ok(())
    }

    fn check_systemd_available(&self) -> Result<()> {
        let output = Command::new("which").arg("systemctl").output()?;

        if !output.status.success() {
            return Err(DhdError::AtomExecution(
                "systemctl command not found. This system may not use systemd.".to_string(),
            ));
        }

        Ok(())
    }

    fn get_service_content(&self) -> Result<Option<String>> {
        let service_path = self.get_service_path();

        if service_path.exists() {
            Ok(Some(fs::read_to_string(&service_path)?))
        } else {
            Ok(None)
        }
    }

    fn is_service_enabled(&self) -> Result<bool> {
        let service_name = self.ensure_service_name();
        let output = self
            .systemctl_command()
            .args(["is-enabled", &service_name])
            .output()?;

        Ok(output.status.success())
    }

    fn is_service_active(&self) -> Result<bool> {
        let service_name = self.ensure_service_name();
        let output = self
            .systemctl_command()
            .args(["is-active", &service_name])
            .output()?;

        Ok(output.status.success())
    }

    fn daemon_reload(&self) -> Result<()> {
        if self.user {
            let status = self.systemctl_command().arg("daemon-reload").status()?;

            if !status.success() {
                return Err(DhdError::AtomExecution(
                    "Failed to reload systemd daemon".to_string(),
                ));
            }
        } else {
            let output = execute_with_privilege_escalation("systemctl", &["daemon-reload"])?;

            if !output.status.success() {
                return Err(DhdError::AtomExecution(
                    "Failed to reload systemd daemon".to_string(),
                ));
            }
        }

        Ok(())
    }
}

impl Atom for SystemdService {
    fn check(&self) -> Result<bool> {
        // Check platform support first
        self.check_platform_support()?;

        // Check if systemd is available
        self.check_systemd_available()?;

        // Check if service file content differs
        let current_content = self.get_service_content()?;

        // Determine the expected content
        let expected_content = if self.content.trim().starts_with('{') {
            // Try to parse as JSON
            match serde_json::from_str::<SystemdServiceContent>(&self.content) {
                Ok(typed_content) => build_service_unit(&typed_content),
                Err(_) => {
                    // If parsing fails, treat as raw content
                    self.content.clone()
                }
            }
        } else {
            // Raw unit file content
            self.content.clone()
        };

        let content_differs = match current_content {
            Some(content) => content.trim() != expected_content.trim(),
            None => true,
        };

        if content_differs {
            return Ok(true);
        }

        // Check if enable state differs
        if self.enable {
            let is_enabled = self.is_service_enabled()?;
            if !is_enabled {
                return Ok(true);
            }
        }

        // Check if start state differs
        if self.start {
            let is_active = self.is_service_active()?;
            if !is_active {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn execute(&self) -> Result<()> {
        // Check platform support first
        self.check_platform_support()?;

        // Check if systemd is available
        self.check_systemd_available()?;

        let service_path = self.get_service_path();
        let service_name = self.ensure_service_name();

        // Determine the content to write
        let unit_content = if self.content.trim().starts_with('{') {
            // Try to parse as JSON
            match serde_json::from_str::<SystemdServiceContent>(&self.content) {
                Ok(typed_content) => build_service_unit(&typed_content),
                Err(_) => {
                    // If parsing fails, treat as raw content
                    self.content.clone()
                }
            }
        } else {
            // Raw unit file content
            self.content.clone()
        };

        // Create parent directory if it doesn't exist
        if let Some(parent) = service_path.parent() {
            if !parent.exists() {
                if self.user {
                    fs::create_dir_all(parent)?;
                } else {
                    let output = execute_with_privilege_escalation(
                        "mkdir",
                        &["-p", parent.to_str().unwrap()],
                    )?;

                    if !output.status.success() {
                        return Err(DhdError::AtomExecution(
                            "Failed to create systemd directory".to_string(),
                        ));
                    }
                }
            }
        }

        // Write the service file
        if self.user {
            fs::write(&service_path, &unit_content)?;
        } else {
            // Write to temp file first, then move with sudo
            let temp_file =
                std::env::temp_dir().join(format!("dhd_service_{}.tmp", std::process::id()));
            fs::write(&temp_file, &unit_content)?;

            let output = execute_with_privilege_escalation(
                "mv",
                &[temp_file.to_str().unwrap(), service_path.to_str().unwrap()],
            )?;

            if !output.status.success() {
                let _ = fs::remove_file(&temp_file);
                return Err(DhdError::AtomExecution(
                    "Failed to install service file".to_string(),
                ));
            }
        }

        tracing::info!("Created service file: {}", service_path.display());

        // Reload systemd daemon if requested
        if self.reload {
            self.daemon_reload()?;
            tracing::info!("Reloaded systemd daemon");
        }

        // Enable the service if requested
        if self.enable {
            if self.user {
                let status = self
                    .systemctl_command()
                    .args(["enable", &service_name])
                    .status()?;

                if !status.success() {
                    return Err(DhdError::AtomExecution(format!(
                        "Failed to enable service: {}",
                        service_name
                    )));
                }
            } else {
                let output =
                    execute_with_privilege_escalation("systemctl", &["enable", &service_name])?;

                if !output.status.success() {
                    return Err(DhdError::AtomExecution(format!(
                        "Failed to enable service: {}",
                        service_name
                    )));
                }
            }

            tracing::info!("Enabled service: {}", service_name);
        }

        // Start the service if requested
        if self.start {
            if self.user {
                let status = self
                    .systemctl_command()
                    .args(["start", &service_name])
                    .status()?;

                if !status.success() {
                    return Err(DhdError::AtomExecution(format!(
                        "Failed to start service: {}",
                        service_name
                    )));
                }
            } else {
                let output =
                    execute_with_privilege_escalation("systemctl", &["start", &service_name])?;

                if !output.status.success() {
                    return Err(DhdError::AtomExecution(format!(
                        "Failed to start service: {}",
                        service_name
                    )));
                }
            }

            tracing::info!("Started service: {}", service_name);
        }

        Ok(())
    }

    fn describe(&self) -> String {
        let service_name = self.ensure_service_name();
        let mut desc = format!(
            "Create {} systemd service: {}",
            if self.user { "user" } else { "system" },
            service_name
        );

        let mut options = Vec::new();
        if self.enable {
            options.push("enable");
        }
        if self.start {
            options.push("start");
        }
        if self.reload {
            options.push("reload");
        }

        if !options.is_empty() {
            desc.push_str(&format!(" ({})", options.join(", ")));
        }

        desc
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ensure_service_name() {
        let service1 = SystemdService::new(
            "myapp".to_string(),
            "[Unit]\nDescription=My App".to_string(),
            false,
            false,
            false,
            false,
        );
        assert_eq!(service1.ensure_service_name(), "myapp.service");

        let service2 = SystemdService::new(
            "myapp.service".to_string(),
            "[Unit]\nDescription=My App".to_string(),
            false,
            false,
            false,
            false,
        );
        assert_eq!(service2.ensure_service_name(), "myapp.service");
    }

    #[test]
    fn test_get_service_path_system() {
        let service = SystemdService::new(
            "test.service".to_string(),
            "[Unit]\nDescription=Test".to_string(),
            false,
            false,
            false,
            false,
        );

        let path = service.get_service_path();
        assert_eq!(path, PathBuf::from("/etc/systemd/system/test.service"));
    }

    #[test]
    fn test_systemd_service_describe() {
        let service = SystemdService::new(
            "myapp".to_string(),
            "[Unit]\nDescription=My App".to_string(),
            true,
            true,
            true,
            true,
        );

        assert_eq!(
            service.describe(),
            "Create user systemd service: myapp.service (enable, start, reload)"
        );
    }

    #[test]
    fn test_systemd_service_describe_minimal() {
        let service = SystemdService::new(
            "myapp.service".to_string(),
            "[Unit]\nDescription=My App".to_string(),
            false,
            false,
            false,
            false,
        );

        assert_eq!(
            service.describe(),
            "Create system systemd service: myapp.service"
        );
    }
}
