use crate::platform::PlatformInfo;
use crate::{Atom, DhdError, Result};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub struct SystemdSocket {
    name: String,
    content: String,
    user: bool, // true for user socket, false for system socket
    enable: bool,
    start: bool,
    reload: bool,
}

impl SystemdSocket {
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

    fn get_socket_path(&self) -> PathBuf {
        if self.user {
            let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"));
            config_dir.join("systemd/user").join(&self.name)
        } else {
            PathBuf::from("/etc/systemd/system").join(&self.name)
        }
    }

    fn ensure_socket_name(&self) -> String {
        let mut name = self.name.clone();
        if !name.ends_with(".socket") {
            name.push_str(".socket");
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

    fn sudo_systemctl_command(&self) -> Command {
        if self.user {
            self.systemctl_command()
        } else {
            Command::new("sudo")
        }
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

    fn get_socket_content(&self) -> Result<Option<String>> {
        let socket_path = self.get_socket_path();

        if socket_path.exists() {
            Ok(Some(fs::read_to_string(&socket_path)?))
        } else {
            Ok(None)
        }
    }

    fn is_socket_enabled(&self) -> Result<bool> {
        let socket_name = self.ensure_socket_name();
        let output = self
            .systemctl_command()
            .args(&["is-enabled", &socket_name])
            .output()?;

        Ok(output.status.success())
    }

    fn is_socket_active(&self) -> Result<bool> {
        let socket_name = self.ensure_socket_name();
        let output = self
            .systemctl_command()
            .args(&["is-active", &socket_name])
            .output()?;

        Ok(output.status.success())
    }

    fn daemon_reload(&self) -> Result<()> {
        let mut cmd = self.sudo_systemctl_command();

        if !self.user {
            cmd.args(&["systemctl", "daemon-reload"]);
        } else {
            cmd.arg("daemon-reload");
        }

        let status = cmd.status()?;

        if !status.success() {
            return Err(DhdError::AtomExecution(
                "Failed to reload systemd daemon".to_string(),
            ));
        }

        Ok(())
    }
}

impl Atom for SystemdSocket {
    fn check(&self) -> Result<bool> {
        // Check platform support first
        self.check_platform_support()?;

        // Check if systemd is available
        self.check_systemd_available()?;

        // Check if socket file content differs
        let current_content = self.get_socket_content()?;
        let content_differs = match current_content {
            Some(content) => content.trim() != self.content.trim(),
            None => true,
        };

        if content_differs {
            return Ok(true);
        }

        // Check if enable state differs
        if self.enable {
            let is_enabled = self.is_socket_enabled()?;
            if !is_enabled {
                return Ok(true);
            }
        }

        // Check if start state differs
        if self.start {
            let is_active = self.is_socket_active()?;
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

        let socket_path = self.get_socket_path();
        let socket_name = self.ensure_socket_name();

        // Create parent directory if it doesn't exist
        if let Some(parent) = socket_path.parent() {
            if !parent.exists() {
                if self.user {
                    fs::create_dir_all(parent)?;
                } else {
                    let status = Command::new("sudo")
                        .args(&["mkdir", "-p", parent.to_str().unwrap()])
                        .status()?;

                    if !status.success() {
                        return Err(DhdError::AtomExecution(
                            "Failed to create systemd directory".to_string(),
                        ));
                    }
                }
            }
        }

        // Write the socket file
        if self.user {
            fs::write(&socket_path, &self.content)?;
        } else {
            // Write to temp file first, then move with sudo
            let temp_file =
                std::env::temp_dir().join(format!("dhd_socket_{}.tmp", std::process::id()));
            fs::write(&temp_file, &self.content)?;

            let status = Command::new("sudo")
                .args(&[
                    "mv",
                    temp_file.to_str().unwrap(),
                    socket_path.to_str().unwrap(),
                ])
                .status()?;

            if !status.success() {
                let _ = fs::remove_file(&temp_file);
                return Err(DhdError::AtomExecution(
                    "Failed to install socket file".to_string(),
                ));
            }
        }

        tracing::info!("Created socket file: {}", socket_path.display());

        // Reload systemd daemon if requested
        if self.reload {
            self.daemon_reload()?;
            tracing::info!("Reloaded systemd daemon");
        }

        // Enable the socket if requested
        if self.enable {
            let mut cmd = self.sudo_systemctl_command();

            if !self.user {
                cmd.args(&["systemctl", "enable", &socket_name]);
            } else {
                cmd.args(&["enable", &socket_name]);
            }

            let status = cmd.status()?;

            if !status.success() {
                return Err(DhdError::AtomExecution(format!(
                    "Failed to enable socket: {}",
                    socket_name
                )));
            }

            tracing::info!("Enabled socket: {}", socket_name);
        }

        // Start the socket if requested
        if self.start {
            let mut cmd = self.sudo_systemctl_command();

            if !self.user {
                cmd.args(&["systemctl", "start", &socket_name]);
            } else {
                cmd.args(&["start", &socket_name]);
            }

            let status = cmd.status()?;

            if !status.success() {
                return Err(DhdError::AtomExecution(format!(
                    "Failed to start socket: {}",
                    socket_name
                )));
            }

            tracing::info!("Started socket: {}", socket_name);
        }

        Ok(())
    }

    fn describe(&self) -> String {
        let socket_name = self.ensure_socket_name();
        let mut desc = format!(
            "Create {} systemd socket: {}",
            if self.user { "user" } else { "system" },
            socket_name
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
    fn test_ensure_socket_name() {
        let socket1 = SystemdSocket::new(
            "myapp".to_string(),
            "[Unit]\nDescription=My App Socket".to_string(),
            false,
            false,
            false,
            false,
        );
        assert_eq!(socket1.ensure_socket_name(), "myapp.socket");

        let socket2 = SystemdSocket::new(
            "myapp.socket".to_string(),
            "[Unit]\nDescription=My App Socket".to_string(),
            false,
            false,
            false,
            false,
        );
        assert_eq!(socket2.ensure_socket_name(), "myapp.socket");
    }

    #[test]
    fn test_get_socket_path_system() {
        let socket = SystemdSocket::new(
            "test.socket".to_string(),
            "[Unit]\nDescription=Test Socket".to_string(),
            false,
            false,
            false,
            false,
        );

        let path = socket.get_socket_path();
        assert_eq!(path, PathBuf::from("/etc/systemd/system/test.socket"));
    }

    #[test]
    fn test_systemd_socket_describe() {
        let socket = SystemdSocket::new(
            "myapp".to_string(),
            "[Unit]\nDescription=My App Socket".to_string(),
            true,
            true,
            true,
            true,
        );

        assert_eq!(
            socket.describe(),
            "Create user systemd socket: myapp.socket (enable, start, reload)"
        );
    }

    #[test]
    fn test_systemd_socket_describe_minimal() {
        let socket = SystemdSocket::new(
            "myapp.socket".to_string(),
            "[Unit]\nDescription=My App Socket".to_string(),
            false,
            false,
            false,
            false,
        );

        assert_eq!(
            socket.describe(),
            "Create system systemd socket: myapp.socket"
        );
    }
}
