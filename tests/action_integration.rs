use dhd::actions::{Action, ExecuteCommand, LinkFile, LinkDirectory, PackageInstall};
use dhd::atoms::package::PackageManager;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_execute_command_with_args_and_escalate() {
    let action = ExecuteCommand {
        shell: None, // Should default to "sh"
        command: "systemctl".to_string(),
        args: Some(vec!["status".to_string(), "docker".to_string()]),
        escalate: Some(true),
        environment: None,
    };

    // Verify the action properties
    assert_eq!(action.command, "systemctl");
    assert_eq!(
        action.args,
        Some(vec!["status".to_string(), "docker".to_string()])
    );
    assert_eq!(action.escalate, Some(true));

    // Test planning
    let atoms = action.plan(std::path::Path::new("."));
    assert_eq!(atoms.len(), 1);

    let atom = &atoms[0];
    // Check that the atom describes a run command
    assert!(atom.describe().contains("Run"));

    // The describe should show escalation
    let description = atom.describe();
    assert!(description.contains("elevated"));
}

#[test]
fn test_execute_command_args_with_spaces() {
    let action = ExecuteCommand {
        shell: Some("bash".to_string()),
        command: "git".to_string(),
        args: Some(vec![
            "commit".to_string(),
            "-m".to_string(),
            "feat: add new feature with spaces".to_string(),
            "--author".to_string(),
            "John Doe <john@example.com>".to_string(),
        ]),
        escalate: Some(false),
        environment: None,
    };

    let atoms = action.plan(std::path::Path::new("."));
    assert_eq!(atoms.len(), 1);

    // The command should properly quote arguments with spaces
    let description = atoms[0].describe();
    assert!(description.contains("git"));
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
            names: vec!["neovim".to_string()],
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
        names: vec!["ripgrep".to_string(), "fd-find".to_string(), "bat".to_string()],
        manager: None, // Should auto-detect
    };

    let atoms = action.plan(std::path::Path::new("."));
    assert_eq!(atoms.len(), 1);

    // Even without a manager specified, it should create a valid atom
    let atom = &atoms[0];
    // Check that the atom describes package installation
    assert!(atom.describe().contains("Install"));
}

#[test]
fn test_link_file_with_force() {
    let temp_dir = TempDir::new().unwrap();
    let source = temp_dir.path().join("link.conf");
    let target = temp_dir.path().join("target.conf");

    // Create target file (what the symlink will point to)
    fs::write(&target, "[user]\n    name = Developer\n    email = dev@example.com").unwrap();

    // Create an existing file at source location that should be overwritten
    fs::write(&source, "# Old git config").unwrap();

    let action = LinkFile {
        source: source.to_string_lossy().to_string(),
        target: target.to_string_lossy().to_string(),
        force: true,
    };

    let atoms = action.plan(temp_dir.path());
    assert_eq!(atoms.len(), 1);

    // Execute the atom
    let result = atoms[0].execute();
    assert!(result.is_ok());

    // Verify the link was created at source location
    assert!(source.exists());
    assert!(source.symlink_metadata().unwrap().file_type().is_symlink());
}

#[test]
fn test_link_directory_basic() {
    let temp_dir = TempDir::new().unwrap();
    let module_dir = temp_dir.path().join("module");
    fs::create_dir(&module_dir).unwrap();
    
    // Create a target directory in the module
    let target_dir = module_dir.join("nvim-config");
    fs::create_dir(&target_dir).unwrap();
    fs::write(target_dir.join("init.lua"), "-- Neovim configuration\nvim.o.number = true").unwrap();
    
    // Create the symlink location
    let source = temp_dir.path().join(".config/nvim");
    
    let action = LinkDirectory {
        source: source.to_string_lossy().to_string(),
        target: "nvim-config".to_string(),
        force: true,
    };
    
    let atoms = action.plan(&module_dir);
    assert_eq!(atoms.len(), 1);
    
    // Execute the atom
    let result = atoms[0].execute();
    assert!(result.is_ok(), "Failed to execute: {:?}", result);
    
    // Verify the symlink was created
    assert!(source.exists(), "Symlink was not created");
    assert!(source.symlink_metadata().unwrap().file_type().is_symlink(), "Created path is not a symlink");
    
    // Verify the symlink points to the correct target
    let link_target = fs::read_link(&source).unwrap();
    assert_eq!(link_target, target_dir, "Symlink points to wrong target");
    
    // Verify we can access content through the symlink
    let config_file = source.join("init.lua");
    assert!(config_file.exists(), "Cannot access file through symlink");
    let content = fs::read_to_string(&config_file).unwrap();
    assert!(content.contains("vim.o.number = true"), "Wrong content through symlink");
}

#[test]
fn test_link_directory_overwrites_with_force() {
    let temp_dir = TempDir::new().unwrap();
    let module_dir = temp_dir.path().join("module");
    fs::create_dir(&module_dir).unwrap();
    
    // Create a target directory in the module
    let target_dir = module_dir.join("ssh-config");
    fs::create_dir(&target_dir).unwrap();
    
    // Create an existing directory at source location
    let source = temp_dir.path().join(".ssh");
    fs::create_dir(&source).unwrap();
    fs::write(source.join("config"), "# Old SSH config").unwrap();
    
    let action = LinkDirectory {
        source: source.to_string_lossy().to_string(),
        target: "ssh-config".to_string(),
        force: true,
    };
    
    let atoms = action.plan(&module_dir);
    assert_eq!(atoms.len(), 1);
    
    // Execute should succeed with force=true
    let result = atoms[0].execute();
    assert!(result.is_ok(), "Failed to execute with force: {:?}", result);
    
    // Verify the symlink replaced the directory
    assert!(source.exists(), "Source path no longer exists");
    assert!(source.symlink_metadata().unwrap().file_type().is_symlink(), "Source is not a symlink");
    
    // Old file should not be accessible
    assert!(!source.join("config").exists(), "Old file still accessible");
}

#[test]
fn test_complex_action_combination() {
    use dhd::actions::{Action, ActionType};

    let actions = vec![
        ActionType::PackageInstall(PackageInstall {
            names: vec!["docker".to_string(), "docker-compose".to_string()],
            manager: Some(PackageManager::Apt),
        }),
        ActionType::ExecuteCommand(ExecuteCommand {
            shell: None,
            command: "systemctl".to_string(),
            args: Some(vec!["enable".to_string(), "docker".to_string()]),
            escalate: Some(true),
            environment: None,
        }),
        ActionType::LinkFile(LinkFile {
            source: "docker-config.json".to_string(),
            target: "~/.docker/config.json".to_string(),
            force: false,
        }),
    ];

    // Test that all actions can be planned
    for action in &actions {
        let atoms = match action {
            ActionType::PackageInstall(a) => a.plan(std::path::Path::new(".")),
            ActionType::ExecuteCommand(a) => a.plan(std::path::Path::new(".")),
            ActionType::LinkFile(a) => a.plan(std::path::Path::new(".")),
            ActionType::LinkDirectory(a) => a.plan(std::path::Path::new(".")),
            ActionType::CopyFile(a) => a.plan(std::path::Path::new(".")),
            ActionType::Directory(a) => a.plan(std::path::Path::new(".")),
            ActionType::HttpDownload(a) => a.plan(std::path::Path::new(".")),
            ActionType::SystemdSocket(a) => a.plan(std::path::Path::new(".")),
            ActionType::SystemdService(a) => a.plan(std::path::Path::new(".")),
            ActionType::Conditional(a) => a.plan(std::path::Path::new(".")),
            ActionType::DconfImport(a) => a.plan(std::path::Path::new(".")),
            ActionType::InstallGnomeExtensions(a) => a.plan(std::path::Path::new(".")),
            ActionType::PackageRemove(a) => a.plan(std::path::Path::new(".")),
            ActionType::SystemdManage(a) => a.plan(std::path::Path::new(".")),
            ActionType::GitConfig(a) => a.plan(std::path::Path::new(".")),
        };
        assert!(!atoms.is_empty());
    }
}
