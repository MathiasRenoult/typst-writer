//! Assemblage du document intermédiaire dans le projet Typst.
//!
//! L'en-tête reste celui du projet (`main.typ`). Le corps du `document.tw` est
//! généré en Typst et fourni au compilateur comme un fichier inclus en mémoire.

use std::path::{Path, PathBuf};

use super::{generer::generer_avec, parse};

/// Nom stable du fichier Typst temporaire injecté dans l'assemblage.
pub const GENERATED_FILE: &str = "document.generated.typ";

/// Nom stable du fichier des pages de garde.
pub const GARDE_FILE: &str = "document.garde.typ";

/// Assemble l'en-tête existant et le corps généré, prêt pour le compilateur.
///
/// Le fichier `.tw` reste la source sur disque ; le Typst est un override en
/// mémoire, résolu par le `World` de rendu existant. La section bibliographique
/// n'est jamais ajoutée par l'assemblage : elle doit être demandée par
/// `{{bib}}` dans le texte. Les directives restées dans `main.typ` sont
/// ignorées dans la variante stricte, sans modifier le fichier.
/// Sans les diagnostics : le raccourci des tests qui n'en ont pas besoin.
#[cfg(test)]
pub fn compilation_unit(
    root: &Path,
    main: &str,
    writing_source: &str,
) -> (String, Vec<(PathBuf, String)>) {
    let (entree, corrections, _) = compilation_unit_avec_diagnostics(root, main, writing_source);
    (entree, corrections)
}

/// Comme [`compilation_unit`], mais dit aussi les citations neutralisées.
///
/// Une citation inconnue devient `[cle]` — un texte, jamais une fausse
/// référence. Ce texte la voit, mais l'auteur aussi doit la **savoir** :
/// chaque neutralisation revient ici avec sa clé, sa ligne et son fichier,
/// pour devenir un avertissement au lieu d'un silence.
#[allow(dead_code)] // Raccourci des épreuves : l'application passe par la variante stricte.
pub fn compilation_unit_avec_diagnostics(
    root: &Path,
    main: &str,
    writing_source: &str,
) -> (
    String,
    Vec<(PathBuf, String)>,
    Vec<crate::langage::diagnostics::Diagnostic>,
) {
    compilation_unit_document_avec_diagnostics(root, main, &parse(writing_source))
}

/// La variante stricte pour un document d'écriture : seule une balise
/// `{{bib}}` du texte peut demander la section bibliographique.
pub fn compilation_unit_avec_diagnostics_explicite(
    root: &Path,
    main: &str,
    writing_source: &str,
) -> (
    String,
    Vec<(PathBuf, String)>,
    Vec<crate::langage::diagnostics::Diagnostic>,
) {
    compilation_unit_document_avec_diagnostics_explicite(root, main, &parse(writing_source))
}

/// Assemble une valeur AST déjà tenue par l'éditeur.
/// Sans les diagnostics : le raccourci des tests qui n'en ont pas besoin.
#[cfg(test)]
pub fn compilation_unit_document(
    root: &Path,
    main: &str,
    document: &super::Document,
) -> (String, Vec<(PathBuf, String)>) {
    let (entree, corrections, _) = compilation_unit_document_avec_diagnostics(root, main, document);
    (entree, corrections)
}

/// Le cœur de l'assemblage, avec la liste des citations neutralisées.
#[allow(dead_code)] // Raccourci des épreuves : l'application passe par la variante stricte.
pub fn compilation_unit_document_avec_diagnostics(
    root: &Path,
    main: &str,
    document: &super::Document,
) -> (
    String,
    Vec<(PathBuf, String)>,
    Vec<crate::langage::diagnostics::Diagnostic>,
) {
    compilation_unit_document_interne(root, main, document, false)
}

/// La compilation d'un document `.tw` : la bibliographie n'existe que si le
/// texte la demande explicitement par `{{bib}}` (ou `{bibliographie}`).
///
/// Les directives `#bibliography(…)` restées dans `main.typ` — ancien
/// assemblage, modèle d'une version précédente — ne sont pas une demande de
/// l'auteur dans le texte : elles sont écartées de l'assemblage compilé. Le
/// fichier sur disque n'est jamais modifié.
pub fn compilation_unit_document_avec_diagnostics_explicite(
    root: &Path,
    main: &str,
    document: &super::Document,
) -> (
    String,
    Vec<(PathBuf, String)>,
    Vec<crate::langage::diagnostics::Diagnostic>,
) {
    compilation_unit_document_interne(root, main, document, true)
}

fn compilation_unit_document_interne(
    root: &Path,
    main: &str,
    document: &super::Document,
    bibliographie_explicite: bool,
) -> (
    String,
    Vec<(PathBuf, String)>,
    Vec<crate::langage::diagnostics::Diagnostic>,
) {
    let include = format!("#include \"{GENERATED_FILE}\"");
    let normes = crate::mise_en_page::charger(root);
    let demande_explicite = document_demande_la_bibliographie(document);
    let assemblee = if bibliographie_explicite {
        if demande_explicite {
            retirer_les_bibliographies(main)
        } else {
            cacher_les_bibliographies(main, root, normes.bibliographie.style.as_deref())
        }
    } else {
        main.to_string()
    };
    let entry = if assemblee.lines().any(|line| line.trim() == include) {
        assemblee
    } else {
        insert_generated_include(&assemblee, &include)
    };
    // Les clés vivantes : celles des bibliographies du projet. Une
    // citation inconnue ne casse jamais le document — elle se montre
    // telle quelle, et le problème se dit ailleurs qu'en une erreur de
    // compilateur.
    let mut rapports = Vec::new();
    let (cles, etiquettes, collisions) = cles_vivantes(root, &entry);
    rapports.extend(collisions);

    // Les clés vivantes du glossaire : une référence à un terme inconnu
    // ne casse jamais le document — elle se neutralise, et se dit.
    let glossaire = crate::project::glossaire::Glossaire::charger(root);
    let cles_glossaire: std::collections::BTreeSet<String> =
        glossaire.entrees.iter().map(|e| e.key.clone()).collect();

    let entry = corriger_le_typst(
        root,
        &entry,
        &cles,
        &cles_glossaire,
        &etiquettes,
        &root.join("main.typ"),
        &mut rapports,
    );
    let mut corrections = corriger_les_inclus(
        root,
        &entry,
        &cles,
        &cles_glossaire,
        &etiquettes,
        &mut rapports,
    );

    // La typographie et le style se lisent dans les normes du projet : c'est
    // là qu'ils se choisissent, et le générateur les applique.
    let entry = appliquer_le_style_du_projet(
        &entry,
        normes.bibliographie.style.as_deref(),
        &root.join("main.typ"),
        &mut rapports,
    );
    let entry = preparer_la_bibliographie(root, &entry, &normes);

    // Un assemblage sans directive n'a pas encore de notices : on les charge
    // derrière une règle de spectacle qui ne réserve aucune place, uniquement
    // pour les citations. La section visible, elle, reste réservée à `{{bib}}`.
    let entry = if bibliographie_explicite
        && !demande_explicite
        && !entry.contains("#bibliography")
        && let Some(cachee) = bibliographie_cachee(root, normes.bibliographie.style.as_deref())
    {
        format!("{cachee}\n{entry}")
    } else {
        entry
    };

    let mut regles = normes.typographie.regles(&normes.references);
    regles.index = normes.index.clone();
    regles.index_personnes = normes.index_personnes.clone();
    regles.personnes = crate::project::personnes::Personnes::charger(root);
    regles.numerotation_figures = normes.numerotation_figures.clone();
    regles.legende_prefixe = if normes.legende_prefixe.trim().is_empty() {
        "Figure".to_string()
    } else {
        normes.legende_prefixe.clone()
    };
    regles.callouts = normes.parametres_plugins.callouts.clone();
    // Les mots de la balise `{date:...}` viennent des normes (2026-11-03).
    regles.dates = crate::langage::dates::Regles {
        siecle: normes.dates.siecle.clone(),
        millenaire: normes.dates.millenaire.clone(),
        epoque_apres: normes.dates.epoque_apres.clone(),
        epoque_avant: normes.dates.epoque_avant.clone(),
        format: normes.dates.format.clone(),
        langue: normes.dates.langue.clone(),
    };
    let generated = generer_avec(document, &regles, &normes.decor);

    // Une page de titre ancienne ne doit pas porter de numéro : le gabarit
    // généré avant cette version est corrigé en mémoire, sans toucher au
    // fichier du projet.
    if normes.page_de_titre
        && let Some(gabarit) = gabarit_du_projet(root, &corrections)
        && let Some(gabarit) = sans_numero_de_la_page_de_titre(&gabarit)
    {
        corrections.push((root.join(crate::mise_en_page::gabarit::FICHIER), gabarit));
    }

    // Une page de titre sans métadonnée n'est pas une page : les anciens
    // gabarits la composaient quand même, et la garde commençait alors par
    // une page blanche.
    if normes.page_de_titre
        && !a_des_metadonnees_de_page_de_titre(root)
        && let Some(gabarit) = gabarit_du_projet(root, &corrections)
        && let Some(gabarit) = retirer_la_page_de_titre_vide(&gabarit)
    {
        corrections.push((root.join(crate::mise_en_page::gabarit::FICHIER), gabarit));
    }

    // Les pages de garde sont hors du corps : elles se composent avant le
    // sommaire, sans numéro, et leurs titres ne descendent pas dans la table
    // des matières. Le gabarit les reçoit par un petit fichier Typst dédié.
    //
    // Un saut de page au début du corps serait doublé par celui qui termine
    // la garde : le lecteur verrait une page blanche entre les deux. Le
    // retrait est en mémoire, comme les corrections de gabarit.
    let generated = if crate::langage::generer::a_des_pages_de_garde(document) {
        sans_le_saut_de_page_initial(&generated)
    } else {
        generated
    };
    let mut generated = generated;
    if crate::langage::generer::a_des_pages_de_garde(document) {
        let mut front =
            crate::langage::generer::generer_front_matter(document, &regles, &normes.decor);
        front = corriger_le_typst(
            root,
            &front,
            &cles,
            &cles_glossaire,
            &etiquettes,
            &root.join(GARDE_FILE),
            &mut rapports,
        );
        // Les imports éventuels (graphiques, callouts) restent au niveau du
        // fichier : ils ne sont pas du contenu de page.
        let (imports, corps) = separer_les_imports(&front);
        let garde = format!(
            "{imports}#let garde-corps = [\n  #set page(numbering: none, header: none, footer: none)\n  #set heading(numbering: none, outlined: false)\n{corps}  #pagebreak()\n  #counter(page).update(1)\n  #counter(heading).update(0)\n  #counter(figure).update(0)\n]\n"
        );
        corrections.push((root.join(GARDE_FILE), garde));

        // Le gabarit du projet est généré par l'application : on lui ajoute
        // l'import et l'appel, sans toucher au fichier sur le disque.
        let gabarit = gabarit_du_projet(root, &corrections);
        match gabarit.as_deref().and_then(brancher_les_pages_de_garde) {
            Some(gabarit) => {
                corrections.push((root.join(crate::mise_en_page::gabarit::FICHIER), gabarit));
            }
            None => {
                // Un gabarit personnalisé peut ne pas ressembler au nôtre.
                // Le contenu ne doit jamais disparaître : il revient au début
                // du corps, quitte à ne pas être détaché du sommaire.
                generated = format!("{front}\n{generated}");
            }
        }
    }

    // La guérison des `.bib` court enfin : la bibliographie
    // conventionnelle existe déjà, et ses chemins sont vivants.
    corrections.extend(guerir_les_bibliographies(root, &entry, &corrections));

    // La section `{bibliographie}` porte sa propre directive : elle se
    // branche aux bibliothèques du projet, comme celle du document
    // (bug-report-9).
    let generated = if generated.lines().any(is_bibliography_directive) {
        crate::project::model::brancher_les_bibliographies(root, &generated).unwrap_or(generated)
    } else {
        generated
    };

    // Le généré aussi : le document source peut porter sa propre
    // directive, et elle y meurt comme ailleurs.
    let generated = corriger_le_typst(
        root,
        &generated,
        &cles,
        &cles_glossaire,
        &etiquettes,
        &root.join(GENERATED_FILE),
        &mut rapports,
    );

    // Le glossaire glossarium : le registre dans le généré, le salon dans
    // l'entrée. Le paquet embarqué se pose localement, sans réseau.
    let (entry, generated) =
        assembler_le_glossaire(root, entry, generated, &glossaire, &mut rapports);

    corrections.push((root.join(GENERATED_FILE), generated));

    (entry, corrections, rapports)
}

/// Le gabarit `mise-en-page.typ` tel que l'assemblage doit le voir :
/// la dernière correction en mémoire, sinon le fichier du projet.
fn gabarit_du_projet(root: &Path, corrections: &[(PathBuf, String)]) -> Option<String> {
    corrections
        .iter()
        .rev()
        .find(|(chemin, _)| chemin == &root.join(crate::mise_en_page::gabarit::FICHIER))
        .map(|(_, contenu)| contenu.clone())
        .or_else(|| std::fs::read_to_string(root.join(crate::mise_en_page::gabarit::FICHIER)).ok())
}

/// Les métadonnées qui donnent un contenu à la page de titre.
fn a_des_metadonnees_de_page_de_titre(root: &Path) -> bool {
    let entrees = crate::project::metadonnees::Metadonnees::charger(root).entrees();
    ["titre", "matiere", "auteur", "date"].iter().any(|cle| {
        entrees
            .iter()
            .any(|(nom, valeur)| nom == cle && !valeur.trim().is_empty())
    })
}

/// Retire la page de titre vide d'un ancien gabarit.
///
/// Le gabarit généré avant cette version composait toujours une page, même
/// quand le projet n'avait aucune métadonnée : elle restait blanche avant la
/// première page de garde. Le retrait est **en mémoire** — le fichier du
/// projet n'est pas modifié — et seulement pour les gabarits anciens : le
/// gabarit neuf porte lui-même sa condition.
fn retirer_la_page_de_titre_vide(gabarit: &str) -> Option<String> {
    let marqueur = "  // La page de titre : les métadonnées du document.\n";
    let debut = gabarit.find(marqueur)?;
    let apres = &gabarit[debut..];

    // Le gabarit neuf contient déjà un `if` : ne pas défaire sa condition.
    if apres.contains("  if tw_titre ") {
        return None;
    }

    let saut = apres.find("  pagebreak()\n")?;
    let apres_saut = &apres[saut + "  pagebreak()\n".len()..];
    let compteur = apres_saut.find("  counter(page).update(1)\n")?;
    let fin = saut + "  pagebreak()\n".len() + compteur + "  counter(page).update(1)\n".len();

    if fin > gabarit.len() - debut {
        return None;
    }

    let mut sortie = String::with_capacity(gabarit.len() - fin);
    sortie.push_str(&gabarit[..debut]);
    sortie.push_str(&gabarit[debut + fin..]);
    Some(sortie)
}

/// Retire le numéro de la page de titre des gabarits générés avant la prise
/// en charge des pages de garde. L'opération est idempotente : un gabarit
/// déjà corrigé ne contient plus le motif.
fn sans_numero_de_la_page_de_titre(gabarit: &str) -> Option<String> {
    let ancien = "page(header: none, footer: none)[";
    let nouveau = "page(numbering: none, header: none, footer: none)[";
    if !gabarit.contains(ancien) {
        return None;
    }

    let mut out = gabarit.replacen(ancien, nouveau, 1);
    let debut = out.find(nouveau)?;
    let apres = &out[debut..];
    let saut = apres.find("  pagebreak()\n")?;
    let insertion = debut + saut + "  pagebreak()\n".len();
    out.insert_str(insertion, "  counter(page).update(1)\n");
    Some(out)
}

/// Retire un `#pagebreak()` au tout début du corps, après ses imports.
///
/// Le corps d'un document qui commence par trois lignes vides commence par un
/// saut de page ; quand une garde le précède, ce saut s'ajoute à celui de la
/// garde et laisse une page blanche. Le corps ne perd que ce saut **initial** :
/// les suivants sont des choix de l'auteur.
fn sans_le_saut_de_page_initial(generated: &str) -> String {
    let mut prefixe = 0;
    for ligne in generated.split_inclusive('\n') {
        let texte = ligne.trim();
        if texte.is_empty() || texte.starts_with("#import ") {
            prefixe += ligne.len();
        } else {
            break;
        }
    }

    let reste = &generated[prefixe..];
    let mut sauts = 0;
    while reste[sauts..].starts_with("#pagebreak()") {
        let fin_ligne = reste[sauts..]
            .find('\n')
            .map_or(reste.len(), |i| sauts + i + 1);
        sauts = fin_ligne;
        while reste[sauts..].starts_with('\n') {
            sauts += 1;
        }
    }

    if sauts == 0 {
        return generated.to_string();
    }
    format!("{}{}", &generated[..prefixe], &reste[sauts..])
}

/// Sépare les imports Typst de tête du contenu de garde.
fn separer_les_imports(front: &str) -> (String, String) {
    let mut imports = String::new();
    let mut corps = String::new();
    let mut en_tete = true;

    for ligne in front.lines() {
        if en_tete && ligne.starts_with("#import ") {
            imports.push_str(ligne);
            imports.push('\n');
        } else if en_tete && ligne.trim().is_empty() {
            // Une ligne vide entre deux imports ne change rien.
        } else {
            en_tete = false;
            corps.push_str(ligne);
            corps.push('\n');
        }
    }

    (imports, corps)
}

/// Branche les pages de garde dans le gabarit généré par l'application.
///
/// Le fichier `mise-en-page.typ` est réécrit à chaque changement de normes :
/// on le corrige **en mémoire**, sans toucher au disque, en ajoutant l'import
/// de `document.garde.typ` et l'appel de `garde-corps` juste avant le sommaire. Un
/// gabarit personnalisé qui ne contient ni `outline()` ni `doc` n'est pas
/// modifié : le contenu est alors rendu au début du corps.
fn brancher_les_pages_de_garde(gabarit: &str) -> Option<String> {
    let import = format!("#import \"{GARDE_FILE}\": garde-corps");

    // Un gabarit peut déjà porter l'import ou l'appel : une version
    // précédente a pu les laisser sur le disque, et le branchement doit
    // rester idempotent. Sans ce nettoyage, chaque compilation ajoutait une
    // garde de plus et une page blanche apparaissait avant le sommaire.
    let mut out = String::with_capacity(gabarit.len() + import.len() + 16);
    out.push_str(&import);
    out.push('\n');
    for ligne in gabarit.lines() {
        let texte = ligne.trim();
        if texte == import || texte == "garde-corps" {
            continue;
        }
        out.push_str(ligne);
        out.push('\n');
    }

    // Chercher la ligne du sommaire, pas la chaîne exacte : un gabarit peut
    // écrire `outline(title: …)` ou `#outline()`.
    if let Some(position) = ligne_dans_le_gabarit(&out, "outline(") {
        let debut = out[..position].rfind('\n').map_or(0, |i| i + 1);
        let indentation = &out[debut..position];
        out.insert_str(debut, &format!("{indentation}garde-corps\n"));
        return Some(out);
    }

    if let Some(position) = ligne_exacte_dans_le_gabarit(&out, "doc") {
        let debut = out[..position].rfind('\n').map_or(0, |i| i + 1);
        let indentation = &out[debut..position];
        out.insert_str(debut, &format!("{indentation}garde-corps\n"));
        return Some(out);
    }

    None
}

/// La position d'une ligne dont le contenu (après indentation) est exactement
/// `texte` — le `doc` du gabarit, pas un mot qui commence par lui.
fn ligne_exacte_dans_le_gabarit(gabarit: &str, texte: &str) -> Option<usize> {
    let mut offset = 0;
    for ligne in gabarit.split_inclusive('\n') {
        if ligne.trim() == texte {
            return Some(offset + ligne.len() - ligne.trim_start().len());
        }
        offset += ligne.len();
    }
    None
}

/// La position d'une ligne dont le début (après indentation) est `debut`.
fn ligne_dans_le_gabarit(gabarit: &str, debut: &str) -> Option<usize> {
    let mut offset = 0;
    for ligne in gabarit.split_inclusive('\n') {
        if ligne.trim_start().starts_with(debut) {
            return Some(offset + ligne.len() - ligne.trim_start().len());
        }
        offset += ligne.len();
    }
    None
}

/// Le document demande-t-il explicitement une section bibliographique ?
fn document_demande_la_bibliographie(document: &super::Document) -> bool {
    fn dans_bloc(bloc: &super::schema::Bloc) -> bool {
        match bloc {
            super::schema::Bloc::Bibliographie => true,
            super::schema::Bloc::Décor { bloc, .. } | super::schema::Bloc::Canonique(bloc) => {
                dans_bloc(bloc)
            }
            super::schema::Bloc::Garde(contenu) => contenu.iter().any(dans_bloc),
            _ => false,
        }
    }

    document.blocs.iter().any(dans_bloc)
}

/// Masque les appels `#bibliography(…)` de l'assemblage sans les supprimer.
///
/// `#show bibliography: it => none` charge les notices et permet aux
/// citations `@clé` de se résoudre, mais ne compose aucune section visible
/// et ne réserve aucune place. C'est ce qu'il faut quand le texte cite des
/// sources sans demander `{{bib}}`.
///
/// Seuls les chemins de bibliographie qui existent vraiment sont repris :
/// une ancienne directive qui pointe vers un fichier disparu ne doit pas
/// refaire naître l'erreur « label does not exist ».
fn cacher_les_bibliographies(main: &str, root: &Path, style: Option<&str>) -> String {
    let mut sortie = String::with_capacity(main.len());
    let mut offset = 0;

    while let Some((ouverture, fermeture)) = bibliography_call_range(&main[offset..]) {
        let ouverture = offset + ouverture;
        let fermeture = offset + fermeture + 1;

        let debut_ligne = main[..ouverture].rfind('\n').map_or(0, |index| index + 1);
        let fin_ligne = main[fermeture..]
            .find('\n')
            .map_or(main.len(), |index| fermeture + index + 1);
        let ligne = main[debut_ligne..fin_ligne]
            .trim_end_matches(['\n', '\r'])
            .trim();

        sortie.push_str(&main[offset..debut_ligne]);
        if let Some(cachee) = bibliographie_cachee_depuis(root, chemins_bibliographie(ligne), style)
        {
            sortie.push_str(&cachee);
        }
        // Une directive sans chemin vivant disparaît : le repli standard
        // ci-dessous chargera les bibliothèques présentes du projet.
        offset = fin_ligne;
    }

    sortie.push_str(&main[offset..]);
    sortie
}

/// Les chemins `.bib` écrits dans une directive, et qui existent sous `root`.
fn chemins_bibliographie(ligne: &str) -> Vec<String> {
    let mut chemins = Vec::new();
    let mut morceaux = ligne.split('"');
    let _ = morceaux.next();
    while let Some(chemin) = morceaux.next() {
        if chemin.ends_with(".bib") && !chemins.iter().any(|deja| deja == chemin) {
            chemins.push(chemin.to_string());
        }
        let _ = morceaux.next();
    }
    chemins
}

/// Construit l'appel caché à partir de chemins déjà validés.
fn bibliographie_cachee_depuis(
    root: &Path,
    chemins: Vec<String>,
    style: Option<&str>,
) -> Option<String> {
    let chemins: Vec<String> = chemins
        .into_iter()
        .filter(|chemin| root.join(chemin).is_file())
        .collect();
    if chemins.is_empty() {
        return None;
    }

    let argument = match chemins.as_slice() {
        [seul] => format!("\"{}\"", echapper_chaine_bibliographie(seul)),
        plusieurs => format!(
            "({})",
            plusieurs
                .iter()
                .map(|nom| format!("\"{}\"", echapper_chaine_bibliographie(nom)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    };
    let style = style
        .filter(|style| crate::typst::styles::style_valide(style))
        .map(|style| format!(", style: \"{}\"", echapper_chaine_bibliographie(style)))
        .unwrap_or_default();

    // `#hide` conserve la place du contenu : une bibliographie cachée y
    // occupait une page entière et laissait une page blanche avant la garde.
    // La règle de spectacle retire l'élément de la composition sans lui
    // donner de place, tout en laissant Typst résoudre les citations.
    Some(format!(
        "#show bibliography: it => none\n#bibliography({argument}{style})\n"
    ))
}

fn echapper_chaine_bibliographie(texte: &str) -> String {
    texte.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Une bibliographie cachée pour les citations qui n'ont pas de directive
/// dans `main.typ` — par exemple un assemblage neuf. Elle ne produit rien à
/// l'écran, mais donne leurs notices aux appels `@clé`.
fn bibliographie_cachee(root: &Path, style: Option<&str>) -> Option<String> {
    let mut noms: Vec<String> = Vec::new();
    for nom in ["references.bib", "references-zotero.bib"] {
        if root.join(nom).is_file() {
            noms.push(nom.to_string());
        }
    }
    if noms.is_empty()
        && let Some(chemin) = crate::project::find_bibliography(root)
        && let Some(nom) = chemin.file_name().and_then(|nom| nom.to_str())
    {
        noms.push(nom.to_string());
    }
    if noms.is_empty() {
        return None;
    }

    bibliographie_cachee_depuis(root, noms, style)
}

/// Retire les appels `#bibliography(…)` de l'assemblage.
///
/// La version sur disque de `main.typ` n'est pas touchée : ce retrait ne vit
/// que dans l'entrée compilée, pour qu'une ancienne directive ne fasse pas
/// apparaître une section que le texte n'a pas demandée.
fn retirer_les_bibliographies(main: &str) -> String {
    let mut sortie = String::with_capacity(main.len());
    let mut offset = 0;

    while let Some((ouverture, fermeture)) = bibliography_call_range(&main[offset..]) {
        let ouverture = offset + ouverture;
        let fermeture = offset + fermeture + 1;

        let debut_ligne = main[..ouverture].rfind('\n').map_or(0, |index| index + 1);
        let fin_ligne = main[fermeture..]
            .find('\n')
            .map_or(main.len(), |index| fermeture + index + 1);

        sortie.push_str(&main[offset..debut_ligne]);
        offset = fin_ligne;
    }

    sortie.push_str(&main[offset..]);
    sortie
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

/// Applique le style du projet aux citations et à la bibliographie — et
/// ne **réécrit jamais** un style qu'une directive impose déjà (R4).
///
/// Les règles de priorité, explicites :
/// - une directive sans style reçoit celui du projet, s'il est connu du
///   moteur ; à défaut, le style de base (« chicago-notes ») ;
/// - une directive qui impose **son** style reste maîtresse : le conflit
///   avec le style du projet se dit, sans rien écraser ;
/// - un style du projet refusé par le moteur n'est jamais appliqué : il
///   se dit, et le style de base prend le relais.
fn appliquer_le_style_du_projet(
    main: &str,
    style_projet: Option<&str>,
    fichier: &Path,
    rapports: &mut Vec<crate::langage::diagnostics::Diagnostic>,
) -> String {
    // Le style du projet doit être connu du moteur — jamais deviné.
    let style_valide = match style_projet {
        Some(style) if crate::typst::styles::style_valide(style) => Some(style),
        Some(style) => {
            rapports.push(crate::langage::diagnostics::Diagnostic {
                origine: crate::langage::diagnostics::Origine::Analyse,
                message: format!(
                    "Le style « {style} » est inconnu du moteur : non appliqué, le style de base le remplace."
                ),
                line: None,
                column: None,
                file: Some(fichier.to_path_buf()),
            });
            None
        }
        None => None,
    };

    // Les appels bibliographiques, tous — chacun porte son style.
    let mut appels = Vec::new();
    let mut offset = 0;

    while let Some((ouverture, fermeture)) = bibliography_call_range(&main[offset..]) {
        appels.push((offset + ouverture, offset + fermeture));
        offset += fermeture;
    }

    // De la fin vers le début : chaque réécriture déplace ce qui suit.
    let mut sortie = main.to_string();

    for (ouverture, fermeture) in appels.into_iter().rev() {
        let arguments = &sortie[ouverture + 1..fermeture];

        if let Some(position) = arguments.find("style:") {
            // Un style est déjà explicite : le document fait autorité.
            let valeur = arguments[position + "style:".len()..]
                .split(',')
                .next()
                .unwrap_or_default()
                .trim()
                .trim_matches('"')
                .to_string();

            if let Some(style) = style_valide
                && valeur != style
            {
                rapports.push(crate::langage::diagnostics::Diagnostic {
                    origine: crate::langage::diagnostics::Origine::Analyse,
                    message: format!(
                        "Le document impose le style « {valeur} » alors que le projet choisit « {style} » : c'est le document qui fait autorité. Accordez les deux pour lever l'ambiguïté."
                    ),
                    line: None,
                    column: None,
                    file: Some(fichier.to_path_buf()),
                });
            }

            continue;
        }

        // Aucun style : celui du projet s'y inscrit, ou le style de base.
        let style = style_valide.unwrap_or("chicago-notes");
        let arguments = arguments.trim_end();
        let insertion = ouverture + 1 + arguments.len();
        let separateur = if arguments.trim().is_empty() {
            ""
        } else if arguments.ends_with(',') {
            " "
        } else {
            ", "
        };

        sortie = format!(
            "{}{separateur}style: \"{style}\"{}",
            &sortie[..insertion],
            &sortie[insertion..]
        );
    }

    sortie
}

/// Prépare l'appel bibliographique **déjà présent** : branchement des
/// bibliothèques, style et placement. Aucun appel n'est ajouté, et le fichier
/// sur disque n'est jamais modifié.
fn preparer_la_bibliographie(
    root: &Path,
    main: &str,
    normes: &crate::mise_en_page::modele::MiseEnPage,
) -> String {
    // Le branchement d'abord : les bibliothèques du projet sont citées,
    // et un fichier manquant ne l'est plus (bug-report-9) — le style
    // par défaut complète ensuite ce qui en manque.
    let main = crate::project::model::brancher_les_bibliographies(root, main)
        .unwrap_or_else(|| main.to_string());

    let main = ensure_default_citation_style(&main);

    placer_la_bibliographie(&main, normes)
}

/// Le placement de la bibliographie : le saut de page avant, le titre de
/// niveau 1 — les normes du document (bug-report-10, 1.1 et 1.2).
fn placer_la_bibliographie(main: &str, normes: &crate::mise_en_page::modele::MiseEnPage) -> String {
    if !normes.bibliographie.saut_de_page && !normes.bibliographie.titre_un {
        return main.to_string();
    }

    let mut out = String::new();

    for ligne in main.split_inclusive('\n') {
        if is_bibliography_directive(ligne) && !out.contains("#bibliography") {
            if normes.bibliographie.saut_de_page {
                out.push_str("#pagebreak()\n\n");
            }

            if normes.bibliographie.titre_un {
                out.push_str("= Bibliographie\n\n");
            }

            // La liste ne se donne pas son propre titre quand le
            // document lui donne le sien.
            if normes.bibliographie.titre_un
                && !ligne.contains("title:")
                && let Some(fermeture) = ligne.rfind(')')
            {
                out.push_str(&ligne[..fermeture]);
                out.push_str(", title: none)");
                out.push_str(&ligne[fermeture + 1..]);

                continue;
            }
        }

        out.push_str(ligne);
    }

    out
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
/// Le texte replaté : les blancs, les césures de fin de ligne et les
/// césures douces sortent. `pdftotext` coupe où la mise en page veut —
/// un nom de projet y passe sur deux lignes — et l'aiguille se
/// compare au **contenu**, jamais à l'habillage.
fn plat(texte: &str) -> String {
    texte
        .chars()
        .filter(|c| !c.is_whitespace() && *c != char::from(0xAD) && *c != char::from(12))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::plat;

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

    /// Un `@` dans une chaîne Typst n'est pas une citation : l'import d'un
    /// paquet local doit traverser l'assemblage intact.
    #[test]
    fn un_import_de_paquet_n_est_pas_une_citation() {
        let root = std::env::temp_dir().join(format!(
            "typst-writer-pipeline-import-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("racine du projet");

        let main = "#import \"@local/primaviz:0.12.1\": *\n";
        let (entree, corrections, rapports) =
            super::compilation_unit_avec_diagnostics(&root, main, "Un texte.\n");

        assert!(
            entree.contains("@local/primaviz:0.12.1"),
            "l'import reste intact : {entree}"
        );
        assert!(
            !entree.contains("[local]"),
            "l'import n'est jamais une citation neutralisée : {entree}"
        );
        assert!(
            !rapports
                .iter()
                .any(|rapport| rapport.message.contains("local")),
            "aucune citation « local » inventée : {rapports:?}"
        );
        assert!(
            corrections.iter().any(
                |(chemin, contenu)| chemin.ends_with("document.generated.typ")
                    && contenu.contains("Un texte.")
                    && !contenu.contains("[local]")
            ),
            "le généré ne porte pas de citation inventée : {corrections:?}"
        );

        let _ = std::fs::remove_dir_all(&root);
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

    /// La section `{bibliographie}` se branche aux bibliothèques du
    /// projet (bug-report-9).
    #[test]
    fn la_section_bibliographie_se_brance_au_projet() {
        let root = std::env::temp_dir().join(format!(
            "typst-writer-langage-section-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("racine du projet");
        std::fs::write(
            root.join("references-zotero.bib"),
            "@book{dupont2023, title = {Un titre}}\n",
        )
        .expect("export zotero");

        let (_entry, corrections, _rapports) = super::compilation_unit_avec_diagnostics(
            &root,
            "#set page(paper: \"a6\")\n",
            "Avant.\n\n{bibliographie}\n",
        );

        let genere = corrections
            .iter()
            .find(|(chemin, _)| chemin.ends_with("document.generated.typ"))
            .map(|(_, contenu)| contenu.clone())
            .expect("le généré");
        assert!(
            genere.contains("full: true"),
            "la section liste la bibliothèque entière : {genere}"
        );
        assert!(
            genere.contains("\"references-zotero.bib\""),
            "elle cite les bibliothèques du projet : {genere}"
        );
        assert!(
            !genere.contains("\"references.bib\""),
            "le fichier absent n'est pas cité : {genere}"
        );

        std::fs::remove_dir_all(&root).ok();
    }

    /// Les schémas de numérotation libres partent au moteur tels quels
    /// (bug-report-9 : la liste blanche de `forme_numerotation` les
    /// écrasait en « 1 »).
    #[test]
    fn les_schemas_de_numerotation_se_rendent_au_moteur() {
        let mut normes = crate::mise_en_page::modele::MiseEnPage::default();
        normes.numerotation.actif = true;
        normes.numerotation.forme = "1)".to_string();
        normes.numerotation_figures = "A.a".to_string();
        normes.titres.numerotation = "1.1.".to_string();

        let gabarit = crate::mise_en_page::gabarit::generer(&normes);

        for schema in ["1)", "A.a", "1.1."] {
            assert!(
                gabarit.contains(schema),
                "le schéma {schema:?} part au moteur : {gabarit}"
            );
        }
    }

    /// Le placement de la bibliographie suit les normes (bug-report-10,
    /// 1.1 et 1.2) : saut de page avant, titre de niveau 1, sans double
    /// titre.
    #[test]
    fn la_bibliographie_se_place_selon_les_normes() {
        let mut normes = crate::mise_en_page::modele::MiseEnPage::default();
        normes.bibliographie.saut_de_page = true;
        normes.bibliographie.titre_un = true;

        let root = std::env::temp_dir().join(format!(
            "typst-writer-langage-placement-biblio-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("racine du projet");
        crate::mise_en_page::appliquer(&root, &normes, false).expect("application");
        std::fs::write(root.join("references.bib"), "").expect("bibliographie");

        let main = format!(
            "{}\n{}\n#bibliography(\"references.bib\")\n",
            crate::mise_en_page::gabarit::accroche(),
            crate::project::model::MAIN_TEMPLATE
        );
        let (entry, _corrections, _rapports) =
            super::compilation_unit_avec_diagnostics(&root, &main, "Du texte.\n");

        assert!(
            entry.contains("#pagebreak()"),
            "le saut de page part avant : {entry}"
        );
        assert!(
            entry.contains("= Bibliographie"),
            "le titre de niveau 1 est là : {entry}"
        );
        assert!(
            entry.contains("title: none"),
            "la liste ne se double pas de son titre : {entry}"
        );

        std::fs::remove_dir_all(&root).ok();
    }

    /// Le lien Zotero branche la bibliographie : les citations de l’export
    /// trouvent leurs clés, et le fichier manquant n’est plus cité
    /// (bug-report-9).
    #[test]
    fn les_citations_zotero_trouvent_leur_bibliographie() {
        let root = std::env::temp_dir().join(format!(
            "typst-writer-langage-zotero-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("racine du projet");
        std::fs::write(
            root.join("references-zotero.bib"),
            "@book{dupont2023, title = {Un titre}}\n",
        )
        .expect("export zotero");

        // Le document modèle cite la locale — qui n’existe pas encore.
        let main = format!(
            "{}\n\n#bibliography(\"references.bib\", style: \"chicago-notes\")\n",
            "#set page(paper: \"a6\")"
        );
        let (entry, corrections, _rapports) =
            super::compilation_unit_avec_diagnostics(&root, &main, "Selon @dupont2023.\n");

        assert!(
            entry.contains("\"references-zotero.bib\""),
            "l’export Zotero est branché : {entry}"
        );
        assert!(
            !entry.contains("\"references.bib\""),
            "le fichier manquant n’est plus cité : {entry}"
        );

        let genere = corrections
            .iter()
            .find(|(chemin, _)| chemin.ends_with("document.generated.typ"))
            .map(|(_, contenu)| contenu.clone())
            .expect("le généré");
        assert!(
            !genere.contains("[dupont2023]"),
            "la citation Zotero est vivante, jamais neutralisée : {genere}"
        );

        std::fs::remove_dir_all(&root).ok();
    }
    /// Une citation inconnue ne devient jamais une fausse référence — et
    /// désormais, elle ne se cache plus : la neutralisation se dit avec sa
    /// clé, sa ligne et son fichier (R3, fidélité des citations).
    #[test]
    fn une_citation_inconnue_est_signalee_et_ne_devient_jamais_une_reference() {
        let root = std::env::temp_dir().join(format!(
            "typst-writer-langage-diagnostic-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("racine du projet");
        std::fs::write(
            root.join("references.bib"),
            "@book{vivante, title = {Un titre}}\n",
        )
        .expect("bibliographie");

        let (entry, corrections, rapports) = super::compilation_unit_avec_diagnostics(
            &root,
            "#set page(paper: \"a6\")\n",
            "Selon @inconnue, et @vivante qui existe.\n",
        );

        let genere = corrections
            .iter()
            .find(|(chemin, _)| chemin.ends_with("document.generated.typ"))
            .map(|(_, contenu)| contenu.clone())
            .expect("le généré");
        assert!(genere.contains("[inconnue]"), "{genere}");
        assert!(
            !genere.contains("[vivante]"),
            "une clé vivante reste une vraie référence : {genere}"
        );
        assert!(entry.contains("#include"), "{entry}");
        assert_eq!(rapports.len(), 1, "une seule neutralisation : {rapports:?}");
        let diagnostic = &rapports[0];
        assert!(
            diagnostic.message.contains("inconnue"),
            "{}",
            diagnostic.message
        );
        assert_eq!(diagnostic.line, Some(1), "{}", diagnostic.message);
        assert!(
            diagnostic
                .file
                .as_ref()
                .is_some_and(|fichier| fichier.ends_with("document.generated.typ")),
            "le fichier est dit : {diagnostic:?}"
        );
        assert!(
            rapports.iter().any(|d| d.origine
                == crate::langage::diagnostics::Origine::Transformation
                && d.message.contains("inconnue")),
            "le diagnostic dit son origine : {rapports:?}"
        );
    }

    /// `#cite(<cle>)` suit exactement le même contrat.
    #[test]
    fn un_cite_inconnu_est_signale() {
        let root = std::env::temp_dir().join(format!(
            "typst-writer-langage-diagnostic-cite-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("racine du projet");
        std::fs::write(
            root.join("references.bib"),
            "@book{vivante, title = {Un titre}}\n",
        )
        .expect("bibliographie");

        let (_, _, rapports) = super::compilation_unit_avec_diagnostics(
            &root,
            "#set page(paper: \"a6\")\n#cite(<perdue>)\n",
            "",
        );

        assert_eq!(rapports.len(), 1, "{rapports:?}");
        assert!(
            rapports[0].message.contains("perdue"),
            "{}",
            rapports[0].message
        );
    }

    /// Une citation perdue dans un chapitre est signalée avec **son** fichier,
    /// pas celui de l'assemblage : c'est là qu'il faut la corriger.
    #[test]
    fn une_citation_perdue_dans_un_chapitre_dit_son_fichier() {
        let root = std::env::temp_dir().join(format!(
            "typst-writer-langage-diagnostic-chapitre-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("chapitres")).expect("arborescence");
        std::fs::write(
            root.join("references.bib"),
            "@book{vivante, title = {Un titre}}\n",
        )
        .expect("bibliographie");
        std::fs::write(
            root.join("chapitres").join("1.typ"),
            "Un texte @perdue au chapitre.\n",
        )
        .expect("chapitre");

        let (_, _, rapports) = super::compilation_unit_avec_diagnostics(
            &root,
            "#include \"chapitres/1.typ\"\n#bibliography(\"references.bib\")\n",
            "",
        );

        assert_eq!(rapports.len(), 1, "{rapports:?}");
        assert!(
            rapports[0]
                .file
                .as_ref()
                .is_some_and(|fichier| fichier.ends_with("1.typ")),
            "le chapitre est nommé : {:?}",
            rapports[0]
        );
    }

    /// Les clés non ASCII sont des clés à part entière — et une clé non
    /// ASCII **manquante** se neutralise comme les autres : elle ne casse
    /// jamais la compilation (acceptation R3 : « clés non ASCII et
    /// manquantes »).
    #[test]
    fn une_cle_non_ascii_manquante_ne_casse_jamais_le_document() {
        let root = std::env::temp_dir().join(format!(
            "typst-writer-langage-cle-unicode-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("racine du projet");
        std::fs::write(
            root.join("references.bib"),
            "@book{vivante, title = {Un titre}}\n",
        )
        .expect("bibliographie");

        let (entry, corrections, rapports) = super::compilation_unit_avec_diagnostics(
            &root,
            "#set page(paper: \"a6\")\n",
            "Selon @été2020, clé non ASCII.\n",
        );

        let genere = corrections
            .iter()
            .find(|(chemin, _)| chemin.ends_with("document.generated.typ"))
            .map(|(_, contenu)| contenu.clone())
            .unwrap_or_default();
        assert!(entry.contains("#include"), "{entry}");
        assert!(
            genere.contains("[été2020]"),
            "la clé manquante devient un texte visible : {genere}"
        );
        assert_eq!(rapports.len(), 1, "{rapports:?}");
        assert!(
            rapports[0].message.contains("été2020"),
            "{}",
            rapports[0].message
        );
    }

    /// Une clé non ASCII vivante reste une vraie référence, jamais
    /// neutralisée à tort.
    #[test]
    fn une_cle_non_ascii_vivante_reste_une_vraie_reference() {
        let root = std::env::temp_dir().join(format!(
            "typst-writer-langage-cle-unicode-vivante-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("racine du projet");
        std::fs::write(
            root.join("references.bib"),
            "@book{été2020, title = {Un titre}}\n",
        )
        .expect("bibliographie");

        let (_, corrections, rapports) = super::compilation_unit_avec_diagnostics(
            &root,
            "#set page(paper: \"a6\")\n",
            "Selon @été2020, qui existe.\n",
        );

        let genere = corrections
            .iter()
            .find(|(chemin, _)| chemin.ends_with("document.generated.typ"))
            .map(|(_, contenu)| contenu.clone())
            .unwrap_or_default();
        assert!(
            genere.contains("@été2020"),
            "la clé vivante reste une référence : {genere}"
        );
        assert!(
            !genere.contains("[été2020]"),
            "jamais neutralisée à tort : {genere}"
        );
        assert!(rapports.is_empty(), "{rapports:?}");
    }

    /// Deux bibliographies différentes, des clés communes — et chaque
    /// citation reste fidèle : jamais neutralisée, jamais renvoyée à
    /// l'autre source en silence (acceptation R3).
    #[test]
    fn deux_bibliographies_a_cles_communes_ne_se_contredisent_pas() {
        let root = std::env::temp_dir().join(format!(
            "typst-writer-langage-deux-bib-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("chapitres")).expect("arborescence");
        std::fs::create_dir_all(root.join("Bibliographie")).expect("arborescence");
        std::fs::write(
            root.join("references.bib"),
            "@book{commune, title = {La première}, author = {Dupont, Jean}}\n",
        )
        .expect("première bibliographie");
        std::fs::write(
            root.join("chapitres").join("ma-bib.bib"),
            "@book{commune, title = {La seconde}, author = {Martin, Marie}}\n\
             @book{secondaire, title = {Un autre}, author = {Bernard, Paul}}\n",
        )
        .expect("seconde bibliographie");
        std::fs::write(
            root.join("chapitres").join("1.typ"),
            "#bibliography(\"ma-bib.bib\")\n\nSelon @commune et @secondaire.\n",
        )
        .expect("chapitre");

        let (_entree, corrections, rapports) = super::compilation_unit_avec_diagnostics(
            &root,
            "#include \"chapitres/1.typ\"\n#bibliography(\"references.bib\")\n",
            "",
        );

        let chapitre = corrections
            .iter()
            .find(|(chemin, _)| chemin.ends_with("1.typ"))
            .map(|(_, contenu)| contenu.clone())
            .unwrap_or_else(|| {
                std::fs::read_to_string(root.join("chapitres").join("1.typ")).expect("chapitre")
            });
        assert!(
            chapitre.contains("@commune") && chapitre.contains("@secondaire"),
            "les deux clés restent des références : {chapitre}"
        );
        assert!(
            !chapitre.contains("[commune]") && !chapitre.contains("[secondaire]"),
            "aucune n'est neutralisée : {chapitre}"
        );

        // La clé commune est dans deux bibliographies : la collision se
        // dit. La clé d'une seule source ne produit aucun bruit.
        assert_eq!(rapports.len(), 1, "{rapports:?}");
        assert!(
            rapports[0].message.contains("commune"),
            "la collision nomme la clé : {}",
            rapports[0].message
        );
        assert!(
            !rapports[0].message.contains("secondaire"),
            "la clé sans ambiguïté n'est pas mêlée : {}",
            rapports[0].message
        );

        // Sans la clé commune, deux bibliographies complémentaires ne
        // disent rien — et le compilateur tranche, sans erreur.
        std::fs::write(
            root.join("chapitres").join("ma-bib.bib"),
            "@book{secondaire, title = {Un autre}, author = {Bernard, Paul}}\n",
        )
        .expect("seconde bibliographie");
        let (entry, corrections, rapports) = super::compilation_unit_avec_diagnostics(
            &root,
            "#include \"chapitres/1.typ\"\n#bibliography(\"references.bib\")\n",
            "",
        );
        assert!(rapports.is_empty(), "{rapports:?}");
        let outcome = crate::typst::render::compile(&root, &entry, &corrections);
        assert!(
            outcome.errors.is_empty(),
            "deux bibliographies complémentaires compilent : {:?}",
            outcome.errors
        );
    }

    /// De l'insertion au PDF : le texte **extrait** porte bien la source
    /// citée — pas seulement un compilateur qui ne râle pas (acceptation R3).
    #[test]
    fn le_texte_extrait_du_pdf_porte_la_source_citee() {
        let root = std::env::temp_dir().join(format!(
            "typst-writer-langage-pdf-texte-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("racine du projet");
        std::fs::write(
            root.join("references.bib"),
            "@book{dupont2023, title = {La théologie du lien}, \
             author = {Dupont, Jean}, year = {2023}}\n",
        )
        .expect("bibliographie");

        let (entry, corrections, _) = super::compilation_unit_avec_diagnostics(
            &root,
            "#set page(paper: \"a6\")\n#bibliography(\"references.bib\")\n",
            "Selon @dupont2023, la thèse est posée.\n",
        );

        let pdf = crate::typst::render::render_pdf(&root, &entry, &corrections)
            .expect("le PDF se produit");
        assert!(
            pdf.starts_with(b"%PDF"),
            "un vrai PDF : {} octets",
            pdf.len()
        );
        let fichier = root.join("sortie.pdf");
        std::fs::write(&fichier, &pdf).expect("PDF sur disque");

        let extrait = std::process::Command::new("pdftotext")
            .arg(&fichier)
            .arg("-")
            .output();
        let Ok(sortie) = extrait else {
            eprintln!("pdftotext absent — épreuve du texte extrait sautée");
            return;
        };
        assert!(sortie.status.success(), "pdftotext s'exécute");
        // La bibliographie met les titres en capitales : la comparaison ne
        // distingue pas les casse.
        let texte = String::from_utf8_lossy(&sortie.stdout).to_lowercase();
        assert!(
            plat(&texte).contains("dupont") && plat(&texte).contains(&plat("théologie")),
            "la source citée est dans le texte extrait du PDF : {texte}"
        );
    }

    /// Auteurs multiples : la note et la bibliographie les portent tous,
    /// et le texte extrait du PDF le prouve (acceptation R3).
    #[test]
    fn les_auteurs_multiples_se_lisent_dans_le_texte_extrait() {
        let root = std::env::temp_dir().join(format!(
            "typst-writer-langage-pdf-auteurs-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("racine du projet");
        std::fs::write(
            root.join("references.bib"),
            "@book{collectif2020, title = {L'ouvrage collectif}, \
             author = {Dupont, Jean and Martin, Marie and Bernard, Paul}, year = {2020}}\n",
        )
        .expect("bibliographie");

        let (entry, corrections, _) = super::compilation_unit_avec_diagnostics(
            &root,
            "#set page(paper: \"a6\")\n#bibliography(\"references.bib\")\n",
            "Selon @collectif2020, la thèse est posée.\n",
        );

        let pdf = crate::typst::render::render_pdf(&root, &entry, &corrections)
            .expect("le PDF se produit");
        let fichier = root.join("sortie.pdf");
        std::fs::write(&fichier, &pdf).expect("PDF sur disque");

        let Ok(sortie) = std::process::Command::new("pdftotext")
            .arg(&fichier)
            .arg("-")
            .output()
        else {
            eprintln!("pdftotext absent — épreuve du texte extrait sautée");
            return;
        };
        assert!(sortie.status.success(), "pdftotext s'exécute");
        let texte = String::from_utf8_lossy(&sortie.stdout).to_lowercase();
        for nom in ["dupont", "martin", "bernard"] {
            assert!(
                plat(&texte).contains(&plat(nom)),
                "l'auteur {nom} est dans le texte extrait : {texte}"
            );
        }
    }

    /// Compilation en erreur : les erreurs du document se disent avec son
    /// message, et les avertissements de préparation — ici une citation
    /// inconnue — restent visibles à côté. Rien n'est avalé (acceptation
    /// R3 : « conflits et compilation en erreur »).
    #[test]
    fn une_compilation_en_erreur_dit_erreurs_et_avertissements() {
        let root = std::env::temp_dir().join(format!(
            "typst-writer-langage-erreur-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("racine du projet");
        std::fs::write(
            root.join("references.bib"),
            "@book{vivante, title = {Un titre}}\n",
        )
        .expect("bibliographie");

        let (entry, corrections, rapports) = super::compilation_unit_avec_diagnostics(
            &root,
            "#set page(paper: \"a6\")\n#bibliography(\"references.bib\")\n#panic(\"panne synthétique\")\n",
            "Selon @inconnue, la thèse est posée.\n",
        );

        // La citation inconnue est dite en préparation…
        assert_eq!(rapports.len(), 1, "{rapports:?}");
        assert!(
            rapports[0].message.contains("inconnue"),
            "{}",
            rapports[0].message
        );

        // …et la panne du document se dit au compilateur, avec son mot.
        let outcome = crate::typst::render::compile(&root, &entry, &corrections);
        assert!(
            !outcome.errors.is_empty(),
            "la compilation en erreur dit ses erreurs"
        );
        assert!(
            outcome
                .errors
                .iter()
                .any(|diagnostic| diagnostic.message.contains("panne synthétique")),
            "le message est celui du document : {:?}",
            outcome.errors
        );
    }

    /// Le style du projet gouverne citations et bibliographie ensemble :
    /// une directive sans style le reçoit (R4).
    #[test]
    fn le_style_du_projet_s_applique_a_la_bibliographie() {
        let root =
            std::env::temp_dir().join(format!("typst-writer-langage-style-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("racine du projet");
        std::fs::write(
            root.join("references.bib"),
            "@book{vivante, title = {Un titre}}\n",
        )
        .expect("bibliographie");
        std::fs::write(
            root.join("mise-en-page.toml"),
            "[bibliographie]\nstyle = \"apa\"\n",
        )
        .expect("normes");

        let (entry, _, rapports) = super::compilation_unit_avec_diagnostics(
            &root,
            "#bibliography(\"references.bib\")\n",
            "",
        );

        assert!(
            entry.contains("style: \"apa\""),
            "le style du projet s'y inscrit : {entry}"
        );
        assert!(rapports.is_empty(), "{rapports:?}");
    }

    /// Une directive qui impose **son** style reste maîtresse : le conflit
    /// avec le style du projet se dit, sans rien écraser (R4).
    #[test]
    fn une_directive_qui_impose_son_style_ne_se_ecrase_jamais() {
        let root = std::env::temp_dir().join(format!(
            "typst-writer-langage-style-conflit-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("racine du projet");
        std::fs::write(
            root.join("references.bib"),
            "@book{vivante, title = {Un titre}}\n",
        )
        .expect("bibliographie");
        std::fs::write(
            root.join("mise-en-page.toml"),
            "[bibliographie]\nstyle = \"apa\"\n",
        )
        .expect("normes");

        let (entry, _, rapports) = super::compilation_unit_avec_diagnostics(
            &root,
            "#bibliography(\"references.bib\", style: \"ieee\")\n",
            "",
        );

        assert!(
            entry.contains("style: \"ieee\""),
            "le style du document reste : {entry}"
        );
        assert!(
            !entry.contains("style: \"apa\""),
            "rien n'est écrasé : {entry}"
        );
        assert_eq!(rapports.len(), 1, "{rapports:?}");
        let message = &rapports[0].message;
        assert!(
            message.contains("ieee") && message.contains("apa"),
            "le conflit nomme les deux styles : {message}"
        );
        assert!(
            message.contains("autorité"),
            "la priorité est dite : {message}"
        );
    }

    /// Un style que le moteur refuse n'est jamais appliqué : il se dit, et
    /// le style de base prend le relais (R4).
    #[test]
    fn un_style_de_projet_inconnu_est_dit_pas_applique() {
        let root = std::env::temp_dir().join(format!(
            "typst-writer-langage-style-inconnu-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("racine du projet");
        std::fs::write(
            root.join("references.bib"),
            "@book{vivante, title = {Un titre}}\n",
        )
        .expect("bibliographie");
        std::fs::write(
            root.join("mise-en-page.toml"),
            "[bibliographie]\nstyle = \"un-style-qui-nexiste-pas\"\n",
        )
        .expect("normes");

        let (entry, _, rapports) = super::compilation_unit_avec_diagnostics(
            &root,
            "#bibliography(\"references.bib\")\n",
            "",
        );

        assert!(
            entry.contains("style: \"chicago-notes\""),
            "le style de base le remplace : {entry}"
        );
        assert!(
            !entry.contains("un-style-qui-nexiste-pas"),
            "le style refusé n'est jamais appliqué : {entry}"
        );
        assert_eq!(rapports.len(), 1, "{rapports:?}");
        assert!(
            rapports[0].message.contains("inconnu"),
            "{}",
            rapports[0].message
        );
    }

    /// Changer de style ne touche jamais aux clés ni aux notices (R4).
    #[test]
    fn changer_le_style_ne_touche_jamais_aux_notices() {
        let root = std::env::temp_dir().join(format!(
            "typst-writer-langage-style-notices-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("racine du projet");
        let bibliotheque = "@book{vivante, title = {Un titre}}\n";
        std::fs::write(root.join("references.bib"), bibliotheque).expect("bibliographie");
        std::fs::write(
            root.join("mise-en-page.toml"),
            "[bibliographie]\nstyle = \"ieee\"\n",
        )
        .expect("normes");

        let _ = super::compilation_unit_avec_diagnostics(
            &root,
            "#bibliography(\"references.bib\")\n",
            "",
        );

        assert_eq!(
            std::fs::read_to_string(root.join("references.bib")).expect("bibliographie"),
            bibliotheque,
            "les notices sont intactes"
        );
    }

    /// Un style `.csl` importé suit le projet hors ligne, jamais l'original —
    /// et se lit dans la directive (R4).
    #[test]
    fn un_style_csl_importe_se_lit_dans_la_directive() {
        let root = std::env::temp_dir().join(format!(
            "typst-writer-langage-csl-import-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("racine du projet");
        std::fs::write(
            root.join("references.bib"),
            "@book{vivante, title = {Un titre}}\n",
        )
        .expect("bibliographie");

        let csl = r#"<?xml version="1.0" encoding="utf-8"?>
<style xmlns="http://purl.org/net/xbiblio/csl" version="1.0" class="note">
  <info>
    <id>https://example.org/styles/test</id>
    <title>Style de test</title>
    <updated>2026-01-01T00:00:00+00:00</updated>
  </info>
  <citation><layout><text variable="title"/></layout></citation>
  <bibliography><layout><text variable="title"/></layout></bibliography>
</style>"#;

        // L'original vit hors du projet et n'en bouge pas.
        let original = std::env::temp_dir().join("style-de-test-externe.csl");
        std::fs::write(&original, csl).expect("style d'origine");

        let crate::project::ImportationStyle::Choisi {
            chemin: relatif,
            avertissement,
        } = crate::project::importer_style_csl(&root, "style-de-test.csl", csl, None)
            .expect("import")
        else {
            panic!("un style indépendant est choisi")
        };
        assert_eq!(avertissement, None, "pas d'écart à dire");
        assert_eq!(
            relatif,
            std::path::PathBuf::from("styles").join("style-de-test.csl")
        );
        assert!(
            root.join("styles").join("style-de-test.csl").is_file(),
            "la copie suit le projet"
        );
        assert_eq!(
            std::fs::read_to_string(&original).expect("original"),
            csl,
            "l'original n'a pas bougé"
        );

        // Une seconde fois : le nom est libre, rien ne s'écrase.
        let crate::project::ImportationStyle::Choisi { chemin: second, .. } =
            crate::project::importer_style_csl(&root, "style-de-test.csl", csl, None)
                .expect("import")
        else {
            panic!("un style indépendant est choisi")
        };
        assert_ne!(relatif, second, "le second import prend un nom libre");

        // Le style du projet se lit dans la directive.
        std::fs::write(
            root.join("mise-en-page.toml"),
            format!("[bibliographie]\nstyle = \"{}\"\n", relatif.display()),
        )
        .expect("normes");
        let (entry, _, rapports) = super::compilation_unit_avec_diagnostics(
            &root,
            "#bibliography(\"references.bib\")\n",
            "",
        );
        assert!(
            entry.contains(&format!("style: \"{}\"", relatif.display())),
            "le style de fichier se lit dans la directive : {entry}"
        );
        assert!(rapports.is_empty(), "{rapports:?}");
    }

    /// Un style dépendant sans parent sur la machine : le style est
    /// conservé, rien n'est choisi, et le geste se dit (R4).
    #[test]
    fn un_style_dependant_sans_parent_dit_le_geste() {
        let root = std::env::temp_dir().join(format!(
            "typst-writer-langage-csl-dependant-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("racine du projet");

        let dependant = DEPENDANT;

        let crate::project::ImportationStyle::ParentManquant {
            chemin,
            titre,
            parent,
        } = crate::project::importer_style_csl(&root, "dependant.csl", dependant, None)
            .expect("conservé")
        else {
            panic!("le parent manque")
        };

        assert_eq!(
            parent, "https://example.org/styles/parent",
            "le parent est dit"
        );
        assert_eq!(titre, "Style dépendant");
        assert!(
            root.join("styles").join("dependant.csl").is_file(),
            "le style demandé est conservé : {}",
            chemin.display()
        );
    }

    /// Le parent, quand il vit dans les styles du projet, résout le style
    /// dépendant — avec l'écart de locale dit (R4).
    #[test]
    fn un_parent_du_projet_resout_le_style_dependant() {
        let root = std::env::temp_dir().join(format!(
            "typst-writer-langage-csl-parent-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("styles")).expect("arborescence");
        std::fs::write(root.join("styles").join("parent.csl"), PARENT).expect("parent");

        let crate::project::ImportationStyle::Choisi {
            chemin,
            avertissement,
        } = crate::project::importer_style_csl(&root, "dependant.csl", DEPENDANT, None)
            .expect("résolu")
        else {
            panic!("le parent local résout le style")
        };

        assert_eq!(
            chemin,
            std::path::PathBuf::from("styles").join("parent.csl"),
            "le moteur utilisera le parent"
        );
        let avertissement = avertissement.expect("l'écart de locale se dit");
        assert!(
            avertissement.contains("locale") && avertissement.contains("dépendant"),
            "{avertissement}"
        );
    }

    /// Un parent installé dans Zotero est trouvé, copié dans le projet, et
    /// résout le style — sans jamais modifier Zotero (R4).
    #[test]
    fn un_parent_de_zotero_resout_le_style_dependant() {
        let root = std::env::temp_dir().join(format!(
            "typst-writer-langage-csl-zotero-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("racine du projet");

        // Un dossier de styles Zotero fabriqué — jamais les données
        // personnelles de la machine.
        let zotero =
            std::env::temp_dir().join(format!("typst-writer-zotero-styles-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&zotero);
        std::fs::create_dir_all(&zotero).expect("styles Zotero");
        std::fs::write(zotero.join("parent-zotero.csl"), PARENT).expect("parent");

        let crate::project::ImportationStyle::Choisi {
            chemin,
            avertissement: _,
        } = crate::project::importer_style_csl(&root, "dependant.csl", DEPENDANT, Some(&zotero))
            .expect("résolu")
        else {
            panic!("le parent de Zotero résout le style")
        };

        assert!(
            root.join(&chemin).is_file(),
            "le parent est copié dans le projet : {}",
            chemin.display()
        );
        assert_eq!(
            std::fs::read_to_string(zotero.join("parent-zotero.csl")).expect("parent"),
            PARENT,
            "Zotero n'est jamais modifié"
        );
    }

    /// Le parent d'un style dépendant et le style lui-même, pour les tests.
    const PARENT: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<style xmlns="http://purl.org/net/xbiblio/csl" version="1.0" class="note">
  <info>
    <id>https://example.org/styles/parent</id>
    <title>Le parent</title>
    <updated>2026-01-01T00:00:00+00:00</updated>
  </info>
  <citation><layout><text variable="title"/></layout></citation>
  <bibliography><layout><text variable="title"/></layout></bibliography>
</style>"#;

    const DEPENDANT: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<style xmlns="http://purl.org/net/xbiblio/csl" version="1.0" class="in-text">
  <info>
    <id>https://example.org/styles/dependent</id>
    <title>Style dépendant</title>
    <updated>2026-01-01T00:00:00+00:00</updated>
    <link href="https://example.org/styles/parent" rel="independent-parent"/>
  </info>
</style>"#;
    /// Acceptation R4 : auteur-date, numérique et notes rendent des textes
    /// **différents** — comparés sur le PDF réel, pas seulement au
    /// compilateur.
    #[test]
    fn les_trois_familles_rendent_des_textes_differents() {
        let mut extraits = Vec::new();

        for style in ["apa", "ieee", "chicago-notes"] {
            let root = std::env::temp_dir().join(format!(
                "typst-writer-langage-familles-{style}-{}",
                std::process::id()
            ));
            let _ = std::fs::remove_dir_all(&root);
            std::fs::create_dir_all(&root).expect("racine du projet");
            std::fs::write(
                root.join("references.bib"),
                "@book{dupont2023, title = {La théologie du lien}, \
                 author = {Dupont, Jean}, year = {2023}}\n",
            )
            .expect("bibliographie");

            let (entry, corrections, _) = super::compilation_unit_avec_diagnostics(
                &root,
                &format!(
                    "#set page(paper: \"a6\")\n#bibliography(\"references.bib\", style: \"{style}\")\n"
                ),
                "Selon @dupont2023, la thèse est posée.\n",
            );

            let pdf = crate::typst::render::render_pdf(&root, &entry, &corrections)
                .expect("le PDF se produit");
            let fichier = root.join("sortie.pdf");
            std::fs::write(&fichier, &pdf).expect("PDF sur disque");

            let Ok(sortie) = std::process::Command::new("pdftotext")
                .arg(&fichier)
                .arg("-")
                .output()
            else {
                eprintln!("pdftotext absent — épreuve des familles sautée");
                return;
            };
            assert!(sortie.status.success(), "pdftotext s'exécute");
            extraits.push((
                style,
                String::from_utf8_lossy(&sortie.stdout).to_lowercase(),
            ));
        }

        // Chaque famille a sa voix : aucune ne ressemble à une autre.
        for (i, (style_a, texte_a)) in extraits.iter().enumerate() {
            for (style_b, texte_b) in extraits.iter().skip(i + 1) {
                assert_ne!(
                    texte_a, texte_b,
                    "{style_a} et {style_b} rendent le même texte : {texte_a}"
                );
            }
        }
    }

    /// Une adresse électronique n'est jamais une clé — le commentaire du
    /// pipeline le promet, voici l'épreuve qui le garantit.
    #[test]
    fn une_adresse_electronique_nest_jamais_une_cle() {
        let root = std::env::temp_dir().join(format!(
            "typst-writer-langage-courriel-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("racine du projet");
        std::fs::write(
            root.join("references.bib"),
            "@book{vivante, title = {Un titre}}\n",
        )
        .expect("bibliographie");

        let (_, corrections, rapports) = super::compilation_unit_avec_diagnostics(
            &root,
            "#set page(paper: \"a6\")\n",
            "Contactez user@example.com svp. Comme dit @vivante. Ensuite.\n",
        );

        let genere = corrections
            .iter()
            .find(|(chemin, _)| chemin.ends_with("document.generated.typ"))
            .map(|(_, contenu)| contenu.clone())
            .unwrap_or_default();
        // Le `@` d'une adresse s'échappe — c'est ainsi que Typst la lit
        // comme du texte. Une fois l'échappement retiré, l'adresse est
        // entière.
        let visible = genere.replace("\\@", "@");
        assert!(
            visible.contains("user@example.com"),
            "l'adresse reste entière : {genere}"
        );
        assert!(
            !genere.contains("[example]"),
            "jamais neutralisée : {genere}"
        );
        // La citation suivie d'un point final : le point est de la
        // ponctuation, la clé reste une vraie référence.
        assert!(
            visible.contains("@vivante."),
            "le point final ne fait pas partie de la clé : {genere}"
        );
        assert!(
            !genere.contains("[vivante]"),
            "jamais neutralisée à tort : {genere}"
        );
        assert!(rapports.is_empty(), "{rapports:?}");
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
        assert_eq!(overrides[0].1, "= Titre <titre-titre>\n\nDu *gras*.\n\n");

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

        let main = format!(
            "{}\n#bibliography(\"references.bib\")\n",
            crate::project::MAIN_TEMPLATE
        );
        let (entry, overrides) =
            compilation_unit(&root, &main, "Selon @dupont2023, le résultat est établi.\n");
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

    /// Une ancienne entrée sans directive ne reçoit plus de bibliographie
    /// automatique : l'insertion reste un geste de l'auteur.
    #[test]
    fn une_ancienne_entree_sans_bibliographie_n_en_recoit_pas() {
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
        let (entry, _overrides) = compilation_unit(
            &root,
            old_main,
            "Selon @dupont2023, le résultat est établi.\n",
        );
        assert!(
            !entry.contains("#bibliography"),
            "aucune section ne s'ajoute sans instruction : {entry}"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Un projet neuf n'a ni bibliographie ni titre automatique : la
    /// fonctionnalité active n'est qu'une autorisation, pas une insertion.
    #[test]
    fn un_projet_neuf_sans_directive_n_affiche_pas_de_bibliographie() {
        let root = std::env::temp_dir().join(format!(
            "typst-writer-pipeline-neuf-bibliographie-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("racine du projet");
        std::fs::write(root.join("references.bib"), "@book{cle, title = {T}}\n")
            .expect("bibliographie d'essai");
        std::fs::write(root.join("main.typ"), crate::project::MAIN_TEMPLATE).expect("assemblage");

        let mut normes = crate::mise_en_page::modele::MiseEnPage::default();
        normes.bibliographie.style = Some("apa".to_string());
        normes.bibliographie.titre_un = true;
        crate::mise_en_page::appliquer(&root, &normes, false).expect("normes");

        let (entry, corrections, rapports) = compilation_unit_avec_diagnostics(
            &root,
            crate::project::MAIN_TEMPLATE,
            "Un paragraphe sans citation.\n",
        );

        assert!(
            !entry.contains("#bibliography"),
            "aucune directive ne s'ajoute : {entry}"
        );
        assert!(
            !entry.to_lowercase().contains("= bibliographie"),
            "aucun titre ne s'ajoute : {entry}"
        );
        assert!(
            !rapports
                .iter()
                .any(|diagnostic| diagnostic.message.contains("conflit")),
            "aucun conflit de style sur un projet neuf : {rapports:?}"
        );
        let _ = corrections;
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
    cles_glossaire: &std::collections::BTreeSet<String>,
    etiquettes: &std::collections::BTreeSet<String>,
    fichier: &Path,
    rapports: &mut Vec<crate::langage::diagnostics::Diagnostic>,
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
            // La neutralisation se dit : la clé, sa ligne, son fichier.
            let ligne = main[..position].matches('\n').count() + 1;
            rapports.push(crate::langage::diagnostics::Diagnostic {
                origine: crate::langage::diagnostics::Origine::Transformation,
                message: format!("Citation « {cle} » inconnue : neutralisée en [{cle}]."),
                line: Some(ligne),
                column: None,
                file: Some(fichier.to_path_buf()),
            });
            main_corrige.push_str(&format!("[{cle}]"));
        }

        curseur = fin;
    }

    main_corrige.push_str(&main[curseur..]);
    let main = main_corrige;
    let main = guerir_les_termes(&main, cles_glossaire, fichier, rapports);

    let mut sortie = String::new();

    for (numero, ligne) in main.split_inclusive('\n').enumerate() {
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

                    sortie
                        .push_str(&ligne.replace(&format!("\"{chemin}\""), &format!("\"{nom}\"")));
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
            if octets[curseur] == b'@' && !dans_une_chaine(ligne, curseur) {
                // Un `@` collé à un mot est une adresse électronique, pas
                // une citation. La frontière est la même que celle du
                // parseur : `user@example.com` reste du texte.
                if !crate::langage::citation::frontiere(&ligne[..curseur]) {
                    ligne_sortie.push('@');
                    curseur += 1;
                    continue;
                }

                // Déjà échappé par l'auteur : la suite est du texte.
                let echappe = ligne[..curseur].ends_with('\\');

                // La clé se lit caractère par caractère, avec la même
                // grammaire que le parseur. Dans ce texte Typst, un
                // deux-points peut désigner une étiquette ; le point, lui,
                // reste un caractère de clé.
                let mut fin_cle = curseur + 1;
                let mut deux_points = false;

                for caractere in ligne[curseur + 1..].chars() {
                    if caractere == ':' {
                        // Un renvoi (`@fig:une`) porte deux points : ce
                        // n'est jamais une citation ordinaire (R5).
                        deux_points = true;
                        fin_cle += caractere.len_utf8();
                    } else if crate::langage::citation::est_caractere_de_cle(caractere) {
                        fin_cle += caractere.len_utf8();
                    } else {
                        break;
                    }
                }

                // Le point final est de la ponctuation, pas de la clé.
                while fin_cle > curseur + 1 && ligne[..fin_cle].ends_with('.') {
                    fin_cle -= 1;
                }

                let cle = &ligne[curseur + 1..fin_cle];

                // Un renvoi traverse intact : étiquette à deux points, ou
                // étiquette définie dans le document. Une cible manquante
                // sera diagnostiquée par le moteur, jamais déguisée en
                // `[cle]` (R5, « diagnostics »).
                let est_renvoi = deux_points || (!cle.is_empty() && etiquettes.contains(cle));

                if !echappe && !est_renvoi && !cle.is_empty() && !cles.contains(cle) {
                    rapports.push(crate::langage::diagnostics::Diagnostic {
                        origine: crate::langage::diagnostics::Origine::Transformation,
                        message: format!("Citation « {cle} » inconnue : neutralisée en [{cle}]."),
                        line: Some(numero + 1),
                        column: None,
                        file: Some(fichier.to_path_buf()),
                    });
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

/// Le caractère à `position` est-il dans une chaîne Typst ?
///
/// Les `@` des chemins de paquets — `#import "@local/primaviz:…"` — ne sont
/// pas des citations : les neutraliser casse l'import. La lecture suit les
/// guillemets et leurs échappements, sans toucher au reste de la ligne.
fn dans_une_chaine(ligne: &str, position: usize) -> bool {
    let mut chaine = false;
    let mut echappe = false;

    for caractere in ligne[..position].chars() {
        if echappe {
            echappe = false;
            continue;
        }
        match caractere {
            '\\' if chaine => echappe = true,
            '"' => chaine = !chaine,
            _ => {}
        }
    }

    chaine
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
    cles_glossaire: &std::collections::BTreeSet<String>,
    etiquettes: &std::collections::BTreeSet<String>,
    rapports: &mut Vec<crate::langage::diagnostics::Diagnostic>,
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
        let corrige = corriger_le_typst(
            &base,
            &contenu,
            cles,
            cles_glossaire,
            etiquettes,
            &chemin,
            rapports,
        );

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
                format!(
                    "{}%{}",
                    &ligne[..ligne.len() - nu.len()],
                    nu.strip_prefix("//").unwrap_or(nu)
                )
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

            // **Tous** les chemins entre guillemets : un appel à
            // plusieurs bibliothèques les cite toutes, et la seconde
            // porte elle aussi ses entrées à guérir (bug-report-10, 6.1).
            for chemin in ligne.split('"').skip(1).step_by(2) {
                citees.push(base.join(chemin));
            }
        }
    }

    let mut guerisons = Vec::new();
    // Les clés déjà prises par une bibliothèque précédente : la première
    // garde son entrée (bug-report-10, 6.1).
    let mut connues = std::collections::BTreeSet::new();

    for chemin in citees {
        let Ok(contenu) = std::fs::read_to_string(&chemin) else {
            continue;
        };

        let original = contenu;
        let contenu = crate::project::retirer_les_entrees_dupliquees(&original, &mut connues);
        let gueri = guerir_commentaires(&contenu);

        // Le dédoublonnage compte comme une guérison : la correction
        // va au moteur même si les commentaires étaient déjà sains.
        if gueri != original {
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
/// Les étiquettes définies par `<nom>` dans un texte — les cibles de
/// renvois de R5. Un `<` de code (comparaison) n'a pas de `>` fermant au
/// nom d'étiquette valide, et ne s'y lit pas.
fn etiquettes_du_texte(texte: &str) -> Vec<String> {
    let mut out = Vec::new();
    let octets = texte.as_bytes();
    let mut curseur = 0usize;

    while curseur < octets.len() {
        if octets[curseur] == b'<'
            && let Some(fin) = texte[curseur + 1..].find('>')
        {
            let nom = &texte[curseur + 1..curseur + 1 + fin];
            let valide = nom
                .chars()
                .next()
                .is_some_and(|lettre| lettre.is_alphabetic() || lettre == '_')
                && nom.chars().all(|lettre| {
                    lettre.is_alphanumeric() || lettre == '_' || lettre == '-' || lettre == ':'
                });

            if valide {
                out.push(nom.to_string());
            }
            curseur += fin + 2;
            continue;
        }
        curseur += 1;
    }

    out
}

/// La guérison des termes du glossaire : `#gls("clé")` et ses trois
/// sœurs (`glspl`, `Gls`, `Glspl`).
///
/// Une clé inconnue se neutralise en `[clé]` — un texte, jamais une
/// erreur de compilateur, et le terme manquant se voit (rapport à la
/// clé, sa ligne, son fichier).
fn guerir_les_termes(
    main: &str,
    cles_glossaire: &std::collections::BTreeSet<String>,
    fichier: &Path,
    rapports: &mut Vec<crate::langage::diagnostics::Diagnostic>,
) -> String {
    let mut sortie = String::new();
    let mut curseur = 0usize;

    while let Some((position, prefixe)) = appel_glos_suivant(&main[curseur..]) {
        let position = curseur + position;
        let ouverture = position + prefixe.len() - 1; // la parenthèse ouvrante

        // L'appel entier, jusqu'à sa parenthèse fermante — le suffixe et
        // les autres arguments en font partie.
        let (cle, ok) = chaine_du_glos(main, ouverture + 1);
        let fin = parenthese_fermante(main, ouverture).unwrap_or(ouverture + 1);

        sortie.push_str(&main[curseur..position]);

        if ok && cles_glossaire.contains(&cle) {
            sortie.push_str(&main[position..fin]);
        } else {
            let ligne = main[..position].matches('\n').count() + 1;
            rapports.push(crate::langage::diagnostics::Diagnostic {
                origine: crate::langage::diagnostics::Origine::Transformation,
                message: format!("Terme « {cle} » du glossaire inconnu : neutralisé en [{cle}]."),
                line: Some(ligne),
                column: None,
                file: Some(fichier.to_path_buf()),
            });
            sortie.push_str(&format!("[{cle}]"));
        }

        curseur = fin;
    }

    sortie.push_str(&main[curseur..]);
    sortie
}

/// La position du prochain appel de terme, et son nom de fonction.
fn appel_glos_suivant(texte: &str) -> Option<(usize, &'static str)> {
    const PREFIXES: [&str; 4] = ["#glspl(", "#Glspl(", "#gls(", "#Gls("];
    let mut meilleur: Option<(usize, &'static str)> = None;
    for prefixe in PREFIXES {
        if let Some(position) = texte.find(prefixe)
            && meilleur.is_none_or(|(plus_tot, _)| position < plus_tot)
        {
            meilleur = Some((position, prefixe));
        }
    }
    meilleur
}

/// La première chaîne d'un appel — sa clé. `ok` dit si elle s'y lit.
fn chaine_du_glos(texte: &str, apres_parenthese: usize) -> (String, bool) {
    let reste = texte[apres_parenthese..].trim_start();
    let Some(reste) = reste.strip_prefix('"') else {
        return (String::new(), false);
    };

    let fin = reste.find('"').unwrap_or(reste.len());
    (reste[..fin].to_string(), true)
}

/// La parenthèse fermante d'un appel ouvert en `ouverture` — chaînes
/// comprises, parenthèses imbriquées comptées.
fn parenthese_fermante(texte: &str, ouverture: usize) -> Option<usize> {
    let mut profondeur = 0usize;
    let mut chaine = false;
    let mut echappe = false;

    for (decalage, c) in texte[ouverture..].char_indices() {
        if echappe {
            echappe = false;
            continue;
        }

        match c {
            '\\' if chaine => echappe = true,
            '"' => chaine = !chaine,
            '(' if !chaine => profondeur += 1,
            ')' if !chaine => {
                profondeur = profondeur.checked_sub(1)?;
                if profondeur == 0 {
                    return Some(ouverture + decalage + 1);
                }
            }
            _ => {}
        }
    }

    None
}

/// Le glossaire entre dans l'assemblage : le **registre** dans le document
/// généré (sa portée est celle du fichier), le **salon** — l'import et
/// `#show: make-glossary` — dans l'entrée, **après** sa première règle
/// `#show:` : les règles de figures de la mise en page restent extérieures,
/// comme glossarium le veut.
fn assembler_le_glossaire(
    root: &Path,
    entry: String,
    generated: String,
    glossaire: &crate::project::glossaire::Glossaire,
    rapports: &mut Vec<crate::langage::diagnostics::Diagnostic>,
) -> (String, String) {
    let usage = est_glos_utilise(&entry) || est_glos_utilise(&generated);
    if !usage && glossaire.entrees.is_empty() {
        return (entry, generated);
    }

    // Le paquet embarqué se pose dans le dossier local — jamais au réseau.
    if crate::typst::paquets::assurer_glossarium().is_err() {
        rapports.push(crate::langage::diagnostics::Diagnostic {
            origine: crate::langage::diagnostics::Origine::Transformation,
            message: "Le paquet glossarium ne s'est pas installé : les termes du glossaire                       sont neutralisés."
                .to_string(),
            line: None,
            column: None,
            file: None,
        });
        let vides = std::collections::BTreeSet::new();
        return (
            guerir_les_termes(&entry, &vides, root, rapports),
            guerir_les_termes(&generated, &vides, root, rapports),
        );
    }

    let mut generated = generated;
    let mut entete = format!(
        "#import \"{}\": make-glossary, register-glossary, print-glossary, gls, glspl, \
         Gls, Glspl\n",
        crate::typst::paquets::GLOSSARIUM
    );
    entete.push_str("#let tw-glossaire = ");
    entete.push_str(&glossaire.registre_typst());
    entete.push_str("\n#register-glossary(tw-glossaire)\n\n");
    generated.insert_str(0, &entete);

    if glossaire.entrees.is_empty() && generated.contains("#print-glossary(") {
        rapports.push(crate::langage::diagnostics::Diagnostic {
            origine: crate::langage::diagnostics::Origine::Transformation,
            message: "La liste du glossaire est vide : ajoutez des termes dans le panneau \
                      Glossaire."
                .to_string(),
            line: None,
            column: None,
            file: None,
        });
    }

    // Sans liste imprimée, les termes n'ont pas d'ancre : ils se
    // composent sans lien.
    if !entry.contains("#print-glossary(") && !generated.contains("#print-glossary(") {
        generated = delier_les_termes(&generated);
    }

    (injecter_le_salon(&entry), generated)
}

/// Les termes d'un document **sans liste** `{glossaire}` se composent
/// sans lien : glossarium pointe chaque terme vers son entrée imprimée,
/// et sans liste il n'y a pas d'ancre où pointer — un lien mort est une
/// erreur de compilation, jamais ça.
fn delier_les_termes(generated: &str) -> String {
    let mut out = String::new();
    let mut curseur = 0usize;

    while let Some((position, prefixe)) = appel_glos_suivant(&generated[curseur..]) {
        let position = curseur + position;
        let ouverture = position + prefixe.len() - 1;
        let fin = parenthese_fermante(generated, ouverture).unwrap_or(ouverture + 1);

        let appel = &generated[position..fin];
        if appel.contains("link:") {
            // L'appel tranche déjà lui-même.
            out.push_str(&generated[curseur..fin]);
        } else {
            out.push_str(&generated[curseur..fin.saturating_sub(1)]);
            out.push_str(", link: false)");
        }

        curseur = fin;
    }

    out.push_str(&generated[curseur..]);
    out
}

/// Le document use-t-il du glossaire ?
fn est_glos_utilise(texte: &str) -> bool {
    texte.contains("#gls(")
        || texte.contains("#glspl(")
        || texte.contains("#Gls(")
        || texte.contains("#Glspl(")
        || texte.contains("#print-glossary(")
}

/// L'import et `#show: make-glossary` dans l'entrée — après sa première
/// règle `#show:`, sinon en tête. Idempotent.
fn injecter_le_salon(entry: &str) -> String {
    if entry.contains("#show: make-glossary") {
        return entry.to_string();
    }

    let salon = format!(
        "#import \"{}\": make-glossary, register-glossary, print-glossary, gls, glspl, \
         Gls, Glspl\n#show: make-glossary\n",
        crate::typst::paquets::GLOSSARIUM
    );

    let mut out = String::new();
    let mut pose = false;
    for ligne in entry.split_inclusive('\n') {
        out.push_str(ligne);
        if !pose && ligne.trim_start().starts_with("#show:") {
            out.push_str(&salon);
            pose = true;
        }
    }

    if !pose {
        out.insert_str(0, &salon);
    }

    out
}

fn cles_vivantes(
    root: &Path,
    entry: &str,
) -> (
    std::collections::BTreeSet<String>,
    std::collections::BTreeSet<String>,
    Vec<crate::langage::diagnostics::Diagnostic>,
) {
    let mut cles = std::collections::BTreeSet::new();
    let mut etiquettes = std::collections::BTreeSet::new();
    let mut collisions = Vec::new();

    // Les étiquettes de l'assemblage : `<nom>` définit une cible de renvoi,
    // jamais une citation (R5).
    etiquettes.extend(etiquettes_du_texte(entry));

    // Toutes les bibliographies citées, et la conventionnelle — celles de
    // l'assemblage **et** celles que portent les chapitres inclus : une clé
    // n'est vivante que si une bibliographie du projet la connaît, où
    // qu'elle soit citée.
    let mut citees = vec![crate::project::find_bibliography(root)];

    for ligne in entry.lines() {
        if is_bibliography_directive(ligne)
            && let Some(chemin) = ligne.split('"').nth(1)
        {
            citees.push(Some(root.join(chemin)));
        }
    }

    let mut vus_fichiers = std::collections::HashSet::new();
    let mut a_voir: Vec<PathBuf> = crate::project::model::read_includes(&root.join("main.typ"));

    // L'entrée peut elle-même en porter — elle est en mémoire, pas encore
    // sur le disque.
    for ligne in entry.lines() {
        if let Some(reste) = ligne.trim().strip_prefix("#include")
            && let Some(chemin) = reste.trim().trim_matches('"').lines().next()
        {
            a_voir.push(PathBuf::from(chemin));
        }
    }

    while let Some(relatif) = a_voir.pop() {
        if !vus_fichiers.insert(relatif.clone()) {
            continue;
        }

        let chemin = root.join(&relatif);
        let Ok(contenu) = std::fs::read_to_string(&chemin) else {
            continue;
        };

        for sous in crate::project::model::read_includes(&chemin) {
            a_voir.push(relatif.parent().unwrap_or(Path::new("")).join(sous));
        }

        etiquettes.extend(etiquettes_du_texte(&contenu));

        for ligne in contenu.lines() {
            // La directive d'un chapitre : son chemin est relatif à **sa**
            // place, pas à celle de l'assemblage.
            if is_bibliography_directive(ligne)
                && let Some(chemin_bib) = ligne.split('"').nth(1)
            {
                let base = chemin.parent().unwrap_or(root);
                citees.push(Some(base.join(chemin_bib)));
            }
        }
    }

    // Les clés, fichier par fichier — et la collision se dit : la même clé
    // dans deux bibliographies rend la citation ambiguë, ce n'est jamais un
    // silence (R3, « détection des erreurs et collisions »).
    let mut origines: std::collections::BTreeMap<String, Vec<PathBuf>> = Default::default();

    for chemin in citees.into_iter().flatten() {
        if let Ok(contenu) = std::fs::read_to_string(&chemin) {
            // Le contenu guéri d'abord : un `.bib` aux commentaires `//`
            // fait perdre ses clés au parseur, et une clé perdue devient
            // une citation neutralisée à tort.
            let gueri = guerir_commentaires(&contenu);

            for entree in crate::project::parse_bibliography_text(&gueri) {
                origines.entry(entree.key).or_default().push(chemin.clone());
            }
        }
    }

    for (cle, fichiers) in &origines {
        let mut uniques: Vec<&PathBuf> = Vec::new();

        for fichier in fichiers {
            if !uniques.contains(&fichier) {
                uniques.push(fichier);
            }
        }

        if uniques.len() > 1 {
            let noms = uniques
                .iter()
                .map(|fichier| fichier.display().to_string())
                .collect::<Vec<_>>()
                .join(", ");

            collisions.push(crate::langage::diagnostics::Diagnostic {
                origine: crate::langage::diagnostics::Origine::Transformation,
                message: format!(
                    "La clé « {cle} » est dans plusieurs bibliographies ({noms}) : la citation est ambiguë."
                ),
                line: None,
                column: None,
                file: Some(uniques[1].clone().clone()),
            });
        }

        cles.insert(cle.clone());
    }

    (cles, etiquettes, collisions)
}

/// **Un document ne compile jamais mal, quoi qu'il arrive** — les
/// trois fautes réelles du rapport, ensemble, jugées par le vrai
/// compilateur : le chemin mort d'un chapitre, le `.bib` aux
/// commentaires `//`, et les citations inconnues sous leurs deux
/// formes.
#[test]
fn un_document_ne_compile_jamais_mal_quoi_qu_il_arrive() {
    if std::process::Command::new("typst")
        .arg("--version")
        .output()
        .is_err()
    {
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
    let (entry, corrections) = compilation_unit_document(
        &root,
        &std::fs::read_to_string(root.join("main.typ")).expect("main"),
        &document,
    );

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

#[test]
fn une_bibliographie_de_main_sans_tag_reste_cachee() {
    let root = std::env::temp_dir().join(format!(
        "typst-writer-bibliographie-main-sans-tag-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("racine");
    std::fs::write(
        root.join("references.bib"),
        "@book{dupont2023, title={T}, author={Dupont, A.}, year={2023}}\n",
    )
    .expect("bibliographie");

    let document = crate::langage::parse("Voir @dupont2023.\n");
    let (entry, corrections, _) = compilation_unit_document_avec_diagnostics_explicite(
        &root,
        "#set page(margin: 2cm)\n#bibliography(\"references.bib\")\n",
        &document,
    );

    assert!(
        !entry.contains("#hide["),
        "la bibliographie n'est pas masquée par #hide, qui réserve une page : {entry}"
    );
    assert!(
        entry.contains("#show bibliography: it => none"),
        "la bibliographie est retirée de la composition : {entry}"
    );
    assert!(
        entry.contains("#bibliography("),
        "les notices restent chargées pour les citations : {entry}"
    );
    let outcome = crate::typst::render::compile(&root, &entry, &corrections);
    assert!(
        outcome.errors.is_empty(),
        "la citation se résout : {:?}",
        outcome.errors
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn le_tag_bib_demande_la_section_bibliographique() {
    let root = std::env::temp_dir().join(format!(
        "typst-writer-bibliographie-tag-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("racine");
    std::fs::write(
        root.join("references.bib"),
        "@book{dupont2023, title={T}, author={Dupont, A.}, year={2023}}\n",
    )
    .expect("bibliographie");

    let document = crate::langage::parse("Voir @dupont2023.\n\n{{bib}}\n");
    let (entry, corrections, _) = compilation_unit_document_avec_diagnostics_explicite(
        &root,
        "#set page(margin: 2cm)\n",
        &document,
    );

    assert!(
        corrections
            .iter()
            .any(|(_, contenu)| contenu.contains("#bibliography")),
        "le tag demande la section visible : {corrections:?}"
    );
    assert!(
        !entry.contains("#show bibliography: it => none"),
        "aucune version cachée ne double la section : {entry}"
    );
    let outcome = crate::typst::render::compile(&root, &entry, &corrections);
    assert!(
        outcome.errors.is_empty(),
        "la section et la citation compilent : {:?}",
        outcome.errors
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn un_projet_sans_citation_et_sans_directive_compile() {
    let root = std::env::temp_dir().join(format!(
        "typst-writer-bibliographie-sans-citation-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("racine");
    std::fs::write(root.join("references.bib"), "% commentaire seulement\n")
        .expect("bibliographie locale");
    std::fs::write(
        root.join("references-zotero.bib"),
        "@book{dupont2023, title={T}, author={Dupont, A.}, year={2023}}\n",
    )
    .expect("export zotero");

    let document = crate::langage::parse("Un texte sans citation.\n");
    let (entry, corrections, _) = compilation_unit_document_avec_diagnostics_explicite(
        &root,
        "#set page(margin: 2cm)\n",
        &document,
    );

    assert!(
        !entry.contains("#hide["),
        "aucune bibliographie visible n'est inventée : {entry}"
    );
    assert!(
        entry.contains("#show bibliography: it => none"),
        "les deux bibliothèques restent disponibles sans occuper de place : {entry}"
    );
    let outcome = crate::typst::render::compile(&root, &entry, &corrections);
    assert!(
        outcome.errors.is_empty(),
        "le projet sans citation compile : {:?}",
        outcome.errors
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn un_chemin_bibliographique_mort_ne_casse_pas_les_citations() {
    let root = std::env::temp_dir().join(format!(
        "typst-writer-bibliographie-chemin-mort-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("racine");
    std::fs::write(
        root.join("references.bib"),
        "@book{dupont2023, title={T}, author={Dupont, A.}, year={2023}}\n",
    )
    .expect("bibliographie");

    let document = crate::langage::parse("Voir @dupont2023.\n");
    let main = "#set page(margin: 2cm)\n#bibliography(\"disparue.bib\")\n";
    let (entry, corrections, _) =
        compilation_unit_document_avec_diagnostics_explicite(&root, main, &document);

    assert!(
        !entry.contains("disparue.bib"),
        "le chemin mort ne reste pas dans l'entrée : {entry}"
    );
    assert!(
        entry.contains("#show bibliography: it => none")
            && entry.contains("#bibliography(\"references.bib\")"),
        "la bibliothèque vivante prend le relais : {entry}"
    );
    let outcome = crate::typst::render::compile(&root, &entry, &corrections);
    assert!(
        outcome.errors.is_empty(),
        "les citations se résolvent malgré le chemin mort : {:?}",
        outcome.errors
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn une_citation_sans_directive_ni_tag_trouve_sa_bibliographie_cachee() {
    let root = std::env::temp_dir().join(format!(
        "typst-writer-bibliographie-sans-directive-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("racine");
    std::fs::write(
        root.join("references.bib"),
        "@book{dupont2023, title={T}, author={Dupont, A.}, year={2023}}\n",
    )
    .expect("bibliographie");

    let document = crate::langage::parse("Voir @dupont2023.\n");
    let (entry, corrections, _) = compilation_unit_document_avec_diagnostics_explicite(
        &root,
        "#set page(margin: 2cm)\n",
        &document,
    );

    assert!(
        entry.contains("#show bibliography: it => none")
            && entry.contains("#bibliography(\"references.bib\")"),
        "l'assemblage neuf reçoit sa bibliothèque cachée : {entry}"
    );
    let outcome = crate::typst::render::compile(&root, &entry, &corrections);
    assert!(
        outcome.errors.is_empty(),
        "la citation se résout sans directive dans main.typ : {:?}",
        outcome.errors
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[cfg(test)]
mod tests_du_pipeline_vrai {
    /// Le pipeline et le compilateur travaillent sur un projet synthétique.
    #[test]
    fn un_projet_synthetique_compile() {
        if std::process::Command::new("typst")
            .arg("--version")
            .output()
            .is_err()
        {
            println!("typst n'est pas installé : test sauté");
            return;
        }
        let root = std::env::temp_dir().join(format!("tw-pipeline-fixture-{}", std::process::id()));
        std::fs::create_dir_all(&root).expect("dossier");
        std::fs::write(
            root.join("references.bib"),
            "@book{ouvrage, title = {Un ouvrage}, author = {Durand, Alice}, year = {2020}}\n",
        )
        .expect("bibliographie");
        let main = "#set text(lang: \"fr\")\n#bibliography(\"references.bib\")\n";
        let document = super::super::parse("# Chapitre\n\nUn texte de test.\n");
        let (entry, corrections) = super::compilation_unit_document(&root, main, &document);
        std::fs::write(root.join("main.typ"), entry).expect("entrée");
        for (chemin, contenu) in corrections {
            std::fs::write(chemin, contenu).expect("correction");
        }
        let rendu = std::process::Command::new("typst")
            .args(["compile", "main.typ"])
            .current_dir(&root)
            .output()
            .expect("typst");
        assert!(
            rendu.status.success(),
            "le projet compile : {}",
            String::from_utf8_lossy(&rendu.stderr)
        );
        assert!(root.join("main.pdf").is_file(), "le PDF est produit");
        std::fs::remove_dir_all(&root).expect("nettoyage");
    }
}

#[cfg(test)]
mod epreuves_de_mise_en_page {
    //! L'application réelle des normes, prouvée dans le PDF extrait — pas
    //! seulement dans le gabarit généré (R5, « réellement appliquée »).
    use super::plat;

    fn racine(nom: &str) -> std::path::PathBuf {
        let root =
            std::env::temp_dir().join(format!("typst-writer-mep-{nom}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("racine");
        root
    }

    /// L'outil de lecture du PDF est-il là ? Un test qui dépend d'un outil
    /// externe **saute** proprement quand il manque, il ne faillit pas —
    /// en CI comme ailleurs (règle des épreuves).
    fn pdftotext_disponible() -> bool {
        std::process::Command::new("pdftotext")
            .arg("-v")
            .output()
            .is_ok()
    }

    fn texte_extrait(root: &std::path::Path, source: &str) -> String {
        let pdf = crate::typst::render::render_pdf(root, source, &[]).expect("PDF");
        let fichier = root.join("sortie.pdf");
        std::fs::write(&fichier, &pdf).expect("PDF sur disque");

        let extrait = std::process::Command::new("pdftotext")
            .arg(&fichier)
            .arg("-")
            .output();
        let Ok(sortie) = extrait else {
            eprintln!("pdftotext absent — épreuve du texte extrait sautée");
            return String::new();
        };
        assert!(sortie.status.success(), "pdftotext s'exécute");
        String::from_utf8_lossy(&sortie.stdout).into_owned()
    }

    /// La casse d'un niveau est appliquée **au rendu** : le titre de
    /// niveau 1 est en capitales dans le PDF, celui de niveau 2 ne bouge
    /// pas — le texte extrait ne se compare pas en minuscules ici, c'est
    /// la casse elle-même qui est éprouvée.
    #[test]
    fn la_casse_par_niveau_s_applique_au_rendu() {
        if !pdftotext_disponible() {
            println!("pdftotext absent : test sauté");
            return;
        }

        let root = racine("casse");
        let mut normes = crate::mise_en_page::modele::MiseEnPage::default();
        normes.titres.casse_1 = Some("majuscules".to_string());

        std::fs::write(
            root.join(crate::mise_en_page::gabarit::FICHIER),
            crate::mise_en_page::gabarit::generer(&normes),
        )
        .expect("gabarit");

        let source = format!(
            "{}= Introduction\n\n== Partie\n",
            crate::mise_en_page::gabarit::accroche()
        );
        let texte = texte_extrait(&root, &source);
        assert!(!texte.is_empty(), "pdftotext présent");

        assert!(
            plat(&texte).contains("INTRODUCTION"),
            "le niveau 1 est en capitales : {texte}"
        );
        assert!(
            !plat(&texte).contains("Introduction"),
            "seule la capitale initiale n'a pas cédé : {texte}"
        );
        assert!(
            !plat(&texte).contains("PARTIE"),
            "le niveau 2 garde sa casse : {texte}"
        );
        assert!(
            plat(&texte).contains("Partie"),
            "le niveau 2 est là : {texte}"
        );
    }

    /// **Bug-report-8, T9.2** : le document de synthèse exploite toutes
    /// les nouveautés du rapport — image d'en-tête et sa hauteur, quick
    /// parts, numérotation personnalisée, couleurs de titres, italique et
    /// souligné, dates localisées — et le tout se compose pour de vrai.
    /// La vérification visuelle du résultat appartient à l'auteur.
    #[test]
    fn le_document_de_synthese_se_compose_pour_de_vrai() {
        for binaire in ["typst", "pdftotext"] {
            if std::process::Command::new(binaire)
                .arg("--version")
                .output()
                .is_err()
            {
                println!("{binaire} n'est pas installé : test sauté");
                return;
            }
        }

        let root = racine("synthese-8");

        // Les normes : tout ce que le rapport a ajouté.
        let mut normes = crate::mise_en_page::modele::MiseEnPage::default();
        normes.entete_pied.entete.gauche = "![logo](logo.png) — {auteur}".to_string();
        normes.entete_pied.images_hauteur = "60".to_string();
        normes.titres.numerotation = "1.1".to_string();
        normes.titres.italique_2 = Some(true);
        normes.titres.souligne_3 = Some(true);
        normes.titres.couleur_1 = Some("#335577".to_string());
        normes.dates.langue = "en".to_string();
        normes.couleurs.texte = "#222222".to_string();

        let gabarit = crate::mise_en_page::gabarit::generer(&normes);
        for attendu in [
            "height: 60%",
            "style: \"italic\"",
            "underline",
            "sys.inputs.at(\"auteur\"",
            "numbering: \"1.1\"",
            "#335577",
            "#222222",
        ] {
            assert!(gabarit.contains(attendu), "{attendu} manque : {gabarit}");
        }

        // Le corps : la balise date, localisée par les normes.
        let mut regles = normes.typographie.regles(&normes.references);
        regles.dates = crate::langage::dates::Regles {
            siecle: normes.dates.siecle.clone(),
            millenaire: normes.dates.millenaire.clone(),
            epoque_apres: normes.dates.epoque_apres.clone(),
            epoque_avant: normes.dates.epoque_avant.clone(),
            format: normes.dates.format.clone(),
            langue: normes.dates.langue.clone(),
        };
        let document = crate::langage::parse(
            "= Le titre\n\nLe {date:1789.07.14 \"dddd d MMMM yyyy\"}, soit le {date:XIXs?}.\n",
        );
        let corps = crate::langage::generer::generer_avec(&document, &regles, &normes.decor);
        assert!(
            corps.contains("Tuesday 14 July 1789"),
            "la date se localise : {corps}"
        );
        assert!(corps.contains("siècle"), "la période se rend : {corps}");

        // Le tout se compose au vrai Typst, avec les entrées du projet.
        std::fs::write(root.join(crate::mise_en_page::gabarit::FICHIER), &gabarit)
            .expect("gabarit");
        std::fs::write(
            root.join("logo.png"),
            [
                137u8, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0,
                0, 1, 8, 6, 0, 0, 0, 31, 21, 196, 137, 0, 0, 0, 13, 73, 68, 65, 84, 120, 218, 99,
                100, 96, 248, 95, 15, 0, 2, 135, 1, 128, 235, 71, 186, 146, 0, 0, 0, 0, 73, 69, 78,
                68, 174, 66, 96, 130,
            ],
        )
        .expect("image");
        let document =
            format!("#import \"mise-en-page.typ\": mise-en-page\n#show: mise-en-page\n{corps}\n");
        std::fs::write(root.join("le-document.typ"), document).expect("document");

        let sortie = std::process::Command::new("typst")
            .args([
                "compile",
                root.join("le-document.typ").to_str().unwrap(),
                "--input",
                "auteur=Alice",
            ])
            .output()
            .expect("typst s'exécute");
        assert!(
            sortie.status.success(),
            "le document de synthèse compile : {}",
            String::from_utf8_lossy(&sortie.stderr)
        );

        let texte = std::process::Command::new("pdftotext")
            .arg(root.join("le-document.pdf"))
            .arg("-")
            .output()
            .expect("pdftotext s'exécute");
        let texte = String::from_utf8_lossy(&texte.stdout).to_string();

        assert!(
            plat(&texte).contains("Alice"),
            "la quick part se rend : {texte}"
        );
        assert!(
            plat(&texte).contains(&plat("Tuesday 14 July 1789")),
            "la date localisée se rend : {texte}"
        );
        assert!(
            plat(&texte).contains(&plat("Le titre")),
            "le titre se rend : {texte}"
        );
    }

    /// Les quick parts vivent dans l'en-tête : `{projet}` vient de
    /// `sys.inputs`, `{titre}` est repris par `query` — et une image y a sa
    /// place, à la hauteur réglée (bug-report-8, T4.4).
    #[test]
    fn les_quick_parts_et_les_images_vivent_dans_l_entete() {
        if !pdftotext_disponible() {
            println!("pdftotext absent : test sauté");
            return;
        }

        let root = racine("quickparts");
        let mut normes = crate::mise_en_page::modele::MiseEnPage::default();
        normes.entete_pied.entete.gauche = "![logo](logo.png) {titre}".to_string();
        normes.entete_pied.pied.droite = "{projet} — p. {page}".to_string();
        normes.entete_pied.images_hauteur = "50".to_string();

        let code = crate::mise_en_page::gabarit::generer(&normes);
        assert!(
            code.contains("#image(\"logo.png\", height: 50%)"),
            "l'image suit la hauteur réglée : {code}"
        );

        // La hauteur est bornée : jamais de débordement de la zone.
        normes.entete_pied.images_hauteur = "500".to_string();
        let code = crate::mise_en_page::gabarit::generer(&normes);
        assert!(
            code.contains("height: 100%"),
            "la hauteur reste dans la zone : {code}"
        );

        std::fs::write(root.join(crate::mise_en_page::gabarit::FICHIER), &code).expect("gabarit");
        // Un vrai petit PNG : l'image doit se poser sans erreur de rendu.
        std::fs::write(
            root.join("logo.png"),
            [
                137u8, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0,
                0, 1, 8, 6, 0, 0, 0, 31, 21, 196, 137, 0, 0, 0, 13, 73, 68, 65, 84, 120, 218, 99,
                100, 96, 248, 95, 15, 0, 2, 135, 1, 128, 235, 71, 186, 146, 0, 0, 0, 0, 73, 69, 78,
                68, 174, 66, 96, 130,
            ],
        )
        .expect("logo");

        let source = format!(
            "{}= Un grand titre\n\nDu texte.\n",
            crate::mise_en_page::gabarit::accroche()
        );
        let texte = texte_extrait(&root, &source);
        assert!(!texte.is_empty(), "pdftotext présent");

        // {projet} vient de sys.inputs, alimenté par la racine elle-même.
        assert!(
            plat(&texte).contains("quickparts"),
            "le nom du projet vit dans l'en-tête : {texte}"
        );
        // {titre} est repris par le moteur, via query sur les titres.
        assert!(
            plat(&texte).contains(&plat("Un grand titre")),
            "le titre du document vit dans l'en-tête : {texte}"
        );
    }

    /// L'aperçu des titres est un **vrai rendu** : les mêmes normes
    /// donnent les mêmes pixels (le moteur est déterministe), et une norme
    /// différente donne une image différente — les pixels portent les
    /// normes, jamais une imitation (R5).
    #[test]
    fn l_apercu_des_titres_est_rendu_par_le_moteur() {
        let root = racine("apercu");

        let mut normes = crate::mise_en_page::modele::MiseEnPage::default();
        let premier = crate::mise_en_page::apercu_titres(&root, &normes).expect("rendu");
        let encore = crate::mise_en_page::apercu_titres(&root, &normes).expect("rendu");
        assert_eq!(
            premier.pixels, encore.pixels,
            "le moteur est déterministe : mêmes normes, mêmes pixels"
        );

        normes.titres.casse_1 = Some("majuscules".to_string());
        let autre = crate::mise_en_page::apercu_titres(&root, &normes).expect("rendu");
        assert_ne!(
            premier.pixels, autre.pixels,
            "la casse change l'image : les normes sont dans le rendu"
        );
    }

    /// L'audit de `cetz`, exigé par R5 **avant** de présenter les
    /// graphiques comme fiables : le moteur embarqué ne résout **pas** les
    /// paquets Typst (`@preview/…` cherche un fichier du projet). Les
    /// graphiques cetz ne sont donc pas fiables aujourd'hui, et le jour où
    /// le monde de rendu les résoudra, cette garde cassera — signal qu'il
    /// faut réévaluer la promesse, pas la faire en silence.
    #[test]
    fn cetz_ne_se_charge_pas_dans_le_moteur_embarque() {
        let root = racine("cetz");
        let source = "#import \"@preview/cetz:0.3.4\": canvas\n#canvas(length: 1cm, {})\n";
        let outcome = crate::typst::render::compile(&root, source, &[]);

        assert!(
            !outcome.errors.is_empty(),
            "cetz se charge soudain : réévaluer les graphiques avant d'en parler"
        );
        assert!(
            outcome
                .errors
                .iter()
                .any(|diagnostic| diagnostic.message.contains("file not found")),
            "le refus est un diagnostic lu, pas un silence : {:?}",
            outcome.errors
        );
    }

    /// Les champs dynamiques du pied sont résolus **par le moteur** :
    /// `{page}` et `{pages}` deviennent les vrais numéros du PDF — jamais
    /// imités, jamais laissés en clair (R5).
    #[test]
    fn les_champs_dynamiques_se_resolvent_par_le_moteur() {
        if !pdftotext_disponible() {
            println!("pdftotext absent : test sauté");
            return;
        }

        let root = racine("champs");
        let mut normes = crate::mise_en_page::modele::MiseEnPage::default();
        normes.entete_pied.pied.centre = "p. {page} — {pages} pages".to_string();

        std::fs::write(
            root.join(crate::mise_en_page::gabarit::FICHIER),
            crate::mise_en_page::gabarit::generer(&normes),
        )
        .expect("gabarit");

        let source = format!(
            "{}Première page\n\n#pagebreak()\n\nSeconde page\n",
            crate::mise_en_page::gabarit::accroche()
        );
        let texte = texte_extrait(&root, &source);
        assert!(!texte.is_empty(), "pdftotext présent");

        assert!(
            plat(&texte).contains(&plat("p. 1 — 2 pages")),
            "la première page dit ses numéros : {texte}"
        );
        assert!(
            plat(&texte).contains(&plat("p. 2 — 2 pages")),
            "la seconde page dit ses numéros : {texte}"
        );
    }
}

#[cfg(test)]
mod epreuves_de_renvois {
    //! Notes, légendes, références croisées : ce que R5 promet « accessible »
    //! est vérifié au rendu — et un renvoi cassé est un diagnostic, jamais un
    //! silence.

    fn racine(nom: &str) -> std::path::PathBuf {
        let root =
            std::env::temp_dir().join(format!("typst-writer-renvois-{nom}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("racine");
        root
    }

    /// Un renvoi à une figure labellisée n'est **jamais** une citation : il
    /// traverse le correcteur intact et se résout au rendu.
    #[test]
    fn un_renvoi_a_une_figure_se_resout_au_rendu() {
        let root = racine("figure");
        let (entry, corrections, rapports) = super::compilation_unit_avec_diagnostics(
            &root,
            "#figure(rect(width: 1cm), caption: [Légende partagée]) <fig:une>\n\nVoir @fig:une.\n",
            "",
        );

        assert!(
            entry.contains("@fig:une"),
            "le renvoi traverse le correcteur intact : {entry}"
        );
        assert!(
            !rapports
                .iter()
                .any(|diagnostic| diagnostic.message.contains("neutralisée")),
            "aucun renvoi neutralisé : {rapports:?}"
        );

        let outcome = crate::typst::render::compile(&root, &entry, &corrections);
        assert!(
            outcome.errors.is_empty(),
            "le renvoi se compile : {:?}",
            outcome.errors
        );
    }

    /// Un lien interne demande le numéro du chapitre ou la page : le moteur
    /// Typst les compose à partir des compteurs, sans numérotation écrite
    /// dans la source.
    #[test]
    fn un_lien_interne_affiche_le_numero_du_chapitre_et_la_page() {
        let root = racine("numerotation");
        let mut normes = crate::mise_en_page::modele::MiseEnPage {
            sommaire: false,
            ..Default::default()
        };
        normes.titres.numerotation = "1.1".to_string();
        std::fs::write(
            root.join(crate::mise_en_page::gabarit::FICHIER),
            crate::mise_en_page::gabarit::generer(&normes),
        )
        .expect("gabarit");

        let (entry, corrections) = super::compilation_unit(
            &root,
            "#import \"mise-en-page.typ\": mise-en-page\n#show: mise-en-page\n",
            "# Adoption\n\n## Coutume\n\nVoir [[titre-coutume|\"chapitre\"]] à la [[titre-coutume|\"page\"]].\n",
        );
        let outcome = crate::typst::render::compile(&root, &entry, &corrections);
        assert!(
            outcome.errors.is_empty(),
            "les liens numérotés compilent : {:?}",
            outcome.errors
        );
        let document = outcome.document.expect("document");
        let pdf = crate::typst::render::pdf_du_document(&document).expect("PDF");
        let fichier = root.join("sortie.pdf");
        std::fs::write(&fichier, &pdf).expect("PDF sur disque");

        let Ok(sortie) = std::process::Command::new("pdftotext")
            .arg(&fichier)
            .arg("-")
            .output()
        else {
            eprintln!("pdftotext absent — épreuve du texte sautée");
            return;
        };
        let texte = String::from_utf8_lossy(&sortie.stdout);
        assert!(
            texte.contains("chapitre 1.1"),
            "le numéro du chapitre est composé : {texte}"
        );
        assert!(texte.contains("page 1"), "la page est composée : {texte}");
    }

    /// Une figure est appelée par son numéro composé, et sa page se lit
    /// comme celle de n'importe quel élément.
    #[test]
    fn un_lien_interne_affiche_le_numero_de_figure_et_sa_page() {
        let root = racine("figure-numerotee");
        let mut normes = crate::mise_en_page::modele::MiseEnPage {
            sommaire: false,
            ..Default::default()
        };
        normes.titres.numerotation = "1.1".to_string();
        normes.legende_prefixe = "Figure".to_string();
        normes.numerotation_figures = "1".to_string();
        std::fs::write(
            root.join(crate::mise_en_page::gabarit::FICHIER),
            crate::mise_en_page::gabarit::generer(&normes),
        )
        .expect("gabarit");
        std::fs::write(
            root.join("carte.png"),
            [
                137u8, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0,
                0, 1, 8, 6, 0, 0, 0, 31, 21, 196, 137, 0, 0, 0, 13, 73, 68, 65, 84, 120, 218, 99,
                100, 96, 248, 95, 15, 0, 2, 135, 1, 128, 235, 71, 186, 146, 0, 0, 0, 0, 73, 69, 78,
                68, 174, 66, 96, 130,
            ],
        )
        .expect("image");

        let (entry, corrections) = super::compilation_unit(
            &root,
            "#import \"mise-en-page.typ\": mise-en-page\n#show: mise-en-page\n",
            "# Une partie\n\n![Une carte](carte.png)\n\nVoir [[une-carte|\"figure\"]] à la [[une-carte|\"page\"]].\n",
        );
        let outcome = crate::typst::render::compile(&root, &entry, &corrections);
        assert!(
            outcome.errors.is_empty(),
            "les liens de figure compilent : {:?}",
            outcome.errors
        );
        let document = outcome.document.expect("document");
        let pdf = crate::typst::render::pdf_du_document(&document).expect("PDF");
        let fichier = root.join("sortie.pdf");
        std::fs::write(&fichier, &pdf).expect("PDF sur disque");

        let Ok(sortie) = std::process::Command::new("pdftotext")
            .arg(&fichier)
            .arg("-")
            .output()
        else {
            eprintln!("pdftotext absent — épreuve du texte sautée");
            return;
        };
        let texte = String::from_utf8_lossy(&sortie.stdout);
        assert!(
            texte.contains("Figure 1"),
            "le numéro de figure est composé : {texte}"
        );
        assert!(texte.contains("page 1"), "la page est composée : {texte}");
    }

    /// Une étiquette du document est une cible de renvoi, pas une citation
    /// manquante : `@chapitre` vers `<chapitre>` ne se neutralise jamais.
    #[test]
    fn une_etiquette_du_document_est_un_renvoi() {
        let root = racine("etiquette");
        let (entry, _, rapports) = super::compilation_unit_avec_diagnostics(
            &root,
            "= Un chapitre <chapitre>\n\nVoir @chapitre.\n",
            "",
        );

        assert!(entry.contains("@chapitre"), "le renvoi survit : {entry}");
        assert!(
            !rapports
                .iter()
                .any(|diagnostic| diagnostic.message.contains("neutralisée")),
            "aucun renvoi neutralisé : {rapports:?}"
        );
    }

    /// Un renvoi sans cible se **diagnostique** (par le moteur, avec son
    /// nom) — jamais neutralisé en silence, jamais un faux `[cle]`.
    #[test]
    fn un_renvoi_sans_cible_est_diagnostique() {
        let root = racine("sans-cible");
        let (entry, corrections, rapports) =
            super::compilation_unit_avec_diagnostics(&root, "Voir @fig:introuvable.\n", "");

        assert!(
            entry.contains("@fig:introuvable"),
            "le renvoi n'est pas déguisé en texte : {entry}"
        );
        assert!(
            !rapports
                .iter()
                .any(|diagnostic| diagnostic.message.contains("neutralisée")),
            "pas de neutralisation : {rapports:?}"
        );

        let outcome = crate::typst::render::compile(&root, &entry, &corrections);
        assert!(
            outcome
                .errors
                .iter()
                .any(|diagnostic| diagnostic.message.contains("fig:introuvable")),
            "le moteur nomme la cible manquante : {:?}",
            outcome.errors
        );
    }

    /// Une note de bas de page se lit dans le PDF — la note exacte, pas
    /// seulement un compilateur content.
    #[test]
    fn une_note_de_bas_de_page_se_lit_dans_le_pdf() {
        let root = racine("note");
        let pdf = crate::typst::render::render_pdf(
            &root,
            "Texte avec note#footnote[La note exacte.] suite.\n",
            &[],
        )
        .expect("PDF");
        let fichier = root.join("sortie.pdf");
        std::fs::write(&fichier, &pdf).expect("PDF sur disque");

        let extrait = std::process::Command::new("pdftotext")
            .arg(&fichier)
            .arg("-")
            .output();
        let Ok(sortie) = extrait else {
            eprintln!("pdftotext absent — épreuve du texte extrait sautée");
            return;
        };
        assert!(sortie.status.success(), "pdftotext s'exécute");
        let texte = String::from_utf8_lossy(&sortie.stdout);

        assert!(
            texte.contains("La note exacte."),
            "la note est dans le PDF : {texte}"
        );
    }

    /// Une citation **dans une note de bas de page** compile réellement et
    /// laisse sa note dans le PDF : le contenu d'une note est un corps Typst
    /// comme un autre, pas une zone opaque.
    #[test]
    fn une_citation_dans_une_note_se_lit_dans_le_pdf() {
        let root = racine("citation-note");
        std::fs::write(
            root.join("references.bib"),
            "@book{dupont2023, title = {Un titre}, author = {Dupont, Jean}, year = {2023}}\n",
        )
        .expect("bibliographie");
        let source = "#bibliography(\"references.bib\", style: \"chicago-notes\")\n\nTexte#footnote[Voir @dupont2023[p. 45].] suite.\n";
        let pdf = crate::typst::render::render_pdf(&root, source, &[]).expect("PDF");
        let fichier = root.join("sortie.pdf");
        std::fs::write(&fichier, &pdf).expect("PDF sur disque");

        let extrait = std::process::Command::new("pdftotext")
            .arg(&fichier)
            .arg("-")
            .output();
        let Ok(sortie) = extrait else {
            eprintln!("pdftotext absent — épreuve du texte extrait sautée");
            return;
        };
        assert!(sortie.status.success(), "pdftotext s'exécute");
        let texte = String::from_utf8_lossy(&sortie.stdout);

        assert!(
            texte.contains("Voir") && texte.contains("45"),
            "la note et sa page sont dans le PDF : {texte}"
        );
    }

    /// La citation dans une note traverse **tout le chemin applicatif** :
    /// `.tw` → parseur → générateur → assemblage → moteur Typst. Ce n'est
    /// pas seulement du Typst écrit à la main dans une épreuve.
    #[test]
    fn une_citation_dans_une_note_traverse_le_pipeline_et_compile() {
        let root = racine("citation-note-pipeline");
        std::fs::write(
            root.join("references.bib"),
            "@book{dupont2023, title = {Un titre}, author = {Dupont, Jean}, year = {2023}}\n",
        )
        .expect("bibliographie");

        let (entree, corrections) = super::compilation_unit(
            &root,
            "#bibliography(\"references.bib\", style: \"chicago-notes\")\n",
            "Texte^[Voir @dupont2023[p. 45].] suite.\n",
        );
        let outcome = crate::typst::render::compile(&root, &entree, &corrections);

        assert!(
            outcome.errors.is_empty(),
            "la citation en note compile par le pipeline : {:?}",
            outcome.errors
        );
    }

    /// La légende d'une figure se lit dans le PDF.
    #[test]
    fn une_legende_de_figure_se_lit_dans_le_pdf() {
        let root = racine("legende");
        let pdf = crate::typst::render::render_pdf(
            &root,
            "#figure(rect(width: 1cm), caption: [La légende exacte.])\n",
            &[],
        )
        .expect("PDF");
        let fichier = root.join("sortie.pdf");
        std::fs::write(&fichier, &pdf).expect("PDF sur disque");

        let extrait = std::process::Command::new("pdftotext")
            .arg(&fichier)
            .arg("-")
            .output();
        let Ok(sortie) = extrait else {
            eprintln!("pdftotext absent — épreuve du texte extrait sautée");
            return;
        };
        assert!(sortie.status.success(), "pdftotext s'exécute");
        let texte = String::from_utf8_lossy(&sortie.stdout);

        assert!(
            texte.contains("La légende exacte."),
            "la légende est dans le PDF : {texte}"
        );
    }
}

/// Le glossaire glossarium : l'assemblage, la guérison, et le rendu réel.
#[cfg(test)]
mod tests_glossaire {
    use std::path::PathBuf;

    /// Le dossier des paquets de test : **partagé** et posé une seule fois
    /// — l'environnement est un état partagé, et `Once` est la barrière
    /// qui rend l'écriture unique et sa lecture postérieure sûre.
    fn paquets_de_test() -> PathBuf {
        static UNE_FOIS: std::sync::Once = std::sync::Once::new();
        let chemin =
            std::env::temp_dir().join(format!("typst-writer-paquets-tests-{}", std::process::id()));
        UNE_FOIS.call_once(|| {
            // SAFETY : une seule écriture du processus, sous `Once`, et
            // tout lecteur passe par la même barrière avant de lire.
            unsafe { std::env::set_var("TYPST_WRITER_PAQUETS", &chemin) };
        });
        chemin
    }

    fn racine(nom: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("typst-writer-mep-glo-{nom}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("racine du test");
        root
    }

    /// Le généré, tiré des corrections de l'assemblage.
    fn le_genere(corrections: &[(PathBuf, String)]) -> String {
        corrections
            .iter()
            .find(|(chemin, _)| chemin.ends_with("document.generated.typ"))
            .map(|(_, contenu)| contenu.clone())
            .expect("le généré")
    }

    const GLOSSAIRE: &str = r#"
[[entrees]]
key = "sdo"
short = "sdo"
long = "docteur scolastique"
plural = "sdo"
longplural = "docteurs scolastiques"
description = "Un théologien du Moyen Âge."
"#;

    /// L'assemblage pose le salon dans l'entrée et le registre dans le
    /// généré — et le document compile **au vrai moteur**.
    #[test]
    fn le_glossaire_s_assemble_et_le_document_compile() {
        let paquets = paquets_de_test();
        let root = racine("assemble");
        std::fs::write(root.join(crate::project::glossaire::FICHIER), GLOSSAIRE)
            .expect("glossaire");

        let main = "#import \"mise-en-page.typ\": mise-en-page\n#show: mise-en-page\n";
        std::fs::write(
            root.join("mise-en-page.typ"),
            crate::mise_en_page::gabarit::generer(
                &crate::mise_en_page::modele::MiseEnPage::default(),
            ),
        )
        .expect("gabarit");

        let (entry, corrections, _) = super::compilation_unit_avec_diagnostics(
            &root,
            main,
            "Les {glo:sdo} ici.\n\n{glossaire}\n",
        );

        // Le salon, dans l'entrée — et **sous** la règle de mise en page :
        // les règles de figures d'abord, comme glossarium le veut.
        let pos_mep = entry.find("#show: mise-en-page").expect("mise-en-page");
        let pos_glo = entry.find("#show: make-glossary").expect("make-glossary");
        assert!(
            pos_mep < pos_glo,
            "le salon vient après la mise en page : {entry}"
        );

        // Le registre, dans le généré — sa portée est celle du fichier.
        let genere = le_genere(&corrections);
        assert!(genere.contains("#let tw-glossaire = "), "{genere}");
        assert!(
            genere.contains("#register-glossary(tw-glossaire)"),
            "{genere}"
        );
        assert!(genere.contains("#gls(\"sdo\")"), "{genere}");
        assert!(genere.contains("#print-glossary(tw-glossaire)"), "{genere}");

        // Le paquet est posé localement — jamais au réseau.
        assert!(
            paquets
                .join("local/glossarium/0.5.10/glossarium.typ")
                .is_file(),
            "le paquet s'est installé dans le dossier local"
        );

        // Le document compile, au moteur embarqué.
        for (chemin, contenu) in &corrections {
            std::fs::write(chemin, contenu).expect("fichier assemblé");
        }
        let outcome = crate::typst::render::compile(&root, &entry, &[]);
        assert!(
            outcome.errors.is_empty(),
            "le document au glossaire compile : {:?}",
            outcome.errors
        );
    }

    /// Un terme inconnu se neutralise en `[clé]` — le document compile
    /// toujours, et le manque se dit.
    #[test]
    fn un_terme_inconnu_ne_casse_jamais_le_document() {
        let paquets = paquets_de_test();
        let _ = &paquets;
        let root = racine("inconnu");
        std::fs::write(
            root.join(crate::project::glossaire::FICHIER),
            "[[entrees]]\nkey = \"vivant\"\n",
        )
        .expect("glossaire");
        std::fs::write(
            root.join("mise-en-page.typ"),
            crate::mise_en_page::gabarit::generer(
                &crate::mise_en_page::modele::MiseEnPage::default(),
            ),
        )
        .expect("gabarit");

        let (entry, corrections, rapports) = super::compilation_unit_avec_diagnostics(
            &root,
            "#import \"mise-en-page.typ\": mise-en-page\n#show: mise-en-page\n",
            "Le {glo:inconnu} et le {glo:vivant} aussi.\n",
        );

        let genere = le_genere(&corrections);
        assert!(genere.contains("[inconnu]"), "neutralisé : {genere}");
        assert!(!genere.contains("#gls(\"inconnu\")"), "{genere}");
        assert!(
            genere.contains("#gls(\"vivant\", link: false)"),
            "le vivant reste, sans lien faute de liste : {genere}"
        );
        assert!(
            rapports.iter().any(|d| d.message.contains("inconnu")),
            "le manque se dit : {rapports:?}"
        );

        for (chemin, contenu) in &corrections {
            std::fs::write(chemin, contenu).expect("fichier assemblé");
        }
        let outcome = crate::typst::render::compile(&root, &entry, &[]);
        assert!(
            outcome.errors.is_empty(),
            "le neutralisé compile aussi : {:?}",
            outcome.errors
        );
    }

    /// Le rendu **réel** : le CLI Typst compose le glossaire, pdftotext
    /// lit ce qui s'est écrit. La sonde d'abord — les assertions sont
    /// façonnées sur l'observé.
    #[test]
    fn le_glossaire_se_rend_pour_de_vrai() {
        for binaire in ["typst", "pdftotext"] {
            if std::process::Command::new(binaire)
                .arg("--version")
                .output()
                .is_err()
            {
                println!("{binaire} n'est pas installé : test sauté");
                return;
            }
        }

        let paquets = paquets_de_test();
        let root = racine("rendu");
        std::fs::write(root.join(crate::project::glossaire::FICHIER), GLOSSAIRE)
            .expect("glossaire");
        std::fs::write(
            root.join("mise-en-page.typ"),
            crate::mise_en_page::gabarit::generer(
                &crate::mise_en_page::modele::MiseEnPage::default(),
            ),
        )
        .expect("gabarit");

        let (entry, corrections, _) = super::compilation_unit_avec_diagnostics(
            &root,
            "#import \"mise-en-page.typ\": mise-en-page\n#show: mise-en-page\n",
            "Le {glo:sdo} d'abord, les {glo:sdo pluriel} ensuite.\n\n{glossaire: tout}\n",
        );
        for (chemin, contenu) in &corrections {
            std::fs::write(chemin, contenu).expect("fichier assemblé");
        }
        std::fs::write(root.join("main.typ"), &entry).expect("entrée");

        // Le CLI voit les paquets sous XDG_DATA_HOME/typst/packages.
        let data = root.join("data");
        crate::typst::paquets::installer_dans(
            &data.join("typst/packages"),
            crate::typst::paquets::GLOSSARIUM,
            crate::typst::paquets::GLOSSARIUM_FICHIERS,
        )
        .expect("installation pour le CLI");
        assert!(paquets.join("local").exists(), "l'app a posé le sien");

        let sortie = std::process::Command::new("typst")
            .args(["compile", "main.typ", "sortie.pdf"])
            .current_dir(&root)
            .env("XDG_DATA_HOME", &data)
            .output()
            .expect("typst s'exécute");
        assert!(
            sortie.status.success(),
            "le CLI compose : {}",
            String::from_utf8_lossy(&sortie.stderr)
        );

        let texte = std::process::Command::new("pdftotext")
            .arg(root.join("sortie.pdf"))
            .arg("-")
            .output()
            .expect("pdftotext s'exécute");
        let texte = String::from_utf8_lossy(&texte.stdout).to_string();

        // Façonné sur le rendu réel (sonde retirée) : la première
        // occurrence développe la forme longue, la reprise garde le
        // sigle, la liste nomme le terme.
        assert!(
            texte.contains("docteur scolastique (sdo)"),
            "la première occurrence se développe : {texte}"
        );
        assert!(
            texte.contains("les sdo ensuite"),
            "le pluriel se rend : {texte}"
        );
        assert!(
            texte.contains("sdo – docteur scolastique"),
            "la liste nomme le terme : {texte}"
        );
        assert!(
            texte.contains("Un théologien du Moyen Âge."),
            "la description se lit : {texte}"
        );
    }
}

/// **Bug-report-9, T1.2 et T1.4** : toutes les propriétés configurées
/// dans les normes correspondent au rendu final — éprouvé au vrai moteur,
/// sur toute la chaîne : appliquer → assembler → composer. L'assemblage
/// porte même un `#set text` hostile, comme les projets réels : les
/// normes doivent l'emporter.
#[cfg(test)]
mod les_normes_se_rendent_au_moteur {
    use std::path::{Path, PathBuf};

    fn racine(nom: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "typst-writer-normes-rendues-{nom}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("racine du test");
        root
    }

    /// Une trame de texte, telle que le rendu la dessine.
    #[derive(Debug)]
    struct Trame {
        texte: String,
        gras: bool,
        italique: bool,
        remplissage: String,
        police: String,
        traits: usize,
    }

    fn sonder(frame: &typst::layout::Frame, sortie: &mut Vec<Trame>) {
        for (_, item) in frame.items() {
            match item {
                typst::layout::FrameItem::Text(texte) => {
                    let variante = texte.font.font().info().variant;
                    sortie.push(Trame {
                        texte: texte.text.to_string(),
                        gras: variante.weight >= typst::text::FontWeight::BOLD,
                        italique: variante.style != typst::text::FontStyle::Normal,
                        remplissage: format!("{:?}", texte.fill),
                        police: texte.font.font().info().family.clone(),
                        traits: 0,
                    });
                }
                typst::layout::FrameItem::Group(groupe) => sonder(&groupe.frame, sortie),
                typst::layout::FrameItem::Shape(forme, _) => {
                    if let Some(trame) = sortie.last_mut()
                        && matches!(forme.geometry, typst::visualize::Geometry::Line(_))
                    {
                        trame.traits += 1;
                    }
                }
                _ => {}
            }
        }
    }

    /// La chaîne réelle : la norme s'applique au projet, le projet
    /// s'assemble avec le gabarit d'origine, et le moteur compose.
    fn rendu(root: &Path, source: &str) -> Vec<Trame> {
        // Le vrai assemblage : l'accroche devant le gabarit d'origine —
        // avec son `#set text` hostile, comme le projet réel.
        let main = format!(
            "{}\n{}",
            crate::mise_en_page::gabarit::accroche(),
            crate::project::model::MAIN_TEMPLATE
        );
        let (entry, corrections, _) = super::compilation_unit_avec_diagnostics(root, &main, source);
        for (chemin, contenu) in &corrections {
            std::fs::write(chemin, contenu).expect("fichier assemblé");
        }

        let outcome = crate::typst::render::compile(root, &entry, &[]);
        assert!(
            outcome.errors.is_empty(),
            "ça compile : {:?}",
            outcome.errors
        );

        let mut trames = Vec::new();
        for page in outcome.document.expect("document").pages() {
            sonder(&page.frame, &mut trames);
        }
        trames
    }

    fn trame<'a>(trames: &'a [Trame], texte: &str) -> &'a Trame {
        trames
            .iter()
            .find(|trame| trame.texte == texte)
            .unwrap_or_else(|| panic!("la trame {texte:?} est là"))
    }

    /// T1.2 : les styles de titres partent au moteur — gras y compris
    /// dans sa désactivation, que le gras par défaut des titres ne
    /// devrait pas survivre.
    #[test]
    fn les_styles_de_titres_se_rendent_au_moteur() {
        let root = racine("styles");

        let mut normes = crate::mise_en_page::modele::MiseEnPage::default();
        normes.titres.gras_1 = Some(false); // le gras désactivé s'enlève
        normes.titres.taille_2 = "18pt".to_string();
        normes.titres.gras_2 = Some(true);
        normes.titres.italique_2 = Some(true);
        normes.titres.souligne_2 = Some(true);
        normes.titres.couleur_2 = Some("#cc0000".to_string());
        normes.titres.casse_2 = Some("majuscules".to_string());

        crate::mise_en_page::appliquer(&root, &normes, false).expect("application");
        let trames = rendu(
            &root,
            "# Un titre\n\n## Un sous-titre\n\nDu texte courant.\n",
        );

        // Le niveau 1 : le gras désactivé s'enlève.
        let titre = trame(&trames, "Un titre");
        assert!(!titre.gras, "le gras désactivé s'enlève : {titre:?}");
        assert!(!titre.italique, "le niveau 1 reste nu : {titre:?}");

        // Le niveau 2 : gras, italique, couleur, soulignement, casse.
        let sous = trame(&trames, "UN SOUS-TITRE");
        assert!(sous.gras, "le gras s'applique : {sous:?}");
        assert!(sous.italique, "l'italique s'applique : {sous:?}");
        assert!(
            sous.remplissage.contains("0.8, 0, 0"),
            "la couleur s'applique : {sous:?}"
        );
        assert!(sous.traits >= 1, "le soulignement s'applique : {sous:?}");
    }

    /// T1.4 : la police des normes l'emporte sur le `#set text` de
    /// l'assemblage — quand le système en donne une, sinon la
    /// vérification n'a pas de repère.
    #[test]
    fn la_police_des_normes_l_emporte_sur_l_assemblage() {
        let root = racine("police");
        // Le contrat : la police des normes l'emporte sur celle de
        // l'assemblage — sans comparer de nom de famille, que chaque
        // plateforme résout à sa façon.
        crate::mise_en_page::appliquer(&root, &Default::default(), false)
            .expect("application par défaut");
        let trames_defaut = rendu(&root, "# Un titre\n\nDu texte courant.\n");
        let corps_defaut = trames_defaut
            .iter()
            .find(|trame| trame.texte.starts_with("Du texte"))
            .expect("le corps est là")
            .police
            .clone();
        // Une famille qui n'est pas celle du défaut — sinon la comparaison
        // ne dirait rien.
        let Some(police) = crate::typst::fonts::familles_pour_choisir()
            .into_iter()
            .find(|p| *p != corps_defaut)
        else {
            println!("une seule famille système : test sauté");
            return;
        };

        let mut normes = crate::mise_en_page::modele::MiseEnPage::default();
        normes.texte.police = police.clone();
        crate::mise_en_page::appliquer(&root, &normes, false).expect("application");

        let trames = rendu(&root, "# Un titre\n\nDu texte courant.\n");
        let corps = trames
            .iter()
            .find(|trame| trame.texte.starts_with("Du texte"))
            .expect("le corps est là");
        assert_ne!(
            corps.police, corps_defaut,
            "la police des normes l'emporte sur l'assemblage : {corps:?}"
        );
    }

    /// Les couleurs de gras, italique et liens arrivent au moteur.
    #[test]
    fn les_couleurs_de_style_se_rendent_au_moteur() {
        let root = racine("couleurs-style");

        let mut normes = crate::mise_en_page::modele::MiseEnPage::default();
        normes.couleurs.gras = "#cc0000".to_string();
        normes.couleurs.italique = "#00cc00".to_string();
        normes.couleurs.liens = "#0000cc".to_string();

        crate::mise_en_page::appliquer(&root, &normes, false).expect("application");
        // La langue du document : `**gras**`, `*italique*`, `[texte](cible)`.
        let trames = rendu(
            &root,
            "Du texte **gras**, un *italique* et un [lien](https://example.org).\n",
        );

        // Observé avant le correctif : Rgb(0, 0, 0, 1) partout — le noir
        // du texte, les couleurs de style écrasées par `fn texte`.
        assert_eq!(
            trame(&trames, "gras").remplissage,
            "Rgb(0.8, 0, 0, 1)",
            "le gras porte sa couleur : {:?}",
            trame(&trames, "gras")
        );
        assert_eq!(
            trame(&trames, "italique").remplissage,
            "Rgb(0, 0.8, 0, 1)",
            "l'italique porte sa couleur : {:?}",
            trame(&trames, "italique")
        );
        assert_eq!(
            trame(&trames, "lien").remplissage,
            "Rgb(0, 0, 0.8, 1)",
            "le lien porte sa couleur : {:?}",
            trame(&trames, "lien")
        );
    }

    /// Deux bibliothèques qui partagent une clé ne cassent plus la
    /// compilation — la première garde son entrée (bug-report-10, 6.1).
    #[test]
    fn deux_bibliotheques_qui_partagent_une_cle_ne_cassent_plus() {
        let root = std::env::temp_dir().join("tw-partage-cle");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("biblio")).unwrap();
        std::fs::write(
            root.join("biblio/a.bib"),
            "@book{dupont, title = {Premier}, author = {Dupont}, year = {2020}}",
        )
        .unwrap();
        std::fs::write(
            root.join("biblio/b.bib"),
            "@book{dupont, title = {Second}, author = {Dupont}, year = {2021}}",
        )
        .unwrap();
        let entry = concat!(
            "#bibliography(\"biblio/a.bib\")",
            "\n",
            "#bibliography(\"biblio/b.bib\")"
        )
        .to_string();
        let guerison = super::guerir_les_bibliographies(&root, &entry, &[]);
        // La première garde son entrée : elle ne bouge pas.
        assert!(
            guerison
                .iter()
                .all(|(chemin, _)| !chemin.ends_with("a.bib")),
            "la première garde son entrée : {:?}",
            guerison
        );
        // La seconde est guérie : la clé en double en part.
        let seconde = guerison
            .iter()
            .find(|(chemin, _)| chemin.ends_with("b.bib"));
        assert!(seconde.is_some(), "la seconde est guérie : {:?}", guerison);
        assert!(
            !seconde.unwrap().1.contains("dupont"),
            "la clé en double est partie"
        );
    }
}

#[cfg(test)]
mod tests_pipeline_complet {
    /// Le pipeline complet (feuille de route, phase 6, point 6.3) : du `.tw`
    /// au PDF, réellement — et l'erreur de compilation remonte.
    #[test]
    fn le_pipeline_complet_rend_un_pdf() {
        let source = "Bonjour.\n\nUn paragraphe simple.\n";
        let document = crate::langage::parse(source);
        let typst = crate::langage::generer::generer(&document);
        let root = std::path::PathBuf::from("/projet");
        let entry = "#include \"chapitre.typ\"".to_string();
        let overrides = vec![(root.join("chapitre.typ"), typst)];

        let outcome = crate::typst::render::compile(&root, &entry, &overrides);
        assert!(
            outcome.errors.is_empty(),
            "la compilation est sans erreur : {:?}",
            outcome.errors
        );

        let octets =
            crate::typst::render::render_pdf(&root, &entry, &overrides).expect("le PDF sort");
        assert!(octets.starts_with(b"%PDF"), "c'est bien un PDF");

        // L'erreur de compilation est remontée, pas avalée.
        let casse = vec![(root.join("chapitre.typ"), "#(".to_string())];
        let outcome = crate::typst::render::compile(&root, &entry, &casse);
        assert!(
            !outcome.errors.is_empty(),
            "une erreur de compilation remonte"
        );
    }
}

/// **Pages de garde** : elles précèdent le sommaire, ne portent pas de numéro
/// et ne descendent pas dans la table des matières.
#[cfg(test)]
mod pages_de_garde {
    use std::path::PathBuf;

    use super::GARDE_FILE;

    fn racine(nom: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("typst-writer-gardes-{nom}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("racine");
        root
    }

    #[test]
    fn une_ancienne_page_de_titre_perd_son_numero() {
        let ancien =
            "  page(header: none, footer: none)[\n    Titre\n  ]\n  pagebreak()\n  outline()\n";
        let corrige = super::sans_numero_de_la_page_de_titre(ancien).expect("page de titre");
        assert!(
            corrige.contains("page(numbering: none, header: none, footer: none)"),
            "{corrige}"
        );
        assert!(
            corrige.contains("pagebreak()\n  counter(page).update(1)"),
            "{corrige}"
        );
        assert!(
            super::sans_numero_de_la_page_de_titre(&corrige).is_none(),
            "la correction est idempotente"
        );
    }

    #[test]
    fn une_page_de_titre_vide_ne_laisse_pas_de_page_blanche() {
        let root = racine("titre-vide");
        std::fs::write(
            root.join(crate::mise_en_page::REGLAGES),
            "page_de_titre = true\n",
        )
        .expect("normes");
        let normes = crate::mise_en_page::modele::MiseEnPage {
            sommaire: false,
            page_de_titre: true,
            ..Default::default()
        };
        std::fs::write(
            root.join(crate::mise_en_page::gabarit::FICHIER),
            crate::mise_en_page::gabarit::generer(&normes),
        )
        .expect("gabarit");
        let main = "#import \"mise-en-page.typ\": mise-en-page\n#show: mise-en-page\n#include \"document.generated.typ\"\n";
        let source = "{{garde}}\n# La page de garde\n\nTexte de garde.\n{{/garde}}\n\n# Introduction\n\nLe corps.\n";
        let (entry, corrections) = super::compilation_unit(&root, main, source);
        let outcome = crate::typst::render::compile(&root, &entry, &corrections);
        assert!(
            outcome.errors.is_empty(),
            "la garde compile : {:?}",
            outcome.errors
        );
        let document = outcome.document.expect("document");
        assert_eq!(
            crate::typst::render::page_count(&document),
            2,
            "la garde et le corps, sans page de titre vide"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn une_page_de_titre_en_blancs_ne_cree_pas_une_page_vide() {
        let racine = racine("titre-blancs-gardes");
        std::fs::write(
            racine.join(crate::mise_en_page::REGLAGES),
            "page_de_titre = true\n",
        )
        .expect("normes");
        std::fs::write(
            racine.join(crate::project::metadonnees::FICHIER),
            "[paires]\ntitre = \"   \"\n",
        )
        .expect("métadonnées");
        let normes = crate::mise_en_page::modele::MiseEnPage {
            sommaire: false,
            page_de_titre: true,
            ..Default::default()
        };
        std::fs::write(
            racine.join(crate::mise_en_page::gabarit::FICHIER),
            crate::mise_en_page::gabarit::generer(&normes),
        )
        .expect("gabarit");
        let main = "#import \"mise-en-page.typ\": mise-en-page\n#show: mise-en-page\n#include \"document.generated.typ\"\n";
        let source = "{{garde}}\n# La page de garde\n\nTexte de garde.\n{{/garde}}\n\n# Introduction\n\nLe corps.\n";
        let (entry, corrections) = super::compilation_unit(&racine, main, source);
        let outcome = crate::typst::render::compile(&racine, &entry, &corrections);
        assert!(outcome.errors.is_empty(), "{:?}", outcome.errors);
        assert_eq!(
            crate::typst::render::page_count(&outcome.document.expect("document")),
            2,
            "la garde et le corps, sans page de titre en blancs"
        );
        let _ = std::fs::remove_dir_all(&racine);
    }

    #[test]
    fn un_saut_de_page_juste_apres_la_garde_ne_fait_pas_une_page_blanche() {
        let racine = racine("garde-saut-apres");
        let normes = crate::mise_en_page::modele::MiseEnPage {
            sommaire: false,
            page_de_titre: false,
            ..Default::default()
        };
        std::fs::write(
            racine.join(crate::mise_en_page::gabarit::FICHIER),
            crate::mise_en_page::gabarit::generer(&normes),
        )
        .expect("gabarit");
        let main = "#import \"mise-en-page.typ\": mise-en-page\n#show: mise-en-page\n#include \"document.generated.typ\"\n";
        let source =
            "{{garde}}\n# La page de garde\n{{/garde}}\n\n\n\n# Introduction\n\nLe corps.\n";
        let (entry, corrections) = super::compilation_unit(&racine, main, source);
        let outcome = crate::typst::render::compile(&racine, &entry, &corrections);
        assert!(outcome.errors.is_empty(), "{:?}", outcome.errors);
        assert_eq!(
            crate::typst::render::page_count(&outcome.document.expect("document")),
            2,
            "la garde et le corps, sans page blanche après la garde"
        );
        let _ = std::fs::remove_dir_all(&racine);
    }

    #[test]
    fn une_garde_qui_commence_par_un_saut_de_page_ne_cree_pas_de_page_blanche() {
        let racine = racine("garde-saut-initial");
        let normes = crate::mise_en_page::modele::MiseEnPage {
            sommaire: false,
            page_de_titre: false,
            ..Default::default()
        };
        std::fs::write(
            racine.join(crate::mise_en_page::gabarit::FICHIER),
            crate::mise_en_page::gabarit::generer(&normes),
        )
        .expect("gabarit");
        let main = "#import \"mise-en-page.typ\": mise-en-page\n#show: mise-en-page\n#include \"document.generated.typ\"\n";
        let source = "{{garde}}\n\n\n# La page de garde\n\nTexte de garde.\n{{/garde}}\n\n# Introduction\n\nLe corps.\n";
        let (entry, corrections) = super::compilation_unit(&racine, main, source);
        let outcome = crate::typst::render::compile(&racine, &entry, &corrections);
        assert!(outcome.errors.is_empty(), "{:?}", outcome.errors);
        assert_eq!(
            crate::typst::render::page_count(&outcome.document.expect("document")),
            2,
            "la garde et le corps, sans page blanche avant la garde"
        );
        let _ = std::fs::remove_dir_all(&racine);
    }

    #[test]
    fn une_page_de_titre_remplie_ne_laisse_pas_de_page_blanche_avant_la_garde() {
        let root = racine("titre-rempli-gardes");
        std::fs::write(
            root.join(crate::mise_en_page::REGLAGES),
            "page_de_titre = true\n",
        )
        .expect("normes");
        std::fs::write(
            root.join(crate::project::metadonnees::FICHIER),
            "[paires]\ntitre = \"Un titre\"\n",
        )
        .expect("métadonnées");
        let normes = crate::mise_en_page::modele::MiseEnPage {
            sommaire: false,
            page_de_titre: true,
            ..Default::default()
        };
        std::fs::write(
            root.join(crate::mise_en_page::gabarit::FICHIER),
            crate::mise_en_page::gabarit::generer(&normes),
        )
        .expect("gabarit");
        let main = "#import \"mise-en-page.typ\": mise-en-page\n#show: mise-en-page\n#include \"document.generated.typ\"\n";
        let source = "{{garde}}\n# La page de garde\n\nTexte de garde.\n{{/garde}}\n\n# Introduction\n\nLe corps.\n";
        let (entry, corrections) = super::compilation_unit(&root, main, source);
        let outcome = crate::typst::render::compile(&root, &entry, &corrections);
        assert!(outcome.errors.is_empty(), "{:?}", outcome.errors);
        assert_eq!(
            crate::typst::render::page_count(&outcome.document.expect("document")),
            3,
            "la page de titre, la garde et le corps"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn les_elements_de_garde_se_placent_par_position() {
        let root = racine("positions");
        let normes = crate::mise_en_page::modele::MiseEnPage {
            sommaire: false,
            page_de_titre: false,
            ..Default::default()
        };
        std::fs::write(
            root.join(crate::mise_en_page::gabarit::FICHIER),
            crate::mise_en_page::gabarit::generer(&normes),
        )
        .expect("gabarit");
        let main = "#import \"mise-en-page.typ\": mise-en-page\n#show: mise-en-page\n#include \"document.generated.typ\"\n";
        let source = "{{garde}}\n{position=haut}\n# La page de garde\n\n{position=centre}\nLe centre.\n\n{position=bas}\nLe pied.\n{{/garde}}\n\n# Introduction\n\nLe corps.\n";
        let (entry, corrections) = super::compilation_unit(&root, main, source);

        let garde = corrections
            .iter()
            .find(|(chemin, _)| chemin.ends_with(GARDE_FILE))
            .map(|(_, contenu)| contenu)
            .expect("fichier de garde");
        assert!(garde.contains("#place(top + center)"), "{garde}");
        assert!(garde.contains("#place(horizon + center)"), "{garde}");
        assert!(garde.contains("#place(bottom + center)"), "{garde}");

        let outcome = crate::typst::render::compile(&root, &entry, &corrections);
        assert!(outcome.errors.is_empty(), "{:?}", outcome.errors);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn un_ancien_gabarit_sans_metadonnees_ne_garde_pas_la_page_vide() {
        let root = racine("ancien-gabarit-vide");
        std::fs::write(
            root.join(crate::mise_en_page::REGLAGES),
            "page_de_titre = true\n",
        )
        .expect("normes");
        let ancien = "#let mise-en-page(doc) = {\n  set page(margin: 2cm)\n  // La page de titre : les métadonnées du document.\n  page(numbering: none, header: none, footer: none)[\n    #v(1fr)\n  ]\n  pagebreak()\n  counter(page).update(1)\n  doc\n}\n";
        std::fs::write(root.join(crate::mise_en_page::gabarit::FICHIER), ancien)
            .expect("ancien gabarit");
        let main = "#import \"mise-en-page.typ\": mise-en-page\n#show: mise-en-page\n#include \"document.generated.typ\"\n";
        let source = "{{garde}}\nLa garde.\n{{/garde}}\n\n# Introduction\n\nLe corps.\n";
        let (entry, corrections) = super::compilation_unit(&root, main, source);
        let gabarit = corrections
            .iter()
            .find(|(chemin, _)| chemin.ends_with(crate::mise_en_page::gabarit::FICHIER))
            .map(|(_, contenu)| contenu)
            .expect("gabarit corrigé");
        assert!(!gabarit.contains("page(numbering"), "{gabarit}");
        let outcome = crate::typst::render::compile(&root, &entry, &corrections);
        assert!(outcome.errors.is_empty(), "{:?}", outcome.errors);
        assert_eq!(
            crate::typst::render::page_count(&outcome.document.expect("document")),
            2,
            "la garde et le corps, sans page vide"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn une_ancienne_page_de_titre_vide_est_retiree() {
        let ancien = "  // La page de titre : les métadonnées du document.\n  page(numbering: none, header: none, footer: none)[\n    Titre\n  ]\n  pagebreak()\n  counter(page).update(1)\n  doc\n";
        let corrige =
            super::retirer_la_page_de_titre_vide(ancien).expect("la page vide est retirée");
        assert!(!corrige.contains("page(numbering"), "{corrige}");
        assert!(corrige.contains("doc"), "{corrige}");
    }

    #[test]
    fn les_pages_de_garde_precedent_le_sommaire_et_restent_hors_numerotation() {
        let root = racine("avant-sommaire");
        let mut normes = crate::mise_en_page::modele::MiseEnPage {
            sommaire: true,
            page_de_titre: false,
            ..Default::default()
        };
        normes.titres.numerotation = "1.1".to_string();
        std::fs::write(
            root.join(crate::mise_en_page::gabarit::FICHIER),
            crate::mise_en_page::gabarit::generer(&normes),
        )
        .expect("gabarit");

        let main = format!(
            "#import \"mise-en-page.typ\": mise-en-page\n#show: mise-en-page\n{}",
            "#include \"document.generated.typ\"\n"
        );
        let source = "{{garde}}\n# La page de garde\n\nTexte de garde.\n{{/garde}}\n\n# Introduction\n\nLe corps du document.\n";
        let (entry, corrections) = super::compilation_unit(&root, &main, source);

        let gabarit = corrections
            .iter()
            .find(|(chemin, _)| chemin.ends_with(crate::mise_en_page::gabarit::FICHIER))
            .map(|(_, contenu)| contenu.clone())
            .expect("gabarit corrigé");
        assert!(
            gabarit.contains(&format!("#import \"{GARDE_FILE}\": garde-corps")),
            "le gabarit importe la garde : {gabarit}"
        );
        assert!(
            gabarit.find("garde-corps").unwrap() < gabarit.find("outline()").unwrap(),
            "la garde est composée avant le sommaire : {gabarit}"
        );
        assert!(
            corrections
                .iter()
                .any(|(chemin, _)| chemin.ends_with(super::GARDE_FILE)),
            "le contenu de garde est fourni"
        );

        let outcome = crate::typst::render::compile(&root, &entry, &corrections);
        assert!(
            outcome.errors.is_empty(),
            "les pages de garde compilent : {:?}",
            outcome.errors
        );
        let document = outcome.document.expect("document compilé");
        let pdf = crate::typst::render::pdf_du_document(&document).expect("PDF");
        std::fs::write(root.join("sortie.pdf"), &pdf).expect("PDF sur disque");

        let Ok(sortie) = std::process::Command::new("pdftotext")
            .arg(root.join("sortie.pdf"))
            .arg("-")
            .output()
        else {
            eprintln!("pdftotext absent — épreuve du texte sautée");
            return;
        };
        assert!(sortie.status.success(), "pdftotext s'exécute");
        let texte = String::from_utf8_lossy(&sortie.stdout);
        assert!(
            texte.contains("La page de garde"),
            "le titre de garde est composé : {texte}"
        );
        assert_eq!(
            texte.matches("La page de garde").count(),
            1,
            "le titre de garde n'apparaît pas dans le sommaire : {texte}"
        );
        assert!(
            texte.contains("Le corps du document."),
            "le corps suit la garde : {texte}"
        );
    }

    #[test]
    fn brancher_les_pages_de_garde_est_idempotent() {
        let gabarit = "#import \"document.garde.typ\": garde-corps\n#let mise-en-page(doc) = {\n  garde-corps\n  outline()\n  doc\n}\n#garde-corps\n";
        let corrige = super::brancher_les_pages_de_garde(gabarit).expect("gabarit corrigé");
        let import = format!("#import \"{GARDE_FILE}\": garde-corps");
        assert_eq!(
            corrige
                .lines()
                .filter(|ligne| ligne.trim() == import)
                .count(),
            1,
            "un seul import : {corrige}"
        );
        assert_eq!(
            corrige
                .lines()
                .filter(|ligne| ligne.trim() == "garde-corps")
                .count(),
            1,
            "un seul appel : {corrige}"
        );
    }

    #[test]
    fn une_bibliographie_cachee_ne_rajoute_pas_une_page_avant_la_garde() {
        let root = racine("bibliographie-cachee");
        std::fs::write(
            root.join("references.bib"),
            "@book{dupont2023, title={T}, author={Dupont, A.}, year={2023}}\n",
        )
        .expect("bibliographie");
        let normes = crate::mise_en_page::modele::MiseEnPage {
            sommaire: false,
            page_de_titre: false,
            ..Default::default()
        };
        std::fs::write(
            root.join(crate::mise_en_page::gabarit::FICHIER),
            crate::mise_en_page::gabarit::generer(&normes),
        )
        .expect("gabarit");

        let main = "#import \"mise-en-page.typ\": mise-en-page\n#show: mise-en-page\n#set page(margin: 2cm)\n";
        let source =
            "{{garde}}\n# La garde\n\nTexte de garde.\n{{/garde}}\n\n# Introduction\n\nLe corps.\n";
        let document = crate::langage::parse(source);
        let (entry, corrections, _) =
            super::compilation_unit_document_avec_diagnostics_explicite(&root, main, &document);

        assert!(
            entry.contains("#show bibliography: it => none"),
            "la bibliographie reste chargée sans occupation : {entry}"
        );
        let outcome = crate::typst::render::compile(&root, &entry, &corrections);
        assert!(
            outcome.errors.is_empty(),
            "le document compile : {:?}",
            outcome.errors
        );
        let document = outcome.document.expect("document");
        assert_eq!(
            crate::typst::render::page_count(&document),
            2,
            "aucune page blanche avant la garde"
        );
        let _ = std::fs::remove_dir_all(&root);
    }
}

/// Plusieurs pages de garde : chacune commence sur sa propre page et aucune
/// ne prend de numéro.
#[cfg(test)]
mod plusieurs_pages_de_garde {
    use super::GARDE_FILE;

    #[test]
    fn deux_gardes_font_deux_pages_sans_numero() {
        let root = std::env::temp_dir().join(format!(
            "typst-writer-gardes-multiples-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("racine");

        let normes = crate::mise_en_page::modele::MiseEnPage {
            sommaire: false,
            ..Default::default()
        };
        std::fs::write(
            root.join(crate::mise_en_page::gabarit::FICHIER),
            crate::mise_en_page::gabarit::generer(&normes),
        )
        .expect("gabarit");
        let main = "#import \"mise-en-page.typ\": mise-en-page\n#show: mise-en-page\n";

        let source = "{{garde}}\nPremière garde.\n{{/garde}}\n\n{{garde}}\nDeuxième garde.\n{{/garde}}\n\n# Introduction\n\nLe corps.\n";
        let (entry, corrections) = super::compilation_unit(&root, main, source);
        assert!(
            corrections
                .iter()
                .any(|(chemin, _)| chemin.ends_with(GARDE_FILE)),
            "le fichier de garde est présent"
        );

        let outcome = crate::typst::render::compile(&root, &entry, &corrections);
        assert!(
            outcome.errors.is_empty(),
            "les deux gardes compilent : {:?}",
            outcome.errors
        );
        let document = outcome.document.expect("document");
        let pdf = crate::typst::render::pdf_du_document(&document).expect("PDF");
        std::fs::write(root.join("sortie.pdf"), &pdf).expect("PDF sur disque");

        let Ok(sortie) = std::process::Command::new("pdftotext")
            .arg("-f")
            .arg("1")
            .arg("-l")
            .arg("2")
            .arg(root.join("sortie.pdf"))
            .arg("-")
            .output()
        else {
            eprintln!("pdftotext absent — épreuve du texte sautée");
            return;
        };
        let texte = String::from_utf8_lossy(&sortie.stdout);
        assert!(texte.contains("Première garde."), "{texte}");
        assert!(texte.contains("Deuxième garde."), "{texte}");
        assert!(
            !texte.lines().any(|ligne| ligne.trim() == "1"),
            "aucune garde ne porte le numéro 1 : {texte}"
        );
    }
}

/// La page de titre des normes ne prend pas de numéro : le compteur repart
/// à 1 pour le corps, et le titre reste hors de la table des matières.
#[cfg(test)]
mod page_de_titre_sans_numero {
    #[test]
    fn le_titre_des_normes_est_hors_numerotation() {
        let root = std::env::temp_dir().join(format!(
            "typst-writer-titre-sans-numero-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("racine");
        std::fs::write(
            root.join("metadonnees.toml"),
            "[paires]\ntitre = \"Titre de l'essai\"\n",
        )
        .expect("métadonnées");

        let normes = crate::mise_en_page::modele::MiseEnPage {
            page_de_titre: true,
            sommaire: false,
            ..Default::default()
        };
        std::fs::write(
            root.join(crate::mise_en_page::gabarit::FICHIER),
            crate::mise_en_page::gabarit::generer(&normes),
        )
        .expect("gabarit");

        let main = "#import \"mise-en-page.typ\": mise-en-page\n#show: mise-en-page\n";
        let (entry, corrections) =
            super::compilation_unit(&root, main, "# Introduction\n\nLe corps.\n");
        let outcome = crate::typst::render::compile(&root, &entry, &corrections);
        assert!(outcome.errors.is_empty(), "{:?}", outcome.errors);
        let document = outcome.document.expect("document");
        let pdf = crate::typst::render::pdf_du_document(&document).expect("PDF");
        std::fs::write(root.join("sortie.pdf"), &pdf).expect("PDF sur disque");

        let Ok(sortie) = std::process::Command::new("pdftotext")
            .arg("-f")
            .arg("1")
            .arg("-l")
            .arg("1")
            .arg(root.join("sortie.pdf"))
            .arg("-")
            .output()
        else {
            eprintln!("pdftotext absent — épreuve du texte sautée");
            return;
        };
        let texte = String::from_utf8_lossy(&sortie.stdout);
        assert!(
            !texte.lines().any(|ligne| ligne.trim() == "1"),
            "la page de titre ne porte pas de numéro : {texte}"
        );
    }
}

#[cfg(test)]
mod tests_personnes_pipeline {
    use super::*;

    #[test]
    fn une_balise_nue_suit_les_formes_du_fichier_de_normes() {
        let root = std::env::temp_dir().join(format!(
            "typst-writer-personnes-normes-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("racine");
        std::fs::write(
            root.join(crate::project::personnes::FICHIER),
            r#"[[entrees]]
cle = "beale"
nom = "G. K. Beale"
nom_simple = "Beale"
"#,
        )
        .expect("registre");
        std::fs::write(
            root.join("mise-en-page.toml"),
            r#"[index_personnes]
forme_premiere = "complet"
forme_suivantes = "simple"
afficher_dates = false
"#,
        )
        .expect("normes");

        let document =
            crate::langage::parse::parse("{{p beale}} écrit, puis {{p beale}} encore.\n");
        let (_, corrections) =
            compilation_unit_document(&root, "#include \"document.generated.typ\"\n", &document);
        let generated = corrections
            .iter()
            .find(|(chemin, _)| chemin.ends_with(GENERATED_FILE))
            .map(|(_, texte)| texte)
            .expect("le corps généré");

        assert!(generated.contains("G. K. Beale"), "{generated}");
        assert!(
            generated.contains("beale") && generated.contains("Beale"),
            "{generated}"
        );
        assert_eq!(generated.matches("G. K. Beale").count(), 1, "{generated}");

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn le_registre_des_personnes_du_projet_alimente_le_texte() {
        let root = std::env::temp_dir().join(format!(
            "typst-writer-personnes-pipeline-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("racine");
        std::fs::write(
            root.join(crate::project::personnes::FICHIER),
            r#"[[entrees]]
cle = "augustin"
nom = "Augustin d'Hippone"
tri = "Augustin"
dates = "354-430"
"#,
        )
        .expect("registre");

        let document = parse("{{p augustin}} a écrit.\n\n{{index-personnes}}\n");
        let (entry, corrections) =
            compilation_unit_document(&root, "#include \"document.generated.typ\"\n", &document);
        assert!(entry.contains("document.generated.typ"));
        let generated = corrections
            .iter()
            .find(|(chemin, _)| chemin.ends_with(GENERATED_FILE))
            .map(|(_, texte)| texte)
            .expect("le corps généré");

        assert!(generated.contains("Augustin d'Hippone"), "{generated}");
        assert!(generated.contains("Index des personnes"), "{generated}");
        assert!(generated.contains("354-430"), "{generated}");

        let _ = std::fs::remove_dir_all(&root);
    }
}
