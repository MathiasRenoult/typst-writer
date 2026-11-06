//! Les styles de bibliographie, tels que le moteur embarqué les connaît.
//!
//! La règle de R4 : « ne pas maintenir à la main une liste qui diverge de
//! cette version ». La liste ci-dessous est une **vitrine** — chaque nom en
//! est validé par le moteur (`ArchivedStyle::by_name`) au moment de
//! l'énumération, et un nom refusé par le moteur n'est jamais appliqué :
//! la divergence est impossible, au pire la vitrine se rétrécit.

use hayagriva::archive::ArchivedStyle;

/// Le type d'un style, tel qu'on l'annonce au choix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeStyle {
    /// Les références en notes de bas de page.
    Notes,
    /// Les références en auteur-date dans le texte.
    AuteurDate,
    /// Les références numérotées dans le texte.
    Numerique,
}

/// La vitrine des styles intégrés : `(nom canonique, type, libellé)`.
///
/// Le libellé n'est que de la présentation ; le nom canonique est celui
/// qu'écrit la directive, et il est **validé par le moteur**.
pub fn styles_integres() -> Vec<(String, TypeStyle, &'static str)> {
    const VITRINE: &[(&str, TypeStyle, &str)] = &[
        ("chicago-notes", TypeStyle::Notes, "Chicago (notes)"),
        (
            "chicago-author-date",
            TypeStyle::AuteurDate,
            "Chicago (auteur-date)",
        ),
        (
            "modern-language-association",
            TypeStyle::Notes,
            "MLA (notes)",
        ),
        ("apa", TypeStyle::AuteurDate, "APA (auteur-date)"),
        ("ieee", TypeStyle::Numerique, "IEEE (numérique)"),
        (
            "gb-7714-2015-numeric",
            TypeStyle::Numerique,
            "GB/T 7714 (numérique)",
        ),
        ("alphanumeric", TypeStyle::Numerique, "Alphanumérique"),
    ];

    VITRINE
        .iter()
        .filter(|(nom, _, _)| style_valide(nom))
        .map(|(nom, type_style, libelle)| ((*nom).to_string(), *type_style, *libelle))
        .collect()
}

/// Un nom de style que le moteur embarqué accepte-t-il ?
///
/// C'est le moteur qui tranche — jamais une liste à la main. Un nom qui
/// porte une extension (« styles/mon-style.csl ») est un **fichier** : le
/// moteur le charge à la compilation, et son contrôle se fait à l'import.
pub fn style_valide(nom: &str) -> bool {
    std::path::Path::new(nom)
        .extension()
        .is_some_and(|extension| extension == "csl")
        || ArchivedStyle::by_name(nom).is_some()
}

/// Un style CSL lu sur le disque, après contrôle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StyleCsl {
    /// Accepté par le moteur tel quel.
    Independant {
        /// Son titre, tel qu'il se présentera.
        titre: String,
    },

    /// Un style **dépendant** : le moteur le refuse — il lui faut son
    /// parent, et ce n'est jamais un silence (R4).
    Dependant {
        /// Son titre.
        titre: String,
        /// Le parent qu'il réclame, s'il le nomme.
        parent: Option<String>,
    },
}

/// Contrôle un fichier CSL comme le moteur le ferait : même analyseur
/// (`citationberg`), mêmes limites.
///
/// Entrée non fiable : le texte est déjà borné en taille par l'appelant, le
/// XML n'est qu'analysé — jamais exécuté — et toute erreur se dit en mots.
pub fn controler_csl(texte: &str) -> Result<StyleCsl, String> {
    let titre = |style: &citationberg::IndependentStyle| style.info.title.value.clone();

    match citationberg::Style::from_xml(texte) {
        Ok(citationberg::Style::Independent(style)) => Ok(StyleCsl::Independant {
            titre: titre(&style),
        }),
        Ok(citationberg::Style::Dependent(style)) => Ok(StyleCsl::Dependant {
            titre: style.info.title.value.clone(),
            parent: Some(style.parent_link.href.clone()).filter(|url| !url.is_empty()),
        }),
        Err(erreur) => Err(format!("Fichier CSL illisible : {erreur}")),
    }
}

/// L'aperçu d'un style : un petit jeu représentatif, rendu par le moteur
/// **même** — jamais une imitation.
///
/// Une notice, une citation avec localisateur, et la bibliographie : de
/// quoi voir comment le style parle. Le texte se lit tel qu'il sort du
/// moteur ; `None` dit que le moteur refuse le style — l'aperçu ne ment
/// jamais (R4).
pub fn apercu_du_style(projet: &std::path::Path, style: &str) -> Option<String> {
    let entree = format!(
        "#set page(paper: \"a6\", margin: 8pt)\n\
         #bibliography(\"references.bib\", style: \"{style}\")\n\n\
         Selon @dupont2023[p. 45], la thèse est posée. Voir aussi @martin2020, \
         et @lecointe2021.\n"
    );
    // L'échantillon exerce ce que la fidélité veut voir (bug-report-8,
    // T7.1) : plusieurs auteurs (ordre et « et al. »), un sous-titre, des
    // dates complètes, des entrées pour les séparateurs.
    let notices = "@book{dupont2023, title = {La théologie du lien}, \
                   author = {Dupont, Jean and Martin, Marie and Lecointe, Paul}, \
                   year = {2023}, month = {7}}\n\
                   @article{martin2020, title = {L'écriture: geste et sens}, \
                   author = {Martin, Marie}, year = {2020}, journal = {Revue des sciences}}\n\
                   @book{lecointe2021, title = {Le lien}, subtitle = {Essai}, \
                   author = {Lecointe, Paul}, year = {2021}, month = {3}}\n";

    // Le jeu est self-contained : sa bibliographie remplace le fichier du
    // projet par un jeu de rôle, sans jamais y toucher.
    let overrides = [(projet.join("references.bib"), notices.to_string())];
    let outcome = crate::typst::render::compile(projet, &entree, &overrides);
    let document = outcome.document.as_deref()?;

    let mut texte = String::new();

    for page in document.pages() {
        extraire_texte(&page.frame, &mut texte);
        texte.push('\n');
    }

    Some(texte.trim_end().to_string())
}

/// Le texte visible d'un cadre, ses groupes dépliés.
/// Le texte d'un cadre, **ses styles compris** : le gras et l'italique
/// se marquent — sans quoi l'aperçu d'un style CSL cacherait exactement
/// ce que la fidélité veut voir (bug-report-8, T7.1).
pub(crate) fn extraire_texte(frame: &typst::layout::Frame, sortie: &mut String) {
    for (_, item) in frame.items() {
        match item {
            typst::layout::FrameItem::Text(text) => {
                let variante = text.font.font().info().variant;
                let italique = variante.style != typst::text::FontStyle::Normal;
                let gras = variante.weight >= typst::text::FontWeight::BOLD;

                if gras {
                    sortie.push('*');
                }
                if italique {
                    sortie.push('_');
                }
                sortie.push_str(&text.text);
                if italique {
                    sortie.push('_');
                }
                if gras {
                    sortie.push('*');
                }
            }
            typst::layout::FrameItem::Group(groupe) => extraire_texte(&groupe.frame, sortie),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Le contrôle dit l'indépendant, le dépendant et l'invalide — avec
    /// les mêmes mots que le moteur (R4).
    #[test]
    fn le_controle_csl_dit_independant_dependant_et_invalide() {
        let independant = r#"<?xml version="1.0" encoding="utf-8"?>
<style xmlns="http://purl.org/net/xbiblio/csl" version="1.0" class="note">
  <info>
    <id>https://example.org/styles/test</id>
    <title>Style de test</title>
    <updated>2026-01-01T00:00:00+00:00</updated>
  </info>
  <citation><layout><text variable="title"/></layout></citation>
  <bibliography><layout><text variable="title"/></layout></bibliography>
</style>"#;
        assert_eq!(
            controler_csl(independant),
            Ok(StyleCsl::Independant {
                titre: "Style de test".to_string()
            })
        );

        let dependant = r#"<?xml version="1.0" encoding="utf-8"?>
<style xmlns="http://purl.org/net/xbiblio/csl" version="1.0" class="in-text">
  <info>
    <id>https://example.org/styles/dependent</id>
    <title>Style dépendant</title>
    <updated>2026-01-01T00:00:00+00:00</updated>
    <link href="https://example.org/styles/parent" rel="independent-parent"/>
  </info>
</style>"#;
        assert_eq!(
            controler_csl(dependant),
            Ok(StyleCsl::Dependant {
                titre: "Style dépendant".to_string(),
                parent: Some("https://example.org/styles/parent".to_string()),
            })
        );

        assert!(controler_csl("ce n'est pas du XML").is_err());
    }

    /// Les styles de fichier passent la validation de nom, les autres non.
    #[test]
    fn un_nom_avec_extension_est_un_fichier() {
        assert!(style_valide("styles/mon-style.csl"));
        assert!(style_valide("apa"));
        assert!(!style_valide("un-style-qui-nexiste-pas"));
    }

    /// L'aperçu est rendu par le moteur même : le texte porte la source
    /// citée, et chaque famille a sa voix (R4).
    #[test]
    fn l_apercu_est_rendu_par_le_moteur() {
        let root =
            std::env::temp_dir().join(format!("typst-writer-apercu-style-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("racine du projet");

        let notes = apercu_du_style(&root, "chicago-notes").expect("l'aperçu se produit");
        assert!(
            notes.to_lowercase().contains("dupont"),
            "la source citée s'y lit : {notes}"
        );

        let auteur_date = apercu_du_style(&root, "apa").expect("l'aperçu se produit");
        let numerique = apercu_du_style(&root, "ieee").expect("l'aperçu se produit");
        assert_ne!(notes, auteur_date, "notes et auteur-date se distinguent");
        assert_ne!(notes, numerique, "notes et numérique se distinguent");
        assert_ne!(
            auteur_date, numerique,
            "auteur-date et numérique se distinguent"
        );

        // Un style refusé par le moteur : pas d'aperçu qui ment.
        assert_eq!(apercu_du_style(&root, "un-style-qui-nexiste-pas"), None);
    }

    /// L'aperçu montre les **styles** du rendu — le gras et l'italique
    /// s'y lisent, comme dans le document (bug-report-8, T7.1).
    #[test]
    fn l_apercu_montre_le_gras_et_l_italique() {
        let root =
            std::env::temp_dir().join(format!("typst-writer-apercu-styles-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("racine du projet");

        let entree = "#set page(paper: \"a6\", margin: 8pt)\n*gras* _italique_ ordinaire\n";
        let outcome = crate::typst::render::compile(&root, entree, &[]);
        let document = outcome.document.as_deref().expect("le document se compose");

        let mut texte = String::new();
        for page in document.pages() {
            extraire_texte(&page.frame, &mut texte);
        }

        assert!(texte.contains("*gras*"), "le gras se marque : {texte}");
        assert!(
            texte.contains("_italique_"),
            "l'italique se marque : {texte}"
        );
        assert!(
            texte.contains("ordinaire"),
            "le texte ordinaire passe : {texte}"
        );
    }

    /// La vitrine ne diverge jamais du moteur embarqué (R4).
    #[test]
    fn les_styles_proposes_sont_ceux_du_moteur() {
        let styles = styles_integres();
        assert!(!styles.is_empty(), "la vitrine n'est pas vide");

        for (nom, _, _) in &styles {
            assert!(
                style_valide(nom),
                "« {nom} » est accepté par le moteur embarqué"
            );
        }

        // Les trois familles de l'acceptation y sont.
        for nom in ["apa", "ieee", "chicago-notes"] {
            assert!(
                styles.iter().any(|(candidat, _, _)| candidat == nom),
                "« {nom} » est proposé"
            );
        }

        // Et un nom inconnu du moteur est refusé, pas deviné.
        assert!(!style_valide("un-style-qui-nexiste-pas"));
    }
}
