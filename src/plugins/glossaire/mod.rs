//! Le plugin glossaire : son état de vue.
//!
//! Le domaine (entrées, fichier, chargement) vit dans `project::glossaire` :
//! le langage le lit aussi, et un plugin ne doit pas se trouver sous lui.
//! Ici vivent ce qu'une vue retient et la fiche d'édition.

pub mod etat;

pub use crate::project::glossaire::{EntreeGlossaire, Glossaire};
pub use etat::{EditionGlossaire, GlossaireEtat};
