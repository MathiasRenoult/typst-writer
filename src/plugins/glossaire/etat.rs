//! L'état de vue du glossaire : sa liste, son filtre, sa fiche.
//!
//! Les entrées appartiennent au projet (`glossaire.toml`) ; ici vivent ce
//! qu'une vue retient : le repli, le filtre, le survol et la fiche ouverte.

use crate::project::glossaire::EntreeGlossaire;

/// La fiche d'édition d'une entrée.
#[derive(Debug, Clone)]
pub struct EditionGlossaire {
    /// La clé — identifiant, appelée par `{glo:clé}`. Immuable en
    /// édition : elle est déjà citée dans les textes.
    pub cle: String,
    /// La forme courte : sigle, abréviation.
    pub court: String,
    /// La forme longue — développée à la première occurrence.
    pub long: String,
    /// La définition, après le terme dans la liste finale.
    pub description: String,
    /// Les formes plurielles.
    pub pluriel_court: String,
    pub pluriel_long: String,
    /// Le groupe : la rubrique de la liste finale.
    pub groupe: String,
    /// Une entrée neuve, pas encore écrite : la clé se choisit.
    pub creation: bool,
}

impl EditionGlossaire {
    /// Une fiche neuve : tous les champs vides, la clé à choisir.
    pub fn nouvelle() -> Self {
        Self {
            cle: String::new(),
            court: String::new(),
            long: String::new(),
            description: String::new(),
            pluriel_court: String::new(),
            pluriel_long: String::new(),
            groupe: String::new(),
            creation: true,
        }
    }

    /// La fiche d'une entrée existante ; sa clé est déjà citée, donc figée.
    pub fn depuis_entree(entree: &EntreeGlossaire) -> Self {
        Self {
            cle: entree.key.clone(),
            court: entree.short.clone().unwrap_or_default(),
            long: entree.long.clone().unwrap_or_default(),
            description: entree.description.clone().unwrap_or_default(),
            pluriel_court: entree.plural.clone().unwrap_or_default(),
            pluriel_long: entree.longplural.clone().unwrap_or_default(),
            groupe: entree.group.clone().unwrap_or_default(),
            creation: false,
        }
    }
}

/// Ce qu'une vue retient du glossaire.
#[derive(Debug, Clone, Default)]
pub struct GlossaireEtat {
    /// Les entrées du projet, dans l'ordre de l'auteur.
    pub entrees: Vec<EntreeGlossaire>,
    /// La recherche est-elle visible ? (Elle ne se montre que sur son icône.)
    pub recherche_visible: bool,
    /// La liste est-elle repliée ?
    pub replie: bool,
    /// Le filtre de la liste.
    pub filtre: String,
    /// L'entrée survolée — ses gestes ne paraissent qu'alors.
    pub survol: Option<String>,
    /// La fenêtre du glossaire est-elle ouverte ?
    pub fenetre: bool,
    /// La fiche d'édition, ouverte ou non.
    pub edition: Option<EditionGlossaire>,
}
