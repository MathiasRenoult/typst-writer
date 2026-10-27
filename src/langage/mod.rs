//! Le langage intermédiaire : la couche entre l'éditeur enrichi et Typst.
//!
//! ```text
//! fichier texte (notre syntaxe)  ──parse──▶  Document structuré (blocs + inline)
//! Document structuré  ──génère──▶  Typst  ──compile──▶  rendu / PDF
//! ```
//!
//! L'éditeur enrichi travaille sur le Document structuré (l'AST), pas sur le
//! texte. Le texte n'est qu'une représentation sérialisée, lisible à la main
//! et diffable. Le Typst reste le maître du rendu final : tout ce que le
//! langage ne sait pas exprimer passe par un bloc brut.
//!
//! Voir `docs/design/langage-intermediaire.md` pour la conception complète.

pub mod blocs;
pub mod generer;
pub mod marques;
pub mod migration;
pub mod parse;
pub mod pipeline;
pub mod schema;
pub mod serialiser;
pub mod typographie;

pub use parse::parse;
pub use schema::Document;
pub use serialiser::serialiser;
