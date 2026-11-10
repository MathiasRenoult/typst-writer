//! Le plugin biblique : son domaine et son état de vue.

pub mod domaine;
pub mod etat;

pub use domaine::*;
pub use etat::{BibleEtat, BiblePanneau, ReferenceCliquee, ReferenceEdition, TriReferences};
