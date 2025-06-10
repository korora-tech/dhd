use std::path::PathBuf;
use std::fs;
use std::process::Command;
use crate::atoms::Atom;

#[derive(Debug, Clone)]
pub struct CopyFile {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub requires_privilege_escalation: bool,
}

impl CopyFile {
    pub fn new(source: PathBuf, destination: PathBuf, requires_privilege_escalation: bool) -> Self {
        Self {
            source,
            destination,
            requires_privilege_escalation,
        }
    }
}

impl Atom for CopyFile {
    fn name(&self) -> &str {
        "CopyFile"
    }

    fn execute(&self) -> Result<(), String> {
        // Check if source exists
        if !self.source.exists() {
            return Err(format!("Source file {} does not exist", self.source.display()));
        }

        // Create parent directories if needed
        if let Some(parent) = self.destination.parent() {
            if !parent.exists() {
                if self.requires_privilege_escalation {
                    let output = Command::new("sudo")
                        .args(["mkdir", "-p", &parent.to_string_lossy()])
                        .output()
                        .map_err(|e| format!("Failed to create parent directory: {}", e))?;
                    
                    if !output.status.success() {
                        return Err(format!("Failed to create parent directory: {}", 
                            String::from_utf8_lossy(&output.stderr)));
                    }
                } else {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create parent directory: {}", e))?;
                }
            }
        }

        // Copy the file
        if self.requires_privilege_escalation {
            let output = Command::new("sudo")
                .args(["cp", &self.source.to_string_lossy(), &self.destination.to_string_lossy()])
                .output()
                .map_err(|e| format!("Failed to copy file: {}", e))?;
            
            if !output.status.success() {
                return Err(format!("Failed to copy file: {}", 
                    String::from_utf8_lossy(&output.stderr)));
            }
        } else {
            fs::copy(&self.source, &self.destination)
                .map_err(|e| format!("Failed to copy {} to {}: {}", 
                    self.source.display(), self.destination.display(), e))?;
        }
        
        Ok(())
    }

    fn describe(&self) -> String {
        format!("Copy {} -> {}", self.source.display(), self.destination.display())
    }
}