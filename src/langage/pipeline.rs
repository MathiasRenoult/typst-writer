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
    // Les clés vivantes : celles des bibliographies du projet. Une
    // citation inconnue ne casse jamais le document — elle se montre
    // telle quelle, et le problème se dit ailleurs qu'en une erreur de
    // compilateur.
    let cles = cles_vivantes(root, &entry);

    let entry = corriger_le_typst(root, &entry, &cles);
    let mut corrections = corriger_les_inclus(root, &entry, &cles);
    let entry = ensure_default_bibliography(root, &entry);

    // La typographie se lit dans les normes du projet : c'est là qu'elle se
    // choisit, et le générateur l'applique au texte qu'il compose.
    let normes = crate::mise_en_page::charger(root);
    let mut regles = normes.typographie.regles();
    regles.numerotation_figures = normes.numerotation_figures.clone();
    let generated = generer_avec(document, &regles, &normes.decor);

    // La guérison des `.bib` court enfin : la bibliographie
    // conventionnelle existe déjà, et ses chemins sont vivants.
    corrections.extend(guerir_les_bibliographies(root, &entry, &corrections));

    // Le généré aussi : le document source peut porter sa propre
    // directive, et elle y meurt comme ailleurs.
    let generated = corriger_le_typst(root, &generated, &cles);

    corrections.push((root.join(GENERATED_FILE), generated));

    (entry, corrections)
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
fn corriger_le_typst(
    root: &Path,
    main: &str,
    cles: &std::collections::BTreeSet<String>,
) -> String {
    // `#cite(<cle>)` d'abord, sur le bloc entier : la clé peut couvrir
    // plusieurs lignes, et une citation inconnue ne casse jamais le
    // document. On construit la sortie en balayant — rien n'est masqué,
    // rien n'est déplacé.
    let mut main_corrige = String::new();
    let mut curseur = 0usize;

    while let Some(position) = main[curseur..].find("#cite(<") {
        let position = curseur + position;
        let debut = position + "#cite(<".len();

        let Some(fin_label) = main[debut..].find(">") else {
            break;
        };

        let cle = main[debut..debut + fin_label].to_string();

        // L'appel entier : `#cite(<cle>, style: "…")` — la parenthèse
        // fermante le ferme, quoi qu'il y ait entre les deux.
        let apres = debut + fin_label + 1;
        let fin = match main[apres..].find(')') {
            Some(fermeture) => apres + fermeture + 1,
            None => apres,
        };

        main_corrige.push_str(&main[curseur..position]);

        if cles.contains(&cle) {
            main_corrige.push_str(&main[position..fin]);
        } else {
            main_corrige.push_str(&format!("[{cle}]"));
        }

        curseur = fin;
    }

    main_corrige.push_str(&main[curseur..]);
    let main = main_corrige;

    let mut sortie = String::new();

    for ligne in main.split_inclusive('\n') {
        if is_bibliography_directive(ligne) {
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

            // Le chemin est mort : la bibliographie du projet le
            // remplace, trouvée par son nom, comme toujours.
            match crate::project::find_bibliography(root) {
                Some(vraie) => {
                    let nom = vraie
                        .file_name()
                        .map(|nom| nom.to_string_lossy().to_string())
                        .unwrap_or_else(|| "references.bib".to_string());

                    sortie.push_str(&ligne.replace(&format!("\"{chemin}\""), &format!("\"{nom}\"")));
                }
                None => {
                    // Aucune bibliographie : la directive se commente.
                    let sans_fin = ligne.trim_end_matches('\n');
                    sortie.push_str(&format!("// {sans_fin}\n"));
                }
            }

            continue;
        }

        // Les citations inconnues : `@cle` et `#cite(<cle>)` deviennent
        // `[cle]` — un texte, pas une référence. Une adresse
        // électronique a un point, et n'est jamais une clé.
        let mut ligne_sortie = String::new();
        let octets = ligne.as_bytes();
        let mut curseur = 0usize;

        while curseur < octets.len() {
            if octets[curseur] == b'@' {
                let mut fin_cle = curseur + 1;

                while fin_cle < octets.len()
                    && (octets[fin_cle].is_ascii_alphanumeric() || octets[fin_cle] == b'_' || octets[fin_cle] == b'-')
                {
                    fin_cle += 1;
                }

                let cle = &ligne[curseur + 1..fin_cle];

                if !cle.is_empty() && !cle.contains('.') && !cles.contains(cle) {
                    ligne_sortie.push_str(&format!("[{cle}]"));
                    curseur = fin_cle;
                    continue;
                }
            }

            let caractere = ligne[curseur..].chars().next().unwrap_or('\n');
            ligne_sortie.push(caractere);
            curseur += caractere.len_utf8();
        }

        sortie.push_str(&ligne_sortie);
    }

    sortie
}

/// Les bibliographies mortes des fichiers **inclus**.
///
/// Un chapitre porte parfois sa propre directive — et le chemin mort y
/// est relatif à **sa** place, pas à celle de l'assemblage. La
/// substitution se fait au compilateur : le fichier sur disque n'est
/// jamais touché, et la bibliographie du projet est trouvée par son nom.
fn corriger_les_inclus(
    root: &Path,
    entry: &str,
    cles: &std::collections::BTreeSet<String>,
) -> Vec<(PathBuf, String)> {
    let mut corrections = Vec::new();
    let mut vus = std::collections::HashSet::new();
    let mut a_voir: Vec<PathBuf> = crate::project::model::read_includes(&root.join("main.typ"));

    // L'entrée peut elle-même en porter.
    for ligne in entry.lines() {
        if let Some(reste) = ligne.trim().strip_prefix("#include")
            && let Some(chemin) = reste.trim().trim_matches('"').lines().next()
        {
            a_voir.push(PathBuf::from(chemin));
        }
    }

    while let Some(relatif) = a_voir.pop() {
        if !vus.insert(relatif.clone()) {
            continue;
        }

        let chemin = root.join(&relatif);
        let Ok(contenu) = std::fs::read_to_string(&chemin) else {
            continue;
        };

        // Les inclusions du chapitre, pour que la récursion suive.
        for sous in crate::project::model::read_includes(&chemin) {
            a_voir.push(relatif.parent().unwrap_or(Path::new("")).join(sous));
        }

        let base = chemin.parent().unwrap_or(root).to_path_buf();
        let corrige = corriger_le_typst(&base, &contenu, cles);

        if corrige != contenu {
            corrections.push((chemin, corrige));
        }
    }

    corrections
}

/// Les `.bib` aux commentaires `//` — BibLaTeX ne connaît que `%`.
///
/// Notre propre modèle écrivait des `//` : chaque projet créé avant la
/// correction porte un fichier que BibLaTeX refuse tout entier. La
/// guérison se fait au compilateur, par substitution — le fichier sur
/// disque n'est pas touché, et le document compile.
/// Les commentaires `//` d'un `.bib` deviennent `%` — BibLaTeX ne
/// connaît que ceux-ci.
fn guerir_commentaires(contenu: &str) -> String {
    contenu
        .lines()
        .map(|ligne| {
            let nu = ligne.trim_start();

            if nu.starts_with("//") {
                format!("{}%{}", &ligne[..ligne.len() - nu.len()], nu.strip_prefix("//").unwrap_or(nu))
            } else {
                ligne.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn guerir_les_bibliographies(
    root: &Path,
    entry: &str,
    corrections: &[(PathBuf, String)],
) -> Vec<(PathBuf, String)> {
    // Les bibliographies citées, avec le dossier de celui qui les cite :
    // un chemin est toujours relatif à **son** fichier.
    let mut citees = Vec::new();

    let sources = std::iter::once((root.to_path_buf(), entry.to_string())).chain(
        corrections
            .iter()
            .filter(|(chemin, _)| chemin.extension().is_some_and(|ext| ext == "typ"))
            .map(|(chemin, contenu)| {
                (
                    chemin.parent().unwrap_or(root).to_path_buf(),
                    contenu.clone(),
                )
            }),
    );

    for (base, source) in sources {
        for ligne in source.lines() {
            if !is_bibliography_directive(ligne) {
                continue;
            }

            if let Some(chemin) = ligne.split('"').nth(1) {
                citees.push(base.join(chemin));
            }
        }
    }

    let mut guerisons = Vec::new();

    for chemin in citees {
        let Ok(contenu) = std::fs::read_to_string(&chemin) else {
            continue;
        };

        let gueri = guerir_commentaires(&contenu);

        if gueri != contenu {
            guerisons.push((chemin, gueri));
        }
    }

    guerisons
}

/// Les `.bib` aux commentaires `//` — BibLaTeX ne connaît que `%`.
///
/// Notre propre modèle écrivait des `//` : chaque projet créé avant la
/// correction porte un fichier que BibLaTeX refuse tout entier. La
/// guérison se fait au compilateur, par substitution — le fichier sur
/// disque n'est pas touché, et le document compile.
fn cles_vivantes(root: &Path, entry: &str) -> std::collections::BTreeSet<String> {
    let mut cles = std::collections::BTreeSet::new();

    // Toutes les bibliographies citées, et la conventionnelle.
    let mut citees = vec![crate::project::find_bibliography(root)];

    for ligne in entry.lines() {
        if is_bibliography_directive(ligne)
            && let Some(chemin) = ligne.split('"').nth(1)
        {
            citees.push(Some(root.join(chemin)));
        }
    }

    for chemin in citees.into_iter().flatten() {
        if let Ok(contenu) = std::fs::read_to_string(&chemin) {
            // Le contenu guéri d'abord : un `.bib` aux commentaires `//`
            // fait perdre ses clés au parseur, et une clé perdue devient
            // une citation neutralisée à tort.
            let gueri = guerir_commentaires(&contenu);

            for entree in crate::project::parse_bibliography_text(&gueri) {
                cles.insert(entree.key);
            }
        }
    }

    cles
}

    /// **Un document ne compile jamais mal, quoi qu'il arrive** — les
    /// trois fautes réelles du rapport, ensemble, jugées par le vrai
    /// compilateur : le chemin mort d'un chapitre, le `.bib` aux
    /// commentaires `//`, et les citations inconnues sous leurs deux
    /// formes.
    #[test]
    fn un_document_ne_compile_jamais_mal_quoi_qu_il_arrive() {
        if std::process::Command::new("typst").arg("--version").output().is_err() {
            println!("typst n'est pas installé : test sauté");
            return;
        }
        let root = std::env::temp_dir().join("typst-writer-epreuve-trois-fautes");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("chapitres")).expect("dossier");

        // La bibliographie du projet, aux commentaires `//` — notre
        // propre modèle d'avant la correction.
        std::fs::write(
            root.join("references.bib"),
            "// Une entrée par référence, au format BibTeX.\n//\n@book{vivante, title = {Vivante}}\n",
        )
        .expect("bib");

        // Le chapitre : le chemin mort, et les deux formes de citation
        // inconnue — l'une couvrant plusieurs lignes.
        std::fs::write(
            root.join("chapitres/1.typ"),
            "Un texte @grudemInconnu.\n\n#cite(<inconnueMultiLignes>,\n  style: \"chicago-notes\")\n\n#cite(<vivante>)\n",
        )
        .expect("chapitre");

        std::fs::write(
            root.join("main.typ"),
            "#include \"chapitres/1.typ\"\n\n#bibliography(\"Bibliographie/Adoption spirituelle.bib\")\n",
        )
        .expect("main");

        let document = parse("Un texte.\n");
        let (entry, corrections) =
            compilation_unit_document(&root, &std::fs::read_to_string(root.join("main.typ")).expect("main"), &document);

        // Les substitutions sont prêtes, et le disque n'a pas bougé.
        assert!(
            std::fs::read_to_string(root.join("chapitres/1.typ"))
                .expect("lu")
                .contains("@grudemInconnu"),
            "le fichier sur disque n'est jamais touché"
        );

        // Tout se pose, et le compilateur juge.
        std::fs::write(root.join("main.typ"), &entry).expect("entrée");
        std::fs::write(root.join(GENERATED_FILE), "Un texte.\n").expect("généré");

        for (chemin, contenu) in &corrections {
            if chemin != &root.join(GENERATED_FILE) {
                std::fs::write(chemin, contenu).expect("correction");
            }
        }

        let rendu = std::process::Command::new("typst")
            .args(["compile", "main.typ"])
            .current_dir(&root)
            .output()
            .expect("typst s'exécute");

        assert!(
            rendu.status.success(),
            "le document compile toujours : {}",
            String::from_utf8_lossy(&rendu.stderr)
        );
    }

#[cfg(test)]
mod tests_du_pipeline_vrai {
    /// Le compilateur `typst` est-il sur le PATH ? Sans lui, ces tests
    /// n'ont rien à juger : ils sautent au lieu de faillir.
    fn typst_disponible() -> bool {
        std::process::Command::new("typst")
            .arg("--version")
            .output()
            .is_ok()
    }

    /// Les vrais projets de l'utilisateur sont-ils là ? En CI, les fichiers
    /// n'existent pas : le test saute.
    fn projets_disponibles() -> bool {
        std::path::Path::new("/home/mat/.typst-writer/projets/mon-travail/document.tw").is_file()
    }

    /// Le pipeline complet sur les vrais projets anciens : l'entrée, les
    /// substitutions, et le compilateur réel.
    #[test]
    fn les_anciens_projets_compilent() {
        if !typst_disponible() {
            println!("typst n'est pas installé : test sauté");
            return;
        }
        if !projets_disponibles() {
            println!("les projets réels ne sont pas là : test sauté");
            return;
        }
        for nom in ["mon-travail", "projet", "projet-2"] {
            let source = std::path::PathBuf::from("/home/mat/.typst-writer/projets").join(nom);
            let root = std::env::temp_dir().join(format!("tw-pipeline-{nom}"));
            let _ = std::fs::remove_dir_all(&root);
            copy_dir(&source, &root);

            let main = std::fs::read_to_string(root.join("main.typ")).unwrap_or_default();
            let document = super::super::parse(&std::fs::read_to_string(root.join("document.tw")).unwrap_or_default());
            let (entry, corrections) = super::compilation_unit_document(&root, &main, &document);

            std::fs::write(root.join("main.typ"), &entry).expect("entrée");
            for (chemin, contenu) in &corrections {
                std::fs::write(chemin, contenu).expect("correction");
            }

            let rendu = std::process::Command::new("typst")
                .args(["compile", "main.typ"])
                .current_dir(&root)
                .output()
                .expect("typst");

            println!("{nom} : {}", if rendu.status.success() { "OK" } else { "ÉCHEC" });

            if !rendu.status.success() {
                println!("{}", String::from_utf8_lossy(&rendu.stderr));
            }

            assert!(rendu.status.success(), "{nom} doit compiler");
        }
    }

    fn copy_dir(source: &std::path::Path, cible: &std::path::Path) {
        std::fs::create_dir_all(cible).expect("dossier");
        for entree in std::fs::read_dir(source).expect("lu") {
            let entree = entree.expect("entrée");
            let chemin = entree.path();
            let dest = cible.join(entree.file_name());
            if chemin.is_dir() {
                copy_dir(&chemin, &dest);
            } else {
                std::fs::copy(&chemin, &dest).expect("copie");
            }
        }
    }
}
