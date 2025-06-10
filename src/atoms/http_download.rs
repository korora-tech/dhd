use std::path::PathBuf;
use std::fs;
use std::process::Command;
use crate::atoms::Atom;

#[derive(Debug, Clone)]
pub struct HttpDownload {
    pub url: String,
    pub destination: PathBuf,
    pub checksum: Option<String>,
    pub mode: Option<u32>,
}

impl HttpDownload {
    pub fn new(url: String, destination: PathBuf, checksum: Option<String>, mode: Option<u32>) -> Self {
        Self {
            url,
            destination,
            checksum,
            mode,
        }
    }
}

impl Atom for HttpDownload {
    fn name(&self) -> &str {
        "HttpDownload"
    }

    fn execute(&self) -> Result<(), String> {
        // Create parent directories if needed
        if let Some(parent) = self.destination.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create parent directory: {}", e))?;
            }
        }

        // Download the file using curl
        let output = Command::new("curl")
            .args(["-L", "-o", &self.destination.to_string_lossy(), &self.url])
            .output()
            .map_err(|e| format!("Failed to download file: {}", e))?;
        
        if !output.status.success() {
            return Err(format!("Failed to download file: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }

        // Verify checksum if provided
        if let Some(expected_checksum) = &self.checksum {
            let output = Command::new("sha256sum")
                .arg(&self.destination)
                .output()
                .map_err(|e| format!("Failed to calculate checksum: {}", e))?;
            
            if !output.status.success() {
                return Err(format!("Failed to calculate checksum: {}", 
                    String::from_utf8_lossy(&output.stderr)));
            }

            let actual_checksum = String::from_utf8_lossy(&output.stdout)
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_string();

            if &actual_checksum != expected_checksum {
                // Remove the downloaded file
                let _ = fs::remove_file(&self.destination);
                return Err(format!("Checksum mismatch: expected {}, got {}", 
                    expected_checksum, actual_checksum));
            }
        }

        // Set file mode if requested
        if let Some(mode) = self.mode {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&self.destination)
                    .map_err(|e| format!("Failed to read file permissions: {}", e))?
                    .permissions();
                perms.set_mode(mode);
                fs::set_permissions(&self.destination, perms)
                    .map_err(|e| format!("Failed to set file permissions: {}", e))?;
            }
        }
        
        Ok(())
    }

    fn describe(&self) -> String {
        format!("Download {} -> {}", self.url, self.destination.display())
    }
}