use std::path::PathBuf;
use std::process::Command;
use crate::atoms::Atom;

#[derive(Debug, Clone)]
pub struct DconfImportAtom {
    pub source: PathBuf,
    pub path: String,
}

impl DconfImportAtom {
    pub fn new(source: PathBuf, path: String) -> Self {
        Self { source, path }
    }
}

impl Atom for DconfImportAtom {
    fn name(&self) -> &str {
        "DconfImport"
    }

    fn execute(&self) -> Result<(), String> {
        // Check if dconf is installed
        let check = Command::new("which")
            .arg("dconf")
            .output()
            .map_err(|e| format!("Failed to check for dconf: {}", e))?;
        
        if !check.status.success() {
            return Err("dconf is not installed. Please install dconf first.".to_string());
        }

        // Check if source file exists
        if !self.source.exists() {
            return Err(format!("Dconf file does not exist: {}", self.source.display()));
        }

        // Import the dconf settings
        let output = Command::new("dconf")
            .arg("load")
            .arg(&self.path)
            .stdin(std::fs::File::open(&self.source)
                .map_err(|e| format!("Failed to open dconf file: {}", e))?)
            .output()
            .map_err(|e| format!("Failed to execute dconf load: {}", e))?;

        if !output.status.success() {
            return Err(format!("Failed to import dconf settings: {}",
                String::from_utf8_lossy(&output.stderr)));
        }

        Ok(())
    }

    fn describe(&self) -> String {
        format!("Import dconf from {} to {}", self.source.display(), self.path)
    }
}