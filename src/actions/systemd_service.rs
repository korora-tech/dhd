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
            name: "postgres-backup.service".to_string(),
            description: "PostgreSQL Automated Backup Service".to_string(),
            exec_start: "/usr/local/bin/pg-backup.sh".to_string(),
            service_type: "oneshot".to_string(),
            scope: "system".to_string(),
            restart: Some("on-failure".to_string()),
            restart_sec: Some(30),
        };

        assert_eq!(action.name, "postgres-backup.service");
        assert_eq!(action.description, "PostgreSQL Automated Backup Service");
        assert_eq!(action.exec_start, "/usr/local/bin/pg-backup.sh");
        assert_eq!(action.service_type, "oneshot");
        assert_eq!(action.scope, "system");
        assert_eq!(action.restart, Some("on-failure".to_string()));
        assert_eq!(action.restart_sec, Some(30));
    }

    #[test]
    fn test_systemd_service_helper_function() {
        let action = systemd_service(SystemdService {
            name: "node-app.service".to_string(),
            description: "Node.js Production Application".to_string(),
            exec_start: "/usr/bin/node /var/www/app/server.js".to_string(),
            service_type: "simple".to_string(),
            scope: "system".to_string(),
            restart: Some("always".to_string()),
            restart_sec: Some(10),
        });

        match action {
            crate::actions::ActionType::SystemdService(service) => {
                assert_eq!(service.name, "node-app.service");
                assert_eq!(service.description, "Node.js Production Application");
                assert_eq!(service.exec_start, "/usr/bin/node /var/www/app/server.js");
                assert_eq!(service.service_type, "simple");
                assert_eq!(service.scope, "system");
                assert_eq!(service.restart, Some("always".to_string()));
                assert_eq!(service.restart_sec, Some(10));
            }
            _ => panic!("Expected SystemdService action type"),
        }
    }

    #[test]
    fn test_systemd_service_name() {
        let action = SystemdService {
            name: "redis-sentinel.service".to_string(),
            description: "Redis Sentinel Service".to_string(),
            exec_start: "/usr/bin/redis-sentinel /etc/redis/sentinel.conf".to_string(),
            service_type: "notify".to_string(),
            scope: "system".to_string(),
            restart: None,
            restart_sec: None,
        };

        assert_eq!(action.name(), "SystemdService");
    }

    #[test]
    fn test_systemd_service_plan() {
        let action = SystemdService {
            name: "docker-cleanup.service".to_string(),
            description: "Docker System Cleanup Timer".to_string(),
            exec_start: "/usr/bin/docker system prune -af".to_string(),
            service_type: "oneshot".to_string(),
            scope: "system".to_string(),
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
            name: "prometheus.service".to_string(),
            description: "Prometheus Monitoring Server".to_string(),
            exec_start: "/usr/local/bin/prometheus --config.file=/etc/prometheus/prometheus.yml".to_string(),
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
            name: "code-server.service".to_string(),
            description: "VS Code Server for Remote Development".to_string(),
            exec_start: "/home/developer/.local/bin/code-server --bind-addr 0.0.0.0:8080".to_string(),
            service_type: "simple".to_string(),
            scope: "user".to_string(),
            restart: Some("on-failure".to_string()),
            restart_sec: Some(5),
        };

        let atoms = action.plan(std::path::Path::new("."));
        assert_eq!(atoms.len(), 1);
        assert!(atoms[0].describe().contains("systemd service"));
    }
}
