use std::path::PathBuf;
use std::fs;
use std::process::Command;
use crate::atoms::Atom;

#[derive(Debug, Clone)]
pub struct CreateDirectory {
    pub path: PathBuf,
    pub requires_privilege_escalation: bool,
}

impl CreateDirectory {
    pub fn new(path: PathBuf, requires_privilege_escalation: bool) -> Self {
        Self {
            path,
            requires_privilege_escalation,
        }
    }
}

impl Atom for CreateDirectory {
    fn name(&self) -> &str {
        "CreateDirectory"
    }

    fn execute(&self) -> Result<(), String> {
        if self.path.exists() && self.path.is_dir() {
            return Ok(());
        }

        if self.requires_privilege_escalation {
            let output = Command::new("sudo")
                .args(["mkdir", "-p", &self.path.to_string_lossy()])
                .output()
                .map_err(|e| format!("Failed to create directory: {}", e))?;
            
            if !output.status.success() {
                return Err(format!("Failed to create directory: {}", 
                    String::from_utf8_lossy(&output.stderr)));
            }
        } else {
            fs::create_dir_all(&self.path)
                .map_err(|e| format!("Failed to create directory {}: {}", 
                    self.path.display(), e))?;
        }
        
        Ok(())
    }

    fn describe(&self) -> String {
        format!("Create directory {}", self.path.display())
    }
}