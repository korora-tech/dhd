pub mod action;
pub mod actions;
pub mod atom;
pub mod atoms;
pub mod dag_executor;
pub mod dependency_resolver;
pub mod discovery;
pub mod error;
pub mod execution;
pub mod loader;
pub mod module;
pub mod platform;
pub mod typescript;
pub mod utils;

// Re-export the main types users need
pub use action::{Action as ActionTrait, PlatformSelect};
pub use actions::{ActionType, PackageInstall, LinkFile, ExecuteCommand, Action};
pub use atom::Atom as AtomTrait;
pub use dag_executor::{DagExecutor, ExecutionSummary};
pub use dependency_resolver::{resolve_dependencies, DependencyError};
pub use discovery::{discover_modules, DiscoveredModule};
pub use error::{DhdError, Result};
pub use execution::ExecutionEngine;
pub use loader::{load_module, load_modules, LoadedModule, LoadError};
pub use module::{Module, ModuleDefinition};
pub use platform::{Platform, LinuxDistro, current_platform};