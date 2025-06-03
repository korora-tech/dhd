use crate::{Atom, Result};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub struct CreateFile {
    path: String,
    force: bool,
}

impl CreateFile {
    pub fn new(path: String, force: bool) -> Self {
        Self { path, force }
    }
}

impl Atom for CreateFile {
    fn check(&self) -> Result<bool> {
        let exists = Path::new(&self.path).exists();
        Ok(self.force || !exists)
    }

    fn execute(&self) -> Result<()> {
        fs::File::create(&self.path)?;
        Ok(())
    }

    fn describe(&self) -> String {
        format!(
            "Create file {}{}",
            self.path,
            if self.force { " (force)" } else { "" }
        )
    }
}

pub struct SetFileContent {
    path: String,
    content: String,
}

impl SetFileContent {
    pub fn new(path: String, content: String) -> Self {
        Self { path, content }
    }
}

impl Atom for SetFileContent {
    fn check(&self) -> Result<bool> {
        if !Path::new(&self.path).exists() {
            return Ok(true);
        }
        let current = fs::read_to_string(&self.path)?;
        Ok(current != self.content)
    }

    fn execute(&self) -> Result<()> {
        fs::write(&self.path, &self.content)?;
        Ok(())
    }

    fn describe(&self) -> String {
        format!("Set content of {}", self.path)
    }
}

pub struct SetFilePermissions {
    path: String,
    mode: u32,
}

impl SetFilePermissions {
    pub fn new(path: String, mode: u32) -> Self {
        Self { path, mode }
    }
}

impl Atom for SetFilePermissions {
    fn check(&self) -> Result<bool> {
        let metadata = fs::metadata(&self.path)?;
        let current_mode = metadata.permissions().mode();
        Ok((current_mode & 0o777) != self.mode)
    }

    fn execute(&self) -> Result<()> {
        let metadata = fs::metadata(&self.path)?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(self.mode);
        fs::set_permissions(&self.path, permissions)?;
        Ok(())
    }

    fn describe(&self) -> String {
        format!("Set permissions of {} to {:o}", self.path, self.mode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_file_check_when_not_exists() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir
            .path()
            .join("test.txt")
            .to_string_lossy()
            .to_string();

        let atom = CreateFile::new(file_path, false);
        assert!(atom.check().unwrap());
    }

    #[test]
    fn test_create_file_check_when_exists_no_force() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::File::create(&file_path).unwrap();

        let atom = CreateFile::new(file_path.to_string_lossy().to_string(), false);
        assert!(!atom.check().unwrap());
    }

    #[test]
    fn test_create_file_check_when_exists_with_force() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::File::create(&file_path).unwrap();

        let atom = CreateFile::new(file_path.to_string_lossy().to_string(), true);
        assert!(atom.check().unwrap());
    }

    #[test]
    fn test_create_file_execute() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        let atom = CreateFile::new(file_path.to_string_lossy().to_string(), false);
        atom.execute().unwrap();

        assert!(file_path.exists());
    }

    #[test]
    fn test_create_file_describe() {
        let atom = CreateFile::new("/test/path.txt".to_string(), false);
        assert_eq!(atom.describe(), "Create file /test/path.txt");

        let atom_force = CreateFile::new("/test/path.txt".to_string(), true);
        assert_eq!(atom_force.describe(), "Create file /test/path.txt (force)");
    }

    #[test]
    fn test_set_file_content_check_when_not_exists() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir
            .path()
            .join("test.txt")
            .to_string_lossy()
            .to_string();

        let atom = SetFileContent::new(file_path, "content".to_string());
        assert!(atom.check().unwrap());
    }

    #[test]
    fn test_set_file_content_check_when_content_differs() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "old content").unwrap();

        let atom = SetFileContent::new(
            file_path.to_string_lossy().to_string(),
            "new content".to_string(),
        );
        assert!(atom.check().unwrap());
    }

    #[test]
    fn test_set_file_content_check_when_content_same() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "same content").unwrap();

        let atom = SetFileContent::new(
            file_path.to_string_lossy().to_string(),
            "same content".to_string(),
        );
        assert!(!atom.check().unwrap());
    }

    #[test]
    fn test_set_file_content_execute() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        let content = "test content";
        let atom =
            SetFileContent::new(file_path.to_string_lossy().to_string(), content.to_string());
        atom.execute().unwrap();

        assert_eq!(fs::read_to_string(&file_path).unwrap(), content);
    }

    #[test]
    fn test_set_file_content_describe() {
        let atom = SetFileContent::new("/test/path.txt".to_string(), "content".to_string());
        assert_eq!(atom.describe(), "Set content of /test/path.txt");
    }

    #[test]
    fn test_set_file_permissions_check_when_different() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::File::create(&file_path).unwrap();

        let atom = SetFilePermissions::new(file_path.to_string_lossy().to_string(), 0o755);
        assert!(atom.check().unwrap());
    }

    #[test]
    fn test_set_file_permissions_execute() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::File::create(&file_path).unwrap();

        let atom = SetFilePermissions::new(file_path.to_string_lossy().to_string(), 0o755);
        atom.execute().unwrap();

        let metadata = fs::metadata(&file_path).unwrap();
        let mode = metadata.permissions().mode();
        assert_eq!(mode & 0o777, 0o755);
    }

    #[test]
    fn test_set_file_permissions_describe() {
        let atom = SetFilePermissions::new("/test/path.txt".to_string(), 0o755);
        assert_eq!(atom.describe(), "Set permissions of /test/path.txt to 755");
    }
}
