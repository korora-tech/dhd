use std::any::Any;

/// Low-level atomic operation that can be executed
pub trait Atom: Send + Sync {
    /// Check if this atom needs to be executed (idempotency check)
    fn check(&self) -> anyhow::Result<bool>;

    /// Execute the atom
    fn execute(&self) -> anyhow::Result<()>;

    /// Get a human-readable description
    fn describe(&self) -> String;

    /// Get the module this atom belongs to
    fn module(&self) -> &str;

    /// Convert to Any for downcasting
    fn as_any(&self) -> &dyn Any;

    /// Get a unique identifier for this atom (for DAG)
    fn id(&self) -> String {
        format!("{}::{}", self.module(), self.describe())
    }

    /// Get dependencies for this atom (empty by default)
    fn dependencies(&self) -> Vec<String> {
        vec![]
    }
}
