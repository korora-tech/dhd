use crate::utils::execute_with_privilege_escalation;
use crate::{Atom, DhdError, Result};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct FileWrite {
    destination: PathBuf,
    content: String,
    mode: Option<u32>,
    privileged: bool,
    backup: bool,
}

impl FileWrite {
    pub fn new(
        destination: String,
        content: String,
        mode: Option<u32>,
        privileged: bool,
        backup: bool,
    ) -> Self {
        Self {
            destination: PathBuf::from(destination),
            content,
            mode,
            privileged,
            backup,
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

    fn create_backup(&self, destination: &Path) -> Result<()> {
        if destination.exists() && self.backup {
            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
            let backup_path = format!("{}.backup.{}", destination.display(), timestamp);

            if self.privileged {
                let output = execute_with_privilege_escalation(
                    "cp",
                    &["-a", destination.to_str().unwrap(), &backup_path],
                )?;

                if !output.status.success() {
                    return Err(DhdError::AtomExecution(
                        "Failed to create backup with privilege escalation".to_string(),
                    ));
                }
            } else {
                fs::copy(destination, &backup_path)?;
            }

            tracing::info!("Created backup: {}", backup_path);
        }
        Ok(())
    }

    fn content_differs(&self, destination: &Path) -> Result<bool> {
        if !destination.exists() {
            return Ok(true);
        }

        let current_content = if self.privileged {
            let output =
                execute_with_privilege_escalation("cat", &[destination.to_str().unwrap()])?;

            if !output.status.success() {
                return Ok(true); // Assume different if we can't read
            }

            String::from_utf8_lossy(&output.stdout).to_string()
        } else {
            fs::read_to_string(destination)?
        };

        Ok(current_content != self.content)
    }

    fn mode_differs(&self, destination: &Path) -> Result<bool> {
        if let Some(expected_mode) = self.mode {
            if !destination.exists() {
                return Ok(true);
            }

            let current_mode = if self.privileged {
                let output = Command::new("stat")
                    .args(["-c", "%a", destination.to_str().unwrap()])
                    .output()?;

                if !output.status.success() {
                    return Ok(true);
                }

                let mode_str = String::from_utf8_lossy(&output.stdout);
                u32::from_str_radix(mode_str.trim(), 8)
                    .map_err(|_| DhdError::AtomExecution("Failed to parse file mode".to_string()))?
            } else {
                let metadata = fs::metadata(destination)?;
                metadata.permissions().mode() & 0o777
            };

            Ok(current_mode != expected_mode)
        } else {
            Ok(false)
        }
    }

    fn write_with_sudo(&self, destination: &Path) -> Result<()> {
        // Create parent directory if needed
        if let Some(parent) = destination.parent() {
            if !parent.exists() {
                let output =
                    execute_with_privilege_escalation("mkdir", &["-p", parent.to_str().unwrap()])?;

                if !output.status.success() {
                    return Err(DhdError::AtomExecution(
                        "Failed to create parent directory with privilege escalation".to_string(),
                    ));
                }
            }
        }

        // Write content to a temp file first
        let temp_file = std::env::temp_dir().join(format!("dhd_write_{}.tmp", std::process::id()));
        fs::write(&temp_file, &self.content)?;

        // Move the file with privilege escalation
        let output = execute_with_privilege_escalation(
            "mv",
            &[temp_file.to_str().unwrap(), destination.to_str().unwrap()],
        )?;

        if !output.status.success() {
            // Clean up temp file if move failed
            let _ = fs::remove_file(&temp_file);
            return Err(DhdError::AtomExecution(
                "Failed to move file with privilege escalation".to_string(),
            ));
        }

        // Set permissions if specified
        if let Some(mode) = self.mode {
            let output = execute_with_privilege_escalation(
                "chmod",
                &[&format!("{:o}", mode), destination.to_str().unwrap()],
            )?;

            if !output.status.success() {
                return Err(DhdError::AtomExecution(
                    "Failed to set file permissions with privilege escalation".to_string(),
                ));
            }
        }

        Ok(())
    }
}

impl Atom for FileWrite {
    fn check(&self) -> Result<bool> {
        let destination = Self::expand_tilde(&self.destination);

        // Check if content differs
        if self.content_differs(&destination)? {
            return Ok(true);
        }

        // Check if mode differs
        if self.mode_differs(&destination)? {
            return Ok(true);
        }

        Ok(false)
    }

    fn execute(&self) -> Result<()> {
        let destination = Self::expand_tilde(&self.destination);

        // Create backup if requested
        self.create_backup(&destination)?;

        if self.privileged {
            self.write_with_sudo(&destination)?;
        } else {
            // Create parent directory if it doesn't exist
            if let Some(parent) = destination.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                    tracing::info!("Created directory: {}", parent.display());
                }
            }

            // Write the file
            fs::write(&destination, &self.content)?;

            // Set permissions if specified
            if let Some(mode) = self.mode {
                let metadata = fs::metadata(&destination)?;
                let mut permissions = metadata.permissions();
                permissions.set_mode(mode);
                fs::set_permissions(&destination, permissions)?;
            }
        }

        tracing::info!(
            "Wrote file: {}{}",
            destination.display(),
            if self.privileged { " (privileged)" } else { "" }
        );

        Ok(())
    }

    fn describe(&self) -> String {
        let mut desc = format!("Write file {}", self.destination.display());

        if self.privileged {
            desc.push_str(" (privileged)");
        }

        if let Some(mode) = self.mode {
            desc.push_str(&format!(" with mode {:o}", mode));
        }

        if self.backup {
            desc.push_str(" (with backup)");
        }

        desc
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_file_write_check_when_file_missing() {
        let temp_dir = TempDir::new().unwrap();
        let destination = temp_dir.path().join("new_file.txt");

        let atom = FileWrite::new(
            destination.to_string_lossy().to_string(),
            "test content".to_string(),
            None,
            false,
            false,
        );

        assert!(atom.check().unwrap());
    }

    #[test]
    fn test_file_write_check_when_content_same() {
        let temp_dir = TempDir::new().unwrap();
        let destination = temp_dir.path().join("existing.txt");

        let content = "same content";
        fs::write(&destination, content).unwrap();

        let atom = FileWrite::new(
            destination.to_string_lossy().to_string(),
            content.to_string(),
            None,
            false,
            false,
        );

        assert!(!atom.check().unwrap());
    }

    #[test]
    fn test_file_write_check_when_content_differs() {
        let temp_dir = TempDir::new().unwrap();
        let destination = temp_dir.path().join("existing.txt");

        fs::write(&destination, "old content").unwrap();

        let atom = FileWrite::new(
            destination.to_string_lossy().to_string(),
            "new content".to_string(),
            None,
            false,
            false,
        );

        assert!(atom.check().unwrap());
    }

    #[test]
    fn test_file_write_check_when_mode_differs() {
        let temp_dir = TempDir::new().unwrap();
        let destination = temp_dir.path().join("existing.txt");

        fs::write(&destination, "content").unwrap();

        // Set initial permissions
        let metadata = fs::metadata(&destination).unwrap();
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o644);
        fs::set_permissions(&destination, permissions).unwrap();

        let atom = FileWrite::new(
            destination.to_string_lossy().to_string(),
            "content".to_string(),
            Some(0o755),
            false,
            false,
        );

        assert!(atom.check().unwrap());
    }

    #[test]
    fn test_file_write_execute_basic() {
        let temp_dir = TempDir::new().unwrap();
        let destination = temp_dir.path().join("new_file.txt");

        let content = "test content\nwith multiple lines";
        let atom = FileWrite::new(
            destination.to_string_lossy().to_string(),
            content.to_string(),
            None,
            false,
            false,
        );

        atom.execute().unwrap();

        assert!(destination.exists());
        assert_eq!(fs::read_to_string(&destination).unwrap(), content);
    }

    #[test]
    fn test_file_write_execute_creates_parent_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let destination = temp_dir.path().join("nested/dir/file.txt");

        let atom = FileWrite::new(
            destination.to_string_lossy().to_string(),
            "content".to_string(),
            None,
            false,
            false,
        );

        atom.execute().unwrap();

        assert!(destination.exists());
        assert!(destination.parent().unwrap().exists());
    }

    #[test]
    fn test_file_write_execute_with_mode() {
        let temp_dir = TempDir::new().unwrap();
        let destination = temp_dir.path().join("script.sh");

        let atom = FileWrite::new(
            destination.to_string_lossy().to_string(),
            "#!/bin/bash\necho 'Hello'".to_string(),
            Some(0o755),
            false,
            false,
        );

        atom.execute().unwrap();

        let metadata = fs::metadata(&destination).unwrap();
        let mode = metadata.permissions().mode();
        assert_eq!(mode & 0o777, 0o755);
    }

    #[test]
    fn test_file_write_execute_with_backup() {
        let temp_dir = TempDir::new().unwrap();
        let destination = temp_dir.path().join("config.txt");

        // Create existing file
        fs::write(&destination, "old content").unwrap();

        let atom = FileWrite::new(
            destination.to_string_lossy().to_string(),
            "new content".to_string(),
            None,
            false,
            true,
        );

        atom.execute().unwrap();

        // Check file was written
        assert_eq!(fs::read_to_string(&destination).unwrap(), "new content");

        // Check backup was created
        let backups: Vec<_> = fs::read_dir(temp_dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .contains("config.txt.backup.")
            })
            .collect();
        assert_eq!(backups.len(), 1);
    }

    #[test]
    fn test_file_write_describe() {
        let atom = FileWrite::new(
            "/etc/config.conf".to_string(),
            "content".to_string(),
            Some(0o644),
            true,
            true,
        );

        assert_eq!(
            atom.describe(),
            "Write file /etc/config.conf (privileged) with mode 644 (with backup)"
        );
    }

    #[test]
    fn test_file_write_describe_simple() {
        let atom = FileWrite::new(
            "~/notes.txt".to_string(),
            "content".to_string(),
            None,
            false,
            false,
        );

        assert_eq!(atom.describe(), "Write file ~/notes.txt");
    }
}
