use crate::utils::execute_with_privilege_escalation;
use crate::{Atom, DhdError, Result};
use std::fs;
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct HttpDownload {
    url: String,
    destination: PathBuf,
    checksum: Option<String>,
    checksum_type: ChecksumType,
    mode: Option<u32>,
    privileged: bool,
}

#[derive(Clone, Copy)]
pub enum ChecksumType {
    Sha256,
    Sha512,
    Md5,
}

impl ChecksumType {
    fn from_string(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "sha256" => Ok(ChecksumType::Sha256),
            "sha512" => Ok(ChecksumType::Sha512),
            "md5" => Ok(ChecksumType::Md5),
            _ => Err(DhdError::AtomExecution(format!(
                "Unsupported checksum type: {}",
                s
            ))),
        }
    }
}

impl HttpDownload {
    pub fn new(
        url: String,
        destination: String,
        checksum: Option<String>,
        checksum_type: Option<String>,
        mode: Option<u32>,
        privileged: bool,
    ) -> Result<Self> {
        let checksum_type = match checksum_type {
            Some(t) => ChecksumType::from_string(&t)?,
            None => ChecksumType::Sha256, // Default to SHA256
        };

        Ok(Self {
            url,
            destination: PathBuf::from(destination),
            checksum,
            checksum_type,
            mode,
            privileged,
        })
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

    fn download_to_temp(&self) -> Result<PathBuf> {
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join(format!("dhd_download_{}.tmp", std::process::id()));

        // Use curl for downloading (widely available on Unix systems)
        let status = Command::new("curl")
            .args([
                "-L", // Follow redirects
                "-f", // Fail on HTTP errors
                "-s", // Silent mode
                "-o",
                temp_file.to_str().unwrap(),
                &self.url,
            ])
            .status()?;

        if !status.success() {
            return Err(DhdError::AtomExecution(format!(
                "Failed to download from URL: {}",
                self.url
            )));
        }

        Ok(temp_file)
    }

    fn verify_checksum(&self, file_path: &Path) -> Result<bool> {
        if let Some(expected_checksum) = &self.checksum {
            let actual_checksum = self.calculate_checksum(file_path)?;
            Ok(actual_checksum.to_lowercase() == expected_checksum.to_lowercase())
        } else {
            Ok(true) // No checksum to verify
        }
    }

    fn calculate_checksum(&self, path: &Path) -> Result<String> {
        use sha2::{Digest, Sha256, Sha512};

        let mut file = fs::File::open(path)?;
        let mut buffer = [0; 8192];

        match self.checksum_type {
            ChecksumType::Sha256 => {
                let mut hasher = Sha256::new();
                loop {
                    let bytes_read = file.read(&mut buffer)?;
                    if bytes_read == 0 {
                        break;
                    }
                    hasher.update(&buffer[..bytes_read]);
                }
                Ok(format!("{:x}", hasher.finalize()))
            }
            ChecksumType::Sha512 => {
                let mut hasher = Sha512::new();
                loop {
                    let bytes_read = file.read(&mut buffer)?;
                    if bytes_read == 0 {
                        break;
                    }
                    hasher.update(&buffer[..bytes_read]);
                }
                Ok(format!("{:x}", hasher.finalize()))
            }
            ChecksumType::Md5 => {
                let mut hasher = md5::Context::new();
                loop {
                    let bytes_read = file.read(&mut buffer)?;
                    if bytes_read == 0 {
                        break;
                    }
                    hasher.consume(&buffer[..bytes_read]);
                }
                Ok(format!("{:x}", hasher.compute()))
            }
        }
    }

    fn install_with_sudo(&self, temp_file: &Path, destination: &Path) -> Result<()> {
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

        // Move the file
        let output = execute_with_privilege_escalation(
            "mv",
            &[temp_file.to_str().unwrap(), destination.to_str().unwrap()],
        )?;

        if !output.status.success() {
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

    fn needs_download(&self, destination: &Path) -> Result<bool> {
        if !destination.exists() {
            return Ok(true);
        }

        // If we have a checksum, verify the existing file
        if self.checksum.is_some() {
            return Ok(!self.verify_checksum(destination)?);
        }

        // Without a checksum, we can't determine if the file needs updating
        // So we'll re-download to be safe
        Ok(true)
    }
}

impl Atom for HttpDownload {
    fn check(&self) -> Result<bool> {
        let destination = Self::expand_tilde(&self.destination);
        self.needs_download(&destination)
    }

    fn execute(&self) -> Result<()> {
        let destination = Self::expand_tilde(&self.destination);

        // Download to temporary file
        let temp_file = self.download_to_temp()?;

        // Verify checksum if provided
        if !self.verify_checksum(&temp_file)? {
            // Clean up temp file
            let _ = fs::remove_file(&temp_file);
            return Err(DhdError::AtomExecution(format!(
                "Checksum verification failed for {}",
                self.url
            )));
        }

        // Install the file
        if self.privileged {
            self.install_with_sudo(&temp_file, &destination)?;
        } else {
            // Create parent directory if it doesn't exist
            if let Some(parent) = destination.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                    tracing::info!("Created directory: {}", parent.display());
                }
            }

            // Move the file
            fs::rename(&temp_file, &destination)?;

            // Set permissions if specified
            if let Some(mode) = self.mode {
                let metadata = fs::metadata(&destination)?;
                let mut permissions = metadata.permissions();
                permissions.set_mode(mode);
                fs::set_permissions(&destination, permissions)?;
            }
        }

        tracing::info!(
            "Downloaded file: {} -> {}{}",
            self.url,
            destination.display(),
            if self.privileged { " (privileged)" } else { "" }
        );

        Ok(())
    }

    fn describe(&self) -> String {
        let mut desc = format!("Download {} to {}", self.url, self.destination.display());

        if self.checksum.is_some() {
            desc.push_str(" (with checksum verification)");
        }

        if self.privileged {
            desc.push_str(" (privileged)");
        }

        if let Some(mode) = self.mode {
            desc.push_str(&format!(" with mode {:o}", mode));
        }

        desc
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_checksum_type_from_string() {
        assert!(matches!(
            ChecksumType::from_string("sha256").unwrap(),
            ChecksumType::Sha256
        ));
        assert!(matches!(
            ChecksumType::from_string("SHA512").unwrap(),
            ChecksumType::Sha512
        ));
        assert!(matches!(
            ChecksumType::from_string("md5").unwrap(),
            ChecksumType::Md5
        ));
        assert!(ChecksumType::from_string("invalid").is_err());
    }

    #[test]
    fn test_http_download_check_when_file_missing() {
        let temp_dir = TempDir::new().unwrap();
        let destination = temp_dir.path().join("downloaded.bin");

        let atom = HttpDownload::new(
            "https://example.com/file.bin".to_string(),
            destination.to_string_lossy().to_string(),
            None,
            None,
            None,
            false,
        )
        .unwrap();

        assert!(atom.check().unwrap());
    }

    #[test]
    fn test_http_download_describe() {
        let atom = HttpDownload::new(
            "https://example.com/file.bin".to_string(),
            "/usr/local/bin/tool".to_string(),
            Some("abc123".to_string()),
            Some("sha256".to_string()),
            Some(0o755),
            true,
        )
        .unwrap();

        assert_eq!(
            atom.describe(),
            "Download https://example.com/file.bin to /usr/local/bin/tool (with checksum verification) (privileged) with mode 755"
        );
    }

    #[test]
    fn test_http_download_describe_simple() {
        let atom = HttpDownload::new(
            "https://example.com/file.bin".to_string(),
            "~/Downloads/file.bin".to_string(),
            None,
            None,
            None,
            false,
        )
        .unwrap();

        assert_eq!(
            atom.describe(),
            "Download https://example.com/file.bin to ~/Downloads/file.bin"
        );
    }
}
