use dhd_macros::{typescript_type, typescript_fn};
use std::path::{Path, PathBuf};
use crate::atoms::AtomCompat;

#[typescript_type]
pub struct CopyFile {
    pub source: String,
    pub target: String,
    pub escalate: bool,
}

#[typescript_fn]
pub fn copy_file(config: CopyFile) -> crate::actions::ActionType {
    crate::actions::ActionType::CopyFile(config)
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

        let target_path = if self.target.starts_with("~/") {
            let home = std::env::var("HOME").unwrap_or_else(|_| String::from("/home/user"));
            PathBuf::from(self.target.replacen("~/", &format!("{}/", home), 1))
        } else {
            PathBuf::from(&self.target)
        };

        vec![Box::new(AtomCompat::new(
            Box::new(crate::atoms::copy_file::CopyFile::new(
                source_path,
                target_path,
                self.escalate,
            )),
            "copy_file".to_string(),
        ))]
    }
}