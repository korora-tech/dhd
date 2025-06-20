use crate::atoms::Atom;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct LinkFile {
    pub source: PathBuf,
    pub target: PathBuf,
    pub force: bool,
}

impl Atom for LinkFile {
    fn name(&self) -> &str {
        "LinkFile"
    }

    fn execute(&self) -> Result<(), String> {
        #[cfg(unix)]
        {
            use std::fs;

            // Check if symlink already exists and points to the correct target
            if self.source.is_symlink() {
                match fs::read_link(&self.source) {
                    Ok(existing_link) => {
                        if existing_link == self.target {
                            // Silently skip if symlink already exists with correct target
                            return Ok(());
                        }
                    }
                    Err(_) => {
                        // Continue with creation/update
                    }
                }
            }

            // If force is enabled, create parent directories and handle existing files
            if self.force {
                // Create parent directories if they don't exist
                if let Some(parent) = self.source.parent() {
                    fs::create_dir_all(parent).map_err(|e| {
                        format!(
                            "Failed to create parent directories for {}: {}",
                            self.source.display(),
                            e
                        )
                    })?;
                }

                // Remove existing file/directory/symlink if it exists
                if self.source.exists() || self.source.is_symlink() {
                    if self.source.is_dir() && !self.source.is_symlink() {
                        // It's a real directory, not a symlink to a directory
                        fs::remove_dir_all(&self.source).map_err(|e| {
                            format!(
                                "Failed to remove existing directory {}: {}",
                                self.source.display(),
                                e
                            )
                        })?;
                    } else {
                        // It's a file or symlink
                        fs::remove_file(&self.source).map_err(|e| {
                            format!(
                                "Failed to remove existing file {}: {}",
                                self.source.display(),
                                e
                            )
                        })?;
                    }
                }
            }

            std::os::unix::fs::symlink(&self.target, &self.source).map_err(|e| {
                format!(
                    "Failed to create symlink at {} pointing to {}: {}",
                    self.source.display(),
                    self.target.display(),
                    e
                )
            })
        }

        #[cfg(not(unix))]
        {
            Err("Symlink creation is only supported on Unix systems".to_string())
        }
    }

    fn describe(&self) -> String {
        format!(
            "Create symlink at {} -> {}",
            self.source.display(),
            self.target.display()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_link_file_name() {
        let atom = LinkFile {
            source: PathBuf::from("/source"),
            target: PathBuf::from("/target"),
            force: false,
        };
        assert_eq!(atom.name(), "LinkFile");
    }

    #[test]
    fn test_link_file_clone() {
        let atom = LinkFile {
            source: PathBuf::from("/source"),
            target: PathBuf::from("/target"),
            force: false,
        };

        let cloned = atom.clone();
        assert_eq!(cloned.source, atom.source);
        assert_eq!(cloned.target, atom.target);
        assert_eq!(cloned.force, atom.force);
    }

    #[test]
    #[cfg(unix)]
    fn test_link_file_execute_success() {
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("link.txt");
        let target_path = temp_dir.path().join("target.txt");

        // Create target file
        fs::write(&target_path, "test content").unwrap();

        let atom = LinkFile {
            source: source_path.clone(),
            target: target_path.clone(),
            force: false,
        };

        let result = atom.execute();
        assert!(result.is_ok());

        // Verify symlink was created
        assert!(source_path.is_symlink());
        let link_target = fs::read_link(&source_path).unwrap();
        assert_eq!(link_target, target_path);
    }

    #[test]
    #[cfg(unix)]
    fn test_link_file_execute_source_exists() {
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("link.txt");
        let target_path = temp_dir.path().join("target.txt");

        // Create both files
        fs::write(&source_path, "existing content").unwrap();
        fs::write(&target_path, "target content").unwrap();

        let atom = LinkFile {
            source: source_path,
            target: target_path,
            force: false,
        };

        let result = atom.execute();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to create symlink"));
    }

    #[test]
    #[cfg(unix)]
    fn test_link_file_execute_target_not_exist() {
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("link.txt");
        let target_path = temp_dir.path().join("nonexistent.txt");

        let atom = LinkFile {
            source: source_path.clone(),
            target: target_path,
            force: false,
        };

        // Should create symlink even if target doesn't exist (dangling symlink)
        let result = atom.execute();
        assert!(result.is_ok());
        assert!(source_path.is_symlink());
    }

    #[test]
    #[cfg(not(unix))]
    fn test_link_file_execute_non_unix() {
        let atom = LinkFile {
            source: PathBuf::from("/source"),
            target: PathBuf::from("/target"),
            force: false,
        };

        let result = atom.execute();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Symlink creation is only supported on Unix systems"
        );
    }

    #[test]
    #[cfg(unix)]
    fn test_link_file_force_creates_parent_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("nested/dir/link.txt");
        let target_path = temp_dir.path().join("target.txt");

        // Create target file
        fs::write(&target_path, "test content").unwrap();

        let atom = LinkFile {
            source: source_path.clone(),
            target: target_path.clone(),
            force: true,
        };

        let result = atom.execute();
        assert!(result.is_ok());

        // Verify symlink was created
        assert!(source_path.is_symlink());
        let link_target = fs::read_link(&source_path).unwrap();
        assert_eq!(link_target, target_path);
    }

    #[test]
    #[cfg(unix)]
    fn test_link_file_force_overwrites_existing() {
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("link.txt");
        let target_path = temp_dir.path().join("target.txt");

        // Create both files
        fs::write(&source_path, "existing content").unwrap();
        fs::write(&target_path, "target content").unwrap();

        let atom = LinkFile {
            source: source_path.clone(),
            target: target_path.clone(),
            force: true,
        };

        let result = atom.execute();
        assert!(result.is_ok());

        // Verify symlink was created and overwrote existing file
        assert!(source_path.is_symlink());
        let link_target = fs::read_link(&source_path).unwrap();
        assert_eq!(link_target, target_path);
    }

    #[test]
    #[cfg(unix)]
    fn test_link_file_force_overwrites_existing_symlink() {
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("link.txt");
        let old_target_path = temp_dir.path().join("old_target.txt");
        let target_path = temp_dir.path().join("target.txt");

        // Create target and old target files
        fs::write(&target_path, "target content").unwrap();
        fs::write(&old_target_path, "old target content").unwrap();

        // Create existing symlink
        std::os::unix::fs::symlink(&old_target_path, &source_path).unwrap();

        let atom = LinkFile {
            source: source_path.clone(),
            target: target_path.clone(),
            force: true,
        };

        let result = atom.execute();
        assert!(result.is_ok());

        // Verify symlink was updated to new target
        assert!(source_path.is_symlink());
        let link_target = fs::read_link(&source_path).unwrap();
        assert_eq!(link_target, target_path);
    }
}
