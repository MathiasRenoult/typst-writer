//! Services communs : stockage, chemins et noms de fichiers.
//!
//! Ces fonctions vivaient dans `project` ; tout module qui écrit un fichier
//! en dépendait, y compris `theming`. Elles n'ont aucune dépendance de domaine.

pub mod stockage;
