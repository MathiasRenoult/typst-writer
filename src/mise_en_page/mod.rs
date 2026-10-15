//! La mise en page du document, et la fenêtre qui la règle.
//!
//! Un document a des normes : une police, une taille, des marges, une
//! numérotation de chapitres, une casse, un interligne. Elles ne devraient
//! pas s'écrire à la main en `#set` pour quiconque veut un travail propre —
//! elles se **choisissent**, et c'est le métier de ce dossier : tenir ces
//! choix dans une structure, et en tirer l'en-tête Typst qui les exprime.
//!
//! L'en-tête vit dans `mise-en-page.typ`, à côté de `main.typ` : le document
//! de l'utilisateur reste le sien, et la mise en page est un objet qu'on peut
//! exporter, importer, et comparer.

// Pièces posées avant d'être raccordées : la fenêtre de réglage les emploie
// dès la tranche suivante. Cet attribut partira avec le raccordement.
#![allow(dead_code)]

pub mod gabarit;
pub mod modele;

use std::path::Path;

/// Le fichier de réglages, dans le projet : c'est lui qui s'importe et
/// s'exporte.
pub const REGLAGES: &str = "mise-en-page.toml";

/// Ce que l'application a fait du projet, dit en une phrase.
#[derive(Debug, PartialEq, Eq)]
pub enum Applique {
    /// Tout était déjà en place.
    RienAjoute,

    /// `main.typ` a reçu les deux lignes d'accroche.
    AccrocheAjoutee,
}

/// Les réglages d'un projet : ceux du fichier, ou les valeurs par défaut.
pub fn charger(projet: &Path) -> modele::MiseEnPage {
    std::fs::read_to_string(projet.join(REGLAGES))
        .ok()
        .and_then(|texte| modele::MiseEnPage::lire(&texte).ok())
        .unwrap_or_default()
}

/// Écrit la mise en page dans le projet : ses réglages, son en-tête, et
/// l'accroche de `main.typ` si elle est demandée.
///
/// `main.typ` n'est jamais réécrit pour le plaisir : ses deux lignes se
/// **placent en tête** quand elles manquent, et tout ce qui suit reste octet
/// pour octet ce qu'il était. Un document ne se réécrit pas dans son dos.
pub fn appliquer(
    projet: &Path,
    mise_en_page: &modele::MiseEnPage,
    accrocher: bool,
) -> Result<Applique, String> {
    std::fs::write(projet.join(REGLAGES), mise_en_page.ecrire())
        .map_err(|erreur| format!("écrire les réglages : {erreur}"))?;

    std::fs::write(projet.join(gabarit::FICHIER), gabarit::generer(mise_en_page))
        .map_err(|erreur| format!("écrire l'en-tête : {erreur}"))?;

    let main = projet.join("main.typ");
    let texte = std::fs::read_to_string(&main).unwrap_or_default();

    if !accrocher || texte.contains("#show: mise-en-page") {
        return Ok(Applique::RienAjoute);
    }

    let reste = texte.trim_start_matches(['\n', '\r', ' ', '\t']);
    let mut nouveau = gabarit::accroche();

    if !reste.is_empty() {
        nouveau.push('\n');
        nouveau.push_str(reste);
    }

    std::fs::write(&main, nouveau).map_err(|erreur| format!("accrocher main.typ : {erreur}"))?;

    Ok(Applique::AccrocheAjoutee)
}

/// Les deux lignes d'accroche manquent-elles à `main.typ` ?
pub fn accroche_manquante(projet: &Path) -> bool {
    std::fs::read_to_string(projet.join("main.typ"))
        .map(|texte| !texte.contains("#show: mise-en-page"))
        .unwrap_or(true)
}

/// `main.typ` porte-t-il déjà des règles `#set` écrites à la main ?
///
/// Elles priment sur la mise en page choisie — c'est ainsi que Typst les
/// ordonne — et il vaut mieux le savoir avant de croire son réglage appliqué.
pub fn regles_ecrites_a_la_main(projet: &Path) -> usize {
    std::fs::read_to_string(projet.join("main.typ"))
        .unwrap_or_default()
        .lines()
        .filter(|ligne| ligne.trim_start().starts_with("#set "))
        .count()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use crate::typst::render::compile;

    /// La taille de page trahit la règle qui a gagné : A4 par défaut,
    /// A5 si `#set page(paper: "a5")` s'est appliqué.
    fn largeur(entry: &str, fichiers: &[(&str, &str)]) -> f64 {
        let root = std::path::PathBuf::from("/projet");
        let overrides: Vec<(PathBuf, String)> = fichiers
            .iter()
            .map(|(nom, texte)| (root.join(nom), (*texte).to_string()))
            .collect();

        let outcome = compile(&root, entry, &overrides);

        assert!(
            outcome.errors.is_empty(),
            "le document d'épreuve ne compile pas : {:?}",
            outcome.errors
                .iter()
                .map(|erreur| erreur.message.clone())
                .collect::<Vec<_>>()
        );

        let document = outcome.document.as_ref().expect("un document");
        let page = document.pages().first().expect("une page");

        page.frame.size().x.to_pt()
    }

    /// `#include` apporte-t-il ses règles au reste du document ?
    #[test]
    fn what_an_include_really_brings() {
        let par_include = largeur(
            "#include \"mise-en-page.typ\"\nBonjour",
            &[("mise-en-page.typ", "#set page(paper: \"a5\")\n")],
        );

        let par_gabarit = largeur(
            "#import \"gabarit.typ\": gabarit\n#show: gabarit\nBonjour",
            &[(
                "gabarit.typ",
                "#let gabarit(doc) = {\n  set page(paper: \"a5\")\n  doc\n}\n",
            )],
        );

        println!("largeur par include  : {par_include:.2} (A4 = 595.28, A5 = 419.53)");
        println!("largeur par gabarit  : {par_gabarit:.2}");
    }
}
