use crate::atoms::Atom;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct SystemdService {
    pub name: String,
    pub description: String,
    pub exec_start: String,
    pub service_type: String,
    pub scope: String,
    pub restart: Option<String>,
    pub restart_sec: Option<u32>,
}

impl SystemdService {
    pub fn new(
        name: String,
        description: String,
        exec_start: String,
        service_type: String,
        scope: String,
        restart: Option<String>,
        restart_sec: Option<u32>,
    ) -> Self {
        Self {
            name,
            description,
            exec_start,
            service_type,
            scope,
            restart,
            restart_sec,
        }
    }

    fn get_service_path(&self) -> PathBuf {
        if self.scope == "user" {
            let home = std::env::var("HOME").unwrap_or_else(|_| String::from("/home/user"));
            PathBuf::from(format!("{}/.config/systemd/user/{}", home, self.name))
        } else {
            PathBuf::from(format!("/etc/systemd/system/{}", self.name))
        }
    }

    fn generate_service_content(&self) -> String {
        let mut content = format!(
            "[Unit]\nDescription={}\n\n[Service]\nType={}\nExecStart={}\n",
            self.description, self.service_type, self.exec_start
        );

        if let Some(restart) = &self.restart {
            content.push_str(&format!("Restart={}\n", restart));
        }

        if let Some(restart_sec) = self.restart_sec {
            content.push_str(&format!("RestartSec={}\n", restart_sec));
        }

        content.push_str("\n[Install]\nWantedBy=default.target\n");
        content
    }
}

impl Atom for SystemdService {
    fn name(&self) -> &str {
        "SystemdService"
    }

    fn execute(&self) -> Result<(), String> {
        let service_path = self.get_service_path();

        // Create parent directories if needed
        if let Some(parent) = service_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create systemd directory: {}", e))?;
            }
        }

        // Write service file
        let content = self.generate_service_content();
        fs::write(&service_path, content)
            .map_err(|e| format!("Failed to write service file: {}", e))?;

        // Reload systemd
        let reload_cmd = if self.scope == "user" {
            Command::new("systemctl")
                .args(["--user", "daemon-reload"])
                .output()
        } else {
            Command::new("systemctl").args(["daemon-reload"]).output()
        };

        if let Ok(output) = reload_cmd {
            if !output.status.success() {
                return Err(format!(
                    "Failed to reload systemd: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
        }

        Ok(())
    }

    fn describe(&self) -> String {
        format!("Create systemd service: {}", self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_systemd_service_creation() {
        let service = SystemdService::new(
            "test.service".to_string(),
            "Test service".to_string(),
            "/bin/test".to_string(),
            "simple".to_string(),
            "user".to_string(),
            Some("on-failure".to_string()),
            Some(5),
        );

        assert_eq!(service.name, "test.service");
        assert_eq!(service.description, "Test service");
        assert_eq!(service.exec_start, "/bin/test");
        assert_eq!(service.service_type, "simple");
        assert_eq!(service.scope, "user");
        assert_eq!(service.restart, Some("on-failure".to_string()));
        assert_eq!(service.restart_sec, Some(5));
    }

    #[test]
    fn test_systemd_service_name() {
        let service = SystemdService::new(
            "test.service".to_string(),
            "Test service".to_string(),
            "/bin/test".to_string(),
            "simple".to_string(),
            "user".to_string(),
            None,
            None,
        );

        assert_eq!(service.name(), "SystemdService");
    }

    #[test]
    fn test_systemd_service_describe() {
        let service = SystemdService::new(
            "test.service".to_string(),
            "Test service".to_string(),
            "/bin/test".to_string(),
            "simple".to_string(),
            "user".to_string(),
            None,
            None,
        );

        assert_eq!(service.describe(), "Create systemd service: test.service");
    }

    #[test]
    fn test_systemd_service_user_path() {
        unsafe {
            std::env::set_var("HOME", "/home/testuser");
        }

        let service = SystemdService::new(
            "test.service".to_string(),
            "Test service".to_string(),
            "/bin/test".to_string(),
            "simple".to_string(),
            "user".to_string(),
            None,
            None,
        );

        let path = service.get_service_path();
        assert_eq!(
            path,
            std::path::PathBuf::from("/home/testuser/.config/systemd/user/test.service")
        );
    }

    #[test]
    fn test_systemd_service_system_path() {
        let service = SystemdService::new(
            "test.service".to_string(),
            "Test service".to_string(),
            "/bin/test".to_string(),
            "simple".to_string(),
            "system".to_string(),
            None,
            None,
        );

        let path = service.get_service_path();
        assert_eq!(
            path,
            std::path::PathBuf::from("/etc/systemd/system/test.service")
        );
    }

    #[test]
    fn test_generate_service_content_basic() {
        let service = SystemdService::new(
            "test.service".to_string(),
            "Test service for application".to_string(),
            "/usr/bin/test".to_string(),
            "simple".to_string(),
            "user".to_string(),
            None,
            None,
        );

        let content = service.generate_service_content();
        assert!(content.contains("[Unit]"));
        assert!(content.contains("Description=Test service for application"));
        assert!(content.contains("[Service]"));
        assert!(content.contains("Type=simple"));
        assert!(content.contains("ExecStart=/usr/bin/test"));
        assert!(content.contains("[Install]"));
        assert!(content.contains("WantedBy=default.target"));
        assert!(!content.contains("Restart="));
        assert!(!content.contains("RestartSec="));
    }

    #[test]
    fn test_generate_service_content_with_restart() {
        let service = SystemdService::new(
            "daemon.service".to_string(),
            "Daemon service".to_string(),
            "/usr/bin/daemon".to_string(),
            "forking".to_string(),
            "system".to_string(),
            Some("always".to_string()),
            Some(10),
        );

        let content = service.generate_service_content();
        assert!(content.contains("[Unit]"));
        assert!(content.contains("Description=Daemon service"));
        assert!(content.contains("[Service]"));
        assert!(content.contains("Type=forking"));
        assert!(content.contains("ExecStart=/usr/bin/daemon"));
        assert!(content.contains("Restart=always"));
        assert!(content.contains("RestartSec=10"));
        assert!(content.contains("[Install]"));
        assert!(content.contains("WantedBy=default.target"));
    }

    #[test]
    fn test_generate_service_content_partial_restart() {
        let service = SystemdService::new(
            "partial.service".to_string(),
            "Partial restart service".to_string(),
            "/bin/partial".to_string(),
            "oneshot".to_string(),
            "user".to_string(),
            Some("on-failure".to_string()),
            None,
        );

        let content = service.generate_service_content();
        assert!(content.contains("Restart=on-failure"));
        assert!(!content.contains("RestartSec="));
    }
}
