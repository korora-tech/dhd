use dhd_macros::typescript_type;
use std::path::{Path, PathBuf};
use crate::atoms::AtomCompat;

#[typescript_type]
pub struct CopyFile {
    pub source: String,
    pub destination: String,
    pub requires_privilege_escalation: bool,
}

impl crate::actions::Action for CopyFile {
    fn name(&self) -> &str {
        "CopyFile"
    }

    fn plan(&self, module_dir: &Path) -> Vec<Box<dyn crate::atom::Atom>> {
        let source_path = if self.source.starts_with('/') {
            PathBuf::from(&self.source)
        } else {
            module_dir.join(&self.source)
        };

        let destination_path = if self.destination.starts_with("~/") {
            let home = std::env::var("HOME").unwrap_or_else(|_| String::from("/home/user"));
            PathBuf::from(self.destination.replacen("~/", &format!("{}/", home), 1))
        } else {
            PathBuf::from(&self.destination)
        };

        vec![Box::new(AtomCompat::new(
            Box::new(crate::atoms::copy_file::CopyFile::new(
                source_path,
                destination_path,
                self.requires_privilege_escalation,
            )),
            "copy_file".to_string(),
        ))]
    }
}