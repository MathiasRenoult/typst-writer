//! La traduction des réglages en en-tête Typst.
//!
//! C'est **le** métier de ce dossier : prendre ce qu'un humain a choisi dans
//! une fenêtre — une police, des marges, une numérotation — et l'écrire en un
//! gabarit que Typst comprend et applique au document entier.
//!
//! Deux vérités mesurées dans le compilateur décident de la forme :
//!
//! - un `#include` n'apporte **pas** ses règles au reste du document — ses
//!   `#set` sont sans effet après lui ; seul un gabarit, appliqué par
//!   `#show:`, enveloppe réellement le document ;
//! - c'est donc une fonction que produit ce fichier, et `main.typ` n'en
//!   reçoit que les deux lignes d'accroche.
//!
//! Une valeur invalide n'atteint jamais le document : elle est **omise**, et
//! c'est la mise en page elle-même qui dit le problème, champ par champ.

use super::modele::MiseEnPage;

/// Le fichier qui porte l'en-tête.
pub const FICHIER: &str = "mise-en-page.typ";

/// La fonction gabarit qu'il définit.
pub const FONCTION: &str = "mise-en-page";

/// Les deux lignes d'accroche, à placer en tête de `main.typ`.
pub fn accroche() -> String {
    format!("#import \"{FICHIER}\": {FONCTION}\n#show: {FONCTION}\n")
}

/// Traduit une mise en page en en-tête Typst.
pub fn generer(mise_en_page: &MiseEnPage) -> String {
    let mut out = String::new();

    out.push_str("// Généré par Typst Writer — la mise en page du document.\n");
    out.push_str("// Elle se règle dans l'application : ce fichier n'en est que la\n");
    out.push_str("// traduction, et se réécrit tout entier à chaque changement.\n\n");

    out.push_str(&format!("#let {FONCTION}(doc) = {{\n"));

    // Les guillemets sont ceux des normes : Typst ne retransforme pas ce
    // que la typographie a composé. En `lang: "fr"`, sa propre conversion
    // changerait tout — même les paires anglaises — en guillemets français.
    out.push_str("  set smartquote(enabled: false)\n");

    out.push_str(&page(mise_en_page));
    out.push_str(&texte(mise_en_page));
    out.push_str(&paragraphes(mise_en_page));
    out.push_str(&titres(mise_en_page));
    out.push_str(&liens(mise_en_page));

    out.push_str("  doc\n}\n");
    out
}

/// La page : son papier, ses marges, ses numéros.
fn page(mise_en_page: &MiseEnPage) -> String {
    let mut lignes = vec!["  set page(".to_string()];

    // Un format inconnu n'est pas écrit : Typst refuserait la page entière.
    match papier_typst(&mise_en_page.papier.format) {
        Some(papier) => lignes.push(format!("    paper: {papier},")),
        None => lignes.push("    paper: auto,".to_string()),
    }

    if mise_en_page.papier.orientation == "paysage" {
        lignes.push("    flipped: true,".to_string());
    }

    let marges = [
        ("top", &mise_en_page.marges.haut),
        ("bottom", &mise_en_page.marges.bas),
        ("left", &mise_en_page.marges.gauche),
        ("right", &mise_en_page.marges.droite),
    ];

    let marges: Vec<String> = marges
        .iter()
        .filter(|(_, valeur)| super::modele::longueur_valide(valeur))
        .map(|(cote, valeur)| format!("{cote}: {valeur}"))
        .collect();

    if !marges.is_empty() {
        lignes.push(format!("    margin: ({}),", marges.join(", ")));
    }

    if super::modele::couleur_valide(&mise_en_page.couleurs.fond) {
        lignes.push(format!(
            "    fill: rgb(\"{}\"),",
            mise_en_page.couleurs.fond
        ));
    }

    // L'en-tête et le pied de page rédigés : leurs trois places, et le
    // numéro qu'ils portent. S'ils ne disent rien, la numérotation de
    // toujours continue de parler.
    let entete_pied = entete_pied(mise_en_page);

    if !entete_pied.is_empty() {
        for ligne in entete_pied.lines() {
            lignes.push(ligne.to_string());
        }
    }

    // Le pied rédigé porte déjà son numéro : la numérotation de la page
    // ne se répéterait pas.
    let pied_redige = !mise_en_page.entete_pied.pied.est_vide();

    if mise_en_page.numerotation.actif && !pied_redige {
        lignes.push(format!(
            "    numbering: \"{}\",",
            forme_numerotation(&mise_en_page.numerotation.forme)
        ));
        lignes.push(format!(
            "    number-align: {},",
            alignement_numero(
                &mise_en_page.numerotation.alignement,
                &mise_en_page.numerotation.position
            )
        ));
    } else {
        lignes.push("    numbering: none,".to_string());
    }

    lignes.push("  )\n".to_string());
    lignes.join("\n")
}

/// Le texte courant : police, corps, langue, couleur.
fn texte(mise_en_page: &MiseEnPage) -> String {
    let mut lignes = vec!["  set text(".to_string()];

    if let Some(police) = police_typst(&mise_en_page.texte.police) {
        lignes.push(format!("    font: {police},"));
    }

    if super::modele::longueur_valide(&mise_en_page.texte.taille) {
        lignes.push(format!("    size: {},", mise_en_page.texte.taille));
    }

    if !mise_en_page.texte.langue.is_empty() {
        lignes.push(format!(
            "    lang: \"{}\",",
            echapper(&mise_en_page.texte.langue)
        ));
    }

    lignes.push(format!(
        "    hyphenate: {},",
        if mise_en_page.texte.coupure {
            "true"
        } else {
            "false"
        }
    ));

    if super::modele::couleur_valide(&mise_en_page.couleurs.texte) {
        lignes.push(format!(
            "    fill: rgb(\"{}\"),",
            mise_en_page.couleurs.texte
        ));
    }

    lignes.push("  )\n".to_string());
    lignes.join("\n")
}

/// Les paragraphes : justification, interligne, espacement, retrait.
fn paragraphes(mise_en_page: &MiseEnPage) -> String {
    let mut lignes = vec!["  set par(".to_string()];

    lignes.push(format!(
        "    justify: {},",
        if mise_en_page.texte.justification {
            "true"
        } else {
            "false"
        }
    ));

    for (champ, valeur) in [
        ("leading", &mise_en_page.texte.interligne),
        ("spacing", &mise_en_page.texte.espacement),
        ("first-line-indent", &mise_en_page.texte.retrait),
    ] {
        if super::modele::longueur_valide(valeur) {
            lignes.push(format!("    {champ}: {valeur},"));
        }
    }

    lignes.push("  )\n".to_string());
    lignes.join("\n")
}

/// Les titres : numérotation, casse, corps par niveau, police, gras.
fn titres(mise_en_page: &MiseEnPage) -> String {
    let mut out = String::new();
    let titres = &mise_en_page.titres;

    if titres.numerotation.trim().is_empty() {
        out.push_str("  set heading(numbering: none)\n");
    } else {
        out.push_str(&format!(
            "  set heading(numbering: \"{}\")\n",
            echapper(titres.numerotation.trim())
        ));
    }

    if super::modele::longueur_valide(&titres.espacement) {
        // L'espacement d'un titre n'est pas un réglage de `heading` mais de
        // son bloc : c'est le compilateur qui le dit, et il a raison.
        out.push_str(&format!(
            "  show heading: set block(above: {}, below: {})\n",
            titres.espacement, titres.espacement
        ));
    }

    if let Some(police) = police_typst(&titres.police) {
        out.push_str(&format!("  show heading: set text(font: {police})\n"));
    }

    // NORMS-04-02 : chaque niveau porte ses réglages, s'ils existent.
    for (niveau, taille) in [
        (1, &titres.taille_1),
        (2, &titres.taille_2),
        (3, &titres.taille_3),
        (4, &titres.taille_4),
        (5, &titres.taille_5),
        (6, &titres.taille_6),
    ] {
        let mut regles = Vec::new();

        if super::modele::longueur_valide(taille) {
            regles.push(format!("size: {taille}"));
        }

        // La graisse par niveau gagne sur le réglage global.
        let gras_niveau = match niveau {
            1 => titres.gras_1,
            2 => titres.gras_2,
            3 => titres.gras_3,
            4 => titres.gras_4,
            5 => titres.gras_5,
            _ => titres.gras_6,
        };
        let gras = gras_niveau.unwrap_or(titres.gras);
        if gras {
            regles.push("weight: \"bold\"".to_string());
        }

        // La couleur par niveau gagne sur la couleur globale de titres.
        let couleur_niveau = match niveau {
            1 => titres.couleur_1.as_deref(),
            2 => titres.couleur_2.as_deref(),
            3 => titres.couleur_3.as_deref(),
            4 => titres.couleur_4.as_deref(),
            5 => titres.couleur_5.as_deref(),
            _ => titres.couleur_6.as_deref(),
        };
        let couleur = couleur_niveau
            .filter(|c| super::modele::couleur_valide(c))
            .unwrap_or(&mise_en_page.couleurs.titres);
        if super::modele::couleur_valide(couleur) {
            regles.push(format!("fill: rgb(\"{}\")", couleur));
        }

        if !regles.is_empty() {
            out.push_str(&format!(
                "  show heading.where(level: {niveau}): set text({})\n",
                regles.join(", ")
            ));
        }

        // L'espacement par niveau gagne sur l'espacement global.
        let espacement_niveau = match niveau {
            1 => titres.espacement_1.as_deref(),
            2 => titres.espacement_2.as_deref(),
            3 => titres.espacement_3.as_deref(),
            4 => titres.espacement_4.as_deref(),
            5 => titres.espacement_5.as_deref(),
            _ => titres.espacement_6.as_deref(),
        };
        let espacement = espacement_niveau
            .filter(|e| super::modele::longueur_valide(e))
            .unwrap_or(&titres.espacement);
        if super::modele::longueur_valide(espacement) {
            out.push_str(&format!(
                "  show heading.where(level: {niveau}): set block(above: {espacement}, below: {espacement})\n"
            ));
        }

        // La casse par niveau gagne sur la casse globale.
        let casse_niveau = match niveau {
            1 => titres.casse_1.as_deref(),
            2 => titres.casse_2.as_deref(),
            3 => titres.casse_3.as_deref(),
            4 => titres.casse_4.as_deref(),
            5 => titres.casse_5.as_deref(),
            _ => titres.casse_6.as_deref(),
        };
        let casse = casse_niveau.unwrap_or(&titres.casse);
        match casse {
            "majuscules" => out.push_str(&format!(
                "  show heading.where(level: {niveau}): it => upper(it)\n"
            )),
            "minuscules" => out.push_str(&format!(
                "  show heading.where(level: {niveau}): it => lower(it)\n"
            )),
            "petites-majuscules" => out.push_str(&format!(
                "  show heading.where(level: {niveau}): it => smallcaps(it)\n"
            )),
            _ => {}
        }
    }

    // La casse globale s'applique aux niveaux sans casse propre.
    if titres.casse_1.is_none() && titres.casse_2.is_none() && titres.casse_3.is_none()
        && titres.casse_4.is_none() && titres.casse_5.is_none() && titres.casse_6.is_none()
    {
        match titres.casse.as_str() {
            "majuscules" => out.push_str("  show heading: it => upper(it)\n"),
            "minuscules" => out.push_str("  show heading: it => lower(it)\n"),
            "petites-majuscules" => out.push_str("  show heading: it => smallcaps(it)\n"),
            _ => {}
        }
    }

    out.push('\n');
    out
}

/// L'en-tête et le pied de page : leurs trois places, et la forme du
/// numéro.
///
/// `{page}` devient le numéro courant, `{pages}` le nombre de pages —
/// tous deux dans la forme choisie (`1`, `a`, `A`, `i`, `I`, `α`, `א`,
/// celles que Typst sait composer). Le texte du voisinage est du texte :
/// « p. {page} » donne « p. 3 », « page {page} sur {pages} » donne
/// « page 3 sur 12 ».
fn entete_pied(mise_en_page: &MiseEnPage) -> String {
    let entete_pied = &mise_en_page.entete_pied;

    if entete_pied.entete.est_vide() && entete_pied.pied.est_vide() {
        return String::new();
    }

    let forme = entete_pied.numerotation.trim();
    let forme = if forme.is_empty() { "1" } else { forme };

    /// Un lieu — gauche, centre, droite — composé : `{page}` et `{pages}`
    /// prennent leur place dans le texte, le reste est du texte.
    fn lieu(texte: &str, forme: &str) -> String {
        /// Le texte d'un lieu est du texte : les caractères qui signifient
        /// quelque chose pour Typst s'échappent, le reste ne bouge pas.
        fn echapper(texte: &str) -> String {
            let mut out = String::new();

            for caractere in texte.chars() {
                if matches!(caractere, '\\' | '[' | ']' | '#' | '$' | '_' | '*') {
                    out.push('\\');
                }

                out.push(caractere);
            }

            out
        }

        let mut out = String::new();

        for (rang, morceau) in texte.split("{page}").enumerate() {
            if rang > 0 {
                out.push_str(&format!("#context counter(page).display(\"{forme}\")"));
            }

            let morceau = morceau.replace("{pages}", "\\x00");
            // Les Quick Parts — NORMS-11 : des marqueurs que l'échappement
            // ne touche pas, remplacés après coup par du code Typst.
            let morceau = morceau.replace("{document}", "\\x01");
            let morceau = morceau.replace("{date}", "\\x02");
            let morceau = morceau.replace("{auteur}", "\\x03");
            let morceau = morceau.replace("{projet}", "\\x04");

            for (rang, morceau) in morceau.split("\\x00").enumerate() {
                if rang > 0 {
                    out.push_str(&format!("#context numbering(\"{forme}\", ..counter(page).final())"));
                }

                // On échappe le texte, puis on replace les marqueurs par
                // leur code Typst — non échappé, c'est du code.
                let texte_echappe = echapper(morceau);
                let texte_echappe = texte_echappe.replace("\\x01", "#sys.inputs.at(\"document\", default: \"\")");
                let texte_echappe = texte_echappe.replace("\\x02", "#datetime.today().display()");
                let texte_echappe = texte_echappe.replace("\\x03", "#sys.inputs.at(\"auteur\", default: \"\")");
                let texte_echappe = texte_echappe.replace("\\x04", "#sys.inputs.at(\"projet\", default: \"\")");

                out.push_str(&texte_echappe);
            }
        }

        out
    }

    let grille = |empl: &crate::mise_en_page::modele::Emplacement| -> String {
        format!(
            "grid(columns: (1fr, 1fr, 1fr), align: (left, center, right), [{gauche}], [{centre}], [{droite}])",
            gauche = lieu(&empl.gauche, forme),
            centre = lieu(&empl.centre, forme),
            droite = lieu(&empl.droite, forme),
        )
    };

    let mut out = String::new();

    if !entete_pied.entete.est_vide() {
        out.push_str(&format!(
            "    header: align(center, {}),\n",
            grille(&entete_pied.entete)
        ));
    }

    if !entete_pied.pied.est_vide() {
        out.push_str(&format!(
            "    footer: align(center, {}),\n",
            grille(&entete_pied.pied)
        ));
    }

    out
}

/// Les liens : leur couleur. Le gras et l'italique, quand ils ont la leur.
fn liens(mise_en_page: &MiseEnPage) -> String {
    let mut out = String::new();

    // Le gras et l'italique ont leur couleur quand elle est dite ; vides,
    // ils gardent celle du texte.
    if super::modele::couleur_valide(&mise_en_page.couleurs.gras) {
        out.push_str(&format!(
            "  show strong: set text(fill: rgb(\"{}\"))\n",
            mise_en_page.couleurs.gras
        ));
    }

    if super::modele::couleur_valide(&mise_en_page.couleurs.italique) {
        out.push_str(&format!(
            "  show emph: set text(fill: rgb(\"{}\"))\n",
            mise_en_page.couleurs.italique
        ));
    }

    if super::modele::couleur_valide(&mise_en_page.couleurs.liens) {
        out.push_str(&format!(
            "  show link: set text(fill: rgb(\"{}\"))\n",
            mise_en_page.couleurs.liens
        ));
    }

    out
}

/// Le papier, dans les mots de Typst.
fn papier_typst(format: &str) -> Option<String> {
    match format.trim().to_lowercase().as_str() {
        "a3" | "a4" | "a5" | "a6" | "b4" | "b5" | "letter" | "legal" | "business-card" => {
            Some(format!("\"{format}\""))
        }
        _ => None,
    }
}

/// La forme de numérotation, dans les mots de Typst.
fn forme_numerotation(forme: &str) -> String {
    match forme {
        "I" => "I".to_string(),
        "i" => "i".to_string(),
        "A" => "A".to_string(),
        "a" => "a".to_string(),
        _ => "1".to_string(),
    }
}

/// L'alignement d'un numéro de page : `center + bottom` et pareils.
fn alignement_numero(alignement: &str, position: &str) -> String {
    let horizontal = match alignement {
        "gauche" => "left",
        "droite" => "right",
        _ => "center",
    };

    let vertical = if position == "haut" { "top" } else { "bottom" };

    format!("{horizontal} + {vertical}")
}

/// Une police, ou plusieurs, dans les mots de Typst.
///
/// Plusieurs se séparent par des virgues : la seconde sert à ce que la
/// première ne sait pas écrire — un grec dans un texte latin, par exemple.
fn police_typst(polices: &str) -> Option<String> {
    let polices: Vec<&str> = polices
        .split(',')
        .map(str::trim)
        .filter(|police| !police.is_empty())
        .collect();

    match polices.len() {
        0 => None,
        1 => Some(format!("\"{}\"", echapper(polices[0]))),
        _ => Some(format!(
            "({})",
            polices
                .iter()
                .map(|police| format!("\"{}\"", echapper(police)))
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
}

/// Les guillemets et antislashs d'une chaîne, échappés pour Typst.
fn echapper(texte: &str) -> String {
    texte.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::typst::render::compile;
    use std::path::PathBuf;
    use std::sync::Arc;
    use typst::layout::{Frame, FrameItem};
    use typst_layout::PagedDocument;

    /// Compile un document qui accroche un en-tête, et renvoie le document.
    fn documenter(mise_en_page: &MiseEnPage, corps: &str) -> Arc<PagedDocument> {
        let root = PathBuf::from("/projet");
        let entry = format!("{}{corps}", accroche());
        let overrides = vec![(root.join(FICHIER), generer(mise_en_page))];

        let outcome = compile(&root, &entry, &overrides);

        assert!(
            outcome.errors.is_empty(),
            "l'en-tête ne compile pas : {:?}",
            outcome
                .errors
                .iter()
                .map(|erreur| erreur.message.clone())
                .collect::<Vec<_>>()
        );

        outcome.document.expect("un document")
    }

    /// Le texte réellement composé, avec la couleur de chaque trait.
    fn composer(document: &PagedDocument) -> Vec<(String, String)> {
        fn marcher(frame: &Frame, out: &mut Vec<(String, String)>) {
            for (_, item) in frame.items() {
                match item {
                    FrameItem::Text(text) => {
                        out.push((text.text.to_string(), format!("{:?}", text.fill)));
                    }
                    FrameItem::Group(group) => marcher(&group.frame, out),
                    _ => {}
                }
            }
        }

        let mut out = Vec::new();

        for page in document.pages() {
            marcher(&page.frame, &mut out);
        }

        out
    }

    /// L'en-tête par défaut est un gabarit qui s'applique au document entier.
    #[test]
    fn the_default_header_applies_to_the_whole_document() {
        let document = documenter(&MiseEnPage::default(), "= Chapitre\n\nDu texte.\n");

        let largeur = document
            .pages()
            .first()
            .expect("une page")
            .frame
            .size()
            .x
            .to_pt();

        assert!(
            (largeur - 595.28).abs() < 0.01,
            "A4 par défaut — vu : {largeur}"
        );
    }

    /// Le papier et l'orientation suivent les réglages — c'est la preuve que
    /// les règles du gabarit atteignent bien le document.
    #[test]
    fn the_geometry_follows_the_config() {
        let mut mise_en_page = MiseEnPage::default();
        mise_en_page.papier.format = "a5".to_string();
        mise_en_page.papier.orientation = "paysage".to_string();

        let document = documenter(&mise_en_page, "= Chapitre\n\nDu texte.\n");
        let page = document.pages().first().expect("une page");

        let (largeur, hauteur) = (page.frame.size().x.to_pt(), page.frame.size().y.to_pt());

        assert!(
            (largeur - 595.28).abs() < 0.01,
            "A5 à l'horizontale : la hauteur devient la largeur — vu : {largeur}"
        );
        assert!((hauteur - 419.53).abs() < 0.01, "vu : {hauteur}");
    }

    /// La casse et la couleur des titres sont celles qu'on a demandées.
    #[test]
    fn the_heading_case_and_colour_are_the_ones_asked_for() {
        let mut mise_en_page = MiseEnPage::default();
        mise_en_page.titres.casse = "majuscules".to_string();
        mise_en_page.couleurs.titres = "#1e1e2e".to_string();

        let document = documenter(&mise_en_page, "= Chapitre\n\nDu texte.\n");
        let traits = composer(&document);

        let (texte, couleur) = traits
            .iter()
            .find(|(texte, _)| texte.contains("CHAPITRE"))
            .expect("le titre en majuscules");

        assert_eq!(texte, "CHAPITRE");
        // #1e1e2e, en flottants normalisés : 30/255, 30/255, 46/255.
        let attendu = format!(
            "Rgb({}, {}, {}, 1)",
            30f32 / 255f32,
            30f32 / 255f32,
            46f32 / 255f32
        );

        assert_eq!(couleur, &attendu, "la couleur du titre est #1e1e2e");
    }

    /// Une valeur invalide n'atteint pas le document : elle est omise, et le
    /// document compile quand même.
    #[test]
    fn an_invalid_value_never_reaches_the_document() {
        let mut mise_en_page = MiseEnPage::default();
        mise_en_page.marges.gauche = "beaucoup".to_string();
        mise_en_page.texte.taille = "".to_string();

        let document = documenter(&mise_en_page, "= Chapitre\n\nDu texte.\n");

        assert_eq!(document.pages().len(), 1);
    }

    /// L'accroche tient en deux lignes, et elles disent ce qu'elles font.
    #[test]
    fn the_hook_is_two_lines() {
        assert_eq!(
            accroche(),
            "#import \"mise-en-page.typ\": mise-en-page\n#show: mise-en-page\n"
        );
    }
}

/// Le pied de page rédigé se compose : ses trois places, et le numéro
/// qu'il porte, dans la forme choisie.
#[test]
fn un_pied_redige_porte_son_numero() {
    let mut mise = MiseEnPage::default();
    mise.entete_pied.pied.droite = "p. {page}".to_string();
    mise.entete_pied.numerotation = "a".to_string();

    let sortie = generer(&mise);
    assert!(
        sortie.contains("footer: align(center, grid("),
        "le pied se compose : {sortie}"
    );
    assert!(
        sortie.contains("context counter(page).display(\"a\")"),
        "le numéro est dans la forme choisie : {sortie}"
    );
    assert!(
        sortie.contains("p. "),
        "le texte du pied s'y trouve : {sortie}"
    );

    // Le total, lui, se prend par {pages}.
    let mut mise = MiseEnPage::default();
    mise.entete_pied.pied.centre = "page {page} sur {pages}".to_string();

    let sortie = generer(&mise);
    assert!(
        sortie.contains("counter(page).final()"),
        "le total s'y trouve : {sortie}"
    );
}

    /// **Un document ne compile jamais mal.** La preuve se donne au vrai
    /// compilateur : le gabarit, pied et en-tête rédigés, forme grecque et
    /// tout, se compile. C'est la règle absolue du rapport 2.
    #[test]
    fn un_document_ne_compile_jamais_mal() {
        if std::process::Command::new("typst").arg("--version").output().is_err() {
            println!("typst n'est pas installé : test sauté");
            return;
        }
        let mut mise = MiseEnPage::default();
        mise.entete_pied.entete.gauche = "Ma thèse".to_string();
        mise.entete_pied.pied.droite = "p. {page}".to_string();
        mise.entete_pied.pied.centre = "page {page} sur {pages}".to_string();
        mise.entete_pied.numerotation = "α".to_string();

        let gabarit = generer(&mise);
        let dossier = std::env::temp_dir().join("typst-writer-epreuve-compile");
        let _ = std::fs::create_dir_all(&dossier);
        let chemin = dossier.join("document.typ");

        let document = format!("{gabarit}\nUn texte d'épreuve.\n");
        std::fs::write(&chemin, &document).expect("le document s'écrit");

        let sortie = std::process::Command::new("typst")
            .args(["compile", chemin.to_str().unwrap()])
            .output()
            .expect("typst s'exécute");

        assert!(
            sortie.status.success(),
            "le document compile toujours : {}",
            String::from_utf8_lossy(&sortie.stderr)
        );
    }
