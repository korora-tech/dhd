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
            value: "John Developer".to_string(),
            add: None,
        },
        GitConfigEntry {
            key: "user.email".to_string(),
            value: "john.developer@company.com".to_string(),
            add: None,
        },
        GitConfigEntry {
            key: "core.editor".to_string(),
            value: "code --wait".to_string(),
            add: None,
        },
        GitConfigEntry {
            key: "init.defaultBranch".to_string(),
            value: "main".to_string(),
            add: None,
        },
        GitConfigEntry {
            key: "pull.rebase".to_string(),
            value: "true".to_string(),
            add: None,
        },
        GitConfigEntry {
            key: "alias.lg".to_string(),
            value: "log --oneline --graph --decorate --all".to_string(),
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
    assert!(content.contains("name = John Developer"), "Missing user.name");
    assert!(content.contains("email = john.developer@company.com"), "Missing user.email");
    assert!(content.contains("[core]"), "Missing [core] section");
    assert!(content.contains("editor = code --wait"), "Missing core.editor");
    assert!(content.contains("[init]"), "Missing [init] section");
    assert!(content.contains("defaultBranch = main"), "Missing init.defaultBranch");
    assert!(content.contains("[pull]"), "Missing [pull] section");
    assert!(content.contains("rebase = true"), "Missing pull.rebase");
    assert!(content.contains("[alias]"), "Missing [alias] section");
    assert!(content.contains("lg = log --oneline --graph --decorate --all"), "Missing alias.lg");
    
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
            value: "git@github.com:company/production-app.git".to_string(),
            add: None,
        },
        GitConfigEntry {
            key: "remote.upstream.url".to_string(),
            value: "https://github.com/original/production-app.git".to_string(),
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
        GitConfigEntry {
            key: "core.hooksPath".to_string(),
            value: ".githooks".to_string(),
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
    assert!(content.contains("url = git@github.com:company/production-app.git"), "Missing remote.origin.url");
    assert!(content.contains("[remote \"upstream\"]"), "Missing [remote \"upstream\"] section");
    assert!(content.contains("url = https://github.com/original/production-app.git"), "Missing remote.upstream.url");
    assert!(content.contains("[branch \"main\"]"), "Missing [branch \"main\"] section");
    assert!(content.contains("remote = origin"), "Missing branch.main.remote");
    assert!(content.contains("merge = refs/heads/main"), "Missing branch.main.merge");
    assert!(content.contains("hooksPath = .githooks"), "Missing core.hooksPath");
    
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
            value: "osxkeychain".to_string(),
            add: Some(true),
        },
        GitConfigEntry {
            key: "credential.helper".to_string(),
            value: "manager-core".to_string(),
            add: Some(true),
        },
        GitConfigEntry {
            key: "http.postBuffer".to_string(),
            value: "524288000".to_string(),
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
    assert!(content.contains("[credential]"), "Missing [credential] section");
    assert!(content.contains("helper = osxkeychain"), "Missing first credential.helper");
    assert!(content.contains("helper = manager-core"), "Missing second credential.helper");
    assert!(content.contains("[http]"), "Missing [http] section");
    assert!(content.contains("postBuffer = 524288000"), "Missing http.postBuffer for large files");
    
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
    name = Old Developer
    email = old.dev@oldcompany.com
    signingkey = ABC123DEF456
[core]
    editor = emacs
    autocrlf = true
[commit]
    gpgsign = true
[extra]
    customSetting = oldValue
"#).unwrap();
    
    // Create test entries that should replace the entire file
    let entries = vec![
        GitConfigEntry {
            key: "user.name".to_string(),
            value: "Current Developer".to_string(),
            add: None,
        },
        GitConfigEntry {
            key: "user.email".to_string(),
            value: "current.dev@newcompany.com".to_string(),
            add: None,
        },
        GitConfigEntry {
            key: "core.editor".to_string(),
            value: "nvim".to_string(),
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
    assert!(content.contains("name = Current Developer"), "Missing new user.name");
    assert!(content.contains("email = current.dev@newcompany.com"), "Missing new user.email");
    assert!(content.contains("editor = nvim"), "Missing new core.editor");
    
    // Check that old values are NOT present (declarative behavior)
    assert!(!content.contains("Old Developer"), "Old user name should not be present");
    assert!(!content.contains("old.dev@oldcompany.com"), "Old email should not be present");
    assert!(!content.contains("signingkey"), "Old signing key should not be present");
    assert!(!content.contains("emacs"), "Old editor should not be present");
    assert!(!content.contains("autocrlf"), "Old autocrlf setting should not be present");
    assert!(!content.contains("[commit]"), "Commit section should not be present");
    assert!(!content.contains("gpgsign"), "GPG sign setting should not be present");
    assert!(!content.contains("[extra]"), "Extra section should not be present");
    assert!(!content.contains("customSetting"), "Custom setting should not be present");
    
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
            value: "CI Bot".to_string(),
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