//! Petites opérations de texte partagées par les recherches de l'interface.

/// Normalise un texte pour une recherche : casse et accents français.
///
/// Les listes de commandes et de livres doivent plier les mêmes caractères ;
/// une seule fonction évite que « genèse » trouve un livre dans un panneau et
/// pas dans l'autre.
pub fn normaliser_recherche(texte: &str) -> String {
    let mut out = String::with_capacity(texte.len());
    for c in texte.to_lowercase().chars() {
        match c {
            'à' | 'á' | 'â' | 'ä' | 'ã' | 'å' => out.push('a'),
            'ç' => out.push('c'),
            'è' | 'é' | 'ê' | 'ë' => out.push('e'),
            'ì' | 'í' | 'î' | 'ï' => out.push('i'),
            'ñ' => out.push('n'),
            'ò' | 'ó' | 'ô' | 'ö' | 'õ' => out.push('o'),
            'ù' | 'ú' | 'û' | 'ü' => out.push('u'),
            'ý' | 'ÿ' => out.push('y'),
            'œ' => out.push_str("oe"),
            'æ' => out.push_str("ae"),
            other => out.push(other),
        }
    }
    out
}

/// Ramène un décalage d'octets au début du caractère qu'il traverse.
///
/// Les positions du texte viennent de sources différentes — curseur du
/// widget, calculs de plage, longueur d'un gabarit. Une seule peut tomber au
/// milieu d'un `é`, et une tranche `&texte[..offset]` ferait alors tomber
/// l'application. Ce garde-fou est le point unique où l'on repasse sur une
/// frontière UTF-8.
pub fn offset_sur_caractere(texte: &str, offset: usize) -> usize {
    let mut offset = offset.min(texte.len());
    while offset > 0 && !texte.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_recherche_plie_les_accents_et_la_casse() {
        assert_eq!(normaliser_recherche("Métadonnée"), "metadonnee");
        assert_eq!(normaliser_recherche("THÈSE"), "these");
        assert_eq!(normaliser_recherche("Cœur"), "coeur");
    }

    #[test]
    fn un_decalage_dans_un_accent_revient_sur_la_frontiere() {
        assert_eq!(offset_sur_caractere("é", 1), 0);
        assert_eq!(offset_sur_caractere("aé", 2), 1);
        assert_eq!(offset_sur_caractere("aé", 3), 3);
        assert_eq!(offset_sur_caractere("é", 99), 2);
    }
}
