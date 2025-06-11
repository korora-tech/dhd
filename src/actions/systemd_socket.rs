use dhd_macros::{typescript_fn, typescript_type};

use crate::atoms::AtomCompat;
use std::path::Path;

#[typescript_type]
pub struct SystemdSocket {
    pub name: String,
    pub description: String,
    pub listen_stream: String,
    pub scope: String, // "user" or "system"
}

impl crate::actions::Action for SystemdSocket {
    fn name(&self) -> &str {
        "SystemdSocket"
    }

    fn plan(&self, _module_dir: &Path) -> Vec<Box<dyn crate::atom::Atom>> {
        vec![Box::new(AtomCompat::new(
            Box::new(crate::atoms::systemd_socket::SystemdSocket::new(
                self.name.clone(),
                self.description.clone(),
                self.listen_stream.clone(),
                self.scope.clone(),
            )),
            "systemd_socket".to_string(),
        ))]
    }
}

#[typescript_fn]
pub fn systemd_socket(config: SystemdSocket) -> crate::actions::ActionType {
    crate::actions::ActionType::SystemdSocket(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::Action;

    #[test]
    fn test_systemd_socket_creation() {
        let action = SystemdSocket {
            name: "test.socket".to_string(),
            description: "Test socket".to_string(),
            listen_stream: "/tmp/test.sock".to_string(),
            scope: "user".to_string(),
        };

        assert_eq!(action.name, "test.socket");
        assert_eq!(action.description, "Test socket");
        assert_eq!(action.listen_stream, "/tmp/test.sock");
        assert_eq!(action.scope, "user");
    }

    #[test]
    fn test_systemd_socket_helper_function() {
        let action = systemd_socket(SystemdSocket {
            name: "app.socket".to_string(),
            description: "App socket".to_string(),
            listen_stream: "/run/app.sock".to_string(),
            scope: "system".to_string(),
        });

        match action {
            crate::actions::ActionType::SystemdSocket(socket) => {
                assert_eq!(socket.name, "app.socket");
                assert_eq!(socket.description, "App socket");
                assert_eq!(socket.listen_stream, "/run/app.sock");
                assert_eq!(socket.scope, "system");
            }
            _ => panic!("Expected SystemdSocket action type"),
        }
    }

    #[test]
    fn test_systemd_socket_name() {
        let action = SystemdSocket {
            name: "test.socket".to_string(),
            description: "Test socket".to_string(),
            listen_stream: "/tmp/test.sock".to_string(),
            scope: "user".to_string(),
        };

        assert_eq!(action.name(), "SystemdSocket");
    }

    #[test]
    fn test_systemd_socket_plan() {
        let action = SystemdSocket {
            name: "test.socket".to_string(),
            description: "Test socket".to_string(),
            listen_stream: "/tmp/test.sock".to_string(),
            scope: "user".to_string(),
        };

        let atoms = action.plan(std::path::Path::new("."));
        assert_eq!(atoms.len(), 1);
        // Check that we got an atom
        assert_eq!(atoms.len(), 1);
    }

    #[test]
    fn test_systemd_socket_user_scope() {
        let action = SystemdSocket {
            name: "user-service.socket".to_string(),
            description: "User service socket".to_string(),
            listen_stream: "/tmp/user.sock".to_string(),
            scope: "user".to_string(),
        };

        let atoms = action.plan(std::path::Path::new("."));
        assert_eq!(atoms.len(), 1);
        assert!(atoms[0].describe().contains("systemd socket"));
    }

    #[test]
    fn test_systemd_socket_system_scope() {
        let action = SystemdSocket {
            name: "system-service.socket".to_string(),
            description: "System service socket".to_string(),
            listen_stream: "/run/system.sock".to_string(),
            scope: "system".to_string(),
        };

        let atoms = action.plan(std::path::Path::new("."));
        assert_eq!(atoms.len(), 1);
        assert!(atoms[0].describe().contains("systemd socket"));
    }
}
