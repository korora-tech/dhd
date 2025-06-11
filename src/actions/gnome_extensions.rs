use super::Action;
use crate::atoms::AtomCompat;
use dhd_macros::{typescript_fn, typescript_type};
use std::path::Path;

/// Install GNOME Shell extensions
#[typescript_type]
pub struct InstallGnomeExtensions {
    /// List of extension IDs to install
    pub extensions: Vec<String>,
}

impl Action for InstallGnomeExtensions {
    fn name(&self) -> &str {
        "InstallGnomeExtensions"
    }

    fn plan(&self, _module_dir: &Path) -> Vec<Box<dyn crate::atom::Atom>> {
        self.extensions
            .iter()
            .map(|ext| {
                Box::new(AtomCompat::new(
                    Box::new(crate::atoms::gnome_extension::InstallGnomeExtension::new(
                        ext.clone(),
                    )),
                    "install_gnome_extension".to_string(),
                )) as Box<dyn crate::atom::Atom>
            })
            .collect()
    }
}

#[typescript_fn]
pub fn install_gnome_extensions(config: InstallGnomeExtensions) -> crate::actions::ActionType {
    crate::actions::ActionType::InstallGnomeExtensions(config)
}
