/// Converts old atoms to new atoms using compatibility wrapper
use crate::atoms::{Atom as OldAtom, AtomCompat};
use crate::atom::Atom as NewAtom;

pub fn wrap_old_atoms(atoms: Vec<Box<dyn OldAtom>>, module: &str) -> Vec<Box<dyn NewAtom>> {
    atoms.into_iter()
        .map(|atom| {
            Box::new(AtomCompat::new(atom, module.to_string())) as Box<dyn NewAtom>
        })
        .collect()
}