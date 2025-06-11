use crate::atoms::AtomCompat;
use dhd_macros::{typescript_fn, typescript_type};
use std::path::Path;

#[typescript_type]
pub struct SystemdService {
    pub name: String,
    pub description: String,
    pub exec_start: String,
    pub service_type: String,
    pub scope: String, // "user" or "system"
    pub restart: Option<String>,
    pub restart_sec: Option<u32>,
}

impl crate::actions::Action for SystemdService {
    fn name(&self) -> &str {
        "SystemdService"
    }

    fn plan(&self, _module_dir: &Path) -> Vec<Box<dyn crate::atom::Atom>> {
        vec![Box::new(AtomCompat::new(
            Box::new(crate::atoms::systemd_service::SystemdService::new(
                self.name.clone(),
                self.description.clone(),
                self.exec_start.clone(),
                self.service_type.clone(),
                self.scope.clone(),
                self.restart.clone(),
                self.restart_sec,
            )),
            "systemd_service".to_string(),
        ))]
    }
}

#[typescript_fn]
pub fn systemd_service(config: SystemdService) -> crate::actions::ActionType {
    crate::actions::ActionType::SystemdService(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::Action;

    #[test]
    fn test_systemd_service_creation() {
        let action = SystemdService {
            name: "test.service".to_string(),
            description: "Test service".to_string(),
            exec_start: "/bin/test".to_string(),
            service_type: "simple".to_string(),
            scope: "user".to_string(),
            restart: Some("on-failure".to_string()),
            restart_sec: Some(5),
        };

        assert_eq!(action.name, "test.service");
        assert_eq!(action.description, "Test service");
        assert_eq!(action.exec_start, "/bin/test");
        assert_eq!(action.service_type, "simple");
        assert_eq!(action.scope, "user");
        assert_eq!(action.restart, Some("on-failure".to_string()));
        assert_eq!(action.restart_sec, Some(5));
    }

    #[test]
    fn test_systemd_service_helper_function() {
        let action = systemd_service(SystemdService {
            name: "app.service".to_string(),
            description: "App service".to_string(),
            exec_start: "/usr/bin/app".to_string(),
            service_type: "forking".to_string(),
            scope: "system".to_string(),
            restart: None,
            restart_sec: None,
        });

        match action {
            crate::actions::ActionType::SystemdService(service) => {
                assert_eq!(service.name, "app.service");
                assert_eq!(service.description, "App service");
                assert_eq!(service.exec_start, "/usr/bin/app");
                assert_eq!(service.service_type, "forking");
                assert_eq!(service.scope, "system");
                assert!(service.restart.is_none());
                assert!(service.restart_sec.is_none());
            }
            _ => panic!("Expected SystemdService action type"),
        }
    }

    #[test]
    fn test_systemd_service_name() {
        let action = SystemdService {
            name: "test.service".to_string(),
            description: "Test service".to_string(),
            exec_start: "/bin/test".to_string(),
            service_type: "simple".to_string(),
            scope: "user".to_string(),
            restart: None,
            restart_sec: None,
        };

        assert_eq!(action.name(), "SystemdService");
    }

    #[test]
    fn test_systemd_service_plan() {
        let action = SystemdService {
            name: "test.service".to_string(),
            description: "Test service".to_string(),
            exec_start: "/bin/test".to_string(),
            service_type: "simple".to_string(),
            scope: "user".to_string(),
            restart: None,
            restart_sec: None,
        };

        let atoms = action.plan(std::path::Path::new("."));
        assert_eq!(atoms.len(), 1);
        // Check that we got an atom
        assert_eq!(atoms.len(), 1);
    }

    #[test]
    fn test_systemd_service_with_restart() {
        let action = SystemdService {
            name: "daemon.service".to_string(),
            description: "Daemon service".to_string(),
            exec_start: "/usr/bin/daemon".to_string(),
            service_type: "simple".to_string(),
            scope: "system".to_string(),
            restart: Some("always".to_string()),
            restart_sec: Some(10),
        };

        let atoms = action.plan(std::path::Path::new("."));
        assert_eq!(atoms.len(), 1);
        assert!(atoms[0].describe().contains("systemd service"));
    }

    #[test]
    fn test_systemd_service_user_scope() {
        let action = SystemdService {
            name: "user-app.service".to_string(),
            description: "User application".to_string(),
            exec_start: "/home/user/bin/app".to_string(),
            service_type: "simple".to_string(),
            scope: "user".to_string(),
            restart: None,
            restart_sec: None,
        };

        let atoms = action.plan(std::path::Path::new("."));
        assert_eq!(atoms.len(), 1);
        assert!(atoms[0].describe().contains("systemd service"));
    }
}
