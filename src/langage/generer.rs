//! Le générateur : Document structuré → Typst.
//!
//! C'est la traduction qui alimente le moteur de rendu existant
//! (`crate::typst::render`), qu'on réutilise tel quel. Le résultat est le
//! **corps d'un chapitre** : l'assemblage (`main.typ`) garde l'en-tête de mise
//! en page et la bibliographie, et `#include` ce texte. Un chapitre n'a donc
//! pas à réimporter les normes — le faire les appliquerait deux fois.
//!
//! Le Typst reste le maître du rendu final : tout ce que le langage ne sait
//! pas exprimer passe par un bloc brut, inséré tel quel.
//!
//! Le texte est **échappé** : une astérisque dans une phrase reste une
//! astérisque, elle n'ouvre pas de gras. C'est la contrepartie du texte nu que
//! le sérialiseur produit.

use super::schema::{Bloc, Document, Inline, Reference};
use super::typographie::{Regles, appliquer};

/// Génère le corps Typst d'un document, sans règle de langue.
#[allow(dead_code)] // Raccourci des tests et de la migration.
pub fn generer(document: &Document) -> String {
    generer_avec(document, &Regles::aucune())
}

/// Génère le corps Typst d'un document, en appliquant les règles de
/// typographie des normes.
///
/// Les littéraux (`\\x` dans la syntaxe) ne reçoivent **aucune** règle :
/// c'est la porte de sortie universelle, promise par la grammaire.
pub fn generer_avec(document: &Document, regles: &Regles) -> String {
    let mut out = String::new();

    for bloc in &document.blocs {
        generer_bloc(bloc, &mut out, regles);
    }

    out
}

/// Génère un bloc, suivi d'une ligne vide — la séparation de blocs de Typst.
fn generer_bloc(bloc: &Bloc, out: &mut String, regles: &Regles) {
    match bloc {
        Bloc::Titre { niveau, contenu } => {
            out.push_str(&"=".repeat(*niveau as usize));
            out.push(' ');
            generer_inline(contenu, out, false, regles);
            out.push_str("\n\n");
        }
        Bloc::Paragraphe(contenu) => {
            generer_inline(contenu, out, true, regles);
            out.push_str("\n\n");
        }
        Bloc::Citation(contenu) => {
            out.push_str("#quote(block: true)[");
            generer_inline(contenu, out, false, regles);
            out.push_str("]\n\n");
        }
        Bloc::ListePuce(items) => {
            for item in items {
                out.push_str("- ");
                generer_inline(item, out, false, regles);
                out.push('\n');
            }

            out.push('\n');
        }
        Bloc::ListeNumero(items) => {
            // Typst numérote avec `+`, et non avec un chiffre : il suit l'ordre
            // autant que l'usure des listes réordonnées.
            for item in items {
                out.push_str("+ ");
                generer_inline(item, out, false, regles);
                out.push('\n');
            }

            out.push('\n');
        }
        Bloc::Brut(texte) => {
            out.push_str(texte);
            out.push_str("\n\n");
        }
    }
}

/// Génère une suite d'éléments inline.
///
/// `debut_de_ligne` dit que le premier caractère écrit tombe en tête de ligne :
/// une `-`, un `+`, un `=` ou un `/` y ouvrirait une liste ou un titre, et doit
/// donc être échappé.
fn generer_inline(
    inline: &[Inline],
    out: &mut String,
    debut_de_ligne: bool,
    regles: &Regles,
) {
    let mut en_tete = debut_de_ligne;

    for element in inline {
        match element {
            Inline::Texte(texte) => {
                // La typographie de la langue d'abord, l'échappement Typst
                // ensuite — et l'insécable devient `~`. Un tilde tapé, lui,
                // a été échappé : il reste un tilde.
                let compose = appliquer(texte, regles);
                let mut echappe = String::new();
                echapper_texte(&compose, &mut echappe, en_tete);
                out.push_str(&echappe.replace('\u{A0}', "~"));
                en_tete = false;
            }
            Inline::Litteral(texte) => {
                // Un littéral est la promesse de la grammaire : aucune
                // interprétation, ni syntaxique, ni typographique.
                echapper_texte(texte, out, en_tete);
                en_tete = false;
            }
            Inline::Gras(contenu) => {
                out.push('*');
                generer_inline(contenu, out, false, regles);
                out.push('*');
                en_tete = false;
            }
            Inline::Italique(contenu) => {
                out.push('_');
                generer_inline(contenu, out, false, regles);
                out.push('_');
                en_tete = false;
            }
            Inline::GrasItalique(contenu) => {
                out.push_str("*_");
                generer_inline(contenu, out, false, regles);
                out.push_str("_*");
                en_tete = false;
            }
            Inline::Code(texte) => {
                out.push('`');
                out.push_str(texte);
                out.push('`');
                en_tete = false;
            }
            Inline::Couleur { couleur, contenu } => {
                out.push_str("#text(fill: ");
                out.push_str(couleur);
                out.push_str(")[");
                generer_inline(contenu, out, false, regles);
                out.push(']');
                en_tete = false;
            }
            Inline::Citation(references) => {
                generer_citation(references, out);
                en_tete = false;
            }
            Inline::Note(contenu) => {
                out.push_str("#footnote[");
                generer_inline(contenu, out, false, regles);
                out.push(']');
                en_tete = false;
            }
            Inline::Lien { texte, cible } => {
                out.push_str("#link(\"");
                echapper_chaine(cible, out);
                out.push_str("\")[");
                generer_inline(texte, out, false, regles);
                out.push(']');
                en_tete = false;
            }
        }
    }
}

/// Génère un appel de citation.
///
/// Une seule référence s'écrit `@clé[p. 45]` : elle rend elle-même sa note.
/// Plusieurs références partagent **une** note, et c'est là que tout se joue :
/// le style de notes regroupe les appels `#cite` **qui se suivent, séparés par
/// un espace** — un point-virgule entre eux serait avalé —, alors que des
/// `@clé` juxtaposés donneraient autant de notes que de références. Mesuré sur
/// le compilateur : c'est toute la raison de cette forme.
pub fn generer_citation(references: &[Reference], out: &mut String) {
    if references.len() == 1 {
        let reference = &references[0];
        out.push('@');
        out.push_str(&reference.cle);

        if let Some(page) = &reference.page {
            out.push('[');
            echapper_texte(page, out, false);
            out.push(']');
        }

        return;
    }

    for (index, reference) in references.iter().enumerate() {
        if index > 0 {
            out.push(' ');
        }

        out.push_str("#cite(<");
        out.push_str(&reference.cle);
        out.push('>');

        if let Some(page) = &reference.page {
            out.push_str(", supplement: [");
            echapper_texte(page, out, false);
            out.push(']');
        }

        out.push(')');
    }
}

/// Écrit du texte en échappant ce que Typst lirait comme du balisage.
fn echapper_texte(texte: &str, out: &mut String, debut_de_ligne: bool) {
    for (index, caractere) in texte.chars().enumerate() {
        let echappe = match caractere {
            '\\' | '#' | '*' | '_' | '`' | '$' | '@' | '^' | '<' | '>' | '[' | ']' | '~' => true,
            // En tête de ligne seulement : ailleurs, ces signes sont du texte.
            '-' | '+' | '=' | '/' => index == 0 && debut_de_ligne,
            _ => false,
        };

        if echappe {
            out.push('\\');
        }

        out.push(caractere);
    }
}

/// Écrit une cible de lien dans une chaîne Typst.
///
/// Une chaîne n'a que deux escapements — la barre oblique et le guillemet —,
/// mais les deux y sont obligés : une cible qui en porterait un romprait la
/// chaîne, et le document ne compilerait plus.
fn echapper_chaine(texte: &str, out: &mut String) {
    for caractere in texte.chars() {
        if matches!(caractere, '\\' | '"') {
            out.push('\\');
        }

        out.push(caractere);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::langage::parse::parse;
    use std::path::PathBuf;
    use typst::foundations::NativeElement;
    use typst::introspection::Introspector;

    /// Compile un texte généré par le vrai moteur Typst.
    ///
    /// C'est la seule preuve qui vaille pour un générateur : le texte peut être
    /// bien formé et refusé, ou l'inverse. On compile l'assemblage minimal —
    /// un `#include` du chapitre — avec le texte en mémoire.
    fn compiler(typst: &str) -> crate::typst::render::CompileOutcome {
        let root = PathBuf::from("/projet");
        let entry = "#include \"chapitre.typ\"\n".to_string();
        let overrides = vec![(root.join("chapitre.typ"), typst.to_string())];

        crate::typst::render::compile(&root, &entry, &overrides)
    }

    /// Le document généré compile sans erreur.
    #[test]
    fn le_texte_genere_compile() {
        let texte = "# Titre\n\n## Sous-titre\n\nUn paragraphe avec du **gras**, de l'*italique*, du `code`, du ***gras italique***, un [lien](https://exemple.fr) et du {red:texte rouge} et une note ^[une précision].\n\n> Une citation de bloc.\n\n- une puce\n- une autre\n\n1. un élément\n2. un autre\n\n```typst\n#let repère = 1\n```\n";

        let document = parse(texte);
        let outcome = compiler(&generer(&document));

        assert!(
            outcome.errors.is_empty(),
            "le Typst généré doit compiler : {:?}",
            outcome
                .errors
                .iter()
                .map(|erreur| erreur.message.clone())
                .collect::<Vec<_>>()
        );
    }

/// L'échappement Typst, éprouvé **par contexte**.
    ///
    /// Chaque contexte a ses pièges : le code est pris au mot, la cible d'un
    /// lien est une chaîne, une couleur est une expression, une note est un
    /// bloc de contenu, un bloc brut est du Typst nu. Le compilateur est le
    /// seul juge — et la forme écrite se relit telle quelle.
    #[test]
    fn l_echappement_est_eprouve_par_contexte() {
        // Le code est littéral : ses signes Typst ne sont pas du balisage,
        // et un crochet y est un crochet.
        let document = Document {
            blocs: vec![Bloc::Paragraphe(vec![Inline::Code("#f(x) * y]".to_string())])],
        };
        let typst = generer(&document);
        assert!(typst.contains("`#f(x) * y]`"), "{typst}");
        assert!(compiler(&typst).errors.is_empty(), "le code doit compiler");

        // La cible d'un lien est une **chaîne** : le guillemet et la barre
        // oblique y sont échappés, sans quoi la chaîne rompt.
        let document = Document {
            blocs: vec![Bloc::Paragraphe(vec![Inline::Lien {
                texte: vec![Inline::Texte("lien".to_string())],
                cible: "https://x.y/a\"b\\c".to_string(),
            }])],
        };
        let typst = generer(&document);
        assert!(typst.contains("#link(\"https://x.y/a\\\"b\\\\c\")"), "{typst}");
        assert!(compiler(&typst).errors.is_empty(), "la chaîne doit tenir");

        // Une couleur est une expression Typst ; son contenu est du texte
        // échappé — ses signes ne sont pas du balisage.
        let document = Document {
            blocs: vec![Bloc::Paragraphe(vec![Inline::Couleur {
                couleur: "red".to_string(),
                contenu: vec![Inline::Texte("2 * 3 et #x et ] ici".to_string())],
            }])],
        };
        let typst = generer(&document);
        assert!(compiler(&typst).errors.is_empty(), "la couleur doit compiler");

        // Une note est un bloc de contenu : son texte s'échappe, et un code
        // qui y porte un crochet ne le referme pas.
        let document = Document {
            blocs: vec![Bloc::Paragraphe(vec![Inline::Note(vec![
                Inline::Texte("2 * 3 et ".to_string()),
                Inline::Code("a]b".to_string()),
                Inline::Texte(" ici".to_string()),
            ])])],
        };
        let typst = generer(&document);
        assert!(compiler(&typst).errors.is_empty(), "la note doit compiler");

        // Un bloc brut est du Typst nu : pris au mot, sans échappement.
        let document = Document {
            blocs: vec![Bloc::Brut("#let mystère = 2\n#let suite = 3".to_string())],
        };
        let typst = generer(&document);
        assert!(typst.starts_with("#let mystère = 2"), "{typst}");
        assert!(compiler(&typst).errors.is_empty(), "le brut doit compiler");
    }

    /// Le texte qui ressemble à du balisage ne devient pas du balisage.
    ///
    /// Sans échappement, « 2 * 3 » ouvrirait un italique jamais fermé, et
    /// « # » en tête de paragraphe ferait une erreur de compilation.
    #[test]
    fn un_texte_qui_ressemble_a_un_marqueur_reste_du_texte() {
        let texte = "Un calcul : 2 * 3 * 4 = 24, et un # en tête.\n\n# Un titre\n";

        let document = parse(texte);
        let outcome = compiler(&generer(&document));

        assert!(
            outcome.errors.is_empty(),
            "le texte nu doit être échappé : {:?}",
            outcome
                .errors
                .iter()
                .map(|erreur| erreur.message.clone())
                .collect::<Vec<_>>()
        );

        assert_eq!(
            outcome
                .document
                .as_deref()
                .map(crate::typst::render::page_count),
            Some(1)
        );
    }

    /// Les blocs deviennent le Typst attendu.
    #[test]
    fn les_blocs_deviennent_le_typst_attendu() {
        let document = parse(
            "# Titre\n\nUn **gras** et un *italique*.\n\n> Une citation.\n\n- une puce\n\n1. un\n",
        );

        let typst = generer(&document);

        assert!(typst.contains("= Titre\n\n"), "{typst}");
        assert!(typst.contains("*gras*"), "{typst}");
        assert!(typst.contains("_italique_"), "{typst}");
        assert!(
            typst.contains("#quote(block: true)[Une citation.]"),
            "{typst}"
        );
        assert!(typst.contains("- une puce"), "{typst}");
        assert!(typst.contains("+ un\n"), "{typst}");
    }

    /// Une citation bibliographique se génère en référence Typst.
    #[test]
    fn une_citation_bibliographique_devient_une_reference() {
        let document = parse("Voir @dupont2023.\n");

        assert!(generer(&document).contains("@dupont2023"));
    }

    /// Le corps généré n'importe pas l'en-tête : l'assemblage s'en charge, et
    /// l'importer deux fois appliquerait les normes deux fois.
    #[test]
    fn le_corps_genere_n_importe_pas_l_entete() {
        let typst = generer(&parse("Bonjour.\n"));

        assert!(!typst.contains("#import"), "{typst}");
        assert!(!typst.contains("#show"), "{typst}");
        assert!(!typst.contains("#bibliography"), "{typst}");
    }

    /// Un appel groupé s'écrit en appels `#cite` qui se suivent, séparés par
    /// un espace — la seule forme que le style de notes réunit en une note.
    #[test]
    fn un_appel_groupe_genere_les_appels_qui_se_suivent() {
        let typst = generer(&parse("Voir @dupont2023[p. 45]; @martin2020.\n"));

        assert!(
            typst.contains("#cite(<dupont2023>, supplement: [p. 45]) #cite(<martin2020>)"),
            "{typst}"
        );
    }

    /// Un appel groupé rend **une** note de bas de page qui les réunit.
    ///
    /// C'est la preuve décisive, lue sur le rendu : deux `@clé` juxtaposés
    /// donneraient deux notes, et une note explicite imbriquerait les notes
    /// dans les notes. Le texte composé sur la page dit ce que le lecteur
    /// verra — les deux références, et la page, dans une seule note.
    #[test]
    fn un_appel_groupe_devient_une_seule_note() {
        let root = std::env::temp_dir().join(format!(
            "typst-writer-groupe-note-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("racine du projet");
        std::fs::write(root.join("references.bib"), BIB).expect("bibliographie d'essai");

        let entree = "#bibliography(\"references.bib\", style: \"chicago-notes\")\n#include \"chapitre.typ\"\n";
        let genere = generer(&parse("Voir @dupont2023[p. 45]; @martin2020.\n"));
        let outcome =
            crate::typst::render::compile(&root, entree, &[(root.join("chapitre.typ"), genere)]);

        assert!(
            outcome.errors.is_empty(),
            "le groupe doit compiler : {:?}",
            outcome
                .errors
                .iter()
                .map(|erreur| erreur.message.clone())
                .collect::<Vec<_>>()
        );

        let document = outcome.document.as_deref().expect("document compilé");
        let notes = document
            .introspector()
            .query(&typst::model::FootnoteElem::ELEM.select());

        assert_eq!(notes.len(), 1, "un seul appel groupé = une seule note");

        let page = composed_text(document);

        assert!(page.contains("p. 45"), "la page se lit dans la note : {page}");
        assert!(
            page.contains("Dupont") && page.contains("Martin"),
            "les deux références s'y retrouvent : {page}"
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    /// Une seule référence avec sa page rend une seule note, avec sa page —
    /// la forme courte `@clé[p. 45]`, sans les appels groupés.
    #[test]
    fn une_reference_avec_sa_page_rend_sa_note() {
        let root = std::env::temp_dir().join(format!(
            "typst-writer-note-page-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("racine du projet");
        std::fs::write(root.join("references.bib"), BIB).expect("bibliographie d'essai");

        let entree = "#bibliography(\"references.bib\", style: \"chicago-notes\")\n#include \"chapitre.typ\"\n";
        let genere = generer(&parse("Voir @dupont2023[p. 45].\n"));
        let outcome =
            crate::typst::render::compile(&root, entree, &[(root.join("chapitre.typ"), genere)]);

        assert!(outcome.errors.is_empty(), "la page doit compiler");

        let document = outcome.document.as_deref().expect("document compilé");
        let notes = document
            .introspector()
            .query(&typst::model::FootnoteElem::ELEM.select());

        assert_eq!(notes.len(), 1, "une référence = une note");

        let page = composed_text(document);

        assert!(page.contains("p. 45"), "la page se lit dans la note : {page}");

        let _ = std::fs::remove_dir_all(&root);
    }

    /// La bibliographie d'essai des deux tests ci-dessus.
    const BIB: &str = "@article{dupont2023,
      title = {Un ouvrage de référence},
      author = {Dupont, Alice},
      year = {2023},
    }
    @book{martin2020,
      title = {Un autre ouvrage},
      author = {Martin, Bernard},
      year = {2020},
    }
    ";

    /// Le texte réellement composé sur les pages d'un document.
    ///
    /// Descend dans les groupes : le texte d'une page est rangé dans des
    /// sous-cadres, et s'arrêter au premier niveau ne lirait rien.
    fn composed_text(document: &typst_layout::PagedDocument) -> String {
        fn frame_text(frame: &typst::layout::Frame, out: &mut String) {
            for (_, item) in frame.items() {
                match item {
                    typst::layout::FrameItem::Text(text) => out.push_str(&text.text),
                    typst::layout::FrameItem::Group(group) => frame_text(&group.frame, out),
                    _ => {}
                }
            }
        }

        let mut texte = String::new();

        for page in document.pages() {
            frame_text(&page.frame, &mut texte);
        }

        texte
    }
}
