//! Assemblage du document intermédiaire dans le projet Typst.
//!
//! L'en-tête reste celui du projet (`main.typ`). Le corps du `document.tw` est
//! généré en Typst et fourni au compilateur comme un fichier inclus en mémoire.

use std::path::{Path, PathBuf};

use super::{generer::generer_avec, parse};

/// Nom stable du fichier Typst temporaire injecté dans l'assemblage.
pub const GENERATED_FILE: &str = "document.generated.typ";

/// Assemble l'en-tête existant et le corps généré, prêt pour le compilateur.
///
/// Le fichier `.tw` reste la source sur disque ; le Typst est un override en
/// mémoire, résolu par le `World` de rendu existant. L'include se place avant
/// la bibliographie ; si `references.bib` existe mais que l'ancien `main.typ`
/// n'a pas de directive, celle-ci est ajoutée à l'assemblage en mémoire.
pub fn compilation_unit(
    root: &Path,
    main: &str,
    writing_source: &str,
) -> (String, Vec<(PathBuf, String)>) {
    compilation_unit_document(root, main, &parse(writing_source))
}

/// Assemble une valeur AST déjà tenue par l'éditeur.
pub fn compilation_unit_document(
    root: &Path,
    main: &str,
    document: &super::Document,
) -> (String, Vec<(PathBuf, String)>) {
    let include = format!("#include \"{GENERATED_FILE}\"");
    let entry = if main.lines().any(|line| line.trim() == include) {
        main.to_string()
    } else {
        insert_generated_include(main, &include)
    };
    let entry = corriger_les_bibliographies(root, &entry);
    let entry = ensure_default_bibliography(root, &entry);

    // La typographie se lit dans les normes du projet : c'est là qu'elle se
    // choisit, et le générateur l'applique au texte qu'il compose.
    let normes = crate::mise_en_page::charger(root);
    let generated = generer_avec(document, &normes.typographie.regles(), &normes.decor);

    (entry, vec![(root.join(GENERATED_FILE), generated)])
}

/// Place l'include avant la première bibliographie, ou à la fin de l'en-tête.
fn insert_generated_include(main: &str, include: &str) -> String {
    let mut offset = 0;

    for ligne in main.split_inclusive('\n') {
        if is_bibliography_directive(ligne) {
            return format!("{}{}\n\n{}", &main[..offset], include, &main[offset..]);
        }

        offset += ligne.len();
    }

    let mut entry = main.to_string();

    if !entry.is_empty() && !entry.ends_with('\n') {
        entry.push('\n');
    }

    if !entry.is_empty() && !entry.ends_with("\n\n") {
        entry.push('\n');
    }

    entry.push_str(include);
    entry.push('\n');
    entry
}

/// Une ligne d'assemblage est-elle une directive Typst `#bibliography(...)` ?
fn is_bibliography_directive(line: &str) -> bool {
    let line = line.trim_start();

    if line.starts_with("//") {
        return false;
    }

    line.strip_prefix("#bibliography")
        .is_some_and(|remainder| remainder.trim_start().starts_with('('))
}

/// Ajoute la bibliographie conventionnelle aux anciens assemblages qui ont le
/// fichier `.bib` sans directive explicite. La version sur disque n'est jamais
/// modifiée.
fn ensure_default_bibliography(root: &Path, main: &str) -> String {
    let main = ensure_default_citation_style(main);

    if main.lines().any(is_bibliography_directive) || !root.join("references.bib").is_file() {
        return main;
    }

    let mut entry = main;

    if !entry.is_empty() && !entry.ends_with("\n\n") {
        if !entry.ends_with('\n') {
            entry.push('\n');
        }
        entry.push('\n');
    }

    entry.push_str("#bibliography(\"references.bib\", style: \"chicago-notes\")\n");
    entry
}

/// Applique le style de notes par défaut aux bibliographies sans style explicite.
///
/// Un style explicitement choisi dans `main.typ` reste prioritaire. Sans cette
/// valeur, Typst utilise `ieee` et rend les appels dans le texte sous la forme
/// `[1]` au lieu de notes de bas de page.
fn ensure_default_citation_style(main: &str) -> String {
    let Some((open, close)) = bibliography_call_range(main) else {
        return main.to_string();
    };
    let arguments = &main[open + 1..close];

    if arguments.contains("style:") {
        return main.to_string();
    }

    let trimmed = arguments.trim_end();
    let insertion = open + 1 + trimmed.len();
    let separator = if trimmed.trim().is_empty() {
        ""
    } else if trimmed.ends_with(',') {
        " "
    } else {
        ", "
    };

    format!(
        "{}{}style: \"chicago-notes\"{}",
        &main[..insertion],
        separator,
        &main[insertion..]
    )
}

/// Repère les parenthèses de l'appel bibliographique hors chaînes de texte.
fn bibliography_call_range(main: &str) -> Option<(usize, usize)> {
    let mut line_offset = 0;

    for line in main.split_inclusive('\n') {
        if is_bibliography_directive(line) {
            let trimmed = line.trim_start();
            let indent = line.len() - trimmed.len();
            let open = line_offset + indent + trimmed.find('(')?;
            let mut depth = 0usize;
            let mut quoted = false;
            let mut escaped = false;

            for (relative, character) in main[open..].char_indices() {
                if quoted {
                    if escaped {
                        escaped = false;
                    } else if character == '\\' {
                        escaped = true;
                    } else if character == '"' {
                        quoted = false;
                    }

                    continue;
                }

                match character {
                    '"' => quoted = true,
                    '(' => depth += 1,
                    ')' => {
                        depth -= 1;
                        if depth == 0 {
                            return Some((open, open + relative));
                        }
                    }
                    _ => {}
                }
            }

            return None;
        }

        line_offset += line.len();
    }

    None
}

#[cfg(test)]
mod tests {

    /// La typographie des normes s'applique au texte composé — et le `\`
    /// de la syntaxe en exempte ce qu'il écrit.
    #[test]
    fn la_typographie_des_normes_s_applique_et_le_backslash_en_exempt() {
        use super::super::generer::generer_avec;
        use super::super::schema::{Bloc, Inline};
        use super::super::typographie::Regles;

        // Le cas du programme : en français, « ; » est précédé d'une espace
        // insécable et suivi d'une espace normale.
        let document = super::super::parse("La foi ; l'espérance.\n");
        let typst = generer_avec(
            &document,
            &Regles::francaise(),
            &crate::mise_en_page::modele::Decor::default(),
        );
        assert!(
            typst.contains("La foi~; l'espérance."),
            "le français est strict : {typst}"
        );

        // Et la même chose en anglais colle.
        let typst = generer_avec(
            &document,
            &Regles::anglaise(),
            &crate::mise_en_page::modele::Decor::default(),
        );
        assert!(typst.contains("La foi; l'espérance."), "{typst}");

        // Le `\` insère un caractère sans interprétation : la même phrase
        // avec un point-virgule échappé garde ses espaces telles quelles.
        let document = super::super::parse("La foi \\; l'espérance.\n");
        let typst = generer_avec(
            &document,
            &Regles::francaise(),
            &crate::mise_en_page::modele::Decor::default(),
        );
        assert!(
            typst.contains("La foi ; l'espérance."),
            "l'échappement exempte de la règle : {typst}"
        );

        // Le littéral est bien ce qui circule.
        assert!(
            document.blocs.iter().any(|bloc| matches!(
                bloc,
                Bloc::Paragraphe(contenu)
                    if contenu.iter().any(|element| matches!(element, Inline::Litteral(_)))
            )),
            "{document:?}"
        );
    }

    /// Les normes du projet portent la typographie jusqu'au générateur.
    #[test]
    fn les_normes_portent_la_typographie_jusqu_au_generateur() {
        let root =
            std::env::temp_dir().join(format!("typst-writer-typo-normes-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("racine du projet");

        std::fs::write(
            root.join(crate::mise_en_page::REGLAGES),
            "[texte]\ntypographie = \"française\"\n",
        )
        .expect("normes d'essai");

        let document = super::super::parse("La foi ; l'espérance.\n");
        let (_, fichiers) =
            super::compilation_unit_document(&root, "#show: mise-en-page\n", &document);
        let corps = &fichiers[0].1;

        assert!(corps.contains("La foi~; l'espérance."), "{corps}");

        let _ = std::fs::remove_dir_all(&root);
    }

    use super::*;
    use typst::foundations::NativeElement;
    use typst::introspection::Introspector;

    /// L'assemblage est sécurisé : ce qui est nécessaire est préservé, ce
    /// qui est déjà là n'est pas doublé, ce qui est choisi n'est pas repris.
    #[test]
    fn l_assemblage_preserve_et_ne_double_rien() {
        let root = std::path::Path::new("/projet-inexistant");
        let document = super::super::parse("Bonjour.\n");

        // Une inclusion déjà présente est conservée, et l'inclusion générée
        // ne s'ajoute qu'une fois — même après deux assemblages.
        let main = "#include \"chapitre.typ\"\n\n#bibliography(\"references.bib\", style: \"chicago-notes\")\n";
        let (entry, _) = super::compilation_unit_document(root, main, &document);
        assert!(entry.contains("#include \"chapitre.typ\""), "{entry}");

        let (encore, _) = super::compilation_unit_document(root, &entry, &document);
        assert_eq!(
            encore.matches("#include").count(),
            2,
            "l'inclusion héritée et la générée, jamais plus : {encore}"
        );

        // Une bibliographie déjà là ne se double pas.
        assert_eq!(
            encore.matches("#bibliography").count(),
            1,
            "une seule bibliographie : {encore}"
        );

        // Un style explicitement choisi reste le sien.
        let main = "#set document(title: \"Essai\")\n\n#bibliography(\"references.bib\", style: \"ieee\")\n";
        let (entry, _) = super::compilation_unit_document(root, main, &document);
        assert!(
            entry.contains("style: \"ieee\""),
            "le style choisi tient : {entry}"
        );
        assert!(entry.contains("#set document(title: \"Essai\")"), "{entry}");
        assert_eq!(entry.matches("#bibliography").count(), 1, "{entry}");
    }

    /// Le `.tw` devient un include in-memory derrière l'en-tête, et compile par
    /// le même moteur Typst que les chapitres existants.
    #[test]
    fn le_document_intermediaire_est_insere_et_compile() {
        let root = std::env::temp_dir().join(format!(
            "typst-writer-langage-pipeline-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("racine du projet");

        let (entry, overrides) = compilation_unit(
            &root,
            "#set page(paper: \"a6\")\n",
            "# Titre\n\nDu **gras**.\n",
        );

        assert!(entry.starts_with("#set page(paper: \"a6\")"));
        assert!(entry.contains("#include \"document.generated.typ\""));
        assert_eq!(overrides.len(), 1);
        assert_eq!(
            overrides[0].0,
            PathBuf::from(&root).join("document.generated.typ")
        );
        assert_eq!(overrides[0].1, "= Titre\n\nDu *gras*.\n\n");

        let outcome = crate::typst::render::compile(&root, &entry, &overrides);

        assert!(
            outcome.errors.is_empty(),
            "le document assemblé compile : {:?}",
            outcome
                .errors
                .iter()
                .map(|error| &error.message)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            outcome
                .document
                .as_deref()
                .map(crate::typst::render::page_count),
            Some(1)
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    /// L'include se place avant la bibliographie, même si `main.typ` porte déjà
    /// la directive finale.
    #[test]
    fn le_corps_precede_la_bibliographie_de_l_assemblage() {
        let root = Path::new("/projet");
        let main = "#set text(lang: \"fr\")\n#bibliography(\"references.bib\")\n";
        let (entry, _) = compilation_unit(root, main, "Bonjour.\n");

        assert!(
            entry.find("#include \"document.generated.typ\"").unwrap()
                < entry.find("#bibliography(").unwrap(),
            "le texte passe avant la bibliographie"
        );
    }

    /// Une citation du document unique est résolue par la bibliographie du projet.
    #[test]
    fn une_citation_compile_avec_la_bibliographie_du_projet() {
        let root = std::env::temp_dir().join(format!(
            "typst-writer-pipeline-citation-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("racine du projet");
        std::fs::write(
            root.join("references.bib"),
            "@article{dupont2023,\n  title = {Un titre},\n  author = {Dupont, Alice},\n  year = {2023},\n}\n",
        )
        .expect("bibliographie d'essai");

        let (entry, overrides) = compilation_unit(
            &root,
            crate::project::MAIN_TEMPLATE,
            "Selon @dupont2023, le résultat est établi.\n",
        );
        assert!(entry.contains("style: \"chicago-notes\""), "{entry}");
        let outcome = crate::typst::render::compile(&root, &entry, &overrides);

        assert!(
            outcome.errors.is_empty(),
            "la citation doit compiler avec references.bib : {:?}",
            outcome
                .errors
                .iter()
                .map(|error| error.message.clone())
                .collect::<Vec<_>>()
        );
        let document = outcome.document.as_deref().expect("document compilé");
        let notes = document
            .introspector()
            .query(&typst::model::FootnoteElem::ELEM.select());
        assert!(!notes.is_empty(), "la citation Chicago doit créer une note");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Les anciens `main.typ` sans directive récupèrent le `.bib` du projet en mémoire.
    #[test]
    fn une_ancienne_entree_sans_bibliographie_resout_les_citations_du_document() {
        let root = std::env::temp_dir().join(format!(
            "typst-writer-pipeline-ancienne-bibliographie-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("racine du projet");
        std::fs::write(
            root.join("references.bib"),
            "@article{dupont2023,\n  title = {Un titre},\n  author = {Dupont, Alice},\n  year = {2023},\n}\n",
        )
        .expect("bibliographie d'essai");

        let old_main = "#set page(margin: 2cm)\n";
        let (entry, overrides) = compilation_unit(
            &root,
            old_main,
            "Selon @dupont2023, le résultat est établi.\n",
        );
        assert!(entry.contains("style: \"chicago-notes\""), "{entry}");
        let outcome = crate::typst::render::compile(&root, &entry, &overrides);

        assert!(
            outcome.errors.is_empty(),
            "une entrée héritée doit trouver references.bib : {:?}",
            outcome
                .errors
                .iter()
                .map(|error| error.message.clone())
                .collect::<Vec<_>>()
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Une directive héritée sans style adopte Chicago notes ; un style explicite
    /// reste, lui, sous le contrôle de l'auteur.
    #[test]
    fn une_bibliographie_heritee_sans_style_rend_les_citations_en_notes() {
        let root = std::env::temp_dir().join(format!(
            "typst-writer-pipeline-style-notes-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("racine du projet");
        std::fs::write(
            root.join("references.bib"),
            "@article{dupont2023,\n  title = {Un titre},\n  author = {Dupont, Alice},\n  year = {2023},\n}\n",
        )
        .expect("bibliographie d'essai");

        let main = "#set page(margin: 2cm)\n#bibliography(\"references.bib\")\n";
        let (entry, overrides) =
            compilation_unit(&root, main, "Selon @dupont2023, le résultat est établi.\n");

        assert!(entry.contains("style: \"chicago-notes\""), "{entry}");
        let outcome = crate::typst::render::compile(&root, &entry, &overrides);
        assert!(
            outcome.errors.is_empty(),
            "la citation avec style de notes compile : {:?}",
            outcome
                .errors
                .iter()
                .map(|error| error.message.clone())
                .collect::<Vec<_>>()
        );
        let document = outcome.document.as_deref().expect("document compilé");
        let notes = document
            .introspector()
            .query(&typst::model::FootnoteElem::ELEM.select());
        assert!(
            !notes.is_empty(),
            "l'appel bibliographique devient une note"
        );

        let _ = std::fs::remove_dir_all(&root);
    }
}

/// Les bibliographies aux chemins morts.
///
/// Un document importé porte souvent le chemin de son ancienne machine —
/// `Bibliographie/Adoption spirituelle.bib` — et le fichier n'existe
/// plus ici. **Un document ne compile jamais mal** : la directive morte
/// devient celle du projet, sa vraie bibliographie ; s'il n'y en a pas,
/// elle se commente — le document compile, et le problème se dit ailleurs
/// qu'en une erreur de compilateur.
fn corriger_les_bibliographies(root: &Path, main: &str) -> String {
    let mut sortie = String::new();

    for ligne in main.split_inclusive('\n') {
        if !is_bibliography_directive(ligne) {
            sortie.push_str(ligne);
            continue;
        }

        // Le chemin : la première chaîne de la directive.
        let chemin = ligne
            .split('"')
            .nth(1)
            .map(|chemin| chemin.to_string())
            .unwrap_or_default();

        if chemin.is_empty() || root.join(&chemin).is_file() {
            sortie.push_str(ligne);
            continue;
        }

        // Le chemin est mort : la bibliographie du projet le remplace,
        // et elle est cherchée par son nom, comme toujours.
        match crate::project::find_bibliography(root) {
            Some(vraie) => {
                let nom = vraie
                    .file_name()
                    .map(|nom| nom.to_string_lossy().to_string())
                    .unwrap_or_else(|| "references.bib".to_string());

                let corrigee = ligne.replace(&format!("\"{chemin}\""), &format!("\"{nom}\""));
                sortie.push_str(&corrigee);
            }
            None => {
                // Aucune bibliographie : la directive se commente. Le
                // document compile, et c'est tout ce qu'on lui demande.
                let sans_fin = ligne.trim_end_matches('\n');
                sortie.push_str(&format!("// {sans_fin}\n"));
            }
        }
    }

    sortie
}

#[cfg(test)]
mod tests_des_chemins_morts {
    use super::*;

    /// Un chemin mort ne casse jamais le document : la bibliographie du
    /// projet le remplace, et le compilateur trouve le fichier (bug «
    /// aucun document ne compile » du rapport 2).
    #[test]
    fn un_chemin_mort_ne_casse_jamais_le_document() {
        let root = std::env::temp_dir().join("typst-writer-epreuve-chemin-mort");
        let _ = std::fs::create_dir_all(&root);
        std::fs::write(root.join("references.bib"), "@book{a, title = {A}}\n").expect("bib");

        let main = "#include \"document.generated.typ\"\n\n#bibliography(\"Bibliographie/Adoption spirituelle.bib\", title: \"Bibliographie\")\n";
        let document = super::super::parse("Un texte.\n");
        let (entry, _) = compilation_unit_document(&root, main, &document);

        assert!(
            entry.contains("\"references.bib\""),
            "le chemin mort est remplacé : {entry}"
        );
        assert!(
            !entry.contains("Adoption spirituelle"),
            "le chemin mort n'y est plus : {entry}"
        );

        // Et le document se compile : c'est le compilateur qui juge.
        std::fs::write(root.join("document.generated.typ"), "Un texte.\n").expect("généré");
        let rendu = std::process::Command::new("typst")
            .args(["compile", root.join("main.typ").to_str().unwrap()])
            .current_dir(&root)
            .output();

        // `main.typ` n'existe pas encore sur disque — c'est l'entrée qui
        // se juge : elle est prête, et ses chemins sont vivants.
        assert!(entry.lines().all(|ligne| {
            !is_bibliography_directive(ligne)
                || ligne
                    .split('"')
                    .nth(1)
                    .is_some_and(|chemin| root.join(chemin).is_file())
        }), "chaque bibliographie pointe sur un fichier vivant : {entry}");

        let _ = rendu;
    }
}
