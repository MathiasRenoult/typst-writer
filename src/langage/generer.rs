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
use super::typographie::{Regles, appliquer_dans};

/// Génère le corps Typst d'un document, sans règle de langue.
#[allow(dead_code)] // Raccourci des tests et de la migration.
pub fn generer(document: &Document) -> String {
    generer_avec(
        document,
        &Regles::aucune(),
        &crate::mise_en_page::modele::Decor::default(),
    )
}

/// Génère le corps Typst d'un document, en appliquant les règles de
/// typographie des normes.
///
/// Les littéraux (`\\x` dans la syntaxe) ne reçoivent **aucune** règle :
/// c'est la porte de sortie universelle, promise par la grammaire.
pub fn generer_avec(
    document: &Document,
    regles: &Regles,
    decor: &crate::mise_en_page::modele::Decor,
) -> String {
    let mut out = String::new();

    // L'index des citations est recalculé à chaque rendu : il lit les
    // références du document, comme la table des matières lit les titres.
    let index = if document.blocs.iter().any(contient_index) {
        crate::langage::index_citations::IndexBiblique::construire(
            document,
            &regles.references,
            &regles.index,
        )
    } else {
        crate::langage::index_citations::IndexBiblique::default()
    };

    // primaviz, pour les balises `{chart}` et les anciens `{graphique}` —
    // le paquet est vendu avec l'application et installé avant le rendu.
    if document.blocs.iter().any(contient_chart) {
        out.push_str("#import \"@local/primaviz:0.12.1\": *\n\n");
    }

    // showybox, pour les callouts — même contrat : le paquet est embarqué.
    if document.blocs.iter().any(contient_callout) {
        out.push_str("#import \"@local/showybox:2.0.4\": showybox\n\n");
    }

    for bloc in &document.blocs {
        generer_bloc(bloc, &mut out, regles, decor, &index);
    }

    out
}

/// Le document contient-il un index biblique, même sous un décor ?
fn contient_index(bloc: &Bloc) -> bool {
    match bloc {
        Bloc::IndexBiblique => true,
        Bloc::Décor { bloc, .. } | Bloc::Canonique(bloc) => contient_index(bloc),
        _ => false,
    }
}

/// Le document contient-il un graphique primaviz, même sous un décor ?
fn contient_chart(bloc: &Bloc) -> bool {
    match bloc {
        Bloc::Chart(_) | Bloc::Graphique { .. } => true,
        Bloc::Décor { bloc, .. } | Bloc::Canonique(bloc) => contient_chart(bloc),
        _ => false,
    }
}

/// Le document contient-il un callout, même sous un autre décor ?
fn contient_callout(bloc: &Bloc) -> bool {
    match bloc {
        Bloc::Décor { proprietes, bloc } => {
            crate::langage::callout::est_callout(proprietes) || contient_callout(bloc)
        }
        Bloc::Canonique(bloc) => contient_callout(bloc),
        _ => false,
    }
}

/// Génère un bloc, suivi d'une ligne vide — la séparation de blocs de Typst.
/// Le premier caractère qui vient du **texte** : les littéraux et les
/// codes sont sautés, car ils ne posent jamais l'espace d'un signe.
fn premier_texte(inline: &[Inline]) -> Option<char> {
    for element in inline {
        let trouve = match element {
            Inline::Texte(texte) => texte.chars().next(),
            Inline::Gras(contenu)
            | Inline::Italique(contenu)
            | Inline::GrasItalique(contenu)
            | Inline::Note(contenu)
            | Inline::Couleur { contenu, .. } => premier_texte(contenu),
            Inline::Canonique(interne) => premier_texte(std::slice::from_ref(interne.as_ref())),
            Inline::Lien { texte, .. } => premier_texte(texte),
            _ => None,
        };

        if trouve.is_some() {
            return trouve;
        }
    }

    None
}

/// Le premier caractère logique d'une suite d'éléments — à travers le
/// balisage, jusqu'au texte.
///
/// Une citation, qui n'a pas de texte, vaut un caractère : ce qui la suit
/// mérite l'espace d'un mot, pas celle du vide.
fn premier_caractere(inline: &[Inline]) -> Option<char> {
    for element in inline {
        let trouve = match element {
            Inline::Texte(texte)
            | Inline::Litteral(texte)
            | Inline::Code(texte)
            | Inline::Formule(texte) => texte.chars().next(),
            Inline::Bible(elements) => crate::langage::references::afficher_elements(
                elements,
                &crate::mise_en_page::modele::References::default(),
            )
            .chars()
            .next(),
            Inline::BibleForme { elements, forme } => {
                crate::langage::references::afficher_elements_forme(
                    elements,
                    &crate::mise_en_page::modele::References::default(),
                    Some(*forme),
                )
                .chars()
                .next()
            }
            Inline::Gras(contenu)
            | Inline::Italique(contenu)
            | Inline::GrasItalique(contenu)
            | Inline::Souligne(contenu)
            | Inline::Barre(contenu)
            | Inline::Note(contenu)
            | Inline::Couleur { contenu, .. } => premier_caractere(contenu),
            Inline::Lien { texte, .. } => premier_caractere(texte),
            Inline::Citation(_) => Some('\u{2039}'),
            // Une date vaut un mot : ce qui la suit mérite l'espace d'un.
            Inline::Date(_) => Some('1'),
            // Une métadonnée rend une valeur : elle vaut un mot.
            Inline::Meta(_) => Some('m'),
            // Un terme du glossaire rend un mot — sa forme courte.
            Inline::Glo { .. } => Some('t'),
            Inline::Canonique(interne) => premier_caractere(std::slice::from_ref(interne.as_ref())),
        };

        if trouve.is_some() {
            return trouve;
        }
    }

    None
}

fn generer_bloc(
    bloc: &Bloc,
    out: &mut String,
    regles: &Regles,
    decor: &crate::mise_en_page::modele::Decor,
    index: &crate::langage::index_citations::IndexBiblique,
) {
    // La typographie court sur **tout le bloc** : le niveau des guillemets
    // et les espaces traversent les éléments, jamais isolés.
    let mut contexte = crate::langage::typographie::Contexte::neuf();
    match bloc {
        Bloc::Titre { niveau, contenu } => {
            out.push_str(&"=".repeat(*niveau as usize));
            out.push(' ');
            generer_inline(contenu, out, false, regles, &mut contexte);
            out.push_str("\n\n");
        }
        Bloc::Paragraphe(contenu) => {
            generer_inline(contenu, out, true, regles, &mut contexte);
            out.push_str("\n\n");
        }
        Bloc::Citation(contenu) => {
            out.push_str("#quote(block: true)[");
            generer_inline(contenu, out, false, regles, &mut contexte);
            out.push_str("]\n\n");
        }
        Bloc::ListePuce(items) => {
            for item in items {
                out.push_str(&"  ".repeat(item.niveau));
                out.push_str("- ");
                generer_inline(&item.contenu, out, false, regles, &mut contexte);
                out.push('\n');
            }

            out.push('\n');
        }
        Bloc::ListeNumero(items) => {
            // Typst numérote avec `+`, et non avec un chiffre : il suit l'ordre
            // autant que l'usure des listes réordonnées.
            for item in items {
                out.push_str(&"  ".repeat(item.niveau));
                out.push_str("+ ");
                generer_inline(&item.contenu, out, false, regles, &mut contexte);
                out.push('\n');
            }

            out.push('\n');
        }
        Bloc::Brut(texte) => {
            out.push_str(texte);
            out.push_str("\n\n");
        }
        Bloc::Tableau {
            entete,
            lignes,
            alignements,
        } => {
            // Le tableau Typst : ses colonnes, ses alignements, son
            // en-tête, puis les lignes — cellule à cellule.
            out.push_str("#table(columns: ");
            out.push_str(&entete.len().to_string());
            out.push_str(", align: (");

            for (rang, alignement) in alignements.iter().enumerate() {
                if rang > 0 {
                    out.push_str(", ");
                }

                out.push_str(match alignement {
                    crate::langage::schema::Alignement::Gauche => "left",
                    crate::langage::schema::Alignement::Centre => "center",
                    crate::langage::schema::Alignement::Droite => "right",
                });
            }

            out.push_str("), table.header(");

            for (rang, cellule) in entete.iter().enumerate() {
                if rang > 0 {
                    out.push_str(", ");
                }

                out.push('[');
                generer_inline(cellule, out, false, regles, &mut contexte);
                out.push(']');
            }

            out.push(')');

            for ligne in lignes {
                for cellule in ligne {
                    out.push_str(", [");
                    generer_inline(cellule, out, false, regles, &mut contexte);
                    out.push(']');
                }
            }

            out.push_str(")\n\n");
        }
        Bloc::Formule(texte) => {
            // La formule est une équation à part : ses dollars la bornent,
            // et le contenu est du math Typst, mot pour mot.
            out.push_str("$\n");
            out.push_str(texte.trim());
            out.push_str("\n$\n\n");
        }
        Bloc::Separateur => {
            // Le séparateur horizontal Typst : `#line(length: 100%)`.
            out.push_str("#line(length: 100%)\n\n");
        }
        Bloc::Commentaire(_) => {
            // Un commentaire ne se rend pas : il vit dans le .tw seulement.
        }
        Bloc::SautDePage => {
            out.push_str("#pagebreak()\n\n");
        }
        Bloc::Image { légende, cible } => {
            // Une légende fait une figure numérotée ; sans elle, l'image
            // seule — pas de vide à composer.
            if légende.is_empty() {
                out.push_str("#image(\"");
            } else {
                out.push_str("#figure(image(\"");
            }

            echapper_chaine(cible, out);

            if légende.is_empty() {
                out.push_str("\")\n\n");
            } else {
                out.push_str("\"), caption: [");
                generer_inline(légende, out, false, regles, &mut contexte);
                out.push(']');
                // La forme de la numérotation, si les normes la choisissent.
                if !regles.numerotation_figures.is_empty() {
                    out.push_str(", numbering: \"");
                    out.push_str(&regles.numerotation_figures);
                    out.push('"');
                }
                out.push_str(")\n\n");
            }
        }
        Bloc::Décor { proprietes, bloc } => {
            // Un callout n'est pas un `#block` : showybox prend le relais et
            // reçoit le corps déjà généré, typographie comprise.
            if crate::langage::callout::est_callout(proprietes) {
                let mut interieur = String::new();
                generer_bloc(bloc, &mut interieur, regles, decor, index);
                out.push_str(&crate::langage::callout::rendre_avec(
                    crate::langage::callout::type_de(proprietes),
                    crate::langage::callout::titre_de(proprietes),
                    interieur.trim_end(),
                    &regles.callouts.presets,
                    proprietes,
                ));
                return;
            }

            // Le manteau : `#block(...)` autour de ce que le bloc génère.
            // Les propriétés deviennent les arguments du bloc Typst — c'est
            // tout le décor, des bordures aux dégradés.
            out.push_str("#block(");
            ecrire_proprietes_typst(proprietes, decor, out);
            out.push_str(")[\n");

            let mut interieur = String::new();
            generer_bloc(bloc, &mut interieur, regles, decor, index);
            out.push_str(interieur.trim_end());

            out.push_str("\n]\n\n");
        }
        Bloc::Glossaire { tout } => {
            // La liste finale : glossarium ne montre que les termes
            // référencés, sauf `{glossaire: tout}`. Le registre
            // `tw-glossaire` est posé en tête du document généré par
            // l'assemblage.
            if *tout {
                out.push_str("#print-glossary(tw-glossaire, show-all: true)\n\n");
            } else {
                out.push_str("#print-glossary(tw-glossaire)\n\n");
            }
        }
        Bloc::Sommaire => {
            // La table des matières : Typst la compose, avec son titre
            // dans la langue du document (bug-report-10, 1.3).
            out.push_str("#outline()\n\n");
        }
        Bloc::Bibliographie => {
            // La section bibliographie : la liste entière du projet, à
            // cette place — le branchement du pipeline y pose les vrais
            // fichiers (bug-report-9).
            out.push_str("#bibliography(\"references.bib\", full: true)\n\n");
        }
        Bloc::IndexBiblique => {
            out.push_str(&index.rendre_typst());
        }
        Bloc::Chart(chart) => match crate::langage::chart::rendre(chart) {
            Ok(code) => out.push_str(&code),
            Err(erreur) => out.push_str(&format!("// graphique invalide : {erreur}\n\n")),
        },
        Bloc::Graphique {
            type_,
            titre,
            donnees,
        } => {
            // Les anciens `{graphique}` passent aussi par primaviz : un seul
            // moteur de graphiques, et les documents existants continuent de
            // se composer sans cetz.
            let type_ = match type_.as_str() {
                "ligne" | "line" => "line",
                "secteur" | "pie" => "pie",
                _ => "bar",
            };
            let donnees = donnees
                .iter()
                .map(|(label, valeur)| format!("\"{}\",{valeur}", label.replace('"', "\"\"")))
                .collect::<Vec<_>>()
                .join("\n");
            let chart = crate::langage::schema::Chart {
                type_: type_.to_string(),
                titre: titre.clone(),
                valeurs: false,
                donnees,
                ..crate::langage::schema::Chart::default()
            };
            match crate::langage::chart::rendre(&chart) {
                Ok(code) => out.push_str(&code),
                Err(erreur) => out.push_str(&format!("// graphique invalide : {erreur}\n\n")),
            }
        }
        Bloc::Canonique(interne) => {
            generer_bloc(interne, out, regles, decor, index);
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
    contexte: &mut crate::langage::typographie::Contexte,
) {
    generer_inline_dans(inline, out, debut_de_ligne, regles, contexte, false);
}

/// Le même générateur, avec le savoir « suis-je dans une note ? ». Les
/// citations d'une note portent un repère distinct : l'index peut alors
/// marquer leur page d'un `n`, comme `6n`.
fn generer_inline_dans(
    inline: &[Inline],
    out: &mut String,
    debut_de_ligne: bool,
    regles: &Regles,
    contexte: &mut crate::langage::typographie::Contexte,
    dans_note: bool,
) {
    let mut en_tete = debut_de_ligne;

    for (rang, element) in inline.iter().enumerate() {
        // Ce qui suit, à travers le balisage : la fin d'un élément n'est
        // pas la fin du texte.
        contexte.suivant = premier_caractere(&inline[rang + 1..]);
        contexte.suivant_texte = premier_texte(&inline[rang + 1..]);

        match element {
            Inline::Bible(elements) => {
                out.push_str(&crate::langage::references::afficher_elements_typst(
                    elements,
                    &regles.references,
                    None,
                ));
                out.push_str(&crate::langage::index_citations::reperes_typst(
                    elements, dans_note,
                ));
            }
            Inline::BibleForme { elements, forme } => {
                out.push_str(&crate::langage::references::afficher_elements_typst(
                    elements,
                    &regles.references,
                    Some(*forme),
                ));
                out.push_str(&crate::langage::index_citations::reperes_typst(
                    elements, dans_note,
                ));
            }
            Inline::Texte(texte) => {
                // La typographie de la langue d'abord, l'échappement Typst
                // ensuite — et l'insécable devient `~`. Un tilde tapé, lui,
                // a été échappé : il reste un tilde.
                let compose = appliquer_dans(texte, regles, contexte);
                // Les siècles se **déclarent** maintenant (`{date:XIXs}`) —
                // la recherche hasardeuse est partie (2026-11-03).
                let mut echappe = String::new();
                echapper_texte(&compose, &mut echappe, en_tete);
                // Les marqueurs de siècles deviennent leur code APRÈS
                // l'échappement — avant, le `#` mourait dans le texte.
                out.push_str(&super::typographie::substituer_siecles(
                    &echappe.replace('\u{A0}', "~"),
                ));

                if compose.chars().any(|c| !c.is_whitespace()) {
                    en_tete = false;
                }
            }
            Inline::Litteral(texte) => {
                contexte.precedent = texte.chars().last().or(contexte.precedent);
                contexte.venant_du_guillemet = false;
                // Un littéral est la promesse de la grammaire : aucune
                // interprétation, ni syntaxique, ni typographique.
                echapper_texte(texte, out, en_tete);

                if texte.chars().any(|c| !c.is_whitespace()) {
                    en_tete = false;
                }
            }
            Inline::Gras(contenu) => {
                out.push('*');
                generer_inline_dans(contenu, out, false, regles, contexte, dans_note);
                out.push('*');
                en_tete = false;
            }
            Inline::Italique(contenu) => {
                out.push('_');
                generer_inline_dans(contenu, out, false, regles, contexte, dans_note);
                out.push('_');
                en_tete = false;
            }
            Inline::GrasItalique(contenu) => {
                out.push_str("*_");
                generer_inline_dans(contenu, out, false, regles, contexte, dans_note);
                out.push_str("_*");
                en_tete = false;
            }
            Inline::Souligne(contenu) => {
                out.push_str("#underline[");
                generer_inline_dans(contenu, out, false, regles, contexte, dans_note);
                out.push(']');
                en_tete = false;
            }
            Inline::Barre(contenu) => {
                out.push_str("#strike[");
                generer_inline_dans(contenu, out, false, regles, contexte, dans_note);
                out.push(']');
                en_tete = false;
            }
            Inline::Code(texte) => {
                out.push('`');
                out.push_str(texte);
                out.push('`');
                en_tete = false;
                contexte.precedent = texte.chars().last().or(contexte.precedent);
                contexte.venant_du_guillemet = false;
            }
            Inline::Date(spec) => {
                // La date se compose avec les mots des normes, puis
                // traverse l'échappement comme le texte — les marqueurs de
                // petite capitale et d'exposant y survivent.
                let compose = crate::langage::dates::analyser(spec)
                    .map(|date| crate::langage::dates::rendre(&date, &regles.dates))
                    .unwrap_or_default();
                let mut echappe = String::new();
                echapper_texte(&compose, &mut echappe, false);
                out.push_str(&super::typographie::substituer_siecles(&echappe));
            }
            Inline::Meta(cle) => {
                // Même source que les quick parts de l'en-tête : la valeur
                // vient de `metadonnees.toml`, passée en `sys.inputs` à la
                // compilation. Une clé absente ne casse pas le document.
                out.push_str("#sys.inputs.at(\"");
                echapper_chaine(cle, out);
                out.push_str("\", default: \"\")");
                en_tete = false;
                contexte.precedent = None;
                contexte.venant_du_guillemet = false;
            }
            Inline::Glo {
                cle,
                majuscule,
                pluriel,
            } => {
                // Le terme du glossaire appelle glossarium — la fonction
                // porte la majuscule et le pluriel. La clé est un
                // identifiant (le parseur l'a dit), elle ne s'échappe pas.
                let fonction = match (*majuscule, *pluriel) {
                    (false, false) => "gls",
                    (false, true) => "glspl",
                    (true, false) => "Gls",
                    (true, true) => "Glspl",
                };
                out.push_str(&format!("#{fonction}(\"{cle}\")"));
                en_tete = false;
                contexte.precedent = None;
                contexte.venant_du_guillemet = false;
            }
            Inline::Couleur { couleur, contenu } => {
                out.push_str("#text(fill: ");
                out.push_str(&expression_couleur(couleur));
                out.push_str(")[");
                generer_inline_dans(contenu, out, false, regles, contexte, dans_note);
                out.push(']');
                en_tete = false;
            }
            Inline::Citation(references) => {
                generer_citation(references, out);
                en_tete = false;
            }
            Inline::Note(contenu) => {
                out.push_str("#footnote[");
                generer_inline_dans(contenu, out, false, regles, contexte, true);
                out.push(']');
                en_tete = false;
            }
            Inline::Formule(texte) => {
                out.push('$');
                out.push_str(texte);
                out.push('$');
            }
            Inline::Lien { texte, cible } => {
                out.push_str("#link(\"");
                echapper_chaine(cible, out);
                out.push_str("\")[");
                generer_inline_dans(texte, out, false, regles, contexte, dans_note);
                out.push(']');
                en_tete = false;
            }
            Inline::Canonique(interne) => {
                // L'enveloppe n'ajoute rien au rendu : elle ne porte que la
                // forme d'écriture. On génère exactement l'élément intérieur.
                generer_inline_dans(
                    std::slice::from_ref(interne.as_ref()),
                    out,
                    en_tete,
                    regles,
                    contexte,
                    dans_note,
                );
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
pub(crate) fn echapper_texte(texte: &str, out: &mut String, debut_de_ligne: bool) {
    // La tête de ligne tient jusqu'au premier caractère utile : un
    // marqueur après un blanc l'est tout autant qu'au premier rang, et
    // Typst en ferait une liste ou un titre.
    let mut tete = debut_de_ligne;

    for caractere in texte.chars() {
        let echappe = match caractere {
            '\\' | '#' | '*' | '_' | '`' | '$' | '@' | '^' | '<' | '>' | '[' | ']' | '~' => true,
            // En tête de ligne seulement : ailleurs, ces signes sont du texte.
            '-' | '+' | '=' | '/' => tete,
            _ => false,
        };

        if echappe {
            out.push('\\');
        }

        out.push(caractere);

        if !caractere.is_whitespace() {
            tete = false;
        }
    }
}

/// Écrit une cible de lien dans une chaîne Typst.
///
/// Une chaîne n'a que deux escapements — la barre oblique et le guillemet —,
/// mais les deux y sont obligés : une cible qui en porterait un romprait la
/// chaîne, et le document ne compilerait plus.
/// Les propriétés deviennent les arguments d'un bloc Typst.
///
/// **Les noms prennent leurs mesures dans les normes du document** :
/// `encadré` y trouve sa couleur, son épaisseur, son rayon et sa marge ;
/// `plein` y trouve son remplissage. Une mesure explicite passe par-dessus
/// — l'exception est possible, mais elle n'est pas la règle.
///
/// Une clé que la table ne connaît pas passe telle quelle : le Typst reste
/// le maître du rendu, et c'est lui qui dira s'il la comprend.
fn ecrire_proprietes_typst(
    proprietes: &crate::langage::schema::Proprietes,
    decor: &crate::mise_en_page::modele::Decor,
    out: &mut String,
) {
    // Les arguments résolus, dans l'ordre : les noms d'abord.
    let mut arguments: Vec<(&str, String)> = Vec::new();

    for nom in &proprietes.noms {
        match nom.as_str() {
            "encadré" | "encadre" => {
                arguments.push((
                    "stroke",
                    format!(
                        "(paint: {}, thickness: {})",
                        couleur_en_typst(&decor.couleur),
                        decor.epaisseur
                    ),
                ));
                arguments.push(("radius", decor.rayon.clone()));
                arguments.push(("inset", decor.marge.clone()));
            }
            "plein" => arguments.push(("fill", couleur_en_typst(&decor.remplissage))),
            // Un nom que le vocabulaire ne connaît pas n'ajoute rien :
            // il reste dans la source, et le jour où il prend sens, il
            // prendra ses mesures ici.
            _ => {}
        }
    }

    // Les paires libres : la valeur d'une couleur se compose, le reste
    // est du Typst et ne bouge pas.
    for (cle, valeur) in &proprietes.paires {
        let argument = match cle.as_str() {
            "largeur" => "width",
            "hauteur" => "height",
            "bordure" => "stroke",
            "remplissage" => "fill",
            "rayon" => "radius",
            "marge" => "inset",
            "marge_ext" => "outset",
            "espace" => "spacing",
            autre => autre,
        };

        // La mesure explicite remplace la résolue : l'exception gagne.
        match arguments.iter_mut().find(|(nom, _)| *nom == argument) {
            Some(place) => place.1 = valeur.clone(),
            None => arguments.push((argument, valeur.clone())),
        }
    }

    for (position, (nom, valeur)) in arguments.iter().enumerate() {
        if position > 0 {
            out.push_str(", ");
        }

        out.push_str(nom);
        out.push_str(": ");
        out.push_str(valeur);
    }
}

fn echapper_chaine(texte: &str, out: &mut String) {
    for caractere in texte.chars() {
        if matches!(caractere, '\\' | '"') {
            out.push('\\');
        }

        out.push(caractere);
    }
}

/// La couleur, telle que Typst la veut dans une expression.
///
/// La syntaxe écrit `#ff0000` : c'est du texte pour Typst, dont `rgb()`
/// est la seule voie. Les expressions déjà faites — `rgb(255, 0, 0)` — et
/// les noms — `red` — passent tels quels.
fn expression_couleur(couleur: &str) -> String {
    let nu = couleur.trim();

    if nu.starts_with('#') {
        return format!("rgb(\"{nu}\")");
    }

    nu.to_string()
}

/// Une couleur, en Typst.
///
/// Un hexadécimal nu — `#1e1e2e` — est invalide dans le code Typst : le
/// `#` y ouvre un appel. Il devient `rgb("#1e1e2e")`. Une expression —
/// `rgb(...)`, un dégradé — est déjà du Typst, et ne bouge pas.
fn couleur_en_typst(valeur: &str) -> String {
    let valeur = valeur.trim();

    if valeur.starts_with('#') {
        format!("rgb(\"{valeur}\")")
    } else {
        valeur.to_string()
    }
}

#[cfg(test)]
mod tests {

    /// Un marqueur de bloc après un blanc est un marqueur : Typst en
    /// ferait une liste. Le rempart tient toute la tête de ligne.
    /// Le bloc se génère tout entier, contexte compris — la porte de la
    /// sonde typographique.
    fn generer_inline_en_clair(
        bloc: &[Inline],
        regles: &crate::langage::typographie::Regles,
    ) -> String {
        let mut out = String::new();
        let mut contexte = crate::langage::typographie::Contexte::neuf();
        super::generer_inline(bloc, &mut out, false, regles, &mut contexte);
        out
    }

    /// Les cinq défauts nommés par la feuille de route (phase 2, point 2.3) :
    /// le traitement typographique court sur tout le bloc, et chaque forme
    /// attendue doit sortir du générateur.
    #[test]
    fn le_traitement_typographique_tient_ses_cinq_defauts() {
        let regles = crate::langage::typographie::Regles::francaise();
        let texte = |mot: &str| Inline::Texte(mot.to_string());

        // 1. L'insécable aux frontières d'éléments : le point-virgule ouvre
        //    un élément de mise en forme, et l'espace le précède encore.
        let sortie =
            generer_inline_en_clair(&[texte("a"), Inline::Gras(vec![texte("; b")])], &regles);
        assert_eq!(sortie, "a*~; b*", "l'insécable tient la frontière");

        // 2. Les espaces autour des signes de ponctuation.
        let sortie = generer_inline_en_clair(&[texte("a: b")], &regles);
        assert_eq!(sortie, "a~: b", "les deux points prennent leur insécable");
        let sortie = generer_inline_en_clair(&[texte("a;b")], &regles);
        assert_eq!(sortie, "a~; b", "le point-virgule se pose de même");

        // 3. L'imbrication des guillemets : les paires suivent le niveau.
        let sortie =
            generer_inline_en_clair(&[texte("\u{AB} un \u{AB} deux \u{BB} fin \u{BB}")], &regles);
        assert_eq!(
            sortie, "\u{AB}~un \u{2039}~deux~\u{203A} fin~\u{BB}",
            "les guillemets s'imbriquent par niveau"
        );

        // 4. La continuité du contexte entre éléments : l'ouvrant est dans
        //    un élément, le fermant dans un autre.
        let sortie = generer_inline_en_clair(
            &[
                Inline::Gras(vec![texte("\u{AB} ouverture")]),
                texte(" milieu "),
                Inline::Italique(vec![texte("fermeture \u{BB}")]),
            ],
            &regles,
        );
        assert_eq!(
            sortie, "*\u{AB}~ouverture* milieu _fermeture~\u{BB}_",
            "le contexte traverse les éléments"
        );

        // 5. L'échappement Typst : l'insécable devient `~`, et survit.
        let sortie = generer_inline_en_clair(&[texte("a; b")], &regles);
        assert_eq!(sortie, "a~; b", "l'insécable survit à l'échappement");
    }

    #[test]
    fn le_repart_de_tete_tient_apres_un_blanc() {
        use super::super::schema::{Bloc, Document, Inline};

        let document = Document {
            blocs: vec![Bloc::Paragraphe(vec![
                Inline::Litteral(" ".to_string()),
                Inline::Texte("- x".to_string()),
            ])],
        };

        let typst = generer(&document);
        assert!(
            typst.contains("\\- x"),
            "le tiret après un blanc doit être protégé : {typst}"
        );

        // Et le document compile sans devenir une liste.
        let outcome = compiler(&typst);
        assert!(outcome.errors.is_empty(), "vu : {:?}", outcome.errors);
    }

    /// La typographie traverse les éléments : l'espace avant un signe, et
    /// le niveau des guillemets, ne se perdent pas aux frontières.
    #[test]
    fn la_typographie_traverse_les_elements() {
        // Le cas de l'audit : le `;` suit un gras, et son insécable se
        // perdait — la règle ne voyait que l'élément isolé.
        let document = crate::langage::parse("La foi *x* ; suite\n");
        let typst = generer_avec(
            &document,
            &Regles::francaise(),
            &crate::mise_en_page::modele::Decor::default(),
        );
        assert!(
            typst.contains("~; suite"),
            "l'insécable traverse le gras : {typst}"
        );

        // L'espace après un point en fin d'élément se pose devant la suite.
        let document = crate::langage::parse("Suite. *Et plus*\n");
        let typst = generer_avec(
            &document,
            &Regles::francaise(),
            &crate::mise_en_page::modele::Decor::default(),
        );
        assert!(
            typst.contains("Suite. "),
            "l'espace après le point se pose : {typst}"
        );
    }

    /// La couleur hex s'écrit `#ff0000` et doit compiler : c'est du texte
    /// pour Typst, dont `rgb()` est la seule voie.
    #[test]
    fn la_couleur_hex_compile() {
        let document = crate::langage::parse("{#ff0000: en rouge}");
        let typst = generer(&document);

        assert!(
            typst.contains("rgb(\"#ff0000\")"),
            "la couleur hex devient une expression : {typst}"
        );

        let outcome = compiler(&typst);
        assert!(outcome.errors.is_empty(), "vu : {:?}", outcome.errors);
    }

    /// Une couleur exprimée ou nommée passe telle quelle.
    #[test]
    fn la_couleur_exprimee_passe_telle_quelle() {
        assert_eq!(expression_couleur("red"), "red");
        assert_eq!(expression_couleur("rgb(255, 0, 0)"), "rgb(255, 0, 0)");
        assert_eq!(expression_couleur("#00ff00"), "rgb(\"#00ff00\")");
    }

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
    pub(crate) fn compiler(typst: &str) -> crate::typst::render::CompileOutcome {
        let root = PathBuf::from("/projet");
        let entry = "#include \"chapitre.typ\"\n".to_string();
        let overrides = vec![(root.join("chapitre.typ"), typst.to_string())];

        crate::typst::render::compile(&root, &entry, &overrides)
    }

    /// Les trois défauts nommés par la feuille de route (phase 2, point 2.4) :
    /// la chaîne produite ne suffit pas — le Typst embarqué doit la compiler.
    #[test]
    fn la_generation_se_compile_sur_les_defauts_nommes() {
        // 1. Couleur hexadécimale ; 2. caractères spéciaux ; 3. échappement
        //    selon la position (un marqueur en tête de ligne).
        let document = Document {
            blocs: vec![
                Bloc::Paragraphe(vec![
                    Inline::Couleur {
                        couleur: "#ff0000".to_string(),
                        contenu: vec![Inline::Texte("rouge".to_string())],
                    },
                    Inline::Texte(" et des spéciaux : ".to_string()),
                    Inline::Litteral("{}[]#@".to_string()),
                ]),
                Bloc::Paragraphe(vec![
                    Inline::Litteral("-".to_string()),
                    Inline::Texte(" puce en tête de ligne".to_string()),
                ]),
            ],
        };
        let typst = generer_avec(
            &document,
            &Regles::francaise(),
            &crate::mise_en_page::modele::Decor::default(),
        );
        let sortie = compiler(&typst);
        assert!(
            sortie.errors.is_empty(),
            "le Typst embarqué compile la génération : {:?} ({typst:?})",
            sortie.errors
        );
    }

    /// La correspondance de positions (feuille de route, phase 2, point 2.5) :
    /// le générateur garde les numéros de ligne du document source — c'est
    /// ce qui rend la position du fichier généré valable dans le `.tw`.
    #[test]
    fn le_generateur_garde_les_numeros_de_ligne() {
        for source in [
            "# Titre\n\nUn paragraphe.\n\n- puce\n- deux\n",
            "Texte *gras* et `code`.\n\n> citation\n",
            "1. un\n2. deux\n\n| a | b |\n| - | - |\n| 1 | 2 |\n",
        ] {
            let document = crate::langage::parse(source);
            let typst = generer(&document);
            let lignes_source = source.split('\n').count();
            let lignes_typst = typst.split('\n').count();
            assert!(
                lignes_typst >= lignes_source - 1 && lignes_typst <= lignes_source + 1,
                "la structure de lignes tient : {lignes_source} vs {lignes_typst} ({typst:?})"
            );
        }
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
            blocs: vec![Bloc::Paragraphe(vec![Inline::Code(
                "#f(x) * y]".to_string(),
            )])],
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
        assert!(
            typst.contains("#link(\"https://x.y/a\\\"b\\\\c\")"),
            "{typst}"
        );
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
        assert!(
            compiler(&typst).errors.is_empty(),
            "la couleur doit compiler"
        );

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

    /// Une citation bibliographique se génère en référence Typst, même
    /// lorsqu'elle est **dans une note de bas de page**.
    #[test]
    fn une_citation_dans_une_note_reste_une_citation() {
        let document = parse("Une note^[Voir @dupont2023[p. 45].]\n");
        let typst = generer(&document);

        assert!(
            typst.contains("#footnote[Voir @dupont2023[p. 45].]"),
            "la citation reste dans la note : {typst}"
        );
    }

    /// Le balisage inline traverse la note : gras, italique, couleur et lien
    /// restent des éléments typographiques, jamais du texte échappé.
    #[test]
    fn les_balises_vivent_dans_une_note() {
        let document = parse(
            "Une note^[Voir **gras**, *italique*, {rouge:rouge} et un [lien](https://exemple.fr).]\n",
        );
        let typst = generer(&document);

        assert!(typst.contains("#footnote["), "{typst}");
        assert!(
            typst.contains("*gras*"),
            "le gras vit dans la note : {typst}"
        );
        assert!(
            typst.contains("_italique_"),
            "l'italique vit dans la note : {typst}"
        );
        assert!(
            typst.contains("#text(fill:"),
            "la couleur vit dans la note : {typst}"
        );
        assert!(
            typst.contains("#link(\"https://exemple.fr\")"),
            "le lien vit dans la note : {typst}"
        );
    }

    /// L'ancien `{graphique}` passe désormais par primaviz, pas par cetz.
    #[test]
    fn un_ancien_graphique_passe_par_primaviz() {
        let typst = generer(&parse(
            "{graphique type=barre titre=\"Ventes\"}\nNord | 30\nSud | 70\n",
        ));

        assert!(
            typst.contains("#import \"@local/primaviz:0.12.1\": *"),
            "le paquet primaviz est importé : {typst}"
        );
        assert!(
            typst.contains("bar-chart("),
            "le graphique est composé : {typst}"
        );
        assert!(!typst.contains("cetz"), "cetz n'est plus là : {typst}");
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
        let root =
            std::env::temp_dir().join(format!("typst-writer-groupe-note-{}", std::process::id()));
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

        assert!(
            page.contains("p. 45"),
            "la page se lit dans la note : {page}"
        );
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
        let root =
            std::env::temp_dir().join(format!("typst-writer-note-page-{}", std::process::id()));
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

        assert!(
            page.contains("p. 45"),
            "la page se lit dans la note : {page}"
        );

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

// ==================== EXPÉRIENCE DOCX/MARKDOWN (R8) ====================
// Une sonde, pas du code livré : elle mesure ce que la chaîne
// DOCX/LibreOffice rend vraiment — titres, tableaux, notes, citations,
// images, aller-retour — avant toute décision de livraison. Le PDF reste
// la référence de rendu.

#[cfg(test)]
mod experience_docx {
    use crate::langage::parse::parse;
    use std::process::Command;

    const PNG: [u8; 70] = [
        137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1, 8, 6,
        0, 0, 0, 31, 21, 196, 137, 0, 0, 0, 13, 73, 68, 65, 84, 120, 218, 99, 100, 96, 248, 95, 15,
        0, 2, 135, 1, 128, 235, 71, 186, 146, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
    ];

    const FIXTURE: &str = "\
# Titre un\n\
\n\
Un paragraphe avec du texte et une note^[la note attachée] et une citation @dupont2023[p. 45].\n\
\n\
## Titre deux\n\
\n\
| Colonne A | Colonne B |\n\
| --- | --- |\n\
| cellule A1 | cellule B1 |\n\
| cellule A2 | cellule B2 |\n\
\n\
![la légende de la figure](images/fig.png)\n";

    /// La fixture devient le HTML que la chaîne DOCX mangera. Ce mapping est
    /// volontairement simple : ce qui se perd ici se perd pour de bon.
    fn vers_html(source: &str) -> String {
        let mut html = String::from("<html><body>\n<meta charset=\"utf-8\">\n");
        let mut lignes = source.lines().peekable();
        while let Some(ligne) = lignes.next() {
            let nu = ligne.trim();
            if nu.is_empty() {
                continue;
            }
            if let Some(reste) = nu.strip_prefix("## ") {
                html.push_str(&format!("<h2>{reste}</h2>\n"));
            } else if let Some(reste) = nu.strip_prefix("# ") {
                html.push_str(&format!("<h1>{reste}</h1>\n"));
            } else if nu.starts_with('|') {
                // Le tableau, entier.
                html.push_str("<table>\n");
                let rangee = |ligne: &str, entete: bool| {
                    let cellules: Vec<&str> =
                        ligne.trim_matches('|').split('|').map(str::trim).collect();
                    let ouvrant = if entete { "th" } else { "td" };
                    let mut r = String::from("<tr>");
                    for cellule in &cellules {
                        r.push_str(&format!("<{ouvrant}>{cellule}</{ouvrant}>"));
                    }
                    r.push_str("</tr>\n");
                    r
                };
                html.push_str(&rangee(nu, true));
                while let Some(suite) = lignes.peek() {
                    let s = suite.trim();
                    if !s.starts_with('|') {
                        break;
                    }
                    if !s.contains("---") {
                        html.push_str(&rangee(s, false));
                    }
                    lignes.next();
                }
                html.push_str("</table>\n");
            } else if nu.starts_with("![") {
                let (legende, cible) = decouper_image(nu);
                html.push_str(&format!(
                    "<figure><img src=\"{cible}\"/><figcaption>{legende}</figcaption></figure>\n"
                ));
            } else {
                // Le paragraphe, ses notes en texte marqué.
                let avec_notes = remplacer_notes(nu);
                html.push_str(&format!("<p>{avec_notes}</p>\n"));
            }
        }
        html.push_str("</body></html>\n");
        html
    }

    fn decouper_image(ligne: &str) -> (String, String) {
        let sans = ligne.trim_start_matches("![").trim_end_matches(']');
        if let Some(ouvrante) = sans.find("](") {
            (
                sans[..ouvrante].to_string(),
                sans[ouvrante + 2..].to_string(),
            )
        } else {
            (String::new(), sans.to_string())
        }
    }

    fn remplacer_notes(paragraphe: &str) -> String {
        let mut sorti = String::new();
        let mut reste = paragraphe;
        while let Some(ouvrante) = reste.find("^[") {
            sorti.push_str(&reste[..ouvrante]);
            reste = &reste[ouvrante + 2..];
            match reste.find(']') {
                Some(fermante) => {
                    sorti.push_str("<span class=\"note\">[note: ");
                    sorti.push_str(&reste[..fermante]);
                    sorti.push_str("]</span>");
                    reste = &reste[fermante + 1..];
                }
                None => break,
            }
        }
        sorti.push_str(reste);
        sorti
    }

    /// LibreOffice, d'un format à l'autre — profil jetable, sinon deux
    /// conversions parallèles se marchent dessus.
    fn soffice(
        entree: &std::path::Path,
        conversion: &str,
        entree_filtre: Option<&str>,
        sortie: &std::path::Path,
    ) -> Result<(), String> {
        let profil = sortie.join("profil");
        std::fs::create_dir_all(&profil).map_err(|e| e.to_string())?;
        let mut commande = Command::new("soffice");
        commande
            .arg(format!("-env:UserInstallation=file://{}", profil.display()))
            .arg("--headless");
        if let Some(filtre) = entree_filtre {
            commande.arg(format!("--infilter={filtre}"));
        }
        let resultat = commande
            .arg("--convert-to")
            .arg(conversion)
            .arg("--outdir")
            .arg(sortie)
            .arg(entree)
            .output()
            .map_err(|e| format!("soffice: {e}"))?;
        if !resultat.status.success() {
            return Err(format!(
                "soffice a refusé : {}",
                String::from_utf8_lossy(&resultat.stderr)
            ));
        }
        Ok(())
    }

    #[test]
    #[ignore = "expérience DOCX : LibreOffice exigé, mesures à la sonde"]
    fn experience_docx_aller_retour() {
        if Command::new("soffice").arg("--version").output().is_err() {
            println!("expérience DOCX : LibreOffice absent, test sauté");
            return;
        }

        let dir = std::env::temp_dir().join(format!("tw-experience-docx-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("images")).expect("dossier");
        std::fs::write(dir.join("images/fig.png"), PNG).expect("image");

        // La vérité de départ, dans le document structuré.
        let document = parse(FIXTURE);
        let mut titres = 0;
        let mut cellules = 0;
        let mut notes = 0;
        let mut citations = 0;
        let mut images = 0;
        for bloc in &document.blocs {
            match bloc {
                crate::langage::schema::Bloc::Titre { .. } => titres += 1,
                crate::langage::schema::Bloc::Tableau { entete, lignes, .. } => {
                    cellules += entete.len() + lignes.iter().map(|l| l.len()).sum::<usize>();
                }
                crate::langage::schema::Bloc::Image { .. } => images += 1,
                crate::langage::schema::Bloc::Paragraphe(inlines) => {
                    for inline in inlines {
                        match inline {
                            crate::langage::schema::Inline::Note(_) => notes += 1,
                            crate::langage::schema::Inline::Citation(_) => citations += 1,
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
        println!(
            "SOURCE : {titres} titres, {cellules} cellules, {notes} note(s), {citations} citation(s), {images} image(s)"
        );

        // L'aller : .tw → HTML → DOCX. Chaque étape a SON dossier : le HTML
        // de retour ne doit jamais écraser celui du départ — sinon le relevé
        // se lit lui-même.
        let html = vers_html(FIXTURE);
        std::fs::create_dir_all(dir.join("aller")).expect("aller");
        let fichier_html = dir.join("aller/fixture.html");
        std::fs::write(&fichier_html, &html).expect("html");
        if let Err(e) = soffice(
            &fichier_html,
            "docx:Office Open XML Text",
            Some("HTML (StarWriter)"),
            &dir.join("aller"),
        ) {
            println!("aller html→docx impossible : {e}");
            return;
        }
        let docx = dir.join("aller/fixture.docx");
        println!("ALLER : docx écrit = {}", docx.is_file());

        // Le retour : DOCX → HTML, et ce qu'il en reste.
        std::fs::create_dir_all(dir.join("retour")).expect("retour");
        if let Err(e) = soffice(&docx, "html:HTML (StarWriter)", None, &dir.join("retour")) {
            println!("retour docx→html impossible : {e}");
            return;
        }
        let retour = std::fs::read_to_string(dir.join("retour/fixture.html")).unwrap_or_default();
        println!("RETOUR : lu {} octets de fixture.html", retour.len());

        // Le relevé des pertes, axe par axe.
        let presente = |aiguille: &str| retour.contains(aiguille);
        println!("RETOUR DOCX→HTML :");
        println!(
            "  titres : texte présent = {}, balises de titre = {}",
            presente("Titre un") && presente("Titre deux"),
            retour.matches("<h1").count() + retour.matches("<h2").count()
        );
        println!(
            "  tableau : cellules A1/B2 présentes = {}, <table> = {}, <td>/<th> = {}",
            presente("cellule A1") && presente("cellule B2"),
            retour.matches("<table").count(),
            retour.matches("<td").count() + retour.matches("<th").count()
        );
        // Aiguilles sans diacritiques ni blancs : le HTML revenu peut
        // ré-encoder les accents et couper les lignes au milieu d'une
        // phrase, et une mesure qui les porterait confondrait habillage et
        // perte.
        let replat = retour.split_whitespace().collect::<String>();
        let present_sans_blancs =
            |aiguille: &str| replat.contains(&aiguille.split_whitespace().collect::<String>());
        println!(
            "  note : texte pr\u{e9}sent = {}, marqu\u{e9}e comme note = {}",
            present_sans_blancs("la note attachée"),
            retour.contains("class=\"note\"") || retour.contains("footnote")
        );
        println!(
            "  citation : chaîne intacte = {}",
            presente("@dupont2023[p. 45]")
        );
        println!(
            "  image : <img> pr\u{e9}sent = {}, l\u{e9}gende pr\u{e9}sente = {}",
            retour.contains("<img"),
            presente("gende de la figure")
        );

        // Et l'aller inverse : le HTML revenu redonne-t-il un .tw lisible ?
        let revu = retour
            .lines()
            .filter(|l| !l.trim_start().starts_with('<') || l.contains("<p") || l.contains("<h"))
            .count();
        println!(
            "HTML revenu : {} lignes de contenu, {} octets",
            revu,
            retour.len()
        );
    }
}

#[cfg(test)]
mod tests_dates_generees {
    use super::*;

    /// La balise `{date:...}` traverse le parse et la génération, avec les
    /// mots des normes (2026-11-03) — la nouvelle méthode, déclarée.
    #[test]
    fn une_date_se_declare_et_se_genere() {
        let document = crate::langage::parse(
            "En {date:XIXs?}, la révolution du {date:1789.07.14 \"dd/MM/yyyy\"}.",
        );
        let mut regles = Regles::aucune();
        regles.dates.epoque_apres = "EC".to_string();
        let sortie = generer_avec(
            &document,
            &regles,
            &crate::mise_en_page::modele::Decor::default(),
        );
        assert!(
            sortie.contains("#smallcaps[XIX]#super[e] siècle EC"),
            "la période longue : {sortie}"
        );
        assert!(sortie.contains("14/07/1789"), "la date pleine : {sortie}");

        // Le court : le romain seul, en petite capitale et exposant.
        let document = crate::langage::parse("Le {date:XIXs} siècle.");
        let sortie = generer_avec(
            &document,
            &regles,
            &crate::mise_en_page::modele::Decor::default(),
        );
        assert!(
            sortie.contains("#smallcaps[XIX]#super[e]"),
            "la période courte : {sortie}"
        );
    }
}

/// Le glossaire rendu : les appels glossarium, et la liste à sa place.
#[cfg(test)]
mod tests_glossaire {
    use super::super::parse::parse;
    use super::generer;

    /// Chaque forme de terme appelle la fonction glossarium qui lui
    /// répond — jamais une imitation.
    #[test]
    fn un_terme_du_glossaire_devient_un_appel_glossarium() {
        let document = parse(
            "Les {glo:sdo}, les {glo:sdo pluriel}, {glo:sdo majuscule} et {glo:sdo majuscule pluriel}.\n",
        );
        let typst = generer(&document);
        assert!(typst.contains("#gls(\"sdo\")"), "{typst}");
        assert!(typst.contains("#glspl(\"sdo\")"), "{typst}");
        assert!(typst.contains("#Gls(\"sdo\")"), "{typst}");
        assert!(typst.contains("#Glspl(\"sdo\")"), "{typst}");
    }

    /// La liste finale se pose où l'auteur l'a posée, dans sa variante.
    #[test]
    fn la_liste_du_glossaire_se_place_ou_l_auteur_l_a_mise() {
        let document = parse("{glossaire}\n");
        let typst = generer(&document);
        assert!(typst.contains("#print-glossary(tw-glossaire)"), "{typst}");
        assert!(!typst.contains("show-all"), "{typst}");

        let document = parse("{glossaire: tout}\n");
        let typst = generer(&document);
        assert!(
            typst.contains("#print-glossary(tw-glossaire, show-all: true)"),
            "{typst}"
        );
    }
}

#[cfg(test)]
mod tests_de_l_index_des_citations {
    use super::*;
    use crate::langage::typographie::Regles;
    use crate::mise_en_page::modele::{Decor, SystemeAbreviation};
    /// L'index suit la forme de livre choisie pour le document, et non une
    /// abréviation figée dans le générateur.
    #[test]
    fn l_index_suit_la_forme_des_livres() {
        let document = crate::langage::parse::parse("{{Jn 3:16}}\n\n{{index}}\n");
        let mut regles = Regles::aucune();
        regles.references.livre = SystemeAbreviation::FrancaiseLongue;

        let typst = generer_avec(&document, &regles, &Decor::default());

        assert!(typst.contains("- Jean 3.16"), "{typst}");
        assert!(!typst.contains("- Jn 3:16"), "{typst}");
    }

    /// Le texte libre d'une citation est rendu comme du texte : un `#` ou
    /// un `*` ne doit jamais devenir une commande Typst.
    #[test]
    fn le_texte_libre_ne_devient_pas_du_balisage() {
        let document = crate::langage::parse("{{Jn 3.16 ; \"cf. #aussi\"}}\n");
        let typst = generer_avec(&document, &Regles::aucune(), &Decor::default());

        assert!(typst.contains("cf. \\#aussi"), "{typst}");
        assert!(!typst.contains("cf. #aussi"), "{typst}");
    }

    /// Un index paginé pose ses repères et sa requête Typst : le tout doit
    /// réellement compiler, pages comprises.
    #[test]
    fn l_index_pagine_se_compile() {
        let document = crate::langage::parse("{{Jn 3.16 ; Gn 1.1}}\n\n{{index}}\n");
        let mut regles = Regles::aucune();
        regles.index = crate::mise_en_page::modele::IndexCitations {
            afficher_pages: true,
            regrouper_par_livre: true,
            forme: Some(crate::mise_en_page::modele::SystemeAbreviation::FrancaiseLongue),
            ..Default::default()
        };
        let typst = generer_avec(&document, &regles, &Decor::default());

        assert!(
            typst.contains("#metadata(\"John|3|3|16|16\") <tw-citation>"),
            "{typst}"
        );
        assert!(typst.contains("[Genèse :]"), "{typst}");
        assert!(typst.contains("[Jean :]"), "{typst}");
        let sortie = super::tests::compiler(&typst);
        assert!(
            sortie.errors.is_empty(),
            "l'index paginé ne compile pas : {:?} ({typst:?})",
            sortie.errors
        );
    }

    /// Une citation en note porte le repère `tw-citation-note` : sa page
    /// sera écrite `6n` dans l'index.
    #[test]
    fn une_citation_en_note_porte_son_repere() {
        let document = crate::langage::parse("Une note ^[{{Jn 3.16}}].\n\n{{index}}\n");
        let mut regles = Regles::aucune();
        regles.index = crate::mise_en_page::modele::IndexCitations {
            afficher_pages: true,
            ..Default::default()
        };
        let typst = generer_avec(&document, &regles, &Decor::default());

        assert!(
            typst.contains("#metadata(\"John|3|3|16|16\") <tw-citation-note>"),
            "{typst}"
        );
        assert!(
            typst.contains("sorted(key: p => p.at(0))"),
            "les pages doivent être rangées dans l'ordre : {typst}"
        );
        assert!(
            typst.contains("if p.at(1) { \"n\" }"),
            "une citation en note doit porter le suffixe n : {typst}"
        );
        let sortie = super::tests::compiler(&typst);
        assert!(
            sortie.errors.is_empty(),
            "l'index avec une note ne compile pas : {:?} ({typst:?})",
            sortie.errors
        );
    }

    /// Un index en colonnes compile : la disposition est demandée à Typst.
    #[test]
    fn l_index_en_colonnes_se_compile() {
        let document = crate::langage::parse("{{Jn 3.16 ; Gn 1.1}}\n\n{{index}}\n");
        let mut regles = Regles::aucune();
        regles.index = crate::mise_en_page::modele::IndexCitations {
            colonnes: 2,
            regrouper_par_livre: true,
            ..Default::default()
        };
        let typst = generer_avec(&document, &regles, &Decor::default());

        assert!(typst.contains("#columns(2)["), "{typst}");
        let sortie = super::tests::compiler(&typst);
        assert!(
            sortie.errors.is_empty(),
            "l'index en colonnes ne compile pas : {:?} ({typst:?})",
            sortie.errors
        );
    }

    /// Une traduction écrite dans la citation est rendue, et seulement elle.
    #[test]
    fn une_traduction_explicite_est_dans_le_rendu() {
        let document = crate::langage::parse("{{Jn 3.16|darby}}\n");
        let typst = generer_avec(&document, &Regles::aucune(), &Decor::default());

        assert!(typst.contains("Jn 3.16 (DARBY)"), "{typst}");
        assert!(!typst.contains("Jn 3.16 (LSG)"), "{typst}");
    }
}

#[cfg(test)]
mod tests_des_callouts {
    use super::*;
    use crate::mise_en_page::modele::Decor;

    /// Un callout importe showybox une seule fois, et rend le bloc de
    /// citation dans le cadre du ton choisi.
    #[test]
    fn un_callout_importe_showybox_et_rend_le_ton() {
        let document = crate::langage::parse(
            "{callout attention \"Vigilance\"}\n> Ne pas confondre les deux sens.\n",
        );
        let typst = generer_avec(&document, &Regles::aucune(), &Decor::default());

        assert_eq!(
            typst.matches("#import \"@local/showybox:2.0.4\"").count(),
            1,
            "un seul import : {typst}"
        );
        assert!(typst.contains("#showybox("), "le cadre est posé : {typst}");
        assert!(typst.contains("title: \"Vigilance\""), "{typst}");
        assert!(typst.contains("border-color: rgb(\"#b23b36\")"), "{typst}");
        assert!(
            typst.contains("#quote(block: true)[Ne pas confondre les deux sens.]"),
            "la citation reste une citation : {typst}"
        );
        assert!(
            !typst.contains("#block("),
            "showybox remplace le bloc : {typst}"
        );
    }

    /// Un callout sans type prend le ton `note` ; la source, elle, ne change
    /// pas.
    #[test]
    fn un_callout_sans_type_prend_le_ton_note() {
        let document = crate::langage::parse("{callout}\n> Une remarque.\n");
        let typst = generer_avec(&document, &Regles::aucune(), &Decor::default());

        assert!(typst.contains("border-color: rgb(\"#3b6ea5\")"), "{typst}");
        assert!(typst.contains("title: \"\""), "{typst}");
    }
}
