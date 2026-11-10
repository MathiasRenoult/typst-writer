//! Le sérialiseur : Document structuré → texte (notre syntaxe).
//!
//! C'est l'écriture sur disque. Le texte produit doit être exactement celui que
//! le parseur relit — c'est la moitié « retour » du round-trip, l'autre étant
//! [`crate::langage::parse`].
//!
//! Le sérialiseur **échappe** ce qui ressemblerait à du balisage : un texte qui
//! contient une étoile, un arobase ou une accolade reste du texte. Sans cela,
//! le round-trip perdrait les documents qui parlent du langage lui-même — et
//! une note de bas de page commençant par `#` deviendrait un titre.

use super::schema::{Bloc, Document, Inline, Reference, ouvre_un_bloc};

/// Caractères qui ouvrent ou ferment un marqueur inline, et qu'un texte nu
/// doit échapper.
///
/// L'ordre n'importe pas : chacun est échappé individuellement. Les fermetures
/// `]` et `}` y sont autant que les ouvertures — un crochet nu dans une note
/// la refermerait avant l'heure —, et `_` ouvre l'italique bas.
const MARQUEURS: [char; 12] = ['\\', '*', '`', '@', '[', '{', '}', ']', '^', '_', '|', '$'];

/// Sérialise un document dans notre syntaxe.
///
/// Les blocs sont séparés par une ligne vide, et le texte se termine par un
/// saut de ligne. Un document vide donne une chaîne vide : c'est ce que le
/// parseur relit comme un document vide.
pub fn serialiser(document: &Document) -> String {
    let mut out = String::new();

    for (index, bloc) in document.blocs.iter().enumerate() {
        if index > 0 {
            out.push('\n');
        }

        match bloc {
            Bloc::Titre { niveau, contenu } => {
                out.push_str(&"#".repeat(*niveau as usize));
                out.push(' ');
                ecrire_inline(contenu, &mut out);
                out.push('\n');
            }
            Bloc::Paragraphe(contenu) => {
                ecrire_ligne(contenu, &mut out);
                out.push('\n');
            }
            Bloc::Citation(contenu) => {
                out.push_str("> ");
                ecrire_inline(contenu, &mut out);
                out.push('\n');
            }
            Bloc::ListePuce(items) => {
                for item in items {
                    out.push_str(&"  ".repeat(item.niveau));
                    out.push_str("- ");
                    ecrire_inline(&item.contenu, &mut out);
                    out.push('\n');
                }
            }
            Bloc::ListeNumero(items) => {
                for (numero, item) in items.iter().enumerate() {
                    out.push_str(&"  ".repeat(item.niveau));
                    out.push_str(&format!("{}. ", numero + 1));
                    ecrire_inline(&item.contenu, &mut out);
                    out.push('\n');
                }
            }
            Bloc::Brut(texte) => {
                let fence = fence_for_raw(texte);
                out.push_str(&fence);
                out.push_str("typst\n");
                out.push_str(texte);
                out.push('\n');
                out.push_str(&fence);
                out.push('\n');
            }
            Bloc::Tableau {
                entete,
                lignes,
                alignements,
            } => {
                out.push_str("| ");
                for (rang, cellule) in entete.iter().enumerate() {
                    if rang > 0 {
                        out.push_str(" | ");
                    }
                    ecrire_inline(cellule, &mut out);
                }
                out.push_str(" |\n");

                // La ligne de séparation dit l'alignement, colonne par colonne.
                out.push('|');
                for alignement in alignements {
                    out.push(' ');
                    out.push_str(alignement.trait_());
                    out.push_str(" |");
                }
                out.push('\n');

                for ligne in lignes {
                    out.push_str("| ");
                    for (rang, cellule) in ligne.iter().enumerate() {
                        if rang > 0 {
                            out.push_str(" | ");
                        }
                        ecrire_inline(cellule, &mut out);
                    }
                    out.push_str(" |\n");
                }
            }
            Bloc::Formule(texte) => {
                out.push_str("$$\n");
                out.push_str(texte);
                out.push_str("\n$$\n");
            }
            Bloc::Separateur => {
                out.push_str("---\n");
            }
            Bloc::Commentaire(texte) => {
                out.push_str("// ");
                out.push_str(texte);
                out.push('\n');
            }
            Bloc::SautDePage => {
                out.push_str("\n\n\n");
            }
            Bloc::Image { légende, cible } => {
                out.push_str("![");
                ecrire_inline(légende, &mut out);
                out.push_str("](");
                echapper_cible(cible, &mut out);
                out.push_str(")\n");
            }
            Bloc::Décor { .. } => {
                // La ligne d'attributs d'abord, puis le bloc habillé —
                // les manteaux emboîtés se fondent en un seul.
                ecrire_seul(bloc, &mut out);
            }
            Bloc::Graphique {
                type_,
                titre,
                donnees,
            } => {
                out.push_str(&format!("{{graphique type={type_}"));
                if !titre.is_empty() {
                    out.push_str(&format!(" titre=\"{titre}\""));
                }
                out.push_str("}\n");
                for (label, valeur) in donnees {
                    out.push_str(&format!("{label} | {valeur}\n"));
                }
                out.push('\n');
            }
            Bloc::Glossaire { tout } => {
                out.push_str("{glossaire");
                if *tout {
                    out.push_str(": tout");
                }
                out.push_str("}\n");
            }
            Bloc::Bibliographie => {
                out.push_str("{bibliographie}\n");
            }
            Bloc::IndexBiblique => {
                out.push_str("{{index}}\n");
            }
            Bloc::Sommaire => {
                out.push_str("[[TOC]]\n");
            }
        }
    }

    out
}

/// Écrit un bloc comme s'il était seul — le corps d'un décor.
///
/// Les manteaux emboîtés se rassemblent : les propriétés se fondent dans
/// l'ordre, du dehors au dedans, et le bloc nu reste seul au centre.
/// C'est la réécriture des formes non canoniques, et elle est idempotente.
fn ecrire_seul(bloc: &Bloc, out: &mut String) {
    let mut fondus = crate::langage::schema::Proprietes::vide();
    let mut nu = bloc;

    while let Bloc::Décor { proprietes, bloc } = nu {
        // Les noms aussi se fondent, sans se dédire — un nom dit deux fois
        // n'en dit pas plus.
        for nom in &proprietes.noms {
            if !fondus.noms.contains(nom) {
                fondus.noms.push(nom.clone());
            }
        }

        for (cle, valeur) in &proprietes.paires {
            fondus.poser(cle.clone(), valeur.clone());
        }

        nu = bloc;
    }

    if !fondus.est_vide() {
        out.push('{');
        ecrire_proprietes(&fondus, out);
        out.push_str("}\n");
    }

    let document = Document {
        blocs: vec![nu.clone()],
    };

    out.push_str(&serialiser(&document));
}

/// Écrit les propriétés : `clé=valeur`, les valeurs à espace acccolées.
fn ecrire_proprietes(proprietes: &crate::langage::schema::Proprietes, out: &mut String) {
    // Les noms d'abord — l'intention lit la première — puis les mesures.
    for (position, nom) in proprietes.noms.iter().enumerate() {
        if position > 0 {
            out.push(' ');
        }

        out.push_str(nom);
    }

    if !proprietes.noms.is_empty() && !proprietes.paires.is_empty() {
        out.push(' ');
    }

    for (position, (cle, valeur)) in proprietes.paires.iter().enumerate() {
        if position > 0 {
            out.push(' ');
        }

        out.push_str(cle);
        out.push('=');

        if valeur.contains(' ') {
            out.push('{');
            out.push_str(valeur);
            out.push('}');
        } else {
            out.push_str(valeur);
        }
    }
}

/// Choisit un fence plus long que tout fence au début d'une ligne du contenu.
fn fence_for_raw(texte: &str) -> String {
    let longest = texte
        .lines()
        .map(|ligne| {
            ligne
                .trim_start()
                .chars()
                .take_while(|caractere| *caractere == '`')
                .count()
        })
        .max()
        .unwrap_or(0);

    "`".repeat(longest.max(2) + 1)
}

/// Écrit une ligne de texte, en échappant un marqueur de bloc en tête.
///
/// Un paragraphe qui commence par `- `, `>`, `# `, un numéro suivi de `. ` ou
/// une clôture de bloc brut serait relu comme un autre bloc. Le sérialiseur le
/// protège par une barre oblique, que le parseur sait retirer.
fn ecrire_ligne(contenu: &[Inline], out: &mut String) {
    let debut = out.len();

    ecrire_inline(contenu, out);

    if ouvre_un_bloc(&out[debut..]) {
        out.insert(debut, '\\');
    }
}

/// Écrit une citation dans sa forme canonique : une référence par membre,
/// le texte libre entre guillemets. La lecture regroupe ensuite les livres.
fn ecrire_citation(elements: &[crate::langage::references::ElementCitation]) -> String {
    crate::langage::references::ecrire_elements(
        elements,
        &crate::mise_en_page::modele::References::default(),
    )
}

/// Écrit une suite d'éléments inline.
fn ecrire_inline(inline: &[Inline], out: &mut String) {
    for element in inline {
        match element {
            Inline::Bible(elements) => {
                out.push_str("{{");
                out.push_str(&ecrire_citation(elements));
                out.push_str("}}");
            }
            Inline::BibleForme { elements, forme } => {
                let suffixe = match forme {
                    crate::langage::references::FormeLivre::FrancaiseLongue => "long",
                    crate::langage::references::FormeLivre::FrancaiseCourte => "court",
                    crate::langage::references::FormeLivre::Osis => "osis",
                    crate::langage::references::FormeLivre::SansPoint => "sanspoint",
                };
                out.push_str("{{");
                out.push_str(&ecrire_citation(elements));
                out.push('|');
                out.push_str(suffixe);
                out.push_str("}}");
            }
            Inline::Texte(texte) => echapper_texte(texte, out),
            Inline::Litteral(texte) => {
                // Un littéral est ce que l'échappement a fait : chaque
                // caractère reprend sa barre, et la lecture retombe sur le
                // même littéral.
                for caractere in texte.chars() {
                    out.push('\\');
                    out.push(caractere);
                }
            }
            Inline::Gras(contenu) => {
                out.push_str("**");
                ecrire_inline(contenu, out);
                out.push_str("**");
            }
            Inline::Italique(contenu) => {
                out.push('*');
                ecrire_inline(contenu, out);
                out.push('*');
            }
            Inline::GrasItalique(contenu) => {
                out.push_str("***");
                ecrire_inline(contenu, out);
                out.push_str("***");
            }
            Inline::Souligne(contenu) => {
                out.push_str("__");
                ecrire_inline(contenu, out);
                out.push_str("__");
            }
            Inline::Barre(contenu) => {
                out.push_str("~~");
                ecrire_inline(contenu, out);
                out.push_str("~~");
            }
            Inline::Formule(texte) => {
                out.push('$');
                out.push_str(texte);
                out.push('$');
            }
            Inline::Code(texte) => {
                out.push('`');
                out.push_str(texte);
                out.push('`');
            }
            Inline::Couleur { couleur, contenu } => {
                out.push('{');
                out.push_str(couleur);
                out.push(':');
                ecrire_inline(contenu, out);
                out.push('}');
            }
            Inline::Date(spec) => {
                out.push_str("{date:");
                out.push_str(spec);
                out.push('}');
            }
            Inline::Glo {
                cle,
                majuscule,
                pluriel,
            } => {
                out.push_str("{glo:");
                out.push_str(cle);
                if *majuscule {
                    out.push_str(" majuscule");
                }
                if *pluriel {
                    out.push_str(" pluriel");
                }
                out.push('}');
            }
            Inline::Citation(references) => {
                out.push_str(&texte_appel(references));
            }
            Inline::Note(contenu) => {
                out.push_str("^[");
                ecrire_inline(contenu, out);
                out.push(']');
            }
            Inline::Lien { texte, cible } => {
                out.push('[');
                ecrire_inline(texte, out);
                out.push_str("](");
                echapper_cible(cible, out);
                out.push(')');
            }
        }
    }
}

/// Écrit du texte nu, en échappant les caractères qui ouvrent un marqueur.
fn echapper_texte(texte: &str, out: &mut String) {
    for caractere in texte.chars() {
        if MARQUEURS.contains(&caractere) {
            out.push('\\');
        }

        out.push(caractere);
    }
}

/// Écrit une mention de page, en échappant **tous** les marqueurs.
///
/// Une région littérale s'écrit comme du texte nu : sinon ses caractères
/// feraient signe aux balayages qui l'entourent — un `*` de page refermerait
/// l'italique du voisinage. La lecture déséchappe génériquement.
fn echapper_page(page: &str, out: &mut String) {
    echapper_texte(page, out);
}

/// Écrit une cible de lien : les marqueurs, et sa propre parenthèse.
fn echapper_cible(cible: &str, out: &mut String) {
    for caractere in cible.chars() {
        if MARQUEURS.contains(&caractere) || caractere == ')' {
            out.push('\\');
        }

        out.push(caractere);
    }
}

/// Le texte d'un appel de citation, dans notre syntaxe.
///
/// C'est **la** forme canonique — `@dupont2023[p. 45]; @martin2020` — que le
/// sérialiseur écrit et que le composeur de notes insère : une seule source,
/// pour que le document lu et le document écrit ne divergent jamais.
/// Le texte qu'une note prend dans le document, forme finale : les
/// citations en Typst incorporé. C'est le format qui sait ce qu'il
/// insère — jamais le générateur, que l'interface ne connaît pas
/// (feuille de route, phase 2, point 2.1).
pub fn texte_citation(references: &[Reference]) -> String {
    let mut out = String::new();
    crate::langage::generer::generer_citation(references, &mut out);
    out
}
pub fn texte_appel(references: &[Reference]) -> String {
    let mut out = String::new();

    for (index, reference) in references.iter().enumerate() {
        if index > 0 {
            out.push_str("; ");
        }

        out.push('@');
        out.push_str(&reference.cle);

        if let Some(page) = &reference.page {
            out.push('[');
            echapper_page(page, &mut out);
            out.push(']');
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use crate::langage::schema::ElementListe;
    // ---- L'extension : blocs et propriétés ------------------------

    /// Un attribut habille le bloc qui suit — et le round-trip est exact.
    #[test]
    fn un_attribut_habille_le_bloc_qui_suit() {
        let source = "{bordure=1pt rayon=4pt}\n> une citation encadrée";
        let document = super::super::parse(source);

        assert_eq!(
            document.blocs,
            vec![Bloc::Décor {
                proprietes: crate::langage::schema::Proprietes {
                    noms: Vec::new(),
                    paires: vec![
                        ("bordure".into(), "1pt".into()),
                        ("rayon".into(), "4pt".into()),
                    ],
                },
                bloc: Box::new(Bloc::Citation(vec![Inline::Texte(
                    "une citation encadrée".into()
                )])),
            }]
        );

        assert_eq!(super::super::parse(&super::serialiser(&document)), document);
        assert!(document.est_canonique());
    }

    /// Une valeur à espace s'accôle, et se relit juste.
    #[test]
    fn une_valeur_a_espace_s_accolle() {
        let source = "{remplissage={gradient.linear(#blue, #red)}}\n> le dégradé";
        let document = super::super::parse(source);

        let Bloc::Décor { proprietes, .. } = &document.blocs[0] else {
            panic!("un décor attendu");
        };

        assert_eq!(
            proprietes.obtenir("remplissage"),
            Some("gradient.linear(#blue, #red)")
        );
        assert_eq!(super::super::parse(&super::serialiser(&document)), document);
    }

    /// L'image se lit, s'écrit, et compose une figure.
    #[test]
    fn l_image_se_lit_et_compose() {
        let source = "![Ma légende](figs/planche.png)";
        let document = super::super::parse(source);

        assert!(matches!(
            &document.blocs[..],
            [Bloc::Image { cible, .. }] if cible == "figs/planche.png"
        ));

        let typst = crate::langage::generer::generer(&document);
        assert!(
            typst.contains("#figure(image(\"figs/planche.png\"), caption: ["),
            "vu : {typst}"
        );
        assert_eq!(super::super::parse(&super::serialiser(&document)), document);
    }

    /// Sans légende, pas de figure : l'image seule.
    #[test]
    fn sans_legende_l_image_est_seule() {
        let document = super::super::parse("![](simple.png)");
        let typst = crate::langage::generer::generer(&document);

        assert!(typst.contains("#image(\"simple.png\")"), "vu : {typst}");
    }

    /// Une légende qui porte un lien se lit juste : le délimiteur est lu
    /// depuis la fin, jamais depuis le premier crochet venu.
    #[test]
    fn une_legende_qui_porte_un_lien_se_lit_juste() {
        let source = "![Voir le [plan](https://x.y)](image.png)";
        let document = super::super::parse(source);

        assert!(matches!(
            &document.blocs[..],
            [Bloc::Image { cible, .. }] if cible == "image.png"
        ));

        assert_eq!(super::super::parse(&super::serialiser(&document)), document);
    }

    /// Une formule en ligne se lit et se relit — opaque, comme le code.
    #[test]
    fn une_formule_en_ligne_se_lit() {
        let document = super::super::parse("Soit $x^2$ dans le texte.");

        assert!(
            matches!(
                &document.blocs[..],
                [Bloc::Paragraphe(contenu)] if contenu.iter().any(|i| matches!(i, Inline::Formule(_)))
            ),
            "vu : {:?}",
            document.blocs
        );
        assert_eq!(super::super::parse(&super::serialiser(&document)), document);
    }

    /// Une formule à part tient sur ses lignes, et compose en math.
    #[test]
    fn une_formule_a_part_compose() {
        let source = "$$\nx^2 + y^2 = z^2\n$$";
        let document = super::super::parse(source);

        assert!(
            matches!(&document.blocs[..], [Bloc::Formule(texte)] if texte.contains("z^2")),
            "vu : {:?}",
            document.blocs
        );

        let typst = crate::langage::generer::generer(&document);
        assert!(typst.contains("$\nx^2 + y^2 = z^2\n$"), "vu : {typst}");
        assert_eq!(super::super::parse(&super::serialiser(&document)), document);
    }

    /// Une formule jamais fermée n'est pas un bloc perdu : elle va
    /// jusqu'à la fin, comme un bloc brut.
    #[test]
    fn une_formule_ouverte_ne_perd_rien() {
        let document = super::super::parse("$$\nx^2");

        assert!(
            matches!(&document.blocs[..], [Bloc::Formule(_)]),
            "vu : {:?}",
            document.blocs
        );
    }

    /// Un tableau se lit, s'écrit et se relit — avec ses alignements.
    #[test]
    fn un_tableau_se_lit_et_se_relit() {
        let source = "| A | B |\n| --- | :-: |\n| 1 | 2 |";
        let document = super::super::parse(source);

        assert!(
            matches!(&document.blocs[..], [Bloc::Tableau { alignements, .. }] if alignements.len() == 2),
            "vu : {:?}",
            document.blocs
        );
        assert!(document.est_canonique());
        assert_eq!(super::super::parse(&super::serialiser(&document)), document);
    }

    /// Une barre dans une cellule s'échappe, et reste du contenu.
    #[test]
    fn une_barre_dans_une_cellule_ne_coupe_rien() {
        let source = "| a \\| b | c |\n| --- | --- |";
        let document = super::super::parse(source);

        let Bloc::Tableau { entete, .. } = &document.blocs[0] else {
            panic!("un tableau attendu");
        };

        assert_eq!(entete.len(), 2, "deux cellules : {entete:?}");
        assert_eq!(super::super::parse(&super::serialiser(&document)), document);
    }

    /// Sans ligne de séparation, ce n'est rien : la ligne est du texte.
    #[test]
    fn sans_separation_ce_n_est_rien() {
        let document = super::super::parse("| a | b |\n| c | d |");

        assert!(
            matches!(
                document.blocs[..],
                [Bloc::Paragraphe(_)] | [Bloc::Paragraphe(_), Bloc::Paragraphe(_)]
            ),
            "vu : {:?}",
            document.blocs
        );
    }

    /// Le tableau compose au Typst : ses colonnes, ses alignements, son
    /// en-tête.
    #[test]
    fn le_tableau_compose() {
        let document = super::super::parse("| A | B |\n| --- | :-: |\n| 1 | 2 |");
        let typst = crate::langage::generer::generer(&document);

        assert!(
            typst.contains("#table(columns: 2, align: (left, center), table.header([A], [B])"),
            "vu : {typst}"
        );
        assert!(typst.contains(", [1], [2])"), "vu : {typst}");
    }

    /// Un nom prend ses mesures dans les normes du document — jamais
    /// dans la source. C'est tout l'objet du décor par défaut.
    #[test]
    fn un_nom_prend_ses_mesures_dans_les_normes() {
        let document = super::super::parse("{encadré}\n> la note");

        let decor = crate::mise_en_page::modele::Decor {
            couleur: "#555".to_string(),
            epaisseur: "2pt".to_string(),
            rayon: "6pt".to_string(),
            marge: "10pt".to_string(),
            ..crate::mise_en_page::modele::Decor::default()
        };

        let typst = crate::langage::generer::generer_avec(
            &document,
            &crate::langage::typographie::Regles::aucune(),
            &decor,
        );

        assert!(
            typst.contains(
                "#block(stroke: (paint: rgb(\"#555\"), thickness: 2pt), radius: 6pt, inset: 10pt)["
            ),
            "les normes disent la mesure : {typst}"
        );
    }

    /// La mesure explicite est l'exception — et elle gagne.
    #[test]
    fn la_mesure_explicite_est_l_exception_qui_gagne() {
        let document = super::super::parse("{encadré rayon=0pt}\n> la note");
        let decor = crate::mise_en_page::modele::Decor::default();

        let typst = crate::langage::generer::generer_avec(
            &document,
            &crate::langage::typographie::Regles::aucune(),
            &decor,
        );

        assert!(typst.contains("radius: 0pt"), "l'exception gagne : {typst}");
    }

    /// Ce qui n'est pas une ligne d'attributs reste du texte — et un
    /// paragraphe qui commence par cette forme se protège.
    #[test]
    fn ce_qui_n_est_pas_un_attribut_reste_du_texte() {
        let document = super::super::parse("{rouge: du texte coloré}");
        assert_eq!(
            document.blocs,
            vec![Bloc::Paragraphe(vec![Inline::Couleur {
                couleur: "rouge".into(),
                contenu: vec![Inline::Texte(" du texte coloré".into())],
            }])]
        );

        // Un paragraphe qui commence par une forme d'attribut se protège
        // : sa barre ne se relit pas comme un littéral.
        let texte = super::serialiser(&super::super::parse("{a=1} le reste"));
        assert!(texte.starts_with("\\"), "vu : {texte}");
    }

    use super::*;
    use crate::langage::parse::parse;

    /// Le round-trip : parse(serialiser(doc)) == doc.
    ///
    /// C'est l'invariant du langage intermédiaire : le même texte donne toujours
    /// le même Document, et le même Document toujours le même texte.
    /// Les cas limites nommés par la feuille de route (phase 2, point 2.2) :
    /// chacun porte une forme canonique, et le contrat est
    /// `parse(serialiser(document)) == document`.
    #[test]
    fn le_round_trip_tient_sur_les_cas_limites_nommes() {
        let cas = [
            // Chaîne vide.
            Document { blocs: vec![] },
            // Caractères spéciaux : ils vivent dans les littéraux.
            Document {
                blocs: vec![Bloc::Paragraphe(vec![
                    Inline::Texte("Spéciaux:".to_string()),
                    Inline::Litteral("{}[]#@".to_string()),
                    Inline::Texte("et".to_string()),
                    Inline::Litteral("$&%|".to_string()),
                ])],
            },
            // Unicode varié.
            Document {
                blocs: vec![Bloc::Paragraphe(vec![Inline::Texte(
                    "éàü 中文 🎉 a\u{0301} fin".to_string(),
                )])],
            },
            // Frontières entre éléments : littéral collé au texte, emphase
            // autour du code.
            Document {
                blocs: vec![Bloc::Paragraphe(vec![
                    Inline::Texte("devant".to_string()),
                    Inline::Litteral("!".to_string()),
                    Inline::Gras(vec![Inline::Code("c".to_string())]),
                    Inline::Texte("derrière".to_string()),
                ])],
            },
            // Structures imbriquées.
            Document {
                blocs: vec![Bloc::Paragraphe(vec![Inline::Gras(vec![
                    Inline::Texte("un ".to_string()),
                    Inline::Litteral("*".to_string()),
                ])])],
            },
            // Lignes vides et blocs successifs.
            Document {
                blocs: vec![
                    Bloc::Paragraphe(vec![Inline::Texte("un".to_string())]),
                    Bloc::Paragraphe(vec![Inline::Texte("deux".to_string())]),
                ],
            },
        ];
        for document in cas {
            assert!(
                document.est_canonique(),
                "{document:?} devrait être canonique"
            );
            let regenere = serialiser(&document);
            let relu = parse(&regenere);
            assert_eq!(
                relu, document,
                "le contrat tient pour {document:?} (sérialisé : {regenere:?})"
            );
        }
    }

    #[test]
    fn le_round_trip_conserve_le_document() {
        let texte = "# Titre\n\nUn paragraphe avec du **gras**, de l'*italique* et du `code`.\n\n> Une citation.\n\n- une puce\n- une autre\n\n1. un élément\n2. un autre\n\n```typst\n#figure(...)\n```\n";

        let document = parse(texte);
        let regenere = serialiser(&document);
        let relu = parse(&regenere);

        assert_eq!(relu, document, "le round-trip doit conserver le document");
        assert_eq!(
            regenere, texte,
            "le texte doit être reproduit à l'identique"
        );
    }

    /// Le round-trip conserve aussi les éléments inline riches.
    #[test]
    fn le_round_trip_conserve_les_inline_riches() {
        let texte = "Voir @dupont2023 et [ce lien](https://exemple.fr) et {rouge:du texte coloré} et ^[une précision].\n";

        let document = parse(texte);

        assert_eq!(parse(&serialiser(&document)), document);
    }

    /// Un appel groupé se réécrit tel quel — et reste **un** appel à la
    /// relecture. C'est la forme que le composeur de notes écrit.
    #[test]
    fn un_appel_groupe_se_recrit_tel_quel() {
        let texte = "Voir @dupont2023[p. 45]; @martin2020 et @bernard2021[pp. 12-15].\n";

        let document = parse(texte);
        let regenere = serialiser(&document);

        assert_eq!(regenere, texte);
        assert_eq!(parse(&regenere), document);
    }

    /// Une mention de page qui contient un crochet fermant revient entière au
    /// round-trip : le crochet s'échappe d'une barre oblique.
    #[test]
    fn une_page_qui_contient_un_crochet_revient_entiere() {
        let document = parse("Voir @dupont2023[p. 45, note 2\\]].\n");
        let regenere = serialiser(&document);

        assert_eq!(regenere, "Voir @dupont2023[p. 45, note 2\\]].\n");
        assert_eq!(parse(&regenere), document);
    }

    /// Un texte qui ressemble à du balisage reste du texte.
    ///
    /// C'est le cas qui casse un sérialiseur naïf : une astérisque dans une
    /// phrase, une arobase dans une adresse, une accolade dans une formule.
    #[test]
    fn un_texte_qui_ressemble_a_un_marqueur_est_echappe() {
        // La forme canonique : le texte nu ne porte pas de marqueur, ils
        // vivent dans les littéraux que la lecture en tire.
        let document = Document {
            blocs: vec![Bloc::Paragraphe(vec![
                Inline::Texte("Une ".to_string()),
                Inline::Litteral("*".to_string()),
                Inline::Texte("étoile".to_string()),
                Inline::Litteral("*".to_string()),
                Inline::Texte(", un ".to_string()),
                Inline::Litteral("@".to_string()),
                Inline::Texte("grep et ".to_string()),
                Inline::Litteral("{".to_string()),
                Inline::Texte("accolades".to_string()),
                Inline::Litteral("}".to_string()),
                Inline::Texte(".".to_string()),
            ])],
        };

        let texte = serialiser(&document);

        assert_eq!(
            texte, "Une \\*étoile\\*, un \\@grep et \\{accolades\\}.\n",
            "chaque marqueur reprend sa barre"
        );
        assert_eq!(parse(&texte), document, "et le parseur le relit tel quel");
    }

    /// Un paragraphe qui commence par un marqueur de bloc reste un paragraphe.
    #[test]
    fn un_paragraphe_qui_commence_par_un_marqueur_reste_un_paragraphe() {
        let document = Document {
            blocs: vec![Bloc::Paragraphe(vec![
                Inline::Litteral("-".to_string()),
                Inline::Texte(" pas une puce".to_string()),
            ])],
        };

        let texte = serialiser(&document);

        assert_eq!(texte, "\\- pas une puce\n");
        assert_eq!(parse(&texte), document, "et le parseur le relit tel quel");
    }

    /// Les blocs sont séparés par une ligne vide, sinon le parseur les
    /// recollerait en un seul paragraphe.
    #[test]
    fn les_blocs_sont_separes_par_une_ligne_vide() {
        let document = Document {
            blocs: vec![
                Bloc::Paragraphe(vec![Inline::Texte("Un".to_string())]),
                Bloc::Paragraphe(vec![Inline::Texte("Deux".to_string())]),
            ],
        };

        assert_eq!(serialiser(&document), "Un\n\nDeux\n");
    }

    /// Un bloc Typst qui contient un fence ne doit pas être tronqué au round-trip.
    #[test]
    fn un_bloc_brut_choisit_un_delimiteur_plus_long_que_son_contenu() {
        let document = Document {
            blocs: vec![Bloc::Brut("commande\n```\nreste".to_string())],
        };

        let source = serialiser(&document);

        assert!(source.starts_with("````typst\n"));
        assert_eq!(parse(&source), document);
    }

    // ---- La propriété de round-trip, sur des documents au hasard ----
    //
    // Le programme de fiabilisation exige `parse(serialiser(AST)) == AST`
    // pour tout AST canonique. Trois tests d'exemple ne prouvent rien : c'est
    // un générateur qui produit les documents, et la propriété qui les juge.

    /// Un congruentiel linéaire : le même fil de nombres à chaque suite, et
    /// un échec reproductible sans dépendance de test.
    struct Hasard(u64);

    impl Hasard {
        fn suivant(&mut self) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            self.0
        }

        fn entier(&mut self, max: usize) -> usize {
            (self.suivant() % max as u64) as usize
        }

        fn un_parmi<'a, T>(&mut self, options: &'a [T]) -> &'a T {
            &options[self.entier(options.len())]
        }
    }

    /// Des mots de vraie vie — et ceux du langage lui-même, qui doivent
    /// rester des mots.
    const MOTS: &[&str] = &[
        "Église",
        "théologie",
        "grâce",
        "un",
        "des",
        "fin.",
        "2 * 3",
        "a_b",
        "a*b",
        "un ]",
        "{x}",
        "^y",
        "@z",
        "2. x",
        "- y",
        "# z",
        "> w",
        "1. v",
        "x\\y",
        "(parenthèse)",
        "10:30",
        "…",
        "'apostrophe'",
        "3.14",
        "-2",
        "_italique_",
        "**gras**",
        "`` ` ``",
        "]}",
        ")(",
        ";",
        "tab\\ulation",
        "^[]",
        "@a[p. 45]",
        "~~",
        "~~x~~",
        "¿",
        "une étoile * ici",
    ];

    /// Un texte nu au hasard — jamais de marqueur : ils vivent dans les
    /// littéraux, et c'est ce qui rend l'écriture univoque.
    fn texte_au_hasard(hasard: &mut Hasard) -> Inline {
        let mots: Vec<&str> = MOTS
            .iter()
            .copied()
            .filter(|mot| !mot.contains(['\\', '*', '`', '@', '[', '{', '}', ']', '^', '_']))
            .collect();

        let mut texte = (*hasard.un_parmi(&mots)).to_string();

        // Un blanc aux bords, parfois : `parse` le rogne, donc ce n'est
        // pas du canonique — et c'est exactement ce que la propriété doit
        // rencontrer pour valoir quelque chose.
        match hasard.entier(6) {
            0 => texte.insert(0, ' '),
            1 => texte.push(' '),
            2 => {
                texte.insert(0, ' ');
                texte.push(' ');
            }
            _ => {}
        }

        Inline::Texte(texte)
    }

    /// Un littéral au hasard : ce que l'échappement insère mot à mot.
    fn litteral_au_hasard(hasard: &mut Hasard) -> Inline {
        Inline::Litteral(
            (*hasard.un_parmi(&[
                "*", "@", "^", "[", "]", "{", "}", "`", "_", "\\", "**", "a*b", "]]", "{}",
            ]))
            .to_string(),
        )
    }

    /// Des références groupées, comme un appel de citation.
    fn references_au_hasard(hasard: &mut Hasard) -> Vec<Reference> {
        let nombre = 1 + hasard.entier(3);

        (0..nombre)
            .map(|_| {
                let cle = format!(
                    "{}{}",
                    hasard.un_parmi(&["dupont", "martin", "x_1", "a-b", "cle9"]),
                    hasard.entier(100)
                );

                match hasard.entier(3) {
                    0 => Reference::new(cle),
                    1 => Reference::new(cle).avec_page(format!("p. {}", 1 + hasard.entier(200))),
                    _ => Reference::new(cle).avec_page(format!(
                        "pp. {}-{}",
                        3,
                        1 + hasard.entier(50)
                    )),
                }
            })
            .collect()
    }

    /// Une emphase plate au hasard : son contenu ne contient aucune emphase.
    fn emphase_au_hasard(hasard: &mut Hasard) -> Inline {
        let contenu = inlines_au_hasard(hasard, true);

        match hasard.entier(3) {
            0 => Inline::Gras(contenu),
            1 => Inline::Italique(contenu),
            _ => Inline::GrasItalique(contenu),
        }
    }

    /// Des éléments inline au hasard ; `dans_emphase` interdit l'emphase.
    ///
    /// La suite est normalisée après coup : deux textes s'y joignent, et un
    /// lien collé à un appel sans page reçoit son espace de séparation. Le
    /// générateur ne produit que du canonique — c'est la prédicat qui le
    /// vérifie, et la propriété qui en découle.
    fn inlines_au_hasard(hasard: &mut Hasard, dans_emphase: bool) -> Vec<Inline> {
        let nombre = 1 + hasard.entier(3);

        let mut suite: Vec<Inline> = (0..nombre)
            .map(|_| inline_au_hasard(hasard, dans_emphase))
            .collect();

        normaliser_la_suite(&mut suite);
        suite
    }

    /// Deux textes se joignent ; un lien sépare d'un appel sans page.
    fn normaliser_la_suite(suite: &mut Vec<Inline>) {
        let appel_nu = |element: &Inline| match element {
            Inline::Citation(references) => references
                .last()
                .is_some_and(|derniere| derniere.page.is_none()),
            _ => false,
        };

        let mut rangee: Vec<Inline> = Vec::new();

        for element in suite.drain(..) {
            // Collé à un appel sans page, un lien reçoit son espace — sinon
            // son crochet serait lu comme la page —, et un texte qui
            // commencerait par un caractère de clé aussi — sinon il
            // grossirait la clé.
            if rangee.last().is_some_and(appel_nu) {
                let colle = match &element {
                    Inline::Lien { .. } => true,
                    Inline::Texte(texte) => {
                        texte.starts_with(|c: char| c.is_alphanumeric() || c == '-' || c == '_')
                    }
                    _ => false,
                };

                if colle {
                    rangee.push(Inline::Texte(" ".to_string()));
                }
            }

            match (rangee.last_mut(), element) {
                (Some(Inline::Texte(tenue)), Inline::Texte(autre)) => tenue.push_str(&autre),
                (Some(Inline::Litteral(tenue)), Inline::Litteral(autre)) => tenue.push_str(&autre),
                (_, element) => rangee.push(element),
            }
        }

        // Dernière passe : un « ; » nu entre deux appels serait lu comme le
        // séparateur du groupe. On lui donne un corps.
        for i in 1..rangee.len().saturating_sub(1) {
            let suspect = matches!(&rangee[i], Inline::Texte(texte) if texte.trim() == ";")
                && matches!(rangee[i - 1], Inline::Citation(_))
                && matches!(rangee[i + 1], Inline::Citation(_));

            if suspect && let Inline::Texte(texte) = &mut rangee[i] {
                texte.push_str(" .");
            }
        }

        *suite = rangee;
    }

    fn inline_au_hasard(hasard: &mut Hasard, dans_emphase: bool) -> Inline {
        // La portée des choix : sans emphase sous une emphase.
        let portee = if dans_emphase { 7 } else { 9 };

        match hasard.entier(portee) {
            0..=1 => texte_au_hasard(hasard),
            2 => litteral_au_hasard(hasard),
            3 => {
                if hasard.entier(2) == 0 {
                    Inline::Code(
                        (*hasard.un_parmi(&["x", "#f()", "arr[0]", "*ptr", "a]b", "}", ")"]))
                            .to_string(),
                    )
                } else {
                    Inline::Formule(formule_au_hasard(hasard))
                }
            }
            4 => Inline::Couleur {
                couleur: (*hasard.un_parmi(&["red", "#ff0000", "rgb(1, 2, 0)", "blue"]))
                    .to_string(),
                contenu: inlines_au_hasard(hasard, dans_emphase),
            },
            5 => Inline::Note(inlines_au_hasard(hasard, dans_emphase)),
            6 => Inline::Lien {
                texte: inlines_au_hasard(hasard, dans_emphase),
                cible: (*hasard.un_parmi(&[
                    "https://x.y/a",
                    "a(b)c",
                    "p\\q",
                    "x)y",
                    "https://x.y/[z]",
                ]))
                .to_string(),
            },
            7 => Inline::Citation(references_au_hasard(hasard)),
            _ => emphase_au_hasard(hasard),
        }
    }

    /// Un paragraphe au hasard, dans la forme canonique : s'il commencerait
    /// par un marqueur de bloc, le caractère se prend en littéral.
    fn paragraphe_au_hasard(hasard: &mut Hasard) -> Vec<Inline> {
        let mut contenu = inlines_au_hasard(hasard, false);

        if let Some(Inline::Texte(texte)) = contenu.first_mut()
            && crate::langage::schema::ouvre_un_bloc(texte)
        {
            let premier = texte.remove(0);
            let suite = texte.clone();
            contenu[0] = Inline::Litteral(premier.to_string());

            if !suite.is_empty() {
                contenu.insert(1, Inline::Texte(suite));
            }
        }

        contenu
    }

    /// Un bloc au hasard.
    fn bloc_au_hasard(hasard: &mut Hasard) -> Bloc {
        // Le décor habille un bloc simple : jamais un décor dans un décor,
        // c'est la forme canonique.
        if hasard.entier(10) == 0 {
            return Bloc::Décor {
                proprietes: proprietes_au_hasard(hasard),
                bloc: Box::new(bloc_simple_au_hasard(hasard)),
            };
        }

        bloc_simple_au_hasard(hasard)
    }

    /// Une image au hasard — légende riche, cible sans parenthèse.
    fn image_au_hasard(hasard: &mut Hasard) -> Bloc {
        Bloc::Image {
            légende: inlines_au_hasard(hasard, false),
            cible: (*hasard.un_parmi(&[
                "image.png",
                "figs/planche-1.jpg",
                "a-b_c.png",
                "dossier/image.png",
            ]))
            .to_string(),
        }
    }

    /// Des propriétés au hasard — y compris des valeurs à espace, qui
    /// s'accollent, et des clés que la table ne connaît pas : le Typst
    /// reste le maître.
    fn proprietes_au_hasard(hasard: &mut Hasard) -> crate::langage::schema::Proprietes {
        let mut proprietes = crate::langage::schema::Proprietes::vide();

        // Les noms d'abord — l'intention — puis les mesures.
        for _ in 0..hasard.entier(3) {
            let nom = *hasard.un_parmi(&["encadré", "plein", "discret", "encadre"]);

            if !proprietes.noms.contains(&nom.to_string()) {
                proprietes.noms.push(nom.to_string());
            }
        }

        for _ in 0..1 + hasard.entier(3) {
            let cle = *hasard.un_parmi(&[
                "largeur",
                "hauteur",
                "bordure",
                "remplissage",
                "rayon",
                "marge",
                "marge_ext",
                "espace",
                "personnalisee",
            ]);
            let valeur = *hasard.un_parmi(&[
                "1pt",
                "50%",
                "auto",
                "luma(240)",
                "#ff0000",
                "4pt",
                "gradient.linear(#blue, #red)",
                "rgb(1, 2, 0)",
            ]);

            // Une clé ne se dit qu'une fois : c'est la règle canonique.
            if proprietes.obtenir(cle).is_none() {
                proprietes.poser(cle, valeur);
            }
        }

        proprietes
    }

    /// Un tableau au hasard : ses colonnes d'accord, ses cellules riches.
    fn tableau_au_hasard(hasard: &mut Hasard) -> Bloc {
        let colonnes = 1 + hasard.entier(3);
        let rangs = 1 + hasard.entier(3);

        let alignements = (0..colonnes)
            .map(|_| {
                use crate::langage::schema::Alignement;

                match hasard.entier(3) {
                    0 => Alignement::Centre,
                    1 => Alignement::Droite,
                    _ => Alignement::Gauche,
                }
            })
            .collect();

        let rang = |hasard: &mut Hasard| -> Vec<Vec<Inline>> {
            (0..colonnes)
                .map(|_| inlines_au_hasard(hasard, false))
                .collect()
        };

        let entete = rang(hasard);
        let lignes = (0..rangs).map(|_| rang(hasard)).collect();

        Bloc::Tableau {
            entete,
            lignes,
            alignements,
        }
    }

    /// Une formule au hasard — jamais de dollar dedans : c'est sa barre.
    fn formule_au_hasard(hasard: &mut Hasard) -> String {
        (*hasard.un_parmi(&[
            "x^2 + y^2 = z^2",
            "sum_(i=1)^n i = (n(n+1))/2",
            "alpha beta gamma",
            "vec(v) = mat(a; b)",
        ]))
        .to_string()
    }

    fn bloc_simple_au_hasard(hasard: &mut Hasard) -> Bloc {
        match hasard.entier(10) {
            0 => Bloc::Titre {
                niveau: (1 + hasard.entier(6)) as u8,
                contenu: inlines_au_hasard(hasard, false),
            },
            1 => Bloc::Paragraphe(paragraphe_au_hasard(hasard)),
            2 => Bloc::Citation(inlines_au_hasard(hasard, false)),
            3 => Bloc::ListePuce(
                (0..1 + hasard.entier(3))
                    .map(|_| ElementListe {
                        niveau: 0,
                        contenu: inlines_au_hasard(hasard, false),
                    })
                    .collect(),
            ),
            4 => Bloc::ListeNumero(
                (0..1 + hasard.entier(3))
                    .map(|_| ElementListe {
                        niveau: 0,
                        contenu: inlines_au_hasard(hasard, false),
                    })
                    .collect(),
            ),
            5 => Bloc::Brut(
                (*hasard.un_parmi(&[
                    "#let x = 1",
                    "#figure(\n  image(\"a.png\"),\n)",
                    "```\nfence interne\n```",
                    "",
                    "une ligne\net une autre",
                ]))
                .to_string(),
            ),
            6 => image_au_hasard(hasard),
            7 => tableau_au_hasard(hasard),
            8 => Bloc::Formule(formule_au_hasard(hasard)),
            _ => Bloc::Paragraphe(paragraphe_au_hasard(hasard)),
        }
    }

    /// Un document au hasard.
    fn document_au_hasard(hasard: &mut Hasard) -> Document {
        Document {
            blocs: (0..1 + hasard.entier(5))
                .map(|_| bloc_au_hasard(hasard))
                .collect(),
        }
    }

    /// La garantie centrale : `parse(serialiser(AST)) == AST` pour tout AST
    /// **canonique** — et pour lui seul. C'est le contrat réel : la
    /// propriété ne promet rien aux formes que `parse` rogne à la lecture,
    /// et doit en rencontrer pour valoir quelque chose.
    #[test]
    fn le_round_trip_tient_sur_tout_ast_canonique() {
        let mut hasard = Hasard(42);
        let mut canoniques = 0;
        let mut refuses = 0;

        for _ in 0..500 {
            let document = document_au_hasard(&mut hasard);

            if !document.est_canonique() {
                refuses += 1;
                continue;
            }

            canoniques += 1;

            let texte = serialiser(&document);
            let relu = parse(&texte);

            assert_eq!(
                relu, document,
                "round-trip rompu.\nSérialisé :\n{texte}\nRelu :\n{relu:?}"
            );

            // Et la forme écrite est stable : rouvrir puis réécrire sans
            // toucher ne reformate pas le fichier.
            assert_eq!(
                serialiser(&relu),
                texte,
                "la forme écrite n'est pas stable :\n{texte}"
            );
        }

        assert!(
            refuses > 0 && canoniques > 0,
            "la propriété doit voir les deux côtés du contrat : \
             {canoniques} canoniques, {refuses} refusés"
        );
    }

    /// Toute sérialisation d'un document **non** canonique retombe sur un
    /// point fixe : rouvrir puis réécrire ne bouge plus.
    ///
    /// C'est la stabilité textuelle : un fichier quelconque se normalise une
    /// fois, puis tient. L'ouverture et la sauvegarde sans modification ne
    /// reformateront jamais indéfiniment.
    #[test]
    fn la_canonicalisation_est_idempotente() {
        let mut hasard = Hasard(1742);

        for _ in 0..500 {
            let document = document_au_hasard(&mut hasard);

            // Une première passe normalise — un blanc aux bords y perd sa
            // place, c'est écrit dans la grammaire. Ce qui compte est que
            // **la seconde ne bouge plus** : le fichier tient.
            let premier = serialiser(&document);
            let second = serialiser(&parse(&premier));
            let troisieme = serialiser(&parse(&second));

            assert_eq!(
                troisieme, second,
                "rouvrir puis réécrire reformate sans fin :\n{second}\n→\n{troisieme}"
            );
        }
    }
}

/// Les balises du glossaire : le round-trip exact, et la grammaire.
#[cfg(test)]
mod tests_glossaire {
    use super::super::parse::parse;
    use super::super::schema::{Bloc, Inline};
    use super::serialiser;

    /// Toutes les formes vont et viennent sans perte — l'invariant du
    /// schéma, `{glo:}` compris.
    #[test]
    fn les_balises_du_glossaire_vont_et_viennent() {
        let source = "Les {glo:sdo} et {glo:sdo pluriel}, {glo:sdo majuscule} puis {glo:sdo majuscule pluriel}.\n\n{glossaire}\n\n{glossaire: tout}\n";
        let document = parse(source);
        assert_eq!(serialiser(&document), source, "le round-trip est exact");
    }

    /// Une clé qui n'est pas un identifiant n'est pas un terme : la
    /// balise reste du voisinage de la couleur, comme avant.
    #[test]
    fn une_cle_qui_n_est_pas_un_identifiant_n_est_pas_un_terme() {
        let document = parse("Le {glo:deux mots} ici.\n");
        let typs = format!("{:?}", document);
        assert!(!typs.contains("Glo"), "{typs}");
    }

    /// Les mots-clés se lisent pour ce qu'ils sont.
    #[test]
    fn les_mots_cles_se_lisent() {
        let document = parse("{glo:sdo majuscule pluriel}\n");
        let (cle, majuscule, pluriel) = match &document.blocs[0] {
            Bloc::Paragraphe(contenu) => match &contenu[0] {
                Inline::Glo {
                    cle,
                    majuscule,
                    pluriel,
                } => (cle.clone(), *majuscule, *pluriel),
                autre => panic!("pas un terme : {autre:?}"),
            },
            autre => panic!("pas un paragraphe : {autre:?}"),
        };
        assert_eq!(cle, "sdo");
        assert!(majuscule);
        assert!(pluriel);

        // Le bloc de liste, lui, connaît sa variante.
        let document = parse("{glossaire: tout}\n");
        assert!(
            matches!(document.blocs[0], Bloc::Glossaire { tout: true }),
            "{:?}",
            document.blocs
        );
    }
}
