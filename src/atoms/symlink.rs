use crate::{Atom, Result};
use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::Path;

pub struct CreateSymlink {
    source: String,
    target: String,
}

impl CreateSymlink {
    pub fn new(source: String, target: String) -> Self {
        Self { source, target }
    }
}

impl Atom for CreateSymlink {
    fn check(&self) -> Result<bool> {
        let target_path = Path::new(&self.target);
        if !target_path.exists() {
            return Ok(true);
        }

        match fs::read_link(target_path) {
            Ok(current) => Ok(current != Path::new(&self.source)),
            Err(_) => Ok(true), // Not a symlink or can't read
        }
    }

    fn execute(&self) -> Result<()> {
        let target_path = Path::new(&self.target);
        if target_path.exists() {
            fs::remove_file(target_path)?;
        }
        unix_fs::symlink(&self.source, &self.target)?;
        Ok(())
    }

    fn describe(&self) -> String {
        format!("Create symlink from {} to {}", self.source, self.target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_symlink_check_when_not_exists() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source.txt");
        let target = temp_dir.path().join("target.txt");
        
        // Create source file
        fs::File::create(&source).unwrap();
        
        let atom = CreateSymlink::new(
            source.to_string_lossy().to_string(),
            target.to_string_lossy().to_string()
        );
        assert!(atom.check().unwrap());
    }

    #[test]
    fn test_create_symlink_check_when_exists_same_target() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source.txt");
        let target = temp_dir.path().join("target.txt");
        
        // Create source file and symlink
        fs::File::create(&source).unwrap();
        unix_fs::symlink(&source, &target).unwrap();
        
        let atom = CreateSymlink::new(
            source.to_string_lossy().to_string(),
            target.to_string_lossy().to_string()
        );
        assert!(!atom.check().unwrap());
    }

    #[test]
    fn test_create_symlink_check_when_exists_different_target() {
        let temp_dir = TempDir::new().unwrap();
        let source1 = temp_dir.path().join("source1.txt");
        let source2 = temp_dir.path().join("source2.txt");
        let target = temp_dir.path().join("target.txt");
        
        // Create source files and symlink to source1
        fs::File::create(&source1).unwrap();
        fs::File::create(&source2).unwrap();
        unix_fs::symlink(&source1, &target).unwrap();
        
        let atom = CreateSymlink::new(
            source2.to_string_lossy().to_string(),
            target.to_string_lossy().to_string()
        );
        assert!(atom.check().unwrap());
    }

    #[test]
    fn test_create_symlink_check_when_regular_file_exists() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source.txt");
        let target = temp_dir.path().join("target.txt");
        
        // Create source file and regular file at target
        fs::File::create(&source).unwrap();
        fs::File::create(&target).unwrap();
        
        let atom = CreateSymlink::new(
            source.to_string_lossy().to_string(),
            target.to_string_lossy().to_string()
        );
        assert!(atom.check().unwrap());
    }

    #[test]
    fn test_create_symlink_execute() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source.txt");
        let target = temp_dir.path().join("target.txt");
        
        // Create source file
        fs::File::create(&source).unwrap();
        
        let atom = CreateSymlink::new(
            source.to_string_lossy().to_string(),
            target.to_string_lossy().to_string()
        );
        atom.execute().unwrap();
        
        assert!(target.exists());
        assert_eq!(fs::read_link(&target).unwrap(), source);
    }

    #[test]
    fn test_create_symlink_execute_replaces_existing() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source.txt");
        let target = temp_dir.path().join("target.txt");
        
        // Create source file and existing file at target
        fs::File::create(&source).unwrap();
        fs::File::create(&target).unwrap();
        
        let atom = CreateSymlink::new(
            source.to_string_lossy().to_string(),
            target.to_string_lossy().to_string()
        );
        atom.execute().unwrap();
        
        assert!(target.exists());
        assert_eq!(fs::read_link(&target).unwrap(), source);
    }

    #[test]
    fn test_create_symlink_describe() {
        let atom = CreateSymlink::new(
            "/path/to/source".to_string(),
            "/path/to/target".to_string()
        );
        assert_eq!(atom.describe(), "Create symlink from /path/to/source to /path/to/target");
    }
}
