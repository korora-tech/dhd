use dhd_macros::typescript_enum;

pub mod package_install;
pub mod package_install_v2;
pub mod link_dotfile;
pub mod execute_command;
pub mod copy_file;
pub mod directory;
pub mod http_download;
pub mod systemd_socket;
pub mod systemd_service;
pub mod compat;

pub use package_install::PackageInstall;
pub use link_dotfile::{LinkDotfile, link_dotfile};
pub use execute_command::ExecuteCommand;
pub use copy_file::CopyFile;
pub use directory::{Directory, directory};
pub use http_download::{HttpDownload, http_download};
pub use systemd_socket::{SystemdSocket, systemd_socket};
pub use systemd_service::{SystemdService, systemd_service};

#[typescript_enum]
pub enum ActionType {
    PackageInstall(PackageInstall),
    LinkDotfile(LinkDotfile),
    ExecuteCommand(ExecuteCommand),
    CopyFile(CopyFile),
    Directory(Directory),
    HttpDownload(HttpDownload),
    SystemdSocket(SystemdSocket),
    SystemdService(SystemdService),
}

pub trait Action {
    fn name(&self) -> &str;
    fn plan(&self, module_dir: &std::path::Path) -> Vec<Box<dyn crate::atom::Atom>>;
}

impl Action for ActionType {
    fn name(&self) -> &str {
        match self {
            ActionType::PackageInstall(action) => action.name(),
            ActionType::LinkDotfile(action) => action.name(),
            ActionType::ExecuteCommand(action) => action.name(),
            ActionType::CopyFile(action) => action.name(),
            ActionType::Directory(action) => action.name(),
            ActionType::HttpDownload(action) => action.name(),
            ActionType::SystemdSocket(action) => action.name(),
            ActionType::SystemdService(action) => action.name(),
        }
    }

    fn plan(&self, module_dir: &std::path::Path) -> Vec<Box<dyn crate::atom::Atom>> {
        match self {
            ActionType::PackageInstall(action) => action.plan(module_dir),
            ActionType::LinkDotfile(action) => action.plan(module_dir),
            ActionType::ExecuteCommand(action) => action.plan(module_dir),
            ActionType::CopyFile(action) => action.plan(module_dir),
            ActionType::Directory(action) => action.plan(module_dir),
            ActionType::HttpDownload(action) => action.plan(module_dir),
            ActionType::SystemdSocket(action) => action.plan(module_dir),
            ActionType::SystemdService(action) => action.plan(module_dir),
        }
    }
}