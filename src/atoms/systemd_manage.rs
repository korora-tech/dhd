use crate::atoms::Atom;
use std::process::Command;

#[derive(Debug, Clone, PartialEq)]
pub enum SystemdOperation {
    Enable,
    Disable,
    Start,
    Stop,
    Restart,
    EnableNow,
    DisableNow,
}

#[derive(Debug, Clone)]
pub struct SystemdManage {
    pub name: String,
    pub operation: SystemdOperation,
    pub scope: String,
}

impl SystemdManage {
    pub fn new(name: String, operation: SystemdOperation, scope: String) -> Self {
        Self {
            name,
            operation,
            scope,
        }
    }

    fn get_systemctl_args(&self) -> Vec<&str> {
        let mut args = Vec::new();

        if self.scope == "user" {
            args.push("--user");
        }

        match self.operation {
            SystemdOperation::Enable => {
                args.push("enable");
                args.push(&self.name);
            }
            SystemdOperation::Disable => {
                args.push("disable");
                args.push(&self.name);
            }
            SystemdOperation::Start => {
                args.push("start");
                args.push(&self.name);
            }
            SystemdOperation::Stop => {
                args.push("stop");
                args.push(&self.name);
            }
            SystemdOperation::Restart => {
                args.push("restart");
                args.push(&self.name);
            }
            SystemdOperation::EnableNow => {
                args.push("enable");
                args.push("--now");
                args.push(&self.name);
            }
            SystemdOperation::DisableNow => {
                args.push("disable");
                args.push("--now");
                args.push(&self.name);
            }
        }

        args
    }
}

impl Atom for SystemdManage {
    fn name(&self) -> &str {
        "SystemdManage"
    }

    fn execute(&self) -> Result<(), String> {
        let args = self.get_systemctl_args();

        let output = Command::new("systemctl")
            .args(&args)
            .output()
            .map_err(|e| format!("Failed to execute systemctl: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "Failed to {} service {}: {}",
                match self.operation {
                    SystemdOperation::Enable => "enable",
                    SystemdOperation::Disable => "disable",
                    SystemdOperation::Start => "start",
                    SystemdOperation::Stop => "stop",
                    SystemdOperation::Restart => "restart",
                    SystemdOperation::EnableNow => "enable and start",
                    SystemdOperation::DisableNow => "disable and stop",
                },
                self.name,
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(())
    }

    fn describe(&self) -> String {
        format!(
            "{} systemd service: {}",
            match self.operation {
                SystemdOperation::Enable => "Enable",
                SystemdOperation::Disable => "Disable",
                SystemdOperation::Start => "Start",
                SystemdOperation::Stop => "Stop",
                SystemdOperation::Restart => "Restart",
                SystemdOperation::EnableNow => "Enable and start",
                SystemdOperation::DisableNow => "Disable and stop",
            },
            self.name
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_systemd_manage_creation() {
        let manage = SystemdManage::new(
            "test.service".to_string(),
            SystemdOperation::Enable,
            "user".to_string(),
        );

        assert_eq!(manage.name, "test.service");
        assert_eq!(manage.operation, SystemdOperation::Enable);
        assert_eq!(manage.scope, "user");
    }

    #[test]
    fn test_systemd_manage_name() {
        let manage = SystemdManage::new(
            "test.service".to_string(),
            SystemdOperation::Start,
            "system".to_string(),
        );

        assert_eq!(manage.name(), "SystemdManage");
    }

    #[test]
    fn test_systemd_manage_describe() {
        let test_cases = vec![
            (
                SystemdOperation::Enable,
                "Enable systemd service: test.service",
            ),
            (
                SystemdOperation::Disable,
                "Disable systemd service: test.service",
            ),
            (
                SystemdOperation::Start,
                "Start systemd service: test.service",
            ),
            (SystemdOperation::Stop, "Stop systemd service: test.service"),
            (
                SystemdOperation::Restart,
                "Restart systemd service: test.service",
            ),
            (
                SystemdOperation::EnableNow,
                "Enable and start systemd service: test.service",
            ),
            (
                SystemdOperation::DisableNow,
                "Disable and stop systemd service: test.service",
            ),
        ];

        for (operation, expected) in test_cases {
            let manage =
                SystemdManage::new("test.service".to_string(), operation, "user".to_string());
            assert_eq!(manage.describe(), expected);
        }
    }

    #[test]
    fn test_get_systemctl_args_user() {
        let manage = SystemdManage::new(
            "test.service".to_string(),
            SystemdOperation::Enable,
            "user".to_string(),
        );

        let args = manage.get_systemctl_args();
        assert_eq!(args, vec!["--user", "enable", "test.service"]);
    }

    #[test]
    fn test_get_systemctl_args_system() {
        let manage = SystemdManage::new(
            "test.service".to_string(),
            SystemdOperation::Start,
            "system".to_string(),
        );

        let args = manage.get_systemctl_args();
        assert_eq!(args, vec!["start", "test.service"]);
    }

    #[test]
    fn test_get_systemctl_args_enable_now() {
        let manage = SystemdManage::new(
            "app.service".to_string(),
            SystemdOperation::EnableNow,
            "user".to_string(),
        );

        let args = manage.get_systemctl_args();
        assert_eq!(args, vec!["--user", "enable", "--now", "app.service"]);
    }

    #[test]
    fn test_get_systemctl_args_disable_now() {
        let manage = SystemdManage::new(
            "daemon.service".to_string(),
            SystemdOperation::DisableNow,
            "system".to_string(),
        );

        let args = manage.get_systemctl_args();
        assert_eq!(args, vec!["disable", "--now", "daemon.service"]);
    }
}
