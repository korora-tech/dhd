use crate::platform::PlatformInfo;
use crate::{Atom, DhdError, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct DconfImport {
    source: PathBuf,
    path: String, // dconf path like "/org/gnome/desktop/interface/"
    backup: bool,
}

impl DconfImport {
    pub fn new(source: String, path: String, backup: bool) -> Self {
        Self {
            source: PathBuf::from(source),
            path,
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

    fn normalize_dconf_path(path: &str) -> String {
        let mut normalized = path.trim().to_string();

        // Ensure path starts with /
        if !normalized.starts_with('/') {
            normalized = format!("/{}", normalized);
        }

        // Ensure path ends with /
        if !normalized.ends_with('/') {
            normalized.push('/');
        }

        normalized
    }

    fn check_platform_support(&self) -> Result<()> {
        let platform = PlatformInfo::current();

        if !platform.has_dconf() {
            return Err(DhdError::AtomExecution(format!(
                "dconf is only supported on Linux systems with GNOME, not on {}",
                platform.description()
            )));
        }

        Ok(())
    }

    fn check_dconf_available(&self) -> Result<()> {
        let output = Command::new("which").arg("dconf").output()?;

        if !output.status.success() {
            return Err(DhdError::AtomExecution(
                "dconf command not found. Please install dconf-cli package.".to_string(),
            ));
        }

        Ok(())
    }

    fn create_backup(&self) -> Result<()> {
        if !self.backup {
            return Ok(());
        }

        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let backup_file = format!(
            "/tmp/dconf_backup_{}_{}.conf",
            self.path.replace('/', "_"),
            timestamp
        );

        let output = Command::new("dconf").args(["dump", &self.path]).output()?;

        if output.status.success() && !output.stdout.is_empty() {
            fs::write(&backup_file, &output.stdout)?;
            tracing::info!("Created dconf backup: {}", backup_file);
        }

        Ok(())
    }

    fn get_current_settings(&self) -> Result<String> {
        let output = Command::new("dconf").args(["dump", &self.path]).output()?;

        if !output.status.success() {
            return Ok(String::new()); // No existing settings
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

impl Atom for DconfImport {
    fn check(&self) -> Result<bool> {
        // Check platform support first
        self.check_platform_support()?;

        // Check if dconf is available
        self.check_dconf_available()?;

        let source = Self::expand_tilde(&self.source);

        // Check if source file exists
        if !source.exists() {
            return Err(DhdError::AtomExecution(format!(
                "Source dconf file does not exist: {}",
                source.display()
            )));
        }

        // Read the source file
        let source_content = fs::read_to_string(&source)?;

        // Get current settings
        let current_settings = self.get_current_settings()?;

        // If settings differ, we need to import
        Ok(source_content.trim() != current_settings.trim())
    }

    fn execute(&self) -> Result<()> {
        // Check platform support first
        self.check_platform_support()?;

        // Check if dconf is available
        self.check_dconf_available()?;

        let source = Self::expand_tilde(&self.source);
        let normalized_path = Self::normalize_dconf_path(&self.path);

        // Ensure source exists
        if !source.exists() {
            return Err(DhdError::AtomExecution(format!(
                "Source dconf file does not exist: {}",
                source.display()
            )));
        }

        // Create backup if requested
        self.create_backup()?;

        // Import the dconf settings
        let mut child = Command::new("dconf")
            .args(["load", &normalized_path])
            .stdin(std::process::Stdio::piped())
            .spawn()?;

        // Read the source file and pipe it to dconf
        let source_content = fs::read_to_string(&source)?;

        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            stdin.write_all(source_content.as_bytes())?;
        }

        let status = child.wait()?;

        if !status.success() {
            return Err(DhdError::AtomExecution(format!(
                "Failed to import dconf settings to {}",
                normalized_path
            )));
        }

        tracing::info!(
            "Imported dconf settings: {} -> {}",
            source.display(),
            normalized_path
        );

        Ok(())
    }

    fn describe(&self) -> String {
        let mut desc = format!(
            "Import dconf settings from {} to {}",
            self.source.display(),
            self.path
        );

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
    fn test_normalize_dconf_path() {
        assert_eq!(
            DconfImport::normalize_dconf_path("org/gnome/desktop/interface"),
            "/org/gnome/desktop/interface/"
        );
        assert_eq!(
            DconfImport::normalize_dconf_path("/org/gnome/desktop/interface"),
            "/org/gnome/desktop/interface/"
        );
        assert_eq!(
            DconfImport::normalize_dconf_path("/org/gnome/desktop/interface/"),
            "/org/gnome/desktop/interface/"
        );
        assert_eq!(
            DconfImport::normalize_dconf_path("  /org/gnome/desktop/interface/  "),
            "/org/gnome/desktop/interface/"
        );
    }

    #[test]
    fn test_dconf_import_check_when_source_missing() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("missing.conf");

        let atom = DconfImport::new(
            source.to_string_lossy().to_string(),
            "/org/gnome/test/".to_string(),
            false,
        );

        let result = atom.check();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Source dconf file does not exist")
        );
    }

    #[test]
    fn test_dconf_import_describe() {
        let atom = DconfImport::new(
            "/home/user/gnome-settings.conf".to_string(),
            "/org/gnome/desktop/".to_string(),
            true,
        );

        assert_eq!(
            atom.describe(),
            "Import dconf settings from /home/user/gnome-settings.conf to /org/gnome/desktop/ (with backup)"
        );
    }

    #[test]
    fn test_dconf_import_describe_no_backup() {
        let atom = DconfImport::new(
            "~/settings.conf".to_string(),
            "/org/gnome/shell/".to_string(),
            false,
        );

        assert_eq!(
            atom.describe(),
            "Import dconf settings from ~/settings.conf to /org/gnome/shell/"
        );
    }
}
