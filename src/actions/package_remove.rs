use dhd_macros::{typescript_type, typescript_fn};
use super::Action;
use std::path::Path;
use crate::atoms::AtomCompat;
use crate::atoms::package::PackageManager;

/// Remove packages from the system
#[typescript_type]
pub struct PackageRemove {
    /// List of package names to remove
    pub names: Vec<String>,
    /// Optional package manager to use
    pub manager: Option<PackageManager>,
}

impl Action for PackageRemove {
    fn name(&self) -> &str {
        "PackageRemove"
    }

    fn plan(&self, _module_dir: &Path) -> Vec<Box<dyn crate::atom::Atom>> {
        vec![Box::new(AtomCompat::new(
            Box::new(crate::atoms::remove_packages::RemovePackages::new(
                self.names.clone(),
                self.manager.clone(),
            )),
            "remove_packages".to_string(),
        ))]
    }
}

#[typescript_fn]
pub fn package_remove(config: PackageRemove) -> crate::actions::ActionType {
    crate::actions::ActionType::PackageRemove(config)
}