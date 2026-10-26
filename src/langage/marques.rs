//! Les notes d'un document.
//!
//! Un appel de références — `@dupont2023[p. 45]; @martin2020` — est **une**
//! note : un seul appel de note réunit ses références, et un seul numéro en
//! sort. Une note libre `^[…]` en est une autre. Toutes partagent un même
//! compteur, dans l'ordre du document : c'est ce que la page montre, et ce
//! que les glyphes doivent montrer à leur tour.
//!
//! Le balayage est purement textuel : il ne s'agit pas de comprendre le
//! document, seulement de trouver les plages que la mise en page fond en un
//! glyphe, et de leur donner leur rang. Le texte reste le vrai — c'est la
//! mise en page qui le tait. Recalculé à chaque frappe, il n'y a rien de
//! paresseux ici : les numéros suivent le document, jamais l'inverse.

/// La nature d'une note.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nature {
    /// Un appel de références : `@dupont2023[p. 45]; @martin2020`.
    Appel,
    /// Une note libre : `^[…]`.
    PiedDePage,
}

/// Une note : sa plage dans le texte, son rang, son glyphe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    /// Le début de la plage, en octets du document.
    pub debut: usize,
    /// La fin de la plage, exclue.
    pub fin: usize,
    /// Son rang dans le compteur partagé, à partir de 1.
    pub numero: usize,
    /// Sa nature.
    pub nature: Nature,
}

/// Les notes d'un document, dans l'ordre, numérotées ensemble.
pub fn notes(texte: &str) -> Vec<Note> {
    let mut trouvees: Vec<(usize, usize, Nature)> = Vec::new();
    let mut octet = 0usize;

    while octet < texte.len() {
        let reste = &texte[octet..];
        let Some(c) = reste.chars().next() else {
            break;
        };
        let suite = octet + c.len_utf8();

        // Un caractère échappé ne déclenche rien.
        if c == '\\' {
            octet = suite + reste[1..].chars().next().map_or(0, char::len_utf8);
            continue;
        }

        // Une note libre : `^[…]`, crochets imbriqués compris. Ce qui s'y
        // trouve appartient à la note — un `@` dedans n'est pas une note de
        // plus.
        if c == '^'
            && texte[suite..].starts_with('[')
            && let Some(fin) = crochet_fermant(texte, suite)
        {
            trouvees.push((octet, fin, Nature::PiedDePage));
            octet = fin;
            continue;
        }

        // Un appel de références : ses références se suivent par `; `, et
        // tout l'appel ne vaut qu'un numéro.
        if c == '@'
            && let Some(fin) = appel_jusqu_a(texte, octet)
        {
            trouvees.push((octet, fin, Nature::Appel));
            octet = fin;
            continue;
        }

        // Un appel de cite brut, tel que le généré l'écrit pour un fichier
        // Typst : `#cite(<a>, …) #cite(<b>)`, un seul espace entre eux.
        if reste.starts_with("#cite(")
            && let Some(fin) = citer_jusqu_a(texte, octet)
        {
            trouvees.push((octet, fin, Nature::Appel));
            octet = fin;
            continue;
        }

        octet = suite;
    }

    trouvees
        .into_iter()
        .enumerate()
        .map(|(rang, (debut, fin, nature))| Note {
            debut,
            fin,
            numero: rang + 1,
            nature,
        })
        .collect()
}

/// L'offset qui suit l'appel de références ouvert à `debut`.
///
/// Une suite de références séparées par `; ` n'est qu'**une** note : c'est
/// elle qui, rendue, ne sort qu'un numéro.
fn appel_jusqu_a(texte: &str, debut: usize) -> Option<usize> {
    let mut octet = debut;

    loop {
        // Le `@` et sa clé.
        octet += 1;
        let cle_debut = octet;
        while let Some(c) = texte[octet..].chars().next() {
            if c.is_alphanumeric() || matches!(c, '_' | ':' | '.' | '-' | '+') {
                octet += c.len_utf8();
            } else {
                break;
            }
        }

        if octet == cle_debut {
            return None; // Un `@` sans clé : ce n'est pas un appel.
        }

        // La page, si elle est là : `[…]`, échappements compris.
        if texte[octet..].starts_with('[') {
            octet = crochet_fermant(texte, octet)?;
        }

        // Une autre référence suit-elle, par `; ` ?
        let reste = texte[octet..].trim_start_matches([' ', '\t']);
        if !(reste.starts_with(';') && reste[1..].trim_start().starts_with('@')) {
            return Some(octet);
        }

        octet += texte[octet..].find('@')?;
    }
}

/// L'offset qui suit la suite d'appels de cite ouverte à `debut`.
fn citer_jusqu_a(texte: &str, debut: usize) -> Option<usize> {
    let mut octet = debut;

    loop {
        let ouvert = texte[octet..].find("#cite(")? + octet;
        let mut profondeur = 0usize;
        let mut curseur = ouvert + "#cite(".len();

        while curseur < texte.len() {
            match texte[curseur..].chars().next()? {
                '\\' => curseur += 1,
                '(' => profondeur += 1,
                ')' => {
                    profondeur = profondeur.saturating_sub(1);
                    if profondeur == 0 {
                        curseur += 1;
                        break;
                    }
                }
                _ => {}
            }

            curseur += texte[curseur..].chars().next()?.len_utf8();
        }

        if profondeur != 0 {
            return None;
        }

        // Un seul espace, puis un autre appel : la suite tient en une note.
        if texte[curseur..].starts_with(" #cite(") {
            octet = curseur + 1;
            continue;
        }

        return Some(curseur);
    }
}

/// L'offset du crochet fermant celui ouvert à `ouvert`, échappements et
/// imbrications compris.
fn crochet_fermant(texte: &str, ouvert: usize) -> Option<usize> {
    let mut profondeur = 0usize;
    let mut octet = ouvert;

    while octet < texte.len() {
        match texte[octet..].chars().next()? {
            '\\' => octet += 1,
            '[' => profondeur += 1,
            ']' => {
                profondeur -= 1;
                if profondeur == 0 {
                    return Some(octet + 1);
                }
            }
            _ => {}
        }

        octet += texte[octet..].chars().next()?.len_utf8();
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Un appel groupé ne vaut qu'un glyphe : une note, pas une par
    /// référence.
    #[test]
    fn une_note_groupee_ne_vaut_qu_un_glyphe() {
        let trouvees = notes("Avant @dupont2023[p. 45]; @martin2020 après.");

        assert_eq!(trouvees.len(), 1);
        assert_eq!(trouvees[0].numero, 1);
        assert_eq!(trouvees[0].nature, Nature::Appel);
        assert_eq!(
            &"Avant @dupont2023[p. 45]; @martin2020 après."[trouvees[0].debut..trouvees[0].fin],
            "@dupont2023[p. 45]; @martin2020"
        );
    }

    /// Les appels et les pieds de page partagent le compteur : c'est la
    /// synchronisation demandée, et elle suit l'ordre du document.
    #[test]
    fn les_appels_et_les_pieds_partagent_le_compteur() {
        let trouvees = notes("@a\n^[Une note libre.]\n@b[p. 2] puis @c.");

        let numeros: Vec<usize> = trouvees.iter().map(|n| n.numero).collect();
        let natures: Vec<Nature> = trouvees.iter().map(|n| n.nature).collect();

        assert_eq!(numeros, vec![1, 2, 3, 4], "un seul rang, dans l'ordre");
        assert_eq!(
            natures,
            vec![
                Nature::Appel,
                Nature::PiedDePage,
                Nature::Appel,
                Nature::Appel
            ]
        );
    }

    /// Le contenu d'un pied de page appartient à la note : un `@` dedans ne
    /// se numérote pas de son côté.
    #[test]
    fn le_contenu_d_un_pied_n_est_pas_une_note_de_plus() {
        let trouvees = notes("^[Voir @dupont2023.] puis rien.");

        assert_eq!(trouvees.len(), 1);
        assert_eq!(trouvees[0].nature, Nature::PiedDePage);
    }

    /// Les numéros sont recalculés : supprimer une note renumérote tout ce
    /// qui suit. C'est là que « dynamique » se prouve.
    #[test]
    fn les_numeros_se_recalculent_avec_le_document() {
        let complet = notes("@a\n@b\n^[note]\n@c");
        let ampute = notes("@b\n^[note]\n@c");

        assert_eq!(
            complet.iter().map(|n| n.numero).collect::<Vec<_>>(),
            vec![1, 2, 3, 4]
        );
        assert_eq!(
            ampute.iter().map(|n| n.numero).collect::<Vec<_>>(),
            vec![1, 2, 3],
            "la note supprimée emporte son rang, et les autres montent"
        );
    }

    /// Un appel de cite brut se reconnaît, et sa suite d'appels ne vaut
    /// toujours qu'une note.
    #[test]
    fn un_appel_de_cite_brut_se_reconnait() {
        let trouvees =
            notes("#cite(<a>, supplement: [p. 45]) #cite(<b>) voilà.");

        assert_eq!(trouvees.len(), 1, "deux appels, une seule note");
        assert_eq!(trouvees[0].nature, Nature::Appel);
    }

    /// Un `@` échappé n'ouvre pas d'appel.
    #[test]
    fn un_arobase_echappe_ne_rien_ouvre() {
        assert!(notes("écrire \\@dupont2023 sans citer.").is_empty());
    }

    /// Un `;` orphelin n'avale jamais la suite : après une suppression qui
    /// laisse « @dupont2023[p. 45]; » seul en bout de ligne, le texte qui
    /// suit reste du texte — sinon le glyphe le cacherait tout entier.
    #[test]
    fn un_point_virgule_orphelin_navale_pas_la_suite() {
        let texte = "Avant @dupont2023[p. 45];\n et tout ce qui suit après.";
        let trouvees = notes(texte);

        assert_eq!(trouvees.len(), 1, "une seule note, celle qui reste");
        assert_eq!(
            &texte[trouvees[0].debut..trouvees[0].fin],
            "@dupont2023[p. 45]",
            "le glyphe ne couvre que l'appel, jamais le texte d'après"
        );
    }
}
