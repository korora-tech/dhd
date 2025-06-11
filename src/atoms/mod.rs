pub mod link_file;
pub mod install_packages;
pub mod install_packages_v2;
pub mod run_command;
pub mod package;
pub mod copy_file;
pub mod create_directory;
pub mod http_download;
pub mod systemd_socket;
pub mod systemd_service;
pub mod compat;
pub mod dconf_import;
pub mod gnome_extension;
pub mod remove_packages;

pub use link_file::LinkFile;
pub use install_packages::InstallPackages;
pub use run_command::RunCommand;
pub use copy_file::CopyFile;
pub use create_directory::CreateDirectory;
pub use http_download::HttpDownload;
pub use compat::AtomCompat;

/// Legacy Atom trait for backwards compatibility
pub trait Atom: Send + Sync {
    fn name(&self) -> &str;
    fn execute(&self) -> Result<(), String>;
    fn describe(&self) -> String;
}