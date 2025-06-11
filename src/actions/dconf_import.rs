use dhd_macros::{typescript_type, typescript_fn};
use super::Action;
use std::path::{Path, PathBuf};
use crate::atoms::AtomCompat;

/// Import dconf settings from a file
#[typescript_type]
pub struct DconfImport {
    /// Path to the dconf file to import
    pub source: String,
    /// The dconf path to import to (e.g., "/org/gnome/desktop/")
    pub path: String,
}

impl Action for DconfImport {
    fn name(&self) -> &str {
        "DconfImport"
    }

    fn plan(&self, module_dir: &Path) -> Vec<Box<dyn crate::atom::Atom>> {
        let source_path = if self.source.starts_with('/') {
            PathBuf::from(&self.source)
        } else {
            module_dir.join(&self.source)
        };

        vec![Box::new(AtomCompat::new(
            Box::new(crate::atoms::dconf_import::DconfImportAtom::new(
                source_path,
                self.path.clone(),
            )),
            "dconf_import".to_string(),
        ))]
    }
}

#[typescript_fn]
pub fn dconf_import(config: DconfImport) -> crate::actions::ActionType {
    crate::actions::ActionType::DconfImport(config)
}