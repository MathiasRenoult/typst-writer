//! Le routage du dispatch par domaine.
//!
//! `update_interne` garde le cycle de vie. Il applique d'abord la porte
//! d'activation des plugins, puis la garde d'écriture : un message classé
//! « modifie le projet » est refusé sans verrou. Les routeurs de domaine
//! reçoivent ensuite le message. Un message d'écriture qui n'est pas dans
//! cette liste doit donc assurer sa propre garde.

pub mod addons;
pub mod aide;
pub mod bible;
pub mod bibliographie;
pub mod confirmation;
pub mod couleurs;
pub mod edition;
pub mod entrees;
pub mod fenetre;
pub mod glossaire;
pub mod modes;
pub mod normes;
pub mod notes;
pub mod onglets;
pub mod projets;
pub mod suppressions;
pub mod tableaux;
pub mod themes_editeur;
pub mod ticks;
