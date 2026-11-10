//! L'état de vue de la bibliographie : sa liste, son filtre, sa notice.
//!
//! Les entrées appartiennent au projet (`.bib`) ; ici vivent ce qu'une vue
//! retient : le repli, la recherche, le survol et la fiche ouverte.

/// La notice en cours d'édition.
#[derive(Debug, Clone)]
pub struct EditionReference {
    /// La clé de citation. Immuable dans une notice — elle est citée — et
    /// libre dans une notice neuve.
    pub cle: String,
    /// Le type de document : `book`, `article`, `inproceedings`…
    pub kind: String,
    /// Le nom de l'auteur.
    pub nom: String,
    /// Son prénom.
    pub prenom: String,
    /// Le titre de l'œuvre.
    pub titre: String,
    /// Le titre court, celui des notes serrées.
    pub titre_court: String,
    /// L'année, si elle est là.
    pub annee: String,
    /// La maison d'édition.
    pub editeur: String,
    /// Le lieu de parution.
    pub lieu: String,
    /// Une notice neuve, pas encore écrite : la clé se choisit.
    pub creation: bool,
    /// La bibliographie est-elle locale ? Seule une bibliothèque locale
    /// s'écrit ; une notice Zotero se lit, jamais.
    pub locale: bool,
}

impl EditionReference {
    /// La fiche d'une notice existante, pour l'éditer sans la citer à neuf.
    ///
    /// `locale` dit si l'écriture est permise : une notice Zotero se lit,
    /// jamais ne s'écrit.
    pub fn depuis_entree(entree: &crate::project::BibliographyEntry, locale: bool) -> Self {
        let (nom, prenom) = entree
            .auteurs
            .first()
            .cloned()
            .unwrap_or((String::new(), String::new()));
        Self {
            cle: entree.key.clone(),
            kind: entree.kind.clone(),
            nom,
            prenom,
            titre: entree.titre.clone(),
            titre_court: entree.titre_court.clone().unwrap_or_default(),
            annee: entree.annee.clone().unwrap_or_default(),
            editeur: entree.editeur.clone().unwrap_or_default(),
            lieu: entree.lieu.clone().unwrap_or_default(),
            creation: false,
            locale,
        }
    }

    /// Une fiche neuve : clé libre, type `book`, champs vides.
    pub fn nouvelle(locale: bool) -> Self {
        Self {
            cle: String::new(),
            kind: "book".to_string(),
            nom: String::new(),
            prenom: String::new(),
            titre: String::new(),
            titre_court: String::new(),
            annee: String::new(),
            editeur: String::new(),
            lieu: String::new(),
            creation: true,
            locale,
        }
    }
}

/// Ce qu'une vue retient de la bibliographie.
#[derive(Debug, Clone, Default)]
pub struct BibliographieEtat {
    /// La liste des références est-elle repliée ?
    pub repliees: bool,
    /// Le filtre de la liste.
    pub filtre: String,
    /// La référence survolée — ses gestes ne paraissent qu'alors.
    pub survol: Option<String>,
    /// La recherche ne se montre que sur son icône.
    pub recherche_visible: bool,
    /// La notice ouverte, s'il y en a une.
    pub edition: Option<EditionReference>,
}
