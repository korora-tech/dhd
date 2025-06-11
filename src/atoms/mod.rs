pub mod compat;
pub mod copy_file;
pub mod create_directory;
pub mod dconf_import;
pub mod git_config;
pub mod gnome_extension;
pub mod http_download;
pub mod install_packages;
pub mod install_packages_v2;
pub mod link_file;
pub mod package;
pub mod remove_packages;
pub mod run_command;
pub mod systemd_manage;
pub mod systemd_service;
pub mod systemd_socket;

pub use compat::AtomCompat;
pub use copy_file::CopyFile;
pub use create_directory::CreateDirectory;
pub use http_download::HttpDownload;
pub use install_packages::InstallPackages;
pub use link_file::LinkFile;
pub use run_command::RunCommand;

/// Legacy Atom trait for backwards compatibility
pub trait Atom: Send + Sync {
    fn name(&self) -> &str;
    fn execute(&self) -> Result<(), String>;
    fn describe(&self) -> String;
}
