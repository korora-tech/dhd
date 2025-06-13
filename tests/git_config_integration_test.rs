use dhd::atoms::git_config::{GitConfig, GitConfigEntry, GitConfigScope};
use dhd::atoms::Atom;
use tempfile::TempDir;
use std::fs;
use std::env;

#[test]
fn test_git_config_writes_global_config() {
    // Create a temporary directory for our test home
    let temp_home = TempDir::new().unwrap();
    let original_home = env::var("HOME").ok();
    
    // Set HOME to our temp directory
    unsafe {
        env::set_var("HOME", temp_home.path());
    }
    
    // Create test entries
    let entries = vec![
        GitConfigEntry {
            key: "user.name".to_string(),
            value: "Test User".to_string(),
            add: None,
        },
        GitConfigEntry {
            key: "user.email".to_string(),
            value: "test@example.com".to_string(),
            add: None,
        },
        GitConfigEntry {
            key: "core.editor".to_string(),
            value: "vim".to_string(),
            add: None,
        },
        GitConfigEntry {
            key: "alias.co".to_string(),
            value: "checkout".to_string(),
            add: None,
        },
    ];
    
    // Create and execute the GitConfig atom
    let git_config = GitConfig::new(entries, GitConfigScope::Global, false);
    let result = git_config.execute();
    assert!(result.is_ok(), "Failed to execute git config: {:?}", result);
    
    // Verify the config file was created in XDG location
    let config_path = temp_home.path().join(".config/git/config");
    assert!(config_path.exists(), "Global git config file was not created at ~/.config/git/config");
    
    // Read and verify the content
    let content = fs::read_to_string(&config_path).unwrap();
    
    // Check that all our values are present
    assert!(content.contains("[user]"), "Missing [user] section");
    assert!(content.contains("name = Test User"), "Missing user.name");
    assert!(content.contains("email = test@example.com"), "Missing user.email");
    assert!(content.contains("[core]"), "Missing [core] section");
    assert!(content.contains("editor = vim"), "Missing core.editor");
    assert!(content.contains("[alias]"), "Missing [alias] section");
    assert!(content.contains("co = checkout"), "Missing alias.co");
    
    // Restore original HOME
    if let Some(home) = original_home {
        unsafe {
            env::set_var("HOME", home);
        }
    }
}

#[test]
fn test_git_config_writes_local_config() {
    // Create a temporary directory for our git repo
    let temp_repo = TempDir::new().unwrap();
    let original_dir = env::current_dir().unwrap();
    
    // Change to temp directory and create .git folder
    env::set_current_dir(&temp_repo).unwrap();
    fs::create_dir(".git").unwrap();
    
    // Create test entries
    let entries = vec![
        GitConfigEntry {
            key: "remote.origin.url".to_string(),
            value: "https://github.com/example/repo.git".to_string(),
            add: None,
        },
        GitConfigEntry {
            key: "branch.main.remote".to_string(),
            value: "origin".to_string(),
            add: None,
        },
        GitConfigEntry {
            key: "branch.main.merge".to_string(),
            value: "refs/heads/main".to_string(),
            add: None,
        },
    ];
    
    // Create and execute the GitConfig atom
    let git_config = GitConfig::new(entries, GitConfigScope::Local, false);
    let result = git_config.execute();
    assert!(result.is_ok(), "Failed to execute git config: {:?}", result);
    
    // Verify the config file was created
    let config_path = temp_repo.path().join(".git/config");
    assert!(config_path.exists(), "Local git config file was not created");
    
    // Read and verify the content
    let content = fs::read_to_string(&config_path).unwrap();
    
    // Check that all our values are present
    assert!(content.contains("[remote \"origin\"]"), "Missing [remote \"origin\"] section");
    assert!(content.contains("url = https://github.com/example/repo.git"), "Missing remote.origin.url");
    assert!(content.contains("[branch \"main\"]"), "Missing [branch \"main\"] section");
    assert!(content.contains("remote = origin"), "Missing branch.main.remote");
    assert!(content.contains("merge = refs/heads/main"), "Missing branch.main.merge");
    
    // Restore original directory
    env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_git_config_handles_multi_valued_keys() {
    // Create a temporary directory for our test home
    let temp_home = TempDir::new().unwrap();
    let original_home = env::var("HOME").ok();
    
    // Set HOME to our temp directory
    unsafe {
        env::set_var("HOME", temp_home.path());
    }
    
    // Create test entries with multi-valued keys
    let entries = vec![
        GitConfigEntry {
            key: "credential.helper".to_string(),
            value: "store".to_string(),
            add: Some(true),
        },
        GitConfigEntry {
            key: "credential.helper".to_string(),
            value: "cache --timeout=3600".to_string(),
            add: Some(true),
        },
    ];
    
    // Create and execute the GitConfig atom
    let git_config = GitConfig::new(entries, GitConfigScope::Global, false);
    let result = git_config.execute();
    assert!(result.is_ok(), "Failed to execute git config: {:?}", result);
    
    // Verify the config file was created in XDG location
    let config_path = temp_home.path().join(".config/git/config");
    assert!(config_path.exists(), "Global git config file was not created at ~/.config/git/config");
    
    // Read and verify the content
    let content = fs::read_to_string(&config_path).unwrap();
    
    // Check that all our values are present
    assert!(content.contains("[credential]"), "Missing [credential] section");
    assert!(content.contains("helper = store"), "Missing first credential.helper");
    assert!(content.contains("helper = cache --timeout=3600"), "Missing second credential.helper");
    
    // Restore original HOME
    if let Some(home) = original_home {
        unsafe {
            env::set_var("HOME", home);
        }
    }
}

#[test]
fn test_git_config_declarative_overwrites_existing() {
    // Create a temporary directory for our test home
    let temp_home = TempDir::new().unwrap();
    let original_home = env::var("HOME").ok();
    
    // Set HOME to our temp directory
    unsafe {
        env::set_var("HOME", temp_home.path());
    }
    
    // Create config directory and write an existing config file with extra content
    let config_dir = temp_home.path().join(".config/git");
    fs::create_dir_all(&config_dir).unwrap();
    let config_path = config_dir.join("config");
    fs::write(&config_path, r#"[user]
    name = Old User
    email = old@example.com
[core]
    editor = nano
[extra]
    setting = value
"#).unwrap();
    
    // Create test entries that should replace the entire file
    let entries = vec![
        GitConfigEntry {
            key: "user.name".to_string(),
            value: "New User".to_string(),
            add: None,
        },
        GitConfigEntry {
            key: "user.email".to_string(),
            value: "new@example.com".to_string(),
            add: None,
        },
    ];
    
    // Create and execute the GitConfig atom
    let git_config = GitConfig::new(entries, GitConfigScope::Global, false);
    let result = git_config.execute();
    assert!(result.is_ok(), "Failed to execute git config: {:?}", result);
    
    // Read and verify the content
    let content = fs::read_to_string(&config_path).unwrap();
    
    // Check that new values are present
    assert!(content.contains("name = New User"), "Missing new user.name");
    assert!(content.contains("email = new@example.com"), "Missing new user.email");
    
    // Check that old values are NOT present (declarative behavior)
    assert!(!content.contains("Old User"), "Old user name should not be present");
    assert!(!content.contains("old@example.com"), "Old email should not be present");
    assert!(!content.contains("editor = nano"), "Old editor should not be present");
    assert!(!content.contains("[extra]"), "Extra section should not be present");
    assert!(!content.contains("setting = value"), "Extra setting should not be present");
    
    // Restore original HOME
    if let Some(home) = original_home {
        unsafe {
            env::set_var("HOME", home);
        }
    }
}

#[test] 
fn test_git_config_error_when_not_in_git_repo() {
    // Create a temporary directory without .git
    let temp_dir = TempDir::new().unwrap();
    let original_dir = env::current_dir().unwrap();
    
    // Change to temp directory (no .git folder)
    env::set_current_dir(&temp_dir).unwrap();
    
    // Create test entries for local config
    let entries = vec![
        GitConfigEntry {
            key: "user.name".to_string(),
            value: "Test User".to_string(),
            add: None,
        },
    ];
    
    // Create and execute the GitConfig atom for local scope
    let git_config = GitConfig::new(entries, GitConfigScope::Local, false);
    let result = git_config.execute();
    
    // Should fail because we're not in a git repository
    assert!(result.is_err(), "Should fail when not in a git repository");
    assert!(result.unwrap_err().contains("Not in a git repository"), "Error should mention not being in a git repo");
    
    // Restore original directory
    env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_git_config_handles_credential_urls() {
    // Create a temporary directory for our test home
    let temp_home = TempDir::new().unwrap();
    let original_home = env::var("HOME").ok();
    
    // Set HOME to our temp directory
    unsafe {
        env::set_var("HOME", temp_home.path());
    }
    
    // Create test entries with credential URLs
    let entries = vec![
        GitConfigEntry {
            key: "credential.https://github.com.helper".to_string(),
            value: "!/usr/bin/gh auth git-credential".to_string(),
            add: None,
        },
        GitConfigEntry {
            key: "credential.https://gist.github.com.helper".to_string(),
            value: "!/usr/bin/gh auth git-credential".to_string(),
            add: None,
        },
    ];
    
    // Create and execute the GitConfig atom
    let git_config = GitConfig::new(entries, GitConfigScope::Global, false);
    let result = git_config.execute();
    assert!(result.is_ok(), "Failed to execute git config: {:?}", result);
    
    // Verify the config file was created in XDG location
    let config_path = temp_home.path().join(".config/git/config");
    assert!(config_path.exists(), "Global git config file was not created at ~/.config/git/config");
    
    // Read and verify the content
    let content = fs::read_to_string(&config_path).unwrap();
    
    // Check that credential sections with URLs are correctly formatted
    assert!(content.contains("[credential \"https://github.com\"]"), "Missing [credential \"https://github.com\"] section");
    assert!(content.contains("[credential \"https://gist.github.com\"]"), "Missing [credential \"https://gist.github.com\"] section");
    assert!(content.contains("helper = !/usr/bin/gh auth git-credential"), "Missing credential helper value");
    
    // Restore original HOME
    if let Some(home) = original_home {
        unsafe {
            env::set_var("HOME", home);
        }
    }
}