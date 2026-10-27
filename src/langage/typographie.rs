//! La typographie selon la langue : des règles strictes, choisies dans les
//! normes du document.
//!
//! L'orthographe française veut une espace **insécable** devant les signes
//! doubles — `;`, `:`, `!`, `?` — et dans les guillemets ; l'anglaise veut,
//! elle, aucune espace devant la ponctuation. Ce sont des règles de
//! grammaire : elles se **choisissent** dans la fenêtre des normes, et le
//! générateur les applique au texte qu'il compose.
//!
//! Deux portes de sortie, et elles suffisent à tout dire :
//!
//! - un **littéral** (`Inline::Litteral`, écrit `\x` dans la syntaxe) ne
//!   reçoit aucune règle : c'est l'escapement universel ;
//! - le code inline et les blocs bruts sont littéraux par nature.
//!
//! L'espace insécable s'écrit `~` en Typst ; le moteur la porte en
//! espace insécable Unicode pendant son travail, que le générateur traduit
//! en `~` — un tilde tapé dans le texte reste un tilde.

/// La typographie appliquée au texte composé.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Typographie {
    /// Aucune règle : le texte est pris tel quel.
    #[default]
    Aucune,

    /// Le français : espace insécable devant les signes doubles et dans les
    /// guillemets, espace normale après.
    Francaise,

    /// L'anglais : aucune espace devant la ponctuation, espace normale après.
    Anglaise,
}

impl Typographie {
    /// La typographie nommée dans les normes : `aucune`, `française`,
    /// `anglaise`.
    pub fn depuis(nom: &str) -> Self {
        match nom.trim().to_lowercase().as_str() {
            "française" | "francaise" | "fr" => Typographie::Francaise,
            "anglaise" | "anglaise " | "en" => Typographie::Anglaise,
            _ => Typographie::Aucune,
        }
    }

}

/// L'espace insécable, portée en Unicode pendant le travail.
const INSECABLE: char = '\u{A0}';

/// Applique les règles de la langue à un texte nu.
///
/// Le résultat porte les insécables en [`INSECABLE`] : c'est au générateur
/// de les traduire en `~` après son échappement, pour qu'un tilde tapé reste
/// un tilde. Un texte sans règle à appliquer sort inchangé.
pub fn appliquer(texte: &str, typographie: Typographie) -> String {
    if typographie == Typographie::Aucune || texte.is_empty() {
        return texte.to_string();
    }

    let francais = typographie == Typographie::Francaise;
    let mut sortie = String::new();
    let caracteres: Vec<char> = texte.chars().collect();

    for (position, caractere) in caracteres.iter().enumerate() {
        let suivant = caracteres.get(position + 1);

        match caractere {
            // L'espace se réduit à une seule, et se pose **avant** le
            // prochain caractère utile — dont la règle décidera de sa nature.
            c if c.is_whitespace() => {
                // La fin du texte : l'espace est du contenu, qui tient entre
                // les éléments. Seules les suites s'absorbent.
                let Some(&prochain) = suivant else {
                    if !sortie.ends_with(' ') && !sortie.ends_with(INSECABLE) {
                        sortie.push(' ');
                    }

                    continue;
                };

                // Une suite d'espaces n'en fait qu'un.
                if prochain.is_whitespace() {
                    continue;
                }

                // En français, l'espace d'avant un signe double est absorbée
                // par l'insécable que le signe posera lui-même.
                if francais && matches!(prochain, ';' | ':' | '!' | '?' | '»') {
                    continue;
                }

                // Un signe qui se colle n'en veut pas davantage : sa branche
                // aura déjà coupé, et il ne faut pas repousser la question.
                if matches!(prochain, ',' | '.') {
                    continue;
                }

                if !sortie.ends_with(' ') && !sortie.ends_with(INSECABLE) {
                    sortie.push(' ');
                }
            }

            // Les signes doubles : en français, insécable devant et espace
            // normale après — l'exemple du programme. En anglais, la
            // ponctuation colle : rien devant, une espace après.
            ';' | ':' | '!' | '?' => {
                while sortie.ends_with(' ') {
                    sortie.pop();
                }

                if francais && !sortie.is_empty() && !sortie.ends_with(INSECABLE) {
                    sortie.push(INSECABLE);
                }

                sortie.push(*caractere);

                if suivant.is_some_and(|c| !c.is_whitespace()) {
                    sortie.push(' ');
                }
            }

            // La virgule et le point ne s'ouvrent jamais — `3.14` tient —
            // mais n'ajoutent pas d'espace après : leur voisinage est au
            // texte (abréviations, décimales, initiales).
            ',' | '.' => {
                while sortie.ends_with(' ') {
                    sortie.pop();
                }

                sortie.push(*caractere);
            }

            // Les guillemets français portent l'insécable dedans.
            '«' => {
                sortie.push(*caractere);

                if francais && suivant.is_some() && !sortie.ends_with(INSECABLE) {
                    sortie.push(INSECABLE);
                }
            }
            '»' => {
                while sortie.ends_with(' ') {
                    sortie.pop();
                }

                if francais && !sortie.is_empty() && !sortie.ends_with(INSECABLE) {
                    sortie.push(INSECABLE);
                }

                sortie.push(*caractere);
            }

            _ => {
                sortie.push(*caractere);
            }
        }
    }

    sortie
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Le cas du programme : en français, `;` est précédé d'une espace
    /// insécable et suivi d'une espace normale — exactement.
    #[test]
    fn en_francais_le_point_virgule_est_strict() {
        let typo = Typographie::Francaise;

        assert_eq!(appliquer("a; b", typo), format!("a{INSECABLE}; b"));
        assert_eq!(appliquer("a ; b", typo), format!("a{INSECABLE}; b"));
        assert_eq!(appliquer("a;b", typo), format!("a{INSECABLE}; b"));
        assert_eq!(appliquer("a ;  b", typo), format!("a{INSECABLE}; b"));
    }

    /// Les signes doubles suivent la même règle, et les guillemets aussi.
    #[test]
    fn en_francais_les_signes_doubles_et_les_guillemets() {
        let typo = Typographie::Francaise;

        assert_eq!(appliquer("oui !", typo), format!("oui{INSECABLE}!"));
        assert_eq!(appliquer("non ?", typo), format!("non{INSECABLE}?"));
        assert_eq!(appliquer("12 : 30", typo), format!("12{INSECABLE}: 30"));
        assert_eq!(
            appliquer("« bonjour »", typo),
            format!("«{INSECABLE}bonjour{INSECABLE}»")
        );
    }

    /// La virgule et le point ne s'ouvrent jamais, en français comme en
    /// anglais — et jamais d'espace forcée après : `3.14` doit tenir.
    #[test]
    fn la_virgule_et_le_point_ne_s_ouvrent_jamais() {
        for typo in [Typographie::Francaise, Typographie::Anglaise] {
            assert_eq!(appliquer("a , b", typo), "a, b", "{typo:?}");
            assert_eq!(appliquer("a,  b", typo), "a, b", "{typo:?}");
            assert_eq!(appliquer("fin .", typo), "fin.", "{typo:?}");
            assert_eq!(appliquer("3.14", typo), "3.14", "{typo:?}");
            assert_eq!(appliquer("1,2", typo), "1,2", "{typo:?}");
        }
    }

    /// En anglais, la ponctuation colle : aucune espace devant, une après.
    #[test]
    fn en_anglais_la_ponctuation_colle() {
        let typo = Typographie::Anglaise;

        assert_eq!(appliquer("a ; b", typo), "a; b");
        assert_eq!(appliquer("a:b", typo), "a: b");
        assert_eq!(appliquer("Hello !", typo), "Hello!");
        assert_eq!(appliquer("why ?", typo), "why?");
    }

    /// Sans typographie, rien ne bouge — et les doubles espaces restent.
    #[test]
    fn sans_typographie_rien_ne_bouge() {
        assert_eq!(appliquer("a ;  b", Typographie::Aucune), "a ;  b");
    }
}
