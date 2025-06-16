use dhd_macros::typescript_enum;

pub mod compat;
pub mod condition;
pub mod conditional;
pub mod copy_file;
pub mod dconf_import;
pub mod directory;
pub mod execute_command;
pub mod git_config;
pub mod gnome_extensions;
pub mod http_download;
pub mod link_directory;
pub mod link_file;
pub mod package_install;
pub mod package_install_v2;
pub mod package_remove;
pub mod systemd_manage;
pub mod systemd_service;
pub mod systemd_socket;

pub use condition::{
    Condition, ComparisonOperator, all_of, any_of, and, or, command, command_exists, 
    command_succeeds, directory_exists, env_var, file_exists, not, property, secret_exists,
};
pub use conditional::{ConditionalAction, only_if, skip_if};
pub use copy_file::{CopyFile, copy_file};
pub use dconf_import::{DconfImport, dconf_import};
pub use directory::{Directory, directory};
pub use execute_command::ExecuteCommand;
pub use git_config::{GitConfig, git_config};
pub use gnome_extensions::{InstallGnomeExtensions, install_gnome_extensions};
pub use http_download::{HttpDownload, http_download};
pub use link_directory::{LinkDirectory, link_directory};
pub use link_file::{LinkFile, link_file};
pub use package_install::{PackageInstall, package_install};
pub use package_remove::{PackageRemove, package_remove};
pub use systemd_manage::{SystemdManage, systemd_manage};
pub use systemd_service::{SystemdService, systemd_service};
pub use systemd_socket::{SystemdSocket, systemd_socket};

#[typescript_enum]
pub enum ActionType {
    PackageInstall(PackageInstall),
    LinkFile(LinkFile),
    LinkDirectory(LinkDirectory),
    ExecuteCommand(ExecuteCommand),
    CopyFile(CopyFile),
    Directory(Directory),
    HttpDownload(HttpDownload),
    SystemdSocket(SystemdSocket),
    SystemdService(SystemdService),
    Conditional(ConditionalAction),
    DconfImport(DconfImport),
    InstallGnomeExtensions(InstallGnomeExtensions),
    PackageRemove(PackageRemove),
    SystemdManage(SystemdManage),
    GitConfig(GitConfig),
}

pub trait Action {
    fn name(&self) -> &str;
    fn plan(&self, module_dir: &std::path::Path) -> Vec<Box<dyn crate::atom::Atom>>;
}

impl Action for ActionType {
    fn name(&self) -> &str {
        match self {
            ActionType::PackageInstall(action) => action.name(),
            ActionType::LinkFile(action) => action.name(),
            ActionType::LinkDirectory(action) => action.name(),
            ActionType::ExecuteCommand(action) => action.name(),
            ActionType::CopyFile(action) => action.name(),
            ActionType::Directory(action) => action.name(),
            ActionType::HttpDownload(action) => action.name(),
            ActionType::SystemdSocket(action) => action.name(),
            ActionType::SystemdService(action) => action.name(),
            ActionType::Conditional(action) => action.name(),
            ActionType::DconfImport(action) => action.name(),
            ActionType::InstallGnomeExtensions(action) => action.name(),
            ActionType::PackageRemove(action) => action.name(),
            ActionType::SystemdManage(action) => action.name(),
            ActionType::GitConfig(action) => action.name(),
        }
    }

    fn plan(&self, module_dir: &std::path::Path) -> Vec<Box<dyn crate::atom::Atom>> {
        match self {
            ActionType::PackageInstall(action) => action.plan(module_dir),
            ActionType::LinkFile(action) => action.plan(module_dir),
            ActionType::LinkDirectory(action) => action.plan(module_dir),
            ActionType::ExecuteCommand(action) => action.plan(module_dir),
            ActionType::CopyFile(action) => action.plan(module_dir),
            ActionType::Directory(action) => action.plan(module_dir),
            ActionType::HttpDownload(action) => action.plan(module_dir),
            ActionType::SystemdSocket(action) => action.plan(module_dir),
            ActionType::SystemdService(action) => action.plan(module_dir),
            ActionType::Conditional(action) => action.plan(module_dir),
            ActionType::DconfImport(action) => action.plan(module_dir),
            ActionType::InstallGnomeExtensions(action) => action.plan(module_dir),
            ActionType::PackageRemove(action) => action.plan(module_dir),
            ActionType::SystemdManage(action) => action.plan(module_dir),
            ActionType::GitConfig(action) => action.plan(module_dir),
        }
    }
}
