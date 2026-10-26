//! Le Document structuré : la source de vérité du langage intermédiaire.
//!
//! L'éditeur enrichi ne manipule pas du texte brut, mais une suite de blocs
//! dont le texte est une suite d'éléments inline. Ce document est la vérité ;
//! le texte sur disque n'est qu'une sérialisation (voir `parse` et `generer`).
//!
//! Deux invariants gouvernent ce schéma :
//!
//! 1. **Round-trip sans perte** : `parse(generer(doc)) == doc`. Le même texte
//!    donne toujours le même Document, et le même Document toujours le même
//!    texte. C'est ce qui rend le fichier lisible à la main et diffable.
//! 2. **Le Typst reste le maître du rendu final.** Tout ce que le langage ne
//!    sait pas exprimer passe par un bloc brut Typst, inséré tel quel dans le
//!    document généré.

/// Un document : une suite ordonnée de blocs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub blocs: Vec<Bloc>,
}

/// Un bloc de niveau document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Bloc {
    /// Un titre de niveau 1 à 6.
    Titre {
        niveau: u8,
        /// Le texte du titre, en éléments inline.
        contenu: Vec<Inline>,
    },
    /// Un paragraphe de texte.
    Paragraphe(Vec<Inline>),
    /// Une citation (bloc de citation).
    Citation(Vec<Inline>),
    /// Une liste à puces.
    ListePuce(Vec<Vec<Inline>>),
    /// Une liste numérotée.
    ListeNumero(Vec<Vec<Inline>>),
    /// Un bloc brut Typst, inséré tel quel dans le document généré.
    Brut(String),
}

/// Un élément inline, dans le texte d'un bloc.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Inline {
    /// Du texte nu.
    Texte(String),
    /// Du texte en gras.
    Gras(Vec<Inline>),
    /// Du texte en italique.
    Italique(Vec<Inline>),
    /// Du texte en gras et en italique.
    GrasItalique(Vec<Inline>),
    /// Du code inline.
    Code(String),
    /// Du texte coloré.
    Couleur {
        /// La couleur, au format Typst (`red`, `#ff0000`, `rgb(255, 0, 0)`…).
        couleur: String,
        contenu: Vec<Inline>,
    },
    /// Un appel de citation : `@clé`, `@clé[p. 45]`, ou plusieurs références
    /// jointes par `; ` — `@clé[p. 45]; @autre`.
    ///
    /// Un appel ne fait jamais qu'**une** note de bas de page : la mesure le
    /// dit, et c'est elle qui dicte la forme du générateur (voir
    /// [`crate::langage::generer::generer`]).
    Citation(Vec<Reference>),
    /// Une note de bas de page : `^[contenu]`.
    Note(Vec<Inline>),
    /// Un lien : `[texte](https://…)`.
    Lien { texte: Vec<Inline>, cible: String },
}

/// Une référence bibliographique dans un appel de citation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reference {
    /// La clé de l'entrée dans `references.bib`.
    pub cle: String,
    /// La mention de page, écrite telle qu'elle apparaîtra : `p. 45`,
    /// `pp. 12-15`, `chap. 3`…
    pub page: Option<String>,
}

impl Reference {
    /// Une référence sans mention de page.
    pub fn new(cle: impl Into<String>) -> Self {
        Self {
            cle: cle.into(),
            page: None,
        }
    }

    /// La même référence, avec sa mention de page.
    pub fn avec_page(mut self, page: impl Into<String>) -> Self {
        self.page = Some(page.into());
        self
    }
}

impl Document {
    /// Un document vide.
    pub fn vide() -> Self {
        Self { blocs: Vec::new() }
    }

    /// Le texte visible du document, sans marqueurs ni blocs vides.
    ///
    /// C'est la base du compteur de mots : titres, citations et listes comptent,
    /// tandis que le balisage et les caractères Typst d'un bloc brut ne comptent
    /// pas comme des mots visibles.
    pub fn texte_nu(&self) -> String {
        self.blocs
            .iter()
            .map(Bloc::texte_nu)
            .filter(|texte| !texte.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl Bloc {
    /// Le texte nu d'un bloc, pour le compte de mots et l'outline.
    ///
    /// Le balisage est retiré : seul le texte visible compte. C'est la même
    /// règle que le compteur de mots actuel, appliquée à l'AST plutôt qu'au
    /// source.
    pub fn texte_nu(&self) -> String {
        match self {
            Bloc::Titre { contenu, .. } | Bloc::Paragraphe(contenu) | Bloc::Citation(contenu) => {
                inline_texte_nu(contenu)
            }
            Bloc::ListePuce(items) | Bloc::ListeNumero(items) => items
                .iter()
                .map(|item| inline_texte_nu(item))
                .collect::<Vec<_>>()
                .join(" "),
            Bloc::Brut(_) => String::new(),
        }
    }
}

/// Le texte nu d'une suite d'éléments inline.
pub fn inline_texte_nu(inline: &[Inline]) -> String {
    let mut out = String::new();

    for element in inline {
        match element {
            Inline::Texte(texte) => out.push_str(texte),
            Inline::Gras(contenu)
            | Inline::Italique(contenu)
            | Inline::GrasItalique(contenu)
            | Inline::Note(contenu)
            | Inline::Couleur { contenu, .. } => out.push_str(&inline_texte_nu(contenu)),
            Inline::Code(texte) => out.push_str(texte),
            Inline::Citation(references) => {
                for (index, reference) in references.iter().enumerate() {
                    if index > 0 {
                        out.push_str("; ");
                    }

                    out.push_str(&reference.cle);

                    if let Some(page) = &reference.page {
                        out.push(' ');
                        out.push_str(page);
                    }
                }
            }
            Inline::Lien { texte, .. } => out.push_str(&inline_texte_nu(texte)),
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Le texte nu retire le balisage : c'est ce que compte le compteur de
    /// mots et ce que montre l'outline.
    #[test]
    fn texte_nu_retire_le_balisage() {
        let document = Document {
            blocs: vec![Bloc::Paragraphe(vec![
                Inline::Texte("Un ".to_string()),
                Inline::Gras(vec![Inline::Texte("mot".to_string())]),
                Inline::Texte(" et ".to_string()),
                Inline::Italique(vec![Inline::Texte("deux".to_string())]),
            ])],
        };

        assert_eq!(document.blocs[0].texte_nu(), "Un mot et deux");
    }

    /// Le document réunit le texte visible des blocs, sans leur balisage.
    #[test]
    fn document_texte_nu_reunit_les_blocs() {
        let document = Document {
            blocs: vec![
                Bloc::Titre {
                    niveau: 1,
                    contenu: vec![Inline::Texte("Titre".to_string())],
                },
                Bloc::Paragraphe(vec![
                    Inline::Texte("Un ".to_string()),
                    Inline::Gras(vec![Inline::Texte("texte".to_string())]),
                ]),
            ],
        };

        assert_eq!(document.texte_nu(), "Titre Un texte");
    }

    /// Une note compte par son contenu, pas par son marqueur.
    #[test]
    fn une_note_compte_comme_son_contenu() {
        let bloc = Bloc::Paragraphe(vec![
            Inline::Texte("Voir ".to_string()),
            Inline::Note(vec![Inline::Texte("une précision".to_string())]),
        ]);

        assert_eq!(bloc.texte_nu(), "Voir une précision");
    }

    /// Le compteur ne prend pas les commandes d'un bloc Typst brut pour de la prose.
    #[test]
    fn le_typst_brut_ne_gonfle_pas_le_compte_de_mots() {
        let document = Document {
            blocs: vec![
                Bloc::Paragraphe(vec![Inline::Texte("Texte visible".to_string())]),
                Bloc::Brut("#figure(image(\"figure.png\"))".to_string()),
            ],
        };

        assert_eq!(document.texte_nu(), "Texte visible");
    }

    /// Une citation bibliographique compte comme son texte : la clé est un mot.
    #[test]
    fn une_citation_compte_comme_son_texte() {
        let bloc = Bloc::Paragraphe(vec![
            Inline::Texte("Voir ".to_string()),
            Inline::Citation(vec![Reference::new("dupont2023")]),
        ]);

        assert_eq!(bloc.texte_nu(), "Voir dupont2023");
    }

    /// La mention de page se lit à côté de sa clé, et un appel groupé les
    /// rejoint par le point-virgule qui les sépare dans le texte.
    #[test]
    fn une_page_et_un_groupe_se_lisent_comme_ils_s_ecrivent() {
        let bloc = Bloc::Paragraphe(vec![Inline::Citation(vec![
            Reference::new("dupont2023").avec_page("p. 45"),
            Reference::new("martin2020"),
        ])]);

        assert_eq!(bloc.texte_nu(), "dupont2023 p. 45; martin2020");
    }
}
