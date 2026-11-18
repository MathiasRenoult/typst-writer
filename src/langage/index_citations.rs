//! L'index des citations bibliques : ses entrées groupées et son rendu Typst.
//!
//! L'index ne relit pas le document structuré : il part du texte sérialisé,
//! la seule définition vivante du balisage `{{…}}`. Les pages ne sont pas
//! calculées ici — Typst les connaît au moment de la composition. Le rendu
//! d'un index paginé pose donc des repères `tw-citation` autour des citations
//! et laisse une `#context` les interroger.

use std::collections::BTreeMap;

use super::references::{
    Reference, afficher, afficher_nombres, forme_du_livre, mention_traduction, nom_du_livre_dans,
    rang_du_livre,
};
use crate::mise_en_page::modele::{Canon, IndexCitations, References};

/// Une entrée de l'index : une référence, et le nombre de ses occurrences.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntreeIndex {
    /// La référence telle qu'elle est citée.
    pub reference: Reference,
    /// Son libellé dans l'index, sans le nom du livre quand il est groupé.
    pub libelle: String,
    /// Le nombre de citations de cette référence dans le document.
    pub occurrences: usize,
}

/// Un livre de l'index, avec ses entrées.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupeIndex {
    /// Le nom du livre tel que les normes le demandent.
    pub nom: String,
    /// Les références du livre, dans l'ordre canonique interne.
    pub entrees: Vec<EntreeIndex>,
    /// Ce livre appartient-il au canon choisi pour l'index ?
    pub canonique: bool,
}

/// L'index préparé pour un document.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IndexBiblique {
    /// Les livres, ou les entrées seules quand le regroupement est désactivé.
    pub groupes: Vec<GroupeIndex>,
    /// Les pages sont-elles montrées ?
    pub afficher_pages: bool,
    /// Les références sont-elles groupées sous leur livre ?
    pub regrouper_par_livre: bool,
    /// Le nombre de colonnes du rendu (1 à 4).
    pub colonnes: usize,
    /// Le canon qui sépare les livres canoniques des non-canoniques.
    pub canon: Canon,
}

impl IndexBiblique {
    /// Construit l'index à partir du document et des normes.
    pub fn construire(
        document: &super::schema::Document,
        normes: &References,
        config: &IndexCitations,
    ) -> Self {
        let texte = super::serialiser::serialiser(document);
        let mut normes = normes.clone();
        if let Some(forme) = config.forme {
            normes.livre = forme;
        }

        let mut references: BTreeMap<String, (Reference, usize)> = BTreeMap::new();
        let mut reste = texte.as_str();
        while let Some(debut) = reste.find("{{") {
            let apres = &reste[debut + 2..];
            let Some(fin) = apres.find("}}") else {
                break;
            };
            if let Some((du_groupe, _)) = super::references::lire_groupe_de_balise(&apres[..fin]) {
                for reference in du_groupe {
                    // Deux traductions peuvent nommer un même livre
                    // différemment : la clé d'index doit donc porter la
                    // version citée, sinon les deux entrées fusionnent et
                    // l'une des formes de livre est perdue.
                    let cle = reference.cle_de_citation();
                    references
                        .entry(cle)
                        .and_modify(|(_, occurrences)| *occurrences += 1)
                        .or_insert((reference, 1));
                }
            }
            reste = &apres[fin + 2..];
        }

        let mut groupes: Vec<(usize, GroupeIndex)> = Vec::new();
        for (reference, occurrences) in references.into_values() {
            let rang = rang_du_livre(&reference.livre).unwrap_or(usize::MAX);
            let nom = nom_du_livre_dans(
                &reference.livre,
                forme_du_livre(normes.livre),
                reference.traduction,
            );
            let mut libelle = if config.regrouper_par_livre {
                afficher_nombres(&reference, &normes)
            } else {
                afficher(&reference, &normes)
            };
            if config.regrouper_par_livre {
                libelle.push_str(&mention_traduction(&reference));
            }
            let entree = EntreeIndex {
                reference: reference.clone(),
                libelle,
                occurrences,
            };

            if let Some((_, groupe)) = groupes
                .iter_mut()
                .find(|(rang_groupe, _)| *rang_groupe == rang)
            {
                groupe.entrees.push(entree);
            } else {
                groupes.push((
                    rang,
                    GroupeIndex {
                        nom,
                        entrees: vec![entree],
                        canonique: config.canon.contient(&reference.livre),
                    },
                ));
            }
        }

        groupes.sort_by_key(|(rang, groupe)| (!groupe.canonique, *rang));
        for (_, groupe) in &mut groupes {
            groupe.entrees.sort_by_key(|entree| {
                (
                    entree.reference.chapitre,
                    entree.reference.chapitre_fin,
                    entree.reference.verset_debut,
                    entree.reference.verset_fin,
                )
            });
        }

        Self {
            groupes: groupes.into_iter().map(|(_, groupe)| groupe).collect(),
            afficher_pages: config.afficher_pages,
            regrouper_par_livre: config.regrouper_par_livre,
            colonnes: config.colonnes.clamp(1, 4),
            canon: config.canon,
        }
    }

    /// L'index est-il vide ?
    pub fn est_vide(&self) -> bool {
        self.groupes.iter().all(|groupe| groupe.entrees.is_empty())
    }

    /// Rend l'index en Typst. Avec les pages, une `#context` interroge les
    /// repères posés auprès des citations ; sans elles, le texte est statique.
    pub fn rendre_typst(&self) -> String {
        let mut out = String::from("= Index des citations\n\n");

        if self.est_vide() {
            out.push_str("_Aucune référence biblique citée._\n\n");
            return out;
        }

        let mut corps = String::new();
        if self.afficher_pages {
            corps.push_str("#context {\n");
            corps.push_str("  let citations = query(<tw-citation>)\n");
            corps.push_str("  let notes = query(<tw-citation-note>)\n");
            corps.push_str("  let occurrences = key => citations.filter(x => x.value == key)\n");
            corps.push_str("  let occurrences_notes = key => notes.filter(x => x.value == key)\n");
            corps.push_str(
                "  let pages = key => (occurrences(key).map(x => (x.location().page(), false)) + occurrences_notes(key).map(x => (x.location().page(), true))).sorted(key: p => p.at(0)).dedup().map(p => str(p.at(0)) + if p.at(1) { \"n\" } else { \"\" }).join(\", \")\n",
            );
            corps.push_str(
                "  let present = key => occurrences(key).len() + occurrences_notes(key).len()\n",
            );

            let mut section = None;
            for groupe in &self.groupes {
                if self.canon != Canon::Tout && section != Some(groupe.canonique) {
                    let titre = if groupe.canonique {
                        "Livres canoniques"
                    } else {
                        "Livres non canoniques"
                    };
                    corps.push_str(&format!("  [== {titre}\n\n]\n"));
                    section = Some(groupe.canonique);
                }
                if self.regrouper_par_livre {
                    // Un titre de livre ne doit jamais rester seul en bas
                    // d'une colonne : Typst déplace le bloc collant avec la
                    // première référence qui le suit.
                    corps.push_str(&format!(
                        "  block(sticky: true)[{} :]\n",
                        echapper_contenu_typst(&groupe.nom)
                    ));
                }
                for entree in &groupe.entrees {
                    let cle = echapper_chaine_typst(&entree.reference.cle_de_citation());
                    let libelle = echapper_contenu_typst(&entree.libelle);
                    corps.push_str(&format!(
                        "  [- {libelle} #if present(\"{cle}\") > 0 [(p. #pages(\"{cle}\"))]]\n",
                    ));
                }
                if self.regrouper_par_livre {
                    corps.push('\n');
                }
            }
            corps.push_str("}\n\n");
        } else {
            let mut section = None;
            for groupe in &self.groupes {
                if self.canon != Canon::Tout && section != Some(groupe.canonique) {
                    let titre = if groupe.canonique {
                        "Livres canoniques"
                    } else {
                        "Livres non canoniques"
                    };
                    corps.push_str(&format!("== {titre}\n\n"));
                    section = Some(groupe.canonique);
                }
                if self.regrouper_par_livre {
                    // Même règle sans colonnes : le titre et la première
                    // référence voyagent ensemble au fil des pages.
                    corps.push_str(&format!(
                        "#block(sticky: true)[{} :]\n\n",
                        echapper_contenu_typst(&groupe.nom)
                    ));
                }
                for entree in &groupe.entrees {
                    corps.push_str("- ");
                    corps.push_str(&echapper_contenu_typst(&entree.libelle));
                    if entree.occurrences > 1 {
                        corps.push_str(&format!(" ({} citations)", entree.occurrences));
                    }
                    corps.push('\n');
                }
                if self.regrouper_par_livre {
                    corps.push('\n');
                }
            }
            corps.push('\n');
        }

        if self.colonnes > 1 {
            out.push_str(&format!("#columns({})[\n", self.colonnes));
            out.push_str(&corps);
            out.push_str("]\n\n");
        } else {
            out.push_str(&corps);
        }
        out
    }
}

/// Échappe une chaîne pour un littéral Typst entre guillemets.
fn echapper_chaine_typst(texte: &str) -> String {
    texte.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Échappe le texte d'une entrée pour qu'il reste du texte Typst.
fn echapper_contenu_typst(texte: &str) -> String {
    let mut sortie = String::new();
    for caractere in texte.chars() {
        if matches!(
            caractere,
            '\\' | '#' | '*' | '_' | '`' | '$' | '@' | '^' | '<' | '>' | '[' | ']' | '~' | '|'
        ) {
            sortie.push('\\');
        }
        sortie.push(caractere);
    }
    sortie
}

/// Les repères Typst posés auprès des citations, pour que l'index paginé
/// retrouve leur page. Un repère par référence, même dans une balise mixte.
pub fn reperes_typst(elements: &[super::references::ElementCitation], dans_note: bool) -> String {
    let label = if dans_note {
        "tw-citation-note"
    } else {
        "tw-citation"
    };
    let mut out = String::new();
    for element in elements {
        if let Some(reference) = element.reference() {
            out.push_str(&format!(
                "#metadata(\"{}\") <{label}>",
                echapper_chaine_typst(&reference.cle_de_citation())
            ));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mise_en_page::modele::SystemeAbreviation;

    fn document(source: &str) -> super::super::schema::Document {
        super::super::parse::parse(source)
    }

    #[test]
    fn les_references_sont_regroupees_par_livre_et_dedoublees() {
        let doc = document("{{Jn 3.16 ; Gn 1.1 ; Jn 3.16 ; Gn 2.1}}\n");
        let config = IndexCitations {
            regrouper_par_livre: true,
            forme: Some(SystemeAbreviation::FrancaiseLongue),
            ..Default::default()
        };
        let index = IndexBiblique::construire(&doc, &References::default(), &config);

        assert_eq!(index.groupes.len(), 2);
        assert_eq!(index.groupes[0].nom, "Genèse");
        assert_eq!(index.groupes[0].entrees.len(), 2);
        assert_eq!(index.groupes[0].entrees[0].libelle, "1.1");
        assert_eq!(index.groupes[0].entrees[0].occurrences, 1);
        assert_eq!(index.groupes[1].nom, "Jean");
        assert_eq!(index.groupes[1].entrees[0].occurrences, 2);
    }

    #[test]
    fn deux_traductions_du_meme_livre_ne_fusionnent_pas_dans_l_index() {
        let doc = document("{{Jn 3.16|lsg ; Jn 3.16|darby}}\n");
        let config = IndexCitations {
            regrouper_par_livre: true,
            ..Default::default()
        };
        let index = IndexBiblique::construire(&doc, &References::default(), &config);

        assert_eq!(
            index.groupes[0].entrees.len(),
            2,
            "la même référence dans deux versions reste deux entrées : {index:?}"
        );
    }

    #[test]
    fn l_index_suit_la_forme_demandee() {
        let doc = document("{{Jn 3.16}}\n");
        let config = IndexCitations {
            forme: Some(SystemeAbreviation::FrancaiseLongue),
            regrouper_par_livre: true,
            ..Default::default()
        };
        let index = IndexBiblique::construire(&doc, &References::default(), &config);

        assert_eq!(index.groupes[0].nom, "Jean");
        assert_eq!(index.groupes[0].entrees[0].libelle, "3.16");
        assert!(index.rendre_typst().contains("Jean :"));
    }

    #[test]
    fn les_livres_hors_canon_sont_separes_par_le_canon() {
        let doc = document("{{Jn 3.16 ; Tob 1.1}}\n");
        let config = IndexCitations {
            canon: crate::mise_en_page::modele::Canon::Protestant,
            regrouper_par_livre: true,
            forme: Some(crate::mise_en_page::modele::SystemeAbreviation::FrancaiseLongue),
            ..Default::default()
        };
        let index = IndexBiblique::construire(&doc, &References::default(), &config);
        assert_eq!(index.groupes.len(), 2);
        assert!(
            index
                .groupes
                .iter()
                .any(|groupe| groupe.canonique && groupe.nom == "Jean"),
            "{index:?}"
        );
        assert!(
            index
                .groupes
                .iter()
                .any(|groupe| !groupe.canonique && groupe.nom == "Tobie"),
            "{index:?}"
        );
        let typst = index.rendre_typst();
        assert!(typst.contains("== Livres canoniques"), "{typst}");
        assert!(typst.contains("== Livres non canoniques"), "{typst}");
        assert!(
            typst.contains("#block(sticky: true)[Jean :]"),
            "le titre du livre reste avec sa première référence : {typst}"
        );

        // « Tout le corpus » ne fabrique pas de section : une seule liste.
        let config = IndexCitations::default();
        let index = IndexBiblique::construire(&doc, &References::default(), &config);
        let typst = index.rendre_typst();
        assert!(!typst.contains("Livres canoniques"), "{typst}");
    }

    #[test]
    fn les_pages_se_demandent_a_typst() {
        let doc = document("{{Jn 3.16 ; Tob 1.1}}\n\n{{index}}\n");
        let config = IndexCitations {
            afficher_pages: true,
            regrouper_par_livre: true,
            canon: crate::mise_en_page::modele::Canon::Protestant,
            ..Default::default()
        };
        let index = IndexBiblique::construire(&doc, &References::default(), &config);
        let typst = index.rendre_typst();

        assert!(typst.contains("#context {"), "{typst}");
        assert!(typst.contains("query(<tw-citation>)"), "{typst}");
        assert!(typst.contains("location().page()"), "{typst}");
        assert!(typst.contains("[- 3.16"), "{typst}");
        assert!(typst.contains("[== Livres canoniques"), "{typst}");
        assert!(
            typst.contains("block(sticky: true)[Jn :]"),
            "le titre du livre reste avec sa première référence : {typst}"
        );
    }

    #[test]
    fn sans_pages_l_index_reste_statique() {
        let doc = document("{{Jn 3.16 ; Jn 3.16}}\n");
        let index =
            IndexBiblique::construire(&doc, &References::default(), &IndexCitations::default());
        let typst = index.rendre_typst();

        assert!(!typst.contains("#context"), "{typst}");
        assert!(typst.contains("- Jn 3.16 (2 citations)"), "{typst}");
    }
}
