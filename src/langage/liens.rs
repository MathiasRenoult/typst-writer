//! Les éléments notables d'un document, et leurs ancres.
//!
//! Un titre, une figure ou une image reçoit une référence stable : c'est elle
//! que l'on écrit dans un lien interne `[[référence|texte]]`. La même
//! traversée sert à la recherche de la palette et à la génération Typst, pour
//! que la suggestion et l'ancre ne divergent jamais.

use std::collections::{HashMap, HashSet};

use super::schema::{Bloc, Document, Inline};

/// La nature d'un élément linkable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NatureLien {
    /// Un titre, avec son niveau de 1 à 6.
    Titre(u8),
    /// Une figure ou une image.
    Figure,
}

/// Ce qu'un lien interne affiche quand son texte est un mot-clé entre
/// guillemets : `[[référence|"chapitre"]]`, `[[référence|"figure"]]` ou
/// `[[référence|"page"]]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AffichageLien {
    /// Le numéro composé de la tête visée — « chapitre 2.3 ».
    Chapitre,
    /// Le numéro composé de la figure visée — « Figure 4 ».
    Figure,
    /// La page composée où se trouve la cible — « page 7 ».
    Page,
}

/// Reconnaît le mot-clé d'affichage automatique d'un lien interne.
///
/// Le mot est volontairement **entre guillemets** : un lien ordinaire qui
/// affiche le mot « page » reste possible, et aucun texte existant ne change
/// de sens sans l'avoir demandé.
pub fn affichage_du_lien(texte: &[Inline]) -> Option<AffichageLien> {
    let [Inline::Texte(texte)] = texte else {
        return None;
    };

    let texte = texte.trim();
    let interieur = texte
        .strip_prefix('"')
        .and_then(|reste| reste.strip_suffix('"'))
        .or_else(|| {
            texte
                .strip_prefix('«')
                .and_then(|reste| reste.strip_suffix('»'))
        })?;

    match interieur.trim().to_lowercase().as_str() {
        "chapitre" | "section" | "titre" => Some(AffichageLien::Chapitre),
        "figure" | "image" => Some(AffichageLien::Figure),
        "page" => Some(AffichageLien::Page),
        _ => None,
    }
}

/// Un élément que l'on peut cibler dans le document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElementLien {
    /// La référence à écrire dans `[[…]]`.
    pub reference: String,
    /// Le texte lisible dans la liste de suggestions.
    pub libelle: String,
    /// Une description courte : « titre de niveau 2 », « figure »…
    pub description: String,
    /// La nature de l'élément.
    pub nature: NatureLien,
}

/// Le compteur qui rend les ancres uniques.
///
/// Deux titres identiques reçoivent `titre-introduction` puis
/// `titre-introduction-2` ; les figures ont leur propre famille d'ancres.
#[derive(Debug, Default)]
pub struct CompteurLiens {
    vus: HashMap<String, usize>,
}

impl CompteurLiens {
    /// L'ancre d'un titre.
    pub fn titre(&mut self, texte: &str) -> String {
        self.prochain("titre", &slug(texte))
    }

    /// L'ancre d'une figure ou d'une image.
    pub fn figure(&mut self, texte: &str) -> String {
        self.prochain("figure", &slug(texte))
    }

    fn prochain(&mut self, prefixe: &str, base: &str) -> String {
        let base = if base.is_empty() { "sans-titre" } else { base };
        let cle = format!("{prefixe}-{base}");
        let compte = self.vus.entry(cle.clone()).or_insert(0);
        *compte += 1;

        if *compte == 1 {
            cle
        } else {
            format!("{cle}-{compte}")
        }
    }
}

/// Le texte qui donne son ancre à une figure : sa légende, ou le nom du
/// fichier quand elle n'en a pas.
pub fn texte_figure(légende: &[super::schema::Inline], cible: &str) -> String {
    if légende.is_empty() {
        std::path::Path::new(cible)
            .file_stem()
            .map(|nom| nom.to_string_lossy().into_owned())
            .unwrap_or_else(|| cible.to_string())
    } else {
        super::schema::inline_texte_nu(légende)
    }
}

/// Les éléments linkables, dans l'ordre du document.
pub fn collecter(document: &Document) -> Vec<ElementLien> {
    let mut compteur = CompteurLiens::default();
    let mut elements = Vec::new();

    for bloc in &document.blocs {
        parcourir(bloc, &mut compteur, &mut elements);
    }

    elements
}

/// Les cibles d'un document, avec leurs formes courtes quand elles sont
/// uniques : `titre-adoption-coutume` peut aussi s'écrire `adoption-coutume`.
///
/// Deux éléments qui partageraient la même forme courte gardent leur ancre
/// complète : une abréviation ambiguë ne doit jamais choisir à la place de
/// l'auteur.
fn cibles_resolues(document: &Document) -> HashMap<String, String> {
    let mut cibles = HashMap::new();
    let mut courtes: HashMap<String, Option<String>> = HashMap::new();

    for element in collecter(document) {
        let complete = element.reference;
        cibles.insert(complete.clone(), complete.clone());

        if let Some(courte) = complete
            .strip_prefix("titre-")
            .or_else(|| complete.strip_prefix("figure-"))
        {
            courtes
                .entry(courte.to_string())
                .and_modify(|entree| *entree = None)
                .or_insert_with(|| Some(complete.clone()));
        }
    }

    for (courte, complete) in courtes {
        if let Some(complete) = complete {
            // Une ancre complète existante gagne toujours : `titre-x` ne doit
            // pas être détourné par la forme courte de `figure-titre-x`.
            cibles.entry(courte).or_insert(complete);
        }
    }

    cibles
}

/// Réécrit les liens courts vers leur ancre complète, pour la génération
/// Typst uniquement : le fichier `.tw` garde la forme écrite par l'auteur.
///
/// Le document n'est copié que si un lien a réellement besoin d'être résolu :
/// la génération d'un document sans ancre courte reste sans allocation.
pub fn resoudre(document: &Document) -> std::borrow::Cow<'_, Document> {
    let cibles = cibles_resolues(document);

    if !document
        .blocs
        .iter()
        .any(|bloc| contient_lien_court(bloc, &cibles))
    {
        return std::borrow::Cow::Borrowed(document);
    }

    let mut copie = document.clone();
    for bloc in &mut copie.blocs {
        resoudre_bloc(bloc, &cibles);
    }

    std::borrow::Cow::Owned(copie)
}

fn contient_lien_court(bloc: &Bloc, cibles: &HashMap<String, String>) -> bool {
    match bloc {
        Bloc::Titre { contenu, .. } | Bloc::Paragraphe(contenu) | Bloc::Citation(contenu) => {
            contient_lien_court_inline(contenu, cibles)
        }
        Bloc::ListePuce(items) | Bloc::ListeNumero(items) => items
            .iter()
            .any(|item| contient_lien_court_inline(&item.contenu, cibles)),
        Bloc::Tableau { entete, lignes, .. } => entete
            .iter()
            .chain(lignes.iter().flatten())
            .any(|cellule| contient_lien_court_inline(cellule, cibles)),
        Bloc::Image { légende, .. } => contient_lien_court_inline(légende, cibles),
        Bloc::Décor { bloc, .. } | Bloc::Canonique(bloc) => contient_lien_court(bloc, cibles),
        Bloc::Garde(contenu) => contenu.iter().any(|bloc| contient_lien_court(bloc, cibles)),
        _ => false,
    }
}

fn contient_lien_court_inline(inline: &[Inline], cibles: &HashMap<String, String>) -> bool {
    inline.iter().any(|element| match element {
        Inline::LienInterne { reference, texte } => {
            cibles
                .get(reference)
                .is_some_and(|complete| complete != reference)
                || contient_lien_court_inline(texte, cibles)
        }
        Inline::Gras(contenu)
        | Inline::Italique(contenu)
        | Inline::GrasItalique(contenu)
        | Inline::Souligne(contenu)
        | Inline::Barre(contenu)
        | Inline::Note(contenu)
        | Inline::Couleur { contenu, .. }
        | Inline::Lien { texte: contenu, .. } => contient_lien_court_inline(contenu, cibles),
        Inline::Canonique(contenu) => {
            contient_lien_court_inline(std::slice::from_ref(contenu.as_ref()), cibles)
        }
        _ => false,
    })
}

fn resoudre_bloc(bloc: &mut Bloc, cibles: &HashMap<String, String>) {
    match bloc {
        Bloc::Titre { contenu, .. } | Bloc::Paragraphe(contenu) | Bloc::Citation(contenu) => {
            resoudre_inline(contenu, cibles)
        }
        Bloc::ListePuce(items) | Bloc::ListeNumero(items) => {
            for item in items {
                resoudre_inline(&mut item.contenu, cibles);
            }
        }
        Bloc::Tableau { entete, lignes, .. } => {
            for cellule in entete.iter_mut().chain(lignes.iter_mut().flatten()) {
                resoudre_inline(cellule, cibles);
            }
        }
        Bloc::Image { légende, .. } => resoudre_inline(légende, cibles),
        Bloc::Décor { bloc, .. } | Bloc::Canonique(bloc) => resoudre_bloc(bloc, cibles),
        Bloc::Garde(contenu) => {
            for bloc in contenu {
                resoudre_bloc(bloc, cibles);
            }
        }
        _ => {}
    }
}

fn resoudre_inline(inline: &mut [Inline], cibles: &HashMap<String, String>) {
    for element in inline {
        match element {
            Inline::LienInterne { reference, texte } => {
                if let Some(complete) = cibles.get(reference) {
                    *reference = complete.clone();
                }
                resoudre_inline(texte, cibles);
            }
            Inline::Gras(contenu)
            | Inline::Italique(contenu)
            | Inline::GrasItalique(contenu)
            | Inline::Souligne(contenu)
            | Inline::Barre(contenu)
            | Inline::Note(contenu)
            | Inline::Couleur { contenu, .. }
            | Inline::Lien { texte: contenu, .. } => resoudre_inline(contenu, cibles),
            Inline::Canonique(contenu) => {
                resoudre_inline(std::slice::from_mut(contenu.as_mut()), cibles)
            }
            _ => {}
        }
    }
}

fn parcourir(bloc: &Bloc, compteur: &mut CompteurLiens, elements: &mut Vec<ElementLien>) {
    match bloc {
        Bloc::Titre { niveau, contenu } => {
            let texte = super::schema::inline_texte_nu(contenu);
            let reference = compteur.titre(&texte);
            let libelle = if texte.trim().is_empty() {
                "Titre sans texte".to_string()
            } else {
                texte.clone()
            };
            elements.push(ElementLien {
                reference,
                libelle,
                description: format!("Titre de niveau {niveau}"),
                nature: NatureLien::Titre(*niveau),
            });
        }
        Bloc::Image { légende, cible } => {
            let texte = texte_figure(légende, cible);
            let reference = compteur.figure(&texte);
            elements.push(ElementLien {
                reference,
                libelle: if texte.trim().is_empty() {
                    "Figure sans légende".to_string()
                } else {
                    texte
                },
                description: if légende.is_empty() {
                    "Image".to_string()
                } else {
                    "Figure".to_string()
                },
                nature: NatureLien::Figure,
            });
        }
        Bloc::Décor { bloc, .. } | Bloc::Canonique(bloc) => {
            parcourir(bloc, compteur, elements);
        }
        _ => {}
    }
}

/// Un lien interne dont la cible n'existe plus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LienCasse {
    /// L'ancre écrite dans la source.
    pub reference: String,
    /// Le texte visible du lien, pour reconnaître la phrase concernée.
    pub texte: String,
}

/// Les liens internes qui ne trouvent aucune cible dans le document.
///
/// C'est le diagnostic le plus utile après un renommage de titre : l'ancre
/// suit le texte, et un lien ancien devient silencieusement mort. Le contrôle
/// parcourt aussi les notes, les tableaux et les blocs canoniques.
pub fn cibles_absentes(document: &Document) -> Vec<LienCasse> {
    let cibles: HashSet<String> = cibles_resolues(document).into_keys().collect();
    let mut sortie = Vec::new();

    for bloc in &document.blocs {
        parcourir_les_liens(bloc, &cibles, &mut sortie);
    }

    sortie
}

fn parcourir_les_liens(bloc: &Bloc, cibles: &HashSet<String>, sortie: &mut Vec<LienCasse>) {
    match bloc {
        Bloc::Titre { contenu, .. } | Bloc::Paragraphe(contenu) | Bloc::Citation(contenu) => {
            parcourir_inline(contenu, cibles, sortie)
        }
        Bloc::Image { légende, .. } => parcourir_inline(légende, cibles, sortie),
        Bloc::ListePuce(elements) | Bloc::ListeNumero(elements) => {
            for element in elements {
                parcourir_inline(&element.contenu, cibles, sortie);
            }
        }
        Bloc::Tableau { entete, lignes, .. } => {
            for cellule in entete.iter().flatten() {
                parcourir_inline(std::slice::from_ref(cellule), cibles, sortie);
            }
            for ligne in lignes {
                for cellule in ligne.iter().flatten() {
                    parcourir_inline(std::slice::from_ref(cellule), cibles, sortie);
                }
            }
        }
        Bloc::Décor { bloc, .. } | Bloc::Canonique(bloc) => {
            parcourir_les_liens(bloc, cibles, sortie);
        }
        _ => {}
    }
}

fn parcourir_inline(inline: &[Inline], cibles: &HashSet<String>, sortie: &mut Vec<LienCasse>) {
    for element in inline {
        match element {
            Inline::LienInterne { reference, texte } => {
                if !cibles.contains(reference) {
                    sortie.push(LienCasse {
                        reference: reference.clone(),
                        texte: super::schema::inline_texte_nu(texte),
                    });
                }
                parcourir_inline(texte, cibles, sortie);
            }
            Inline::Gras(contenu)
            | Inline::Italique(contenu)
            | Inline::GrasItalique(contenu)
            | Inline::Souligne(contenu)
            | Inline::Barre(contenu)
            | Inline::Note(contenu)
            | Inline::Couleur { contenu, .. }
            | Inline::Lien { texte: contenu, .. } => parcourir_inline(contenu, cibles, sortie),
            Inline::Canonique(interne) => {
                parcourir_inline(std::slice::from_ref(interne.as_ref()), cibles, sortie);
            }
            Inline::Texte(_)
            | Inline::Litteral(_)
            | Inline::Date(_)
            | Inline::Glo { .. }
            | Inline::Code(_)
            | Inline::Formule(_)
            | Inline::Citation(_)
            | Inline::Meta(_)
            | Inline::Bible(_)
            | Inline::BibleForme { .. } => {}
        }
    }
}

/// Une base d'ancre : minuscules, sans accents, séparée par des tirets.
pub fn slug(texte: &str) -> String {
    let normalise = crate::base::texte::normaliser_recherche(texte);
    let mut out = String::new();
    let mut tiret = false;

    for caractere in normalise.chars() {
        if caractere.is_ascii_alphanumeric() {
            out.push(caractere);
            tiret = false;
        } else if !tiret && !out.is_empty() {
            out.push('-');
            tiret = true;
        }
    }

    while out.ends_with('-') {
        out.pop();
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn les_ancres_sont_lisibles_et_uniques() {
        assert_eq!(slug("Le sens de l'Écriture"), "le-sens-de-l-ecriture");
        assert_eq!(slug("  "), "");
        assert_eq!(slug("Déjà vu"), "deja-vu");

        let mut compteur = CompteurLiens::default();
        assert_eq!(compteur.titre("Introduction"), "titre-introduction");
        assert_eq!(compteur.titre("Introduction"), "titre-introduction-2");
        assert_eq!(compteur.figure("Introduction"), "figure-introduction");
    }

    #[test]
    fn les_titres_et_figures_sont_collectes() {
        let document = super::super::parse::parse(
            "Un avant-propos.\n\n# Introduction\n\n![Une carte](carte.png)\n\n## Introduction\n",
        );
        let elements = collecter(&document);
        assert_eq!(elements.len(), 3, "{elements:?}");
        assert_eq!(elements[0].reference, "titre-introduction");
        assert_eq!(elements[1].reference, "figure-une-carte");
        assert_eq!(elements[2].reference, "titre-introduction-2");
        assert_eq!(elements[0].nature, NatureLien::Titre(1));
    }

    #[test]
    fn un_lien_interne_se_lit_et_se_relit() {
        let source = "Voir [[titre-introduction|introduction]] et [[figure-carte|la carte]].\n";
        let document = super::super::parse::parse(source);
        let inline = match &document.blocs[0] {
            crate::langage::schema::Bloc::Paragraphe(inline) => inline,
            autre => panic!("un paragraphe attendu : {autre:?}"),
        };
        assert!(matches!(
            inline[1],
            crate::langage::schema::Inline::LienInterne { .. }
        ));
        assert_eq!(super::super::serialiser(&document), source);
    }

    #[test]
    fn un_lien_interne_devient_un_lien_typst() {
        let document =
            super::super::parse::parse("# Introduction\n\nVoir [[titre-introduction|le début]].\n");
        let typst = super::super::generer::generer(&document);
        assert!(
            typst.contains("= Introduction <titre-introduction>"),
            "{typst}"
        );
        assert!(
            typst.contains("#link(<titre-introduction>)[le début]"),
            "{typst}"
        );
    }

    #[test]
    fn un_mot_cle_entre_guillemets_demande_une_numerotation() {
        let document = super::super::parse::parse(
            "# Adoption\n\n## Coutume\n\nVoir [[titre-coutume|\"chapitre\"]] et [[titre-adoption|\"page\"]].\n",
        );
        let typst = super::super::generer::generer(&document);
        assert!(
            typst.contains(
                "#link(<titre-coutume>)[chapitre #context counter(heading).display(at: <titre-coutume>)]"
            ),
            "la tête affiche son numéro : {typst}"
        );
        assert!(
            typst.contains(
                "#link(<titre-adoption>)[page #context counter(page).display(at: <titre-adoption>)]"
            ),
            "la page est calculée : {typst}"
        );
    }

    #[test]
    fn une_ancre_courte_est_resolue_vers_son_titre() {
        let document = super::super::parse::parse(
            "# Adoption coutume\n\nVoir [[adoption-coutume|\"chapitre\"]].\n",
        );
        let typst = super::super::generer::generer(&document);
        assert!(
            typst.contains("#link(<titre-adoption-coutume>)[chapitre #context counter(heading).display(at: <titre-adoption-coutume>)]"),
            "la forme courte choisit la bonne ancre : {typst}"
        );
        assert!(
            super::cibles_absentes(&document).is_empty(),
            "la cible courte n'est pas signalée comme absente"
        );
    }

    #[test]
    fn une_image_sans_legende_se_cite_par_sa_page() {
        let document = super::super::parse::parse(
            "![](le-fils-prodigue-rembrandt.png)\n\nVoir [[le-fils-prodigue-rembrandt|\"page\"]].\n",
        );
        let typst = super::super::generer::generer(&document);
        assert!(
            typst.contains("#link(<figure-le-fils-prodigue-rembrandt>)[page #context counter(page).display(at: <figure-le-fils-prodigue-rembrandt>)]"),
            "l'image sans légende reste une cible de page : {typst}"
        );
        assert!(
            super::cibles_absentes(&document).is_empty(),
            "la cible courte de l'image est reconnue"
        );
    }

    #[test]
    fn un_lien_numerote_se_relit_a_l_identique() {
        let source = "Voir [[adoption-coutume|\"chapitre\"]] et [[planche|\"page\"]].\n";
        let document = super::super::parse::parse(source);
        assert_eq!(super::super::serialiser::serialiser(&document), source);
    }

    #[test]
    fn les_liens_sans_cible_sont_signales_dans_tout_le_document() {
        let document = super::super::parse::parse(
            "# Introduction\n\n\
             Voir [[titre-introduction|le début]] et [[titre-absente|ailleurs]].\n\n\
             | Une cellule [[figure-inconnue|une figure]] |\n\
             | --- |\n\
             | Un texte |\n\n\
             Une note.^[Un renvoi [[titre-introduction-9|perdu]].]\n",
        );
        let absents = cibles_absentes(&document);
        assert_eq!(absents.len(), 3, "{absents:?}");
        assert!(absents.iter().any(|lien| lien.reference == "titre-absente"));
        assert!(
            absents
                .iter()
                .any(|lien| lien.reference == "figure-inconnue")
        );
        assert!(
            absents
                .iter()
                .any(|lien| lien.reference == "titre-introduction-9")
        );
    }

    #[test]
    fn un_lien_interne_ne_contient_pas_un_autre_lien() {
        use crate::langage::schema::{Bloc, Document, Inline};

        let document = Document {
            blocs: vec![Bloc::Paragraphe(vec![Inline::LienInterne {
                reference: "titre-exterieur".to_string(),
                texte: vec![Inline::LienInterne {
                    reference: "titre-interieur".to_string(),
                    texte: vec![Inline::Texte("dedans".to_string())],
                }],
            }])],
        };

        assert!(
            !document.est_canonique(),
            "les crochets d'un lien intérieur rendraient la source ambiguë"
        );
    }
}
