use std::path::PathBuf;
use crate::atoms::Atom;

#[derive(Debug, Clone)]
pub struct SymlinkFile {
    pub source: PathBuf,
    pub target: PathBuf,
    pub force: bool,
}

impl Atom for SymlinkFile {
    fn name(&self) -> &str {
        "SymlinkFile"
    }

    fn execute(&self) -> Result<(), String> {
        #[cfg(unix)]
        {
            use std::fs;
            
            // Check if symlink already exists and points to the correct source
            if self.target.is_symlink() {
                match fs::read_link(&self.target) {
                    Ok(existing_link) => {
                        if existing_link == self.source {
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
                if let Some(parent) = self.target.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create parent directories for {}: {}", 
                            self.target.display(), e))?;
                }
                
                // Remove existing file/symlink if it exists
                if self.target.exists() || self.target.is_symlink() {
                    fs::remove_file(&self.target)
                        .map_err(|e| format!("Failed to remove existing file {}: {}", 
                            self.target.display(), e))?;
                }
            }
            
            std::os::unix::fs::symlink(&self.source, &self.target)
                .map_err(|e| format!("Failed to symlink {} to {}: {}", 
                    self.source.display(), self.target.display(), e))
        }
        
        #[cfg(not(unix))]
        {
            Err("Symlink creation is only supported on Unix systems".to_string())
        }
    }

    fn describe(&self) -> String {
        format!("Link {} -> {}", self.source.display(), self.target.display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_symlink_file_name() {
        let atom = SymlinkFile {
            source: PathBuf::from("/source"),
            target: PathBuf::from("/target"),
            force: false,
        };
        assert_eq!(atom.name(), "SymlinkFile");
    }

    #[test]
    fn test_symlink_file_clone() {
        let atom = SymlinkFile {
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
    fn test_symlink_file_execute_success() {
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("source.txt");
        let target_path = temp_dir.path().join("target.txt");
        
        // Create source file
        fs::write(&source_path, "test content").unwrap();
        
        let atom = SymlinkFile {
            source: source_path.clone(),
            target: target_path.clone(),
            force: false,
        };
        
        let result = atom.execute();
        assert!(result.is_ok());
        
        // Verify symlink was created
        assert!(target_path.is_symlink());
        let link_target = fs::read_link(&target_path).unwrap();
        assert_eq!(link_target, source_path);
    }

    #[test]
    #[cfg(unix)]
    fn test_symlink_file_execute_target_exists() {
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("source.txt");
        let target_path = temp_dir.path().join("target.txt");
        
        // Create both files
        fs::write(&source_path, "source content").unwrap();
        fs::write(&target_path, "target content").unwrap();
        
        let atom = SymlinkFile {
            source: source_path,
            target: target_path,
            force: false,
        };
        
        let result = atom.execute();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to symlink"));
    }

    #[test]
    #[cfg(unix)]
    fn test_symlink_file_execute_source_not_exist() {
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("nonexistent.txt");
        let target_path = temp_dir.path().join("target.txt");
        
        let atom = SymlinkFile {
            source: source_path,
            target: target_path.clone(),
            force: false,
        };
        
        // Should create symlink even if source doesn't exist (dangling symlink)
        let result = atom.execute();
        assert!(result.is_ok());
        assert!(target_path.is_symlink());
    }

    #[test]
    #[cfg(not(unix))]
    fn test_symlink_file_execute_non_unix() {
        let atom = SymlinkFile {
            source: PathBuf::from("/source"),
            target: PathBuf::from("/target"),
            force: false,
        };
        
        let result = atom.execute();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Symlink creation is only supported on Unix systems");
    }

    #[test]
    #[cfg(unix)]
    fn test_symlink_file_force_creates_parent_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("source.txt");
        let target_path = temp_dir.path().join("nested/dir/target.txt");
        
        // Create source file
        fs::write(&source_path, "test content").unwrap();
        
        let atom = SymlinkFile {
            source: source_path.clone(),
            target: target_path.clone(),
            force: true,
        };
        
        let result = atom.execute();
        assert!(result.is_ok());
        
        // Verify symlink was created
        assert!(target_path.is_symlink());
        let link_target = fs::read_link(&target_path).unwrap();
        assert_eq!(link_target, source_path);
    }

    #[test]
    #[cfg(unix)]
    fn test_symlink_file_force_overwrites_existing() {
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("source.txt");
        let target_path = temp_dir.path().join("target.txt");
        
        // Create both files
        fs::write(&source_path, "source content").unwrap();
        fs::write(&target_path, "existing content").unwrap();
        
        let atom = SymlinkFile {
            source: source_path.clone(),
            target: target_path.clone(),
            force: true,
        };
        
        let result = atom.execute();
        assert!(result.is_ok());
        
        // Verify symlink was created and overwrote existing file
        assert!(target_path.is_symlink());
        let link_target = fs::read_link(&target_path).unwrap();
        assert_eq!(link_target, source_path);
    }

    #[test]
    #[cfg(unix)]
    fn test_symlink_file_force_overwrites_existing_symlink() {
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("source.txt");
        let old_target_path = temp_dir.path().join("old_target.txt");
        let target_path = temp_dir.path().join("target.txt");
        
        // Create source and old target files
        fs::write(&source_path, "source content").unwrap();
        fs::write(&old_target_path, "old target content").unwrap();
        
        // Create existing symlink
        std::os::unix::fs::symlink(&old_target_path, &target_path).unwrap();
        
        let atom = SymlinkFile {
            source: source_path.clone(),
            target: target_path.clone(),
            force: true,
        };
        
        let result = atom.execute();
        assert!(result.is_ok());
        
        // Verify symlink was updated to new target
        assert!(target_path.is_symlink());
        let link_target = fs::read_link(&target_path).unwrap();
        assert_eq!(link_target, source_path);
    }
}