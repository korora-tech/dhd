use crate::{Atom, DhdError, Result};
use chrono::Local;
use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::{Path, PathBuf};

pub struct LinkDotfile {
    source: PathBuf,
    target: Option<PathBuf>,
    backup: bool,
    force: bool,
}

impl LinkDotfile {
    pub fn new(source: String, target: Option<String>, backup: bool, force: bool) -> Self {
        Self {
            source: PathBuf::from(source),
            target: target.map(PathBuf::from),
            backup,
            force,
        }
    }

    fn expand_tilde(path: &Path) -> PathBuf {
        if let Some(path_str) = path.to_str() {
            if let Some(stripped) = path_str.strip_prefix("~/") {
                if let Some(home) = dirs::home_dir() {
                    return home.join(stripped);
                }
            }
        }
        path.to_path_buf()
    }

    fn get_target_path(&self) -> Result<PathBuf> {
        match &self.target {
            Some(target) => {
                let target_str = target
                    .to_str()
                    .ok_or_else(|| DhdError::AtomExecution("Invalid target path".to_string()))?;

                // If target is absolute or starts with ~, use it as-is (with tilde expansion)
                if target_str.starts_with('/') || target_str.starts_with('~') {
                    Ok(Self::expand_tilde(target))
                } else {
                    // Relative target - relative to XDG_CONFIG_HOME
                    let config_dir = dirs::config_dir().ok_or_else(|| {
                        DhdError::AtomExecution("Could not determine XDG_CONFIG_HOME".to_string())
                    })?;
                    Ok(config_dir.join(target))
                }
            }
            None => {
                // No target specified - use source filename in XDG_CONFIG_HOME
                let source_filename = self.source.file_name().ok_or_else(|| {
                    DhdError::AtomExecution("Source path has no filename".to_string())
                })?;

                let config_dir = dirs::config_dir().ok_or_else(|| {
                    DhdError::AtomExecution("Could not determine XDG_CONFIG_HOME".to_string())
                })?;

                Ok(config_dir.join(source_filename))
            }
        }
    }

    fn create_backup(&self, target: &Path) -> Result<()> {
        if target.exists() && self.backup {
            let timestamp = Local::now().format("%Y%m%d_%H%M%S");
            let backup_path = format!("{}.backup.{}", target.display(), timestamp);
            fs::rename(target, &backup_path)?;
            tracing::info!("Created backup: {}", backup_path);
        }
        Ok(())
    }
}

impl Atom for LinkDotfile {
    fn check(&self) -> Result<bool> {
        let source = Self::expand_tilde(&self.source);
        let target = self.get_target_path()?;

        // Check if source exists
        if !source.exists() {
            return Err(DhdError::AtomExecution(format!(
                "Source file does not exist: {}",
                source.display()
            )));
        }

        // If target doesn't exist, we need to create the link
        if !target.exists() {
            return Ok(true);
        }

        // If force is set, always execute
        if self.force {
            return Ok(true);
        }

        // Check if target is already a symlink to the correct source
        match fs::read_link(&target) {
            Ok(current) => Ok(current != source),
            Err(_) => {
                // Target exists but is not a symlink
                // We'll always remove and recreate it
                Ok(true)
            }
        }
    }

    fn execute(&self) -> Result<()> {
        let source = Self::expand_tilde(&self.source);
        let target = self.get_target_path()?;

        // Ensure source exists
        if !source.exists() {
            return Err(DhdError::AtomExecution(format!(
                "Source file does not exist: {}",
                source.display()
            )));
        }

        // Create parent directory if it doesn't exist
        if let Some(parent) = target.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
                tracing::info!("Created directory: {}", parent.display());
            }
        }

        // Handle existing target
        if target.exists() || target.symlink_metadata().is_ok() {
            tracing::debug!("Target exists at: {}", target.display());

            // Check if it's already the correct symlink
            if let Ok(current) = fs::read_link(&target) {
                if current == source {
                    tracing::info!("Symlink already correct: {}", target.display());
                    return Ok(());
                }
                tracing::debug!(
                    "Existing symlink points to: {}, need to update",
                    current.display()
                );
            }

            // Create backup if requested
            self.create_backup(&target)?;

            // Remove existing file/symlink/broken symlink
            // Using symlink_metadata to detect broken symlinks
            match target.symlink_metadata() {
                Ok(metadata) => {
                    if metadata.is_dir() && !metadata.is_symlink() {
                        tracing::debug!("Removing existing directory: {}", target.display());
                        fs::remove_dir_all(&target)?;
                    } else {
                        tracing::debug!("Removing existing file/symlink: {}", target.display());
                        fs::remove_file(&target)?;
                    }
                }
                Err(_) => {
                    tracing::debug!("Target doesn't exist after backup");
                    // If symlink_metadata fails, the file doesn't exist
                    // (or we don't have permissions, which will fail at symlink creation anyway)
                }
            }
        }

        // Create the symlink
        unix_fs::symlink(&source, &target).map_err(|e| {
            DhdError::AtomExecution(format!(
                "Failed to create symlink {} -> {}: {}",
                target.display(),
                source.display(),
                e
            ))
        })?;
        tracing::info!(
            "Created symlink: {} -> {}",
            target.display(),
            source.display()
        );

        Ok(())
    }

    fn describe(&self) -> String {
        let target_str = match &self.target {
            Some(t) => t.display().to_string(),
            None => format!(
                "$XDG_CONFIG_HOME/{}",
                self.source
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
            ),
        };

        let mut desc = format!(
            "Link dotfile from {} to {}",
            self.source.display(),
            target_str
        );
        if self.backup {
            desc.push_str(" (with backup)");
        }
        if self.force {
            desc.push_str(" (force)");
        }
        desc
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_link_dotfile_check_when_source_missing() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("missing.conf");
        let target = temp_dir.path().join(".config");

        let atom = LinkDotfile::new(
            source.to_string_lossy().to_string(),
            Some(target.to_string_lossy().to_string()),
            false,
            false,
        );

        let result = atom.check();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Source file does not exist")
        );
    }

    #[test]
    fn test_link_dotfile_check_when_target_missing() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source.conf");
        let target = temp_dir.path().join(".config");

        // Create source file
        File::create(&source).unwrap();

        let atom = LinkDotfile::new(
            source.to_string_lossy().to_string(),
            Some(target.to_string_lossy().to_string()),
            false,
            false,
        );

        assert!(atom.check().unwrap());
    }

    #[test]
    fn test_link_dotfile_check_when_correct_symlink_exists() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source.conf");
        let target = temp_dir.path().join(".config");

        // Create source and correct symlink
        File::create(&source).unwrap();
        unix_fs::symlink(&source, &target).unwrap();

        let atom = LinkDotfile::new(
            source.to_string_lossy().to_string(),
            Some(target.to_string_lossy().to_string()),
            false,
            false,
        );

        assert!(!atom.check().unwrap());
    }

    #[test]
    fn test_link_dotfile_check_with_force() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source.conf");
        let target = temp_dir.path().join(".config");

        // Create source and correct symlink
        File::create(&source).unwrap();
        unix_fs::symlink(&source, &target).unwrap();

        let atom = LinkDotfile::new(
            source.to_string_lossy().to_string(),
            Some(target.to_string_lossy().to_string()),
            false,
            true, // force
        );

        assert!(atom.check().unwrap());
    }

    #[test]
    fn test_link_dotfile_execute_basic() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source.conf");
        let target = temp_dir.path().join(".config");

        // Create source file with content
        let mut file = File::create(&source).unwrap();
        writeln!(file, "test config").unwrap();

        let atom = LinkDotfile::new(
            source.to_string_lossy().to_string(),
            Some(target.to_string_lossy().to_string()),
            false,
            false,
        );

        atom.execute().unwrap();

        assert!(target.exists());
        assert_eq!(fs::read_link(&target).unwrap(), source);
    }

    #[test]
    fn test_link_dotfile_execute_creates_parent_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source.conf");
        let target = temp_dir.path().join("nested/dir/.config");

        // Create source file
        File::create(&source).unwrap();

        let atom = LinkDotfile::new(
            source.to_string_lossy().to_string(),
            Some(target.to_string_lossy().to_string()),
            false,
            false,
        );

        atom.execute().unwrap();

        assert!(target.exists());
        assert!(target.parent().unwrap().exists());
    }

    #[test]
    fn test_link_dotfile_execute_with_backup() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source.conf");
        let target = temp_dir.path().join(".config");

        // Create source and existing target
        File::create(&source).unwrap();
        let mut existing = File::create(&target).unwrap();
        writeln!(existing, "old config").unwrap();
        drop(existing); // Close the file

        let atom = LinkDotfile::new(
            source.to_string_lossy().to_string(),
            Some(target.to_string_lossy().to_string()),
            true, // backup
            false,
        );

        atom.execute().unwrap();

        // Check symlink was created
        assert!(target.exists());
        assert_eq!(fs::read_link(&target).unwrap(), source);

        // Check backup was created
        let backups: Vec<_> = fs::read_dir(temp_dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains(".config.backup."))
            .collect();
        assert_eq!(backups.len(), 1);
    }

    #[test]
    fn test_link_dotfile_execute_replaces_existing_symlink() {
        let temp_dir = TempDir::new().unwrap();
        let source1 = temp_dir.path().join("source1.conf");
        let source2 = temp_dir.path().join("source2.conf");
        let target = temp_dir.path().join(".config");

        // Create sources and existing symlink
        File::create(&source1).unwrap();
        File::create(&source2).unwrap();
        unix_fs::symlink(&source1, &target).unwrap();

        let atom = LinkDotfile::new(
            source2.to_string_lossy().to_string(),
            Some(target.to_string_lossy().to_string()),
            false,
            true, // force
        );

        atom.execute().unwrap();

        assert_eq!(fs::read_link(&target).unwrap(), source2);
    }

    #[test]
    fn test_link_dotfile_describe() {
        let atom = LinkDotfile::new(
            "/home/user/dotfiles/vimrc".to_string(),
            Some("~/.vimrc".to_string()),
            true,
            false,
        );

        assert_eq!(
            atom.describe(),
            "Link dotfile from /home/user/dotfiles/vimrc to ~/.vimrc (with backup)"
        );
    }

    #[test]
    fn test_link_dotfile_describe_with_force() {
        let atom = LinkDotfile::new(
            "/home/user/dotfiles/vimrc".to_string(),
            Some("~/.vimrc".to_string()),
            false,
            true,
        );

        assert_eq!(
            atom.describe(),
            "Link dotfile from /home/user/dotfiles/vimrc to ~/.vimrc (force)"
        );
    }

    #[test]
    fn test_link_dotfile_with_no_target() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("zellij.kdl");

        // Create source file
        File::create(&source).unwrap();

        let atom = LinkDotfile::new(source.to_string_lossy().to_string(), None, false, false);

        // Should not error during check
        assert!(atom.check().is_ok());
    }

    #[test]
    fn test_link_dotfile_with_relative_target() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("zellij.kdl");

        // Create source file
        File::create(&source).unwrap();

        let atom = LinkDotfile::new(
            source.to_string_lossy().to_string(),
            Some("zellij/config.kdl".to_string()),
            false,
            false,
        );

        // Should not error during check
        assert!(atom.check().is_ok());
    }

    #[test]
    fn test_link_dotfile_describe_with_no_target() {
        let atom = LinkDotfile::new("zellij.kdl".to_string(), None, false, false);

        assert_eq!(
            atom.describe(),
            "Link dotfile from zellij.kdl to $XDG_CONFIG_HOME/zellij.kdl"
        );
    }
}
