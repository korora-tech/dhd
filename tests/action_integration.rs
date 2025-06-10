use dhd::actions::{ExecuteCommand, PackageInstall, LinkDotfile, Action};
use dhd::atoms::package::PackageManager;
use tempfile::TempDir;
use std::fs;

#[test]
fn test_execute_command_with_args_and_escalate() {
    let action = ExecuteCommand {
        shell: None, // Should default to "sh"
        command: "systemctl".to_string(),
        args: Some(vec!["status".to_string(), "docker".to_string()]),
        escalate: true,
    };
    
    // Verify the action properties
    assert_eq!(action.command, "systemctl");
    assert_eq!(action.args, Some(vec!["status".to_string(), "docker".to_string()]));
    assert_eq!(action.escalate, true);
    
    // Test planning
    let atoms = action.plan(std::path::Path::new("."));
    assert_eq!(atoms.len(), 1);
    
    let atom = &atoms[0];
    assert_eq!(atom.name(), "RunCommand");
    
    // The describe should show escalation
    let description = atom.describe();
    assert!(description.contains("elevated"));
}

#[test]
fn test_execute_command_args_with_spaces() {
    let action = ExecuteCommand {
        shell: Some("bash".to_string()),
        command: "echo".to_string(),
        args: Some(vec![
            "hello world".to_string(),
            "with spaces".to_string(),
            "and \"quotes\"".to_string(),
        ]),
        escalate: false,
    };
    
    let atoms = action.plan(std::path::Path::new("."));
    assert_eq!(atoms.len(), 1);
    
    // The command should properly quote arguments with spaces
    let description = atoms[0].describe();
    assert!(description.contains("echo"));
}

#[test]
fn test_package_install_with_different_managers() {
    let managers = vec![
        PackageManager::Apt,
        PackageManager::Brew,
        PackageManager::Bun,
        PackageManager::Cargo,
        PackageManager::Flatpak,
    ];
    
    for manager in managers {
        let action = PackageInstall {
            names: vec!["test-package".to_string()],
            manager: Some(manager.clone()),
        };
        
        let atoms = action.plan(std::path::Path::new("."));
        assert_eq!(atoms.len(), 1);
        
        let description = atoms[0].describe();
        let provider = manager.get_provider();
        assert!(description.contains(provider.name()));
    }
}

#[test]
fn test_package_install_auto_detect() {
    let action = PackageInstall {
        names: vec!["vim".to_string(), "git".to_string()],
        manager: None, // Should auto-detect
    };
    
    let atoms = action.plan(std::path::Path::new("."));
    assert_eq!(atoms.len(), 1);
    
    // Even without a manager specified, it should create a valid atom
    let atom = &atoms[0];
    assert_eq!(atom.name(), "InstallPackages");
}

#[test]
fn test_link_dotfile_with_force() {
    let temp_dir = TempDir::new().unwrap();
    let source = temp_dir.path().join("source.conf");
    let target = temp_dir.path().join("target.conf");
    
    // Create source file
    fs::write(&source, "test content").unwrap();
    
    // Create an existing target that should be overwritten
    fs::write(&target, "old content").unwrap();
    
    let action = LinkDotfile {
        from: source.to_string_lossy().to_string(),
        to: target.to_string_lossy().to_string(),
        force: true,
    };
    
    let atoms = action.plan(&temp_dir.path());
    assert_eq!(atoms.len(), 1);
    
    // Execute the atom
    let result = atoms[0].execute();
    assert!(result.is_ok());
    
    // Verify the link was created
    assert!(target.exists());
    assert!(target.symlink_metadata().unwrap().file_type().is_symlink());
}

#[test]
fn test_complex_action_combination() {
    use dhd::actions::{ActionType, Action};
    
    let actions = vec![
        ActionType::PackageInstall(PackageInstall {
            names: vec!["docker".to_string(), "docker-compose".to_string()],
            manager: Some(PackageManager::Apt),
        }),
        ActionType::ExecuteCommand(ExecuteCommand {
            shell: None,
            command: "systemctl".to_string(),
            args: Some(vec!["enable".to_string(), "docker".to_string()]),
            escalate: true,
        }),
        ActionType::LinkDotfile(LinkDotfile {
            from: "docker-config.json".to_string(),
            to: "~/.docker/config.json".to_string(),
            force: false,
        }),
    ];
    
    // Test that all actions can be planned
    for action in &actions {
        let atoms = match action {
            ActionType::PackageInstall(a) => a.plan(std::path::Path::new(".")),
            ActionType::ExecuteCommand(a) => a.plan(std::path::Path::new(".")),
            ActionType::LinkDotfile(a) => a.plan(std::path::Path::new(".")),
            ActionType::CopyFile(a) => a.plan(std::path::Path::new(".")),
            ActionType::Directory(a) => a.plan(std::path::Path::new(".")),
            ActionType::HttpDownload(a) => a.plan(std::path::Path::new(".")),
            ActionType::SystemdSocket(a) => a.plan(std::path::Path::new(".")),
            ActionType::SystemdService(a) => a.plan(std::path::Path::new(".")),
        };
        assert!(!atoms.is_empty());
    }
}