use crate::utils::execute_with_privilege_escalation;
use crate::{Atom, DhdError, Result};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct CopyFile {
    source: PathBuf,
    destination: PathBuf,
    privileged: bool,
    mode: Option<u32>,
    backup: bool,
}

impl CopyFile {
    pub fn new(
        source: String,
        destination: String,
        privileged: bool,
        mode: Option<u32>,
        backup: bool,
    ) -> Self {
        Self {
            source: PathBuf::from(source),
            destination: PathBuf::from(destination),
            privileged,
            mode,
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

    fn files_are_same(&self, source: &Path, destination: &Path) -> Result<bool> {
        if !destination.exists() {
            return Ok(false);
        }

        // Compare file sizes first
        let source_meta = fs::metadata(source)?;

        if self.privileged {
            // Use stat command for privileged files
            let output = Command::new("stat")
                .args(&["-c", "%s", destination.to_str().unwrap()])
                .output()?;

            if !output.status.success() {
                return Ok(false);
            }

            let size_str = String::from_utf8_lossy(&output.stdout);
            let size: u64 = size_str
                .trim()
                .parse()
                .map_err(|_| DhdError::AtomExecution("Failed to parse file size".to_string()))?;

            if size != source_meta.len() {
                return Ok(false);
            }
        } else {
            let dest_meta = fs::metadata(destination)?;
            if dest_meta.len() != source_meta.len() {
                return Ok(false);
            }
        }

        // Compare permissions if mode is specified
        if let Some(expected_mode) = self.mode {
            let current_mode = if self.privileged {
                let output = Command::new("stat")
                    .args(&["-c", "%a", destination.to_str().unwrap()])
                    .output()?;

                if !output.status.success() {
                    return Ok(false);
                }

                let mode_str = String::from_utf8_lossy(&output.stdout);
                u32::from_str_radix(mode_str.trim(), 8)
                    .map_err(|_| DhdError::AtomExecution("Failed to parse file mode".to_string()))?
            } else {
                let dest_meta = fs::metadata(destination)?;
                dest_meta.permissions().mode() & 0o777
            };

            if current_mode != expected_mode {
                return Ok(false);
            }
        }

        // Compare content using checksums
        let source_checksum = self.calculate_checksum(source)?;
        let dest_checksum = if self.privileged {
            let output =
                execute_with_privilege_escalation("sha256sum", &[destination.to_str().unwrap()])?;

            if !output.status.success() {
                return Ok(false);
            }

            String::from_utf8_lossy(&output.stdout)
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_string()
        } else {
            self.calculate_checksum(destination)?
        };

        Ok(source_checksum == dest_checksum)
    }

    fn calculate_checksum(&self, path: &Path) -> Result<String> {
        use sha2::{Digest, Sha256};
        use std::io::Read;

        let mut file = fs::File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0; 8192];

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    fn copy_with_sudo(&self, source: &Path, destination: &Path) -> Result<()> {
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

        // Copy the file
        let output = execute_with_privilege_escalation(
            "cp",
            &[
                "-a",
                source.to_str().unwrap(),
                destination.to_str().unwrap(),
            ],
        )?;

        if !output.status.success() {
            return Err(DhdError::AtomExecution(
                "Failed to copy file with privilege escalation".to_string(),
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

impl Atom for CopyFile {
    fn check(&self) -> Result<bool> {
        let source = Self::expand_tilde(&self.source);
        let destination = Self::expand_tilde(&self.destination);

        // Check if source exists
        if !source.exists() {
            return Err(DhdError::AtomExecution(format!(
                "Source file does not exist: {}",
                source.display()
            )));
        }

        // Check if files are already the same
        Ok(!self.files_are_same(&source, &destination)?)
    }

    fn execute(&self) -> Result<()> {
        let source = Self::expand_tilde(&self.source);
        let destination = Self::expand_tilde(&self.destination);

        // Ensure source exists
        if !source.exists() {
            return Err(DhdError::AtomExecution(format!(
                "Source file does not exist: {}",
                source.display()
            )));
        }

        // Create backup if requested
        self.create_backup(&destination)?;

        if self.privileged {
            self.copy_with_sudo(&source, &destination)?;
        } else {
            // Create parent directory if it doesn't exist
            if let Some(parent) = destination.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                    tracing::info!("Created directory: {}", parent.display());
                }
            }

            // Copy the file
            fs::copy(&source, &destination)?;

            // Set permissions if specified
            if let Some(mode) = self.mode {
                let metadata = fs::metadata(&destination)?;
                let mut permissions = metadata.permissions();
                permissions.set_mode(mode);
                fs::set_permissions(&destination, permissions)?;
            }
        }

        tracing::info!(
            "Copied file: {} -> {}{}",
            source.display(),
            destination.display(),
            if self.privileged { " (privileged)" } else { "" }
        );

        Ok(())
    }

    fn describe(&self) -> String {
        let mut desc = format!(
            "Copy file from {} to {}",
            self.source.display(),
            self.destination.display()
        );

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
    use std::fs::File;
    
    use tempfile::TempDir;

    #[test]
    fn test_copy_file_check_when_source_missing() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("missing.conf");
        let destination = temp_dir.path().join("dest.conf");

        let atom = CopyFile::new(
            source.to_string_lossy().to_string(),
            destination.to_string_lossy().to_string(),
            false,
            None,
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
    fn test_copy_file_check_when_destination_missing() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source.conf");
        let destination = temp_dir.path().join("dest.conf");

        // Create source file
        File::create(&source).unwrap();

        let atom = CopyFile::new(
            source.to_string_lossy().to_string(),
            destination.to_string_lossy().to_string(),
            false,
            None,
            false,
        );

        assert!(atom.check().unwrap());
    }

    #[test]
    fn test_copy_file_check_when_files_identical() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source.conf");
        let destination = temp_dir.path().join("dest.conf");

        // Create identical files
        let content = "test content";
        fs::write(&source, content).unwrap();
        fs::write(&destination, content).unwrap();

        let atom = CopyFile::new(
            source.to_string_lossy().to_string(),
            destination.to_string_lossy().to_string(),
            false,
            None,
            false,
        );

        assert!(!atom.check().unwrap());
    }

    #[test]
    fn test_copy_file_execute_basic() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source.conf");
        let destination = temp_dir.path().join("dest.conf");

        // Create source file with content
        let content = "test config";
        fs::write(&source, content).unwrap();

        let atom = CopyFile::new(
            source.to_string_lossy().to_string(),
            destination.to_string_lossy().to_string(),
            false,
            None,
            false,
        );

        atom.execute().unwrap();

        assert!(destination.exists());
        assert_eq!(fs::read_to_string(&destination).unwrap(), content);
    }

    #[test]
    fn test_copy_file_execute_creates_parent_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source.conf");
        let destination = temp_dir.path().join("nested/dir/dest.conf");

        // Create source file
        fs::write(&source, "content").unwrap();

        let atom = CopyFile::new(
            source.to_string_lossy().to_string(),
            destination.to_string_lossy().to_string(),
            false,
            None,
            false,
        );

        atom.execute().unwrap();

        assert!(destination.exists());
        assert!(destination.parent().unwrap().exists());
    }

    #[test]
    fn test_copy_file_execute_with_mode() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source.conf");
        let destination = temp_dir.path().join("dest.conf");

        // Create source file
        fs::write(&source, "content").unwrap();

        let atom = CopyFile::new(
            source.to_string_lossy().to_string(),
            destination.to_string_lossy().to_string(),
            false,
            Some(0o600),
            false,
        );

        atom.execute().unwrap();

        let metadata = fs::metadata(&destination).unwrap();
        let mode = metadata.permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    #[test]
    fn test_copy_file_execute_with_backup() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source.conf");
        let destination = temp_dir.path().join("dest.conf");

        // Create source and existing destination
        fs::write(&source, "new content").unwrap();
        fs::write(&destination, "old content").unwrap();

        let atom = CopyFile::new(
            source.to_string_lossy().to_string(),
            destination.to_string_lossy().to_string(),
            false,
            None,
            true,
        );

        atom.execute().unwrap();

        // Check file was copied
        assert_eq!(fs::read_to_string(&destination).unwrap(), "new content");

        // Check backup was created
        let backups: Vec<_> = fs::read_dir(temp_dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .contains("dest.conf.backup.")
            })
            .collect();
        assert_eq!(backups.len(), 1);
    }

    #[test]
    fn test_copy_file_describe() {
        let atom = CopyFile::new(
            "/etc/hosts".to_string(),
            "/etc/hosts.new".to_string(),
            true,
            Some(0o644),
            true,
        );

        assert_eq!(
            atom.describe(),
            "Copy file from /etc/hosts to /etc/hosts.new (privileged) with mode 644 (with backup)"
        );
    }
}
