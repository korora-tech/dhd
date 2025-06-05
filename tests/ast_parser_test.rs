#[cfg(feature = "ast-parser")]
#[cfg(test)]
mod ast_parser_tests {
    use dhd::modules::ast_parser::AstModuleLoader;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    #[test]
    fn test_parse_simple_module() {
        let temp_dir = TempDir::new().unwrap();
        let module_path = temp_dir.path().join("test.ts");

        let module_content = r#"
import { defineModule, packageInstall } from "@dhd/types";

export default defineModule("test-module")
    .description("Test module")
    .tags("test", "example")
    .with(() => [
        packageInstall({
            names: ["vim", "git"],
            manager: "apt"
        })
    ]);
"#;

        fs::write(&module_path, module_content).unwrap();

        let loader = AstModuleLoader::new();
        let module = loader.load_module(&module_path).unwrap();

        assert_eq!(module.name, "test-module");
        assert_eq!(module.description, Some("Test module".to_string()));
        assert_eq!(module.tags, vec!["test", "example"]);
        assert_eq!(module.actions.len(), 1);

        let action = &module.actions[0];
        assert_eq!(action.action_type, "packageInstall");
        assert!(
            action
                .params
                .iter()
                .any(|(k, v)| k == "packages" && v == "vim, git")
        );
        assert!(
            action
                .params
                .iter()
                .any(|(k, v)| k == "manager" && v == "apt")
        );
    }

    #[test]
    fn test_parse_execute_command_with_privilege_escalation() {
        let temp_dir = TempDir::new().unwrap();
        let module_path = temp_dir.path().join("docker.ts");

        let module_content = r#"
import { defineModule, executeCommand, userGroup } from "@dhd/types";

export default defineModule("docker")
    .description("Docker setup")
    .with(() => [
        executeCommand({
            command: "systemctl",
            args: ["disable", "docker.service"],
            privilegeEscalation: true
        }),
        userGroup({
            user: "current",
            groups: ["docker", "libvirt"],
            append: true
        })
    ]);
"#;

        fs::write(&module_path, module_content).unwrap();

        let loader = AstModuleLoader::new();
        let module = loader.load_module(&module_path).unwrap();

        assert_eq!(module.name, "docker");
        assert_eq!(module.actions.len(), 2);

        // Check executeCommand
        let exec_action = &module.actions[0];
        assert_eq!(exec_action.action_type, "executeCommand");
        assert!(
            exec_action
                .params
                .iter()
                .any(|(k, v)| k == "command" && v == "systemctl")
        );
        assert!(
            exec_action
                .params
                .iter()
                .any(|(k, v)| k == "args" && v == "disable, docker.service")
        );
        assert!(
            exec_action
                .params
                .iter()
                .any(|(k, v)| k == "privilege_escalation" && v == "true")
        );

        // Check userGroup
        let user_action = &module.actions[1];
        assert_eq!(user_action.action_type, "userGroup");
        assert!(
            user_action
                .params
                .iter()
                .any(|(k, v)| k == "user" && v == "current")
        );
        assert!(
            user_action
                .params
                .iter()
                .any(|(k, v)| k == "groups" && v == "docker, libvirt")
        );
        assert!(
            user_action
                .params
                .iter()
                .any(|(k, v)| k == "append" && v == "true")
        );
    }

    #[test]
    fn test_parse_platform_select() {
        let temp_dir = TempDir::new().unwrap();
        let module_path = temp_dir.path().join("platform.ts");

        let module_content = r#"
import { defineModule, packageInstall } from "@dhd/types";

export default defineModule("platform-test")
    .description("Platform selection test")
    .with((ctx) => [
        packageInstall({
            names: ctx.platform.select({
                default: ["htop"],
                mac: ["htop", "btop"],
                windows: ["process-explorer"],
                linux: ["htop-linux"]
            })
        })
    ]);
"#;

        fs::write(&module_path, module_content).unwrap();

        let loader = AstModuleLoader::new();
        let module = loader.load_module(&module_path).unwrap();

        assert_eq!(module.name, "platform-test");
        assert_eq!(module.actions.len(), 1);

        let action = &module.actions[0];
        assert_eq!(action.action_type, "packageInstall");

        // The actual package selected depends on the current platform
        // We just verify that a selection was made
        assert!(action.params.iter().any(|(k, _)| k == "packages"));
    }

    #[test]
    fn test_parse_ctx_user() {
        let temp_dir = TempDir::new().unwrap();
        let module_path = temp_dir.path().join("user-test.ts");
        
        let module_content = r#"
import { defineModule, userGroup, linkDotfile } from "@dhd/types";

export default defineModule("user-test")
    .description("User context test")
    .with((ctx) => [
        userGroup({
            user: ctx.user,
            groups: ["docker", "wheel"]
        }),
        linkDotfile({
            source: "config/app.conf",
            target: ctx.user.homedir + "/.config/app.conf"
        })
    ]);
"#;
        
        fs::write(&module_path, module_content).unwrap();
        
        let loader = AstModuleLoader::new();
        let module = loader.load_module(&module_path).unwrap();
        
        assert_eq!(module.name, "user-test");
        assert_eq!(module.actions.len(), 2);
        
        // Check userGroup with ctx.user
        let user_action = &module.actions[0];
        assert_eq!(user_action.action_type, "userGroup");
        // Should have the actual username, not "ctx.user"
        let user_param = user_action.params.iter().find(|(k, _)| k == "user");
        assert!(user_param.is_some());
        let (_, username) = user_param.unwrap();
        assert!(!username.is_empty());
        assert_ne!(username, "ctx.user");
        
        // Check linkDotfile with ctx.user.homedir
        let link_action = &module.actions[1];
        assert_eq!(link_action.action_type, "linkDotfile");
        let target_param = link_action.params.iter().find(|(k, _)| k == "target");
        assert!(target_param.is_some());
        let (_, target) = target_param.unwrap();
        assert!(target.ends_with("/.config/app.conf"));
        assert!(!target.contains("ctx.user.homedir"));
    }

    #[test]
    fn test_parse_module_with_dependencies() {
        let temp_dir = TempDir::new().unwrap();
        let module_path = temp_dir.path().join("app.ts");

        let module_content = r#"
import { defineModule, linkDotfile } from "@dhd/types";

export default defineModule("app")
    .description("Application config")
    .depends("base", "git")
    .with(() => [
        linkDotfile({
            source: ".config/app.conf",
            target: "$HOME/.config/app.conf",
            backup: true
        })
    ]);
"#;

        fs::write(&module_path, module_content).unwrap();

        let loader = AstModuleLoader::new();
        let module = loader.load_module(&module_path).unwrap();

        assert_eq!(module.name, "app");
        assert_eq!(module.dependencies, vec!["base", "git"]);
        assert_eq!(module.actions.len(), 1);

        let action = &module.actions[0];
        assert_eq!(action.action_type, "linkDotfile");
        assert!(
            action
                .params
                .iter()
                .any(|(k, v)| k == "backup" && v == "true")
        );
    }
}
