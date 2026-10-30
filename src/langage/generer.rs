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

    // L'import `cetz` pour les graphiques, seulement si le document en
    // contient — un import inutile alourdit le préambule.
    let a_graphique = document.blocs.iter().any(|b| {
        matches!(b, Bloc::Graphique { .. })
    });
    if a_graphique {
        out.push_str("#import \"@preview/cetz:0.3.2\": chart\n\n");
    }

    for bloc in &document.blocs {
        generer_bloc(bloc, &mut out, regles, decor);
    }

    out
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
            Inline::Gras(contenu)
            | Inline::Italique(contenu)
            | Inline::GrasItalique(contenu)
            | Inline::Souligne(contenu)
            | Inline::Barre(contenu)
            | Inline::Note(contenu)
            | Inline::Couleur { contenu, .. } => premier_caractere(contenu),
            Inline::Lien { texte, .. } => premier_caractere(texte),
            Inline::Citation(_) => Some('\u{2039}'),
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
                out.push_str("- ");
                generer_inline(item, out, false, regles, &mut contexte);
                out.push('\n');
            }

            out.push('\n');
        }
        Bloc::ListeNumero(items) => {
            // Typst numérote avec `+`, et non avec un chiffre : il suit l'ordre
            // autant que l'usure des listes réordonnées.
            for item in items {
                out.push_str("+ ");
                generer_inline(item, out, false, regles, &mut contexte);
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
            // Le manteau : `#block(...)` autour de ce que le bloc génère.
            // Les propriétés deviennent les arguments du bloc Typst — c'est
            // tout le décor, des bordures aux dégradés.
            out.push_str("#block(");
            ecrire_proprietes_typst(proprietes, decor, out);
            out.push_str(")[\n");

            let mut interieur = String::new();
            generer_bloc(bloc, &mut interieur, regles, decor);
            out.push_str(interieur.trim_end());

            out.push_str("\n]\n\n");
        }
        Bloc::Graphique { type_, titre, donnees } => {
            // Un graphique `cetz` : le paquet est importé par le préambule
            // du document généré. Les données deviennent un tableau de
            // couples (label, valeur) que `cetz` sait dessiner.
            out.push_str("#figure(\n  ");
            match type_.as_str() {
                "ligne" => out.push_str("chart.linechart("),
                "secteur" => out.push_str("chart.piechart("),
                _ => out.push_str("chart.barchart("),
            }
            out.push('(');
            for (i, (label, valeur)) in donnees.iter().enumerate() {
                if i > 0 { out.push_str(", "); }
                out.push_str("(label: \"");
                echapper_chaine(label, out);
                out.push_str(&format!("\", values: ({valeur},))"));
            }
            out.push_str("),");
            if !titre.is_empty() {
                out.push_str("\n  caption: [");
                echapper_chaine(titre, out);
                out.push_str("],");
            }
            out.push_str("\n)\n\n");
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
    let mut en_tete = debut_de_ligne;

    for (rang, element) in inline.iter().enumerate() {
        // Ce qui suit, à travers le balisage : la fin d'un élément n'est
        // pas la fin du texte.
        contexte.suivant = premier_caractere(&inline[rang + 1..]);
        contexte.suivant_texte = premier_texte(&inline[rang + 1..]);

        match element {
            Inline::Texte(texte) => {
                // La typographie de la langue d'abord, l'échappement Typst
                // ensuite — et l'insécable devient `~`. Un tilde tapé, lui,
                // a été échappé : il reste un tilde.
                let compose = appliquer_dans(texte, regles, contexte);
                let compose = super::typographie::petites_capitales_siecles(&compose);
                let mut echappe = String::new();
                echapper_texte(&compose, &mut echappe, en_tete);
                out.push_str(&echappe.replace('\u{A0}', "~"));

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
                generer_inline(contenu, out, false, regles, contexte);
                out.push('*');
                en_tete = false;
            }
            Inline::Italique(contenu) => {
                out.push('_');
                generer_inline(contenu, out, false, regles, contexte);
                out.push('_');
                en_tete = false;
            }
            Inline::GrasItalique(contenu) => {
                out.push_str("*_");
                generer_inline(contenu, out, false, regles, contexte);
                out.push_str("_*");
                en_tete = false;
            }
            Inline::Souligne(contenu) => {
                out.push_str("#underline[");
                generer_inline(contenu, out, false, regles, contexte);
                out.push(']');
                en_tete = false;
            }
            Inline::Barre(contenu) => {
                out.push_str("#strike[");
                generer_inline(contenu, out, false, regles, contexte);
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
            Inline::Couleur { couleur, contenu } => {
                out.push_str("#text(fill: ");
                out.push_str(&expression_couleur(couleur));
                out.push_str(")[");
                generer_inline(contenu, out, false, regles, contexte);
                out.push(']');
                en_tete = false;
            }
            Inline::Citation(references) => {
                generer_citation(references, out);
                en_tete = false;
            }
            Inline::Note(contenu) => {
                out.push_str("#footnote[");
                generer_inline(contenu, out, false, regles, contexte);
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
                generer_inline(texte, out, false, regles, contexte);
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

