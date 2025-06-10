use std::fs;
use std::path::PathBuf;
use std::process::Command;
use crate::atoms::Atom;

#[derive(Debug, Clone)]
pub struct SystemdSocket {
    pub name: String,
    pub description: String,
    pub listen_stream: String,
    pub scope: String,
}

impl SystemdSocket {
    pub fn new(name: String, description: String, listen_stream: String, scope: String) -> Self {
        Self {
            name,
            description,
            listen_stream,
            scope,
        }
    }

    fn get_socket_path(&self) -> PathBuf {
        if self.scope == "user" {
            let home = std::env::var("HOME").unwrap_or_else(|_| String::from("/home/user"));
            PathBuf::from(format!("{}/.config/systemd/user/{}", home, self.name))
        } else {
            PathBuf::from(format!("/etc/systemd/system/{}", self.name))
        }
    }

    fn generate_socket_content(&self) -> String {
        format!(
            "[Unit]\nDescription={}\n\n[Socket]\nListenStream={}\n\n[Install]\nWantedBy=sockets.target\n",
            self.description, self.listen_stream
        )
    }
}

impl Atom for SystemdSocket {
    fn name(&self) -> &str {
        "SystemdSocket"
    }

    fn execute(&self) -> Result<(), String> {
        let socket_path = self.get_socket_path();
        
        // Create parent directories if needed
        if let Some(parent) = socket_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create systemd directory: {}", e))?;
            }
        }

        // Write socket file
        let content = self.generate_socket_content();
        fs::write(&socket_path, content)
            .map_err(|e| format!("Failed to write socket file: {}", e))?;

        // Reload systemd and enable socket
        let reload_cmd = if self.scope == "user" {
            Command::new("systemctl")
                .args(["--user", "daemon-reload"])
                .output()
        } else {
            Command::new("systemctl")
                .args(["daemon-reload"])
                .output()
        };

        if let Ok(output) = reload_cmd {
            if !output.status.success() {
                return Err(format!("Failed to reload systemd: {}", 
                    String::from_utf8_lossy(&output.stderr)));
            }
        }

        Ok(())
    }

    fn describe(&self) -> String {
        format!("Create systemd socket: {}", self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_systemd_socket_creation() {
        let socket = SystemdSocket::new(
            "test.socket".to_string(),
            "Test socket".to_string(),
            "/tmp/test.sock".to_string(),
            "user".to_string(),
        );
        
        assert_eq!(socket.name, "test.socket");
        assert_eq!(socket.description, "Test socket");
        assert_eq!(socket.listen_stream, "/tmp/test.sock");
        assert_eq!(socket.scope, "user");
    }

    #[test]
    fn test_systemd_socket_name() {
        let socket = SystemdSocket::new(
            "test.socket".to_string(),
            "Test socket".to_string(),
            "/tmp/test.sock".to_string(),
            "user".to_string(),
        );
        
        assert_eq!(socket.name(), "SystemdSocket");
    }

    #[test]
    fn test_systemd_socket_describe() {
        let socket = SystemdSocket::new(
            "test.socket".to_string(),
            "Test socket".to_string(),
            "/tmp/test.sock".to_string(),
            "user".to_string(),
        );
        
        assert_eq!(socket.describe(), "Create systemd socket: test.socket");
    }

    #[test]
    fn test_systemd_socket_user_path() {
        unsafe { std::env::set_var("HOME", "/home/testuser"); }
        
        let socket = SystemdSocket::new(
            "test.socket".to_string(),
            "Test socket".to_string(),
            "/tmp/test.sock".to_string(),
            "user".to_string(),
        );
        
        let path = socket.get_socket_path();
        assert_eq!(path, std::path::PathBuf::from("/home/testuser/.config/systemd/user/test.socket"));
    }

    #[test]
    fn test_systemd_socket_system_path() {
        let socket = SystemdSocket::new(
            "test.socket".to_string(),
            "Test socket".to_string(),
            "/tmp/test.sock".to_string(),
            "system".to_string(),
        );
        
        let path = socket.get_socket_path();
        assert_eq!(path, std::path::PathBuf::from("/etc/systemd/system/test.socket"));
    }

    #[test]
    fn test_generate_socket_content() {
        let socket = SystemdSocket::new(
            "test.socket".to_string(),
            "Test socket for application".to_string(),
            "/run/test.sock".to_string(),
            "user".to_string(),
        );
        
        let content = socket.generate_socket_content();
        assert!(content.contains("[Unit]"));
        assert!(content.contains("Description=Test socket for application"));
        assert!(content.contains("[Socket]"));
        assert!(content.contains("ListenStream=/run/test.sock"));
        assert!(content.contains("[Install]"));
        assert!(content.contains("WantedBy=sockets.target"));
    }
}
