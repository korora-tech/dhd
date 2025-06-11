use dhd_macros::{typescript_fn, typescript_type};
use std::path::Path;
use crate::atoms::AtomCompat;

#[typescript_type]
pub struct SystemdManage {
    pub name: String,
    pub operation: String, // "enable", "disable", "start", "stop", "restart", "enable-now", "disable-now"
    pub scope: String,     // "user" or "system"
}

impl crate::actions::Action for SystemdManage {
    fn name(&self) -> &str {
        "SystemdManage"
    }

    fn plan(&self, _module_dir: &Path) -> Vec<Box<dyn crate::atom::Atom>> {
        let operation = match self.operation.as_str() {
            "enable" => crate::atoms::systemd_manage::SystemdOperation::Enable,
            "disable" => crate::atoms::systemd_manage::SystemdOperation::Disable,
            "start" => crate::atoms::systemd_manage::SystemdOperation::Start,
            "stop" => crate::atoms::systemd_manage::SystemdOperation::Stop,
            "restart" => crate::atoms::systemd_manage::SystemdOperation::Restart,
            "enable-now" => crate::atoms::systemd_manage::SystemdOperation::EnableNow,
            "disable-now" => crate::atoms::systemd_manage::SystemdOperation::DisableNow,
            _ => crate::atoms::systemd_manage::SystemdOperation::Enable, // Default to enable
        };

        vec![Box::new(AtomCompat::new(
            Box::new(crate::atoms::systemd_manage::SystemdManage::new(
                self.name.clone(),
                operation,
                self.scope.clone(),
            )),
            "systemd_manage".to_string(),
        ))]
    }
}

#[typescript_fn]
pub fn systemd_manage(config: SystemdManage) -> crate::actions::ActionType {
    crate::actions::ActionType::SystemdManage(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::Action;

    #[test]
    fn test_systemd_manage_creation() {
        let action = SystemdManage {
            name: "test.service".to_string(),
            operation: "enable".to_string(),
            scope: "user".to_string(),
        };

        assert_eq!(action.name, "test.service");
        assert_eq!(action.operation, "enable");
        assert_eq!(action.scope, "user");
    }

    #[test]
    fn test_systemd_manage_helper_function() {
        let action = systemd_manage(SystemdManage {
            name: "app.service".to_string(),
            operation: "start".to_string(),
            scope: "system".to_string(),
        });

        match action {
            crate::actions::ActionType::SystemdManage(manage) => {
                assert_eq!(manage.name, "app.service");
                assert_eq!(manage.operation, "start");
                assert_eq!(manage.scope, "system");
            }
            _ => panic!("Expected SystemdManage action type"),
        }
    }

    #[test]
    fn test_systemd_manage_name() {
        let action = SystemdManage {
            name: "test.service".to_string(),
            operation: "enable".to_string(),
            scope: "user".to_string(),
        };

        assert_eq!(action.name(), "SystemdManage");
    }

    #[test]
    fn test_systemd_manage_plan() {
        let action = SystemdManage {
            name: "test.service".to_string(),
            operation: "enable-now".to_string(),
            scope: "user".to_string(),
        };

        let atoms = action.plan(std::path::Path::new("."));
        assert_eq!(atoms.len(), 1);
    }

    #[test]
    fn test_systemd_manage_operations() {
        let operations = vec![
            "enable", "disable", "start", "stop", "restart", "enable-now", "disable-now"
        ];

        for op in operations {
            let action = SystemdManage {
                name: "test.service".to_string(),
                operation: op.to_string(),
                scope: "user".to_string(),
            };

            let atoms = action.plan(std::path::Path::new("."));
            assert_eq!(atoms.len(), 1);
        }
    }
}