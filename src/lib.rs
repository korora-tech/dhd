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
pub mod system_info;
pub mod typescript;
pub mod utils;

// Re-export the main types users need
pub use action::{Action as ActionTrait, PlatformSelect};
pub use actions::{Action, ActionType, ExecuteCommand, LinkFile, PackageInstall};
pub use atom::Atom as AtomTrait;
pub use dag_executor::{DagExecutor, ExecutionSummary};
pub use dependency_resolver::{DependencyError, resolve_dependencies};
pub use discovery::{DiscoveredModule, discover_modules};
pub use error::{DhdError, Result};
pub use execution::ExecutionEngine;
pub use loader::{LoadError, LoadedModule, load_module, load_modules};
pub use module::{Module, ModuleDefinition};
pub use platform::{LinuxDistro, Platform, current_platform};
