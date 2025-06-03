use thiserror::Error;

#[derive(Error, Debug)]
pub enum DhdError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Atom execution failed: {0}")]
    AtomExecution(String),

    #[error("Module parse error: {0}")]
    ModuleParse(String),

    #[error("DAG cycle detected")]
    DagCycle,
}

pub type Result<T> = std::result::Result<T, DhdError>;

pub trait Atom: Send + Sync {
    fn check(&self) -> Result<bool>;
    fn execute(&self) -> Result<()>;
    fn describe(&self) -> String;
}

pub type NodeId = usize;

pub struct ExecutionPlan {
    pub nodes: Vec<Box<dyn Atom>>,
    pub edges: Vec<(NodeId, NodeId)>,
}

pub mod atoms;
pub mod commands;
pub mod config;
pub mod dag;
pub mod gui;
pub mod modules;
pub mod tui;
