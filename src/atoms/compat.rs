/// Compatibility adapter to bridge old Atom trait to new Atom trait
use crate::atom::Atom as NewAtom;
use std::any::Any;

pub struct AtomCompat {
    inner: Box<dyn super::Atom>,
    module: String,
}

impl AtomCompat {
    pub fn new(inner: Box<dyn super::Atom>, module: String) -> Self {
        Self { inner, module }
    }
}

impl NewAtom for AtomCompat {
    fn check(&self) -> anyhow::Result<bool> {
        // Old atoms don't have check, always return true
        Ok(true)
    }

    fn execute(&self) -> anyhow::Result<()> {
        self.inner.execute().map_err(|e| anyhow::anyhow!("{}", e))
    }

    fn describe(&self) -> String {
        self.inner.describe()
    }

    fn module(&self) -> &str {
        &self.module
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn id(&self) -> String {
        format!("{}::{}", self.module, self.inner.name())
    }
}
