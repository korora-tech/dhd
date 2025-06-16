use thiserror::Error;

#[derive(Error, Debug)]
pub enum DhdError {
    #[error("Module loading error: {0}")]
    ModuleLoad(String),

    #[error("Action planning error: {0}")]
    ActionPlan(String),

    #[error("Atom execution error: {0}")]
    AtomExecution(String),

    #[error("Dependency resolution error: {0}")]
    DependencyResolution(String),

    #[error("Platform detection error: {0}")]
    PlatformDetection(String),

    #[error("Package manager error: {0}")]
    PackageManager(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Execution engine error: {0}")]
    ExecutionEngine(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, DhdError>;
