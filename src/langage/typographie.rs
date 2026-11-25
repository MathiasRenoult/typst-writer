//! La typographie selon la langue : des règles strictes, **réglables**.
//!
//! Les normes du document portent un jeu de règles — la typographie — que le
//! générateur applique au texte composé. Un **défaut** de langue les charge
//! (française, anglaise, aucune), et chaque règle se reprend ensuite : c'est
//! le panneau de typographie de la fenêtre des normes.
//!
//! Deux portes de sortie, et elles suffisent à tout dire :
//!
//! - un **littéral** (`Inline::Litteral`, écrit `\x` dans la syntaxe) ne
//!   reçoit aucune règle : c'est l'escapement universel ;
//! - le code inline et les blocs bruts sont littéraux par nature.
//!
//! Les guillemets s'écrivent `«` et `»` dans la syntaxe — c'est la notation
//! du langage, à l'abri des apostrophes —, et le moteur les **compose** avec
//! les caractères choisis par niveau d'imbrication : `«`, `"` puis `'` en
//! français ; `"`, `'` puis `"` en anglais.
//!
//! L'espace insécable s'écrit `~` en Typst ; le moteur la porte en insécable
//! Unicode pendant son travail, que le générateur traduit en `~` — un tilde
//! tapé dans le texte reste un tilde.

/// La place d'une espace dans la composition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Espace {
    /// Aucune espace : le signe colle.
    #[default]
    Aucune,
    /// Une espace normale.
    Normale,
    /// Une espace insécable — `~` en Typst.
    Insécable,
    /// Une espace **fine** insécable — devant `; : ! ?`, la française
    /// véritable : moins large, et jamais cassée.
    FineInsécable,
}

impl Espace {
    /// L'espace nommée dans les normes : `aucune`, `normale`, `insécable`.
    pub fn depuis(nom: &str) -> Self {
        match nom.trim().to_lowercase().as_str() {
            "normale" => Espace::Normale,
            "insécable" => Espace::Insécable,
            "fine-insécable" | "fine" | "fine insécable" => Espace::FineInsécable,
            _ => Espace::Aucune,
        }
    }

    /// Le nom porté dans les normes.
    pub fn nom(self) -> &'static str {
        match self {
            Espace::Aucune => "aucune",
            Espace::Normale => "normale",
            Espace::Insécable => "insécable",
            Espace::FineInsécable => "fine-insécable",
        }
    }

    /// Le caractère de l'espace, ou rien.
    fn caractere(self) -> Option<char> {
        match self {
            Espace::Aucune => None,
            Espace::Normale => Some(' '),
            Espace::Insécable => Some(INSECABLE),
            Espace::FineInsécable => Some(FINE),
        }
    }
}

/// L'espace fine insécable — U+202F.
pub const FINE: char = '\u{202f}';

/// Les règles de typographie appliquées au texte composé.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Regles {
    /// L'espace devant les signes doubles `; : ! ?`.
    pub avant_signes: Espace,

    /// L'espace après les signes doubles `; : ! ?`.
    pub apres_signes: Espace,

    /// L'espace après la virgule et le point, **hors nombres** : `3.14` et
    /// `1,2` sont pris tels quels, `fin. La suite` reçoit sa espace.
    pub apres_virgule_point: Espace,

    /// L'espace à l'intérieur des guillemets.
    pub espace_guillemets: Espace,

    /// Les paires de guillemets, **par niveau d'imbrication** : la première
    /// pour les citations les plus hautes, les suivantes pour ce qu'elles
    /// contiennent. Le dernier niveau sert au-delà.
    pub guillemets: Vec<(char, char)>,

    /// Les caractères de frappe qui **ouvrent** une citation. Ce sont des
    /// déclencheurs : la paire produite reste celle de `guillemets`.
    pub guillemets_ouvrants: Vec<char>,

    /// Les caractères de frappe qui **ferment** une citation.
    pub guillemets_fermants: Vec<char>,
    /// La syntaxe des références bibliques, telle que les normes la
    /// choisissent (le système de références).
    pub references: crate::mise_en_page::modele::References,

    /// La configuration de l'index des citations.
    pub index: crate::mise_en_page::modele::IndexCitations,

    /// Les personnes du registre, pour que `{{p clé}}` affiche son nom.
    pub personnes: crate::project::personnes::Personnes,

    /// La configuration de l'index des personnes.
    pub index_personnes: crate::mise_en_page::modele::IndexPersonnes,

    /// La forme de la numérotation des figures : "1", "A", "a", "I", "i", "1.1".
    /// Vide = les défauts de Typst.
    pub numerotation_figures: String,

    /// Le mot qui précède le numéro d'une figure : « Figure », « Fig. ».
    pub legende_prefixe: String,

    /// Les callouts prédéfinis du document, pour que `{callout note}` trouve
    /// ses couleurs et ses mesures dans la norme.
    pub callouts: crate::mise_en_page::modele::ParametresCallouts,

    /// Les mots et formats de la balise `{date:...}` — siècles,
    /// millénaires, dates pleines.
    pub dates: crate::langage::dates::Regles,
}

impl Regles {
    /// Aucune règle : le texte est pris tel quel.
    pub fn aucune() -> Self {
        Self {
            avant_signes: Espace::Aucune,
            apres_signes: Espace::Aucune,
            apres_virgule_point: Espace::Aucune,
            espace_guillemets: Espace::Aucune,
            guillemets: vec![('«', '»'), ('"', '"'), ('\'', '\'')],
            guillemets_ouvrants: vec!['«'],
            guillemets_fermants: vec!['»'],
            numerotation_figures: String::new(),
            legende_prefixe: "Figure".to_string(),
            callouts: crate::mise_en_page::modele::ParametresCallouts::default(),
            dates: crate::langage::dates::Regles::default(),
            references: crate::mise_en_page::modele::References::default(),
            index: crate::mise_en_page::modele::IndexCitations::default(),
            personnes: crate::project::personnes::Personnes::default(),
            index_personnes: crate::mise_en_page::modele::IndexPersonnes::default(),
        }
    }

    /// Le français : insécable devant les signes doubles et à l'intérieur des
    /// guillemets ; `«», ‹›, ""` par niveau d'imbrication.
    pub fn francaise() -> Self {
        Self {
            avant_signes: Espace::Insécable,
            apres_signes: Espace::Normale,
            apres_virgule_point: Espace::Normale,
            espace_guillemets: Espace::Insécable,
            guillemets: vec![('«', '»'), ('‹', '›'), ('"', '"')],
            guillemets_ouvrants: vec!['«'],
            guillemets_fermants: vec!['»'],
            numerotation_figures: String::new(),
            legende_prefixe: "Figure".to_string(),
            callouts: crate::mise_en_page::modele::ParametresCallouts::default(),
            dates: crate::langage::dates::Regles::default(),
            references: crate::mise_en_page::modele::References::default(),
            index: crate::mise_en_page::modele::IndexCitations::default(),
            personnes: crate::project::personnes::Personnes::default(),
            index_personnes: crate::mise_en_page::modele::IndexPersonnes::default(),
        }
    }

    /// L'anglais : la ponctuation colle ; `"`, `'` puis `"` par niveau.
    pub fn anglaise() -> Self {
        Self {
            avant_signes: Espace::Aucune,
            apres_signes: Espace::Normale,
            apres_virgule_point: Espace::Normale,
            espace_guillemets: Espace::Aucune,
            guillemets: vec![('"', '"'), ('\'', '\''), ('"', '"')],
            guillemets_ouvrants: vec!['«'],
            guillemets_fermants: vec!['»'],
            numerotation_figures: String::new(),
            legende_prefixe: "Figure".to_string(),
            callouts: crate::mise_en_page::modele::ParametresCallouts::default(),
            dates: crate::langage::dates::Regles::default(),
            references: crate::mise_en_page::modele::References::default(),
            index: crate::mise_en_page::modele::IndexCitations::default(),
            personnes: crate::project::personnes::Personnes::default(),
            index_personnes: crate::mise_en_page::modele::IndexPersonnes::default(),
        }
    }

    /// L'allemand : la ponctuation colle ; `„…"`, `‚…'` puis `»…«`
    /// par niveau d'imbrication — l'ordre typographique allemand.
    pub fn allemande() -> Self {
        Self {
            avant_signes: Espace::Aucune,
            apres_signes: Espace::Normale,
            apres_virgule_point: Espace::Normale,
            espace_guillemets: Espace::Aucune,
            guillemets: vec![('„', '“'), ('‚', '‘'), ('»', '«')],
            guillemets_ouvrants: vec!['«'],
            guillemets_fermants: vec!['»'],
            numerotation_figures: String::new(),
            legende_prefixe: "Figure".to_string(),
            callouts: crate::mise_en_page::modele::ParametresCallouts::default(),
            dates: crate::langage::dates::Regles::default(),
            references: crate::mise_en_page::modele::References::default(),
            index: crate::mise_en_page::modele::IndexCitations::default(),
            personnes: crate::project::personnes::Personnes::default(),
            index_personnes: crate::mise_en_page::modele::IndexPersonnes::default(),
        }
    }

    /// Le défaut nommé dans les normes.
    pub fn depuis(nom: &str) -> Self {
        match nom.trim().to_lowercase().as_str() {
            "française" | "francaise" | "fr" => Self::francaise(),
            "anglaise" | "en" => Self::anglaise(),
            "allemande" | "de" => Self::allemande(),
            _ => Self::aucune(),
        }
    }
}

/// L'espace insécable, portée en Unicode pendant le travail.
const INSECABLE: char = '\u{A0}';

/// Applique les règles à un texte nu.
///
/// Le résultat porte les insécables en [`INSECABLE`] : c'est au générateur
/// de les traduire en `~` après son échappement, pour qu'un tilde tapé reste
/// un tilde. Un texte sans règle à appliquer sort inchangé.
/// Ce qui entoure un texte, à travers les éléments d'un bloc.
///
/// La typographie ne se joue pas sur un mot isolé : l'insécable avant un
/// `;`, l'espace après un `.`, le niveau d'un guillemet — tout cela court
/// d'un élément à l'autre. Le contexte porte ce qui vient d'avant et ce
/// qui vient d'après, et le niveau d'imbrication, **pour tout le bloc**.
pub struct Contexte {
    /// Le dernier caractère logique sorti du bloc.
    pub precedent: Option<char>,

    /// Le prochain caractère logique du bloc, s'il y en a un.
    pub suivant: Option<char>,

    /// Le prochain caractère qui vient du **texte** — pas d'un littéral,
    /// ni d'un code.
    ///
    /// C'est lui qui décide de l'absorption : un signe du texte posera son
    /// espace lui-même, un signe échappé non, et lui voler la sienne
    /// collerait le mot d'avant.
    pub suivant_texte: Option<char>,

    /// L'élément suivant est une citation bibliographique.
    ///
    /// Une citation vaut un mot pour la typographie, mais elle n'est pas un
    /// mot de texte : un guillemet juste avant elle la ferme, au lieu de
    /// croire qu'elle ouvre une nouvelle citation.
    pub suivant_citation: bool,

    /// Le niveau d'imbrication des guillemets.
    pub profondeur: usize,

    /// Un guillemet ouvrant vient d'être posé.
    pub venant_du_guillemet: bool,
}

impl Contexte {
    /// Un contexte neuf : le début d'un bloc.
    pub fn neuf() -> Self {
        Self {
            precedent: None,
            suivant: None,
            suivant_texte: None,
            suivant_citation: false,
            profondeur: 0,
            venant_du_guillemet: false,
        }
    }
}

/// La composition d'un texte isolé — celle des tests et de la lecture.
///
/// Le générateur, lui, travaille par [`appliquer_dans`], qui porte le
/// contexte du bloc.
#[allow(dead_code)]
pub fn appliquer(texte: &str, regles: &Regles) -> String {
    let mut contexte = Contexte::neuf();
    appliquer_dans(texte, regles, &mut contexte)
}

/// La composition, dans le contexte du bloc.
pub fn appliquer_dans(texte: &str, regles: &Regles, contexte: &mut Contexte) -> String {
    if *regles == Regles::aucune() || texte.is_empty() {
        return texte.to_string();
    }

    let mut sortie = String::new();
    let caracteres: Vec<char> = texte.chars().collect();

    for (position, caractere) in caracteres.iter().enumerate() {
        // La fin du texte n'est pas la fin du bloc : le contexte sait ce
        // qui vient d'après, à travers le balisage.
        let suivant = caracteres.get(position + 1).or(contexte.suivant.as_ref());

        if !caractere.is_whitespace() {
            contexte.venant_du_guillemet = false;
        }

        match caractere {
            // L'espace se réduit à une seule, et se pose **avant** le
            // prochain caractère utile — dont la règle décidera de sa nature.
            c if c.is_whitespace() => {
                // L'espace qui suit un guillemet ouvrant est celui de la
                // règle : le texte ne le redit pas.
                if contexte.venant_du_guillemet {
                    continue;
                }

                // La fin du texte : l'espace est du contenu, qui tient entre
                // les éléments. Seules les suites s'absorbent.
                let Some(&prochain) = suivant else {
                    if !sortie.ends_with(' ') && !sortie.ends_with(INSECABLE) {
                        sortie.push(' ');
                    }

                    continue;
                };

                if prochain.is_whitespace() {
                    continue;
                }

                // Les points de suspension ne sont jamais suivis d'une
                // espace : ils appartiennent au mot qu'ils suspendent. La
                // règle vaut pour le caractère Unicode et pour la suite de
                // trois points écrite au clavier.
                if sortie.ends_with('…') || sortie.ends_with("...") {
                    continue;
                }

                // Ce que la suite posera elle-même n'en veut pas deux — sauf
                // un nombre, que l'on ne touche pas. La règle ne vaut qu'à la
                // **fin de ce texte** : `suivant_texte` décrit l'élément qui
                // vient après, pas le caractère suivant à l'intérieur de la
                // phrase. Sans cette borne, une citation suivie d'un point
                // absorbait toutes les espaces du paragraphe.
                let fin_du_texte = position + 1 == caracteres.len();
                if fin_du_texte
                    && contexte
                        .suivant
                        .is_some_and(|c| matches!(c, ';' | ':' | '!' | '?' | ',' | '.' | '»' | '›'))
                    && matches!(
                        contexte.suivant_texte,
                        Some(';' | ':' | '!' | '?' | ',' | '.' | '»' | '›')
                    )
                    && !est_un_nombre(&caracteres, position, contexte.precedent, contexte.suivant)
                {
                    continue;
                }

                if !sortie.ends_with(' ') && !sortie.ends_with(INSECABLE) {
                    sortie.push(' ');
                }
            }

            // Les signes doubles : l'espace d'avant et l'espace d'après sont
            // ceux que les règles ont choisis.
            ';' | ':' | '!' | '?' => {
                while sortie.ends_with(' ') {
                    sortie.pop();
                }

                if let Some(espace) = regles.avant_signes.caractere() {
                    let plein = !sortie.is_empty() || contexte.precedent.is_some();
                    let deja = sortie.ends_with(espace) || contexte.precedent == Some(espace);

                    if plein && !deja {
                        sortie.push(espace);
                    }
                }

                sortie.push(*caractere);
                pousser_apres(regles, regles.apres_signes, suivant, &mut sortie);
            }

            // La virgule et le point, hors nombres : la même discipline,
            // avec l'espace qu'on leur a choisie. `3.14` et `1,2` sont des
            // nombres — ils ne sont pas touchés.
            ',' | '.'
                if !est_un_nombre(&caracteres, position, contexte.precedent, contexte.suivant) =>
            {
                while sortie.ends_with(' ') {
                    sortie.pop();
                }

                sortie.push(*caractere);
                pousser_apres(regles, regles.apres_virgule_point, suivant, &mut sortie);
            }

            // Les déclencheurs de guillemets sont ceux des normes : par
            // défaut `«` et `»`, mais l'auteur peut y ajouter les siens.
            // Un caractère présent des deux côtés est **contextuel** :
            // devant un blanc il ferme, sinon il ouvre.
            c if regles.guillemets_ouvrants.contains(c)
                || regles.guillemets_fermants.contains(c) =>
            {
                let ambigu = regles.guillemets_ouvrants.contains(c)
                    && regles.guillemets_fermants.contains(c);
                let ouvrir = if ambigu {
                    if ferme_apres_suite(caracteres.as_slice(), position, contexte, *c) {
                        false
                    } else {
                        suivant.is_some_and(|suivant| !suivant.is_whitespace())
                    }
                } else {
                    regles.guillemets_ouvrants.contains(c)
                };

                if ouvrir {
                    contexte.profondeur += 1;
                    let (ouvrant, _) = paire(regles, contexte.profondeur);
                    sortie.push(ouvrant);
                    contexte.venant_du_guillemet = true;

                    if let Some(espace) = regles.espace_guillemets.caractere()
                        && suivant.is_some()
                    {
                        sortie.push(espace);
                    }
                } else {
                    while sortie.ends_with(' ') {
                        sortie.pop();
                    }

                    let (_, fermant) = paire(regles, contexte.profondeur.max(1));

                    if let Some(espace) = regles.espace_guillemets.caractere()
                        && (!sortie.is_empty()
                            || contexte.precedent.is_some_and(|c| !c.is_whitespace()))
                        && !sortie.ends_with(espace)
                    {
                        sortie.push(espace);
                    }

                    sortie.push(fermant);
                    contexte.profondeur = contexte.profondeur.saturating_sub(1);
                }
            }

            // Le guillemet droit : celui du clavier. Il s'ouvre devant ce
            // qui n'est pas un blanc, et se ferme ailleurs — sur le même
            // empilement que les marques explicites. **Jamais d'espace** :
            // l'espace de composition n'appartient qu'aux `«»`, et un
            // guillemet droit, comme une apostrophe, colle à son texte.
            '"' => {
                let ouvrant = !ferme_apres_suite(caracteres.as_slice(), position, contexte, '"')
                    && suivant.is_some_and(|suivant| !suivant.is_whitespace());

                if ouvrant {
                    contexte.profondeur += 1;
                    let (cote, _) = paire(regles, contexte.profondeur);
                    sortie.push(cote);
                    contexte.venant_du_guillemet = true;
                } else {
                    while sortie.ends_with(' ') {
                        sortie.pop();
                    }

                    let (_, fermant) = paire(regles, contexte.profondeur.max(1));
                    sortie.push(fermant);
                    contexte.profondeur = contexte.profondeur.saturating_sub(1);
                }
            }

            _ => {
                sortie.push(*caractere);
            }
        }
    }

    contexte.precedent = sortie.chars().last().or(contexte.precedent);

    sortie
}

/// Un guillemet ferme-t-il la citation après une suite de marques ?
///
/// Un guillemet placé après un mot et devant une ponctuation ferme la
/// citation, même si cette ponctuation n'est pas précédée d'un blanc. Une
/// suite de marques identiques (`"""` à la fermeture) est jugée par ce qui
/// suit la suite : sans cela, chaque marque de la suite serait prise pour
/// une nouvelle ouverture.
///
/// C'est le cas `« bonjour ».` d'un guillemet droit : sans cette règle, le
/// point qui suit ferait croire au guillemet qu'il ouvre une nouvelle
/// citation, et la fermeture ne serait jamais produite.
fn ferme_apres_suite(
    caracteres: &[char],
    position: usize,
    contexte: &Contexte,
    marque: char,
) -> bool {
    let precedent = position
        .checked_sub(1)
        .and_then(|position| caracteres.get(position))
        .copied()
        .or(contexte.precedent);
    // Une suite de guillemets identiques est jugée par ce qui la suit :
    // `""test""` ouvre sur `test` et ferme sur la fin du texte. Sans cela,
    // le second guillemet d'une fermeture `"""` serait pris pour une
    // nouvelle ouverture.
    let suivant = caracteres[position + 1..]
        .iter()
        .copied()
        .find(|caractere| *caractere != marque)
        .or(contexte.suivant);
    let precede_du_texte = precedent.is_some_and(|caractere| !caractere.is_whitespace());
    let suite_ferme = (contexte.suivant_citation && position + 1 == caracteres.len())
        || suivant.is_none_or(|caractere| {
            caractere.is_whitespace()
                || matches!(
                    caractere,
                    '.' | ',' | ';' | ':' | '!' | '?' | ')' | ']' | '}' | '…' | '»' | '›'
                )
        });

    precede_du_texte && suite_ferme
}

/// Le signe est-il au milieu d'un nombre ?
///
/// Une virgule ou un point entre deux chiffres est décimal : `3.14`, `1,2`.
/// Tout le reste — `fin.`, `a, b` — est de la ponctuation.
fn est_un_nombre(
    caracteres: &[char],
    position: usize,
    precedent: Option<char>,
    suivant: Option<char>,
) -> bool {
    let avant = position
        .checked_sub(1)
        .and_then(|i| caracteres.get(i))
        .copied()
        .or(precedent);
    let apres = caracteres.get(position + 1).copied().or(suivant);

    avant.is_some_and(|c| c.is_ascii_digit()) && apres.is_some_and(|c| c.is_ascii_digit())
}

/// Pousse l'espace d'après un signe, si quelque chose suit.
fn pousser_apres(regles: &Regles, espace: Espace, suivant: Option<&char>, sortie: &mut String) {
    // Un guillemet fermant posera sa propre espace — jamais deux.
    let suit =
        suivant.is_some_and(|c| !c.is_whitespace() && !regles.guillemets_fermants.contains(c));

    if suit
        && !sortie.ends_with(' ')
        && !sortie.ends_with(INSECABLE)
        && let Some(caractere) = espace.caractere()
    {
        sortie.push(caractere);
    }
}

/// La paire de guillemets du niveau demandé ; le dernier sert au-delà.
fn paire(regles: &Regles, profondeur: usize) -> (char, char) {
    let dernier = regles.guillemets.len().saturating_sub(1);
    let index = profondeur.saturating_sub(1).min(dernier);
    regles.guillemets.get(index).copied().unwrap_or(('«', '»'))
}

/// Les marqueurs de siècles deviennent leur code Typst — appelée **après**
/// l'échappement, pour que le code reste vivant.
pub fn substituer_siecles(texte: &str) -> String {
    let mut sortie = String::new();
    let chars: Vec<char> = texte.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\u{2}' {
            let mut j = i + 1;
            let mut corps = String::new();
            while j < chars.len() && chars[j] != '\u{2}' {
                corps.push(chars[j]);
                j += 1;
            }
            sortie.push_str(&format!("#smallcaps[{corps}]"));
            i = j + 1;
        } else if chars[i] == '\u{3}' {
            let mut j = i + 1;
            let mut exposant = String::new();
            while j < chars.len() && chars[j] != '\u{3}' {
                exposant.push(chars[j]);
                j += 1;
            }
            sortie.push_str(&format!("#super[{exposant}]"));
            i = j + 1;
        } else {
            sortie.push(chars[i]);
            i += 1;
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
        let regles = Regles::francaise();

        assert_eq!(appliquer("a; b", &regles), format!("a{INSECABLE}; b"));
        assert_eq!(appliquer("a ; b", &regles), format!("a{INSECABLE}; b"));
        assert_eq!(appliquer("a;b", &regles), format!("a{INSECABLE}; b"));
        assert_eq!(appliquer("a ;  b", &regles), format!("a{INSECABLE}; b"));
    }

    /// Les signes doubles suivent la même règle.
    #[test]
    fn en_francais_les_signes_doubles() {
        let regles = Regles::francaise();

        assert_eq!(appliquer("oui !", &regles), format!("oui{INSECABLE}!"));
        assert_eq!(appliquer("non ?", &regles), format!("non{INSECABLE}?"));
        assert_eq!(appliquer("12 : 30", &regles), format!("12{INSECABLE}: 30"));
    }

    /// Les points de suspension ne reçoivent pas d'espace après eux.
    #[test]
    fn les_points_de_suspension_ne_suivent_pas_d_espace() {
        let regles = Regles::francaise();

        assert_eq!(appliquer("mot… suite", &regles), "mot…suite");
        assert_eq!(appliquer("mot... suite", &regles), "mot...suite");
        assert_eq!(appliquer("mot…  suite", &regles), "mot…suite");
    }

    /// La virgule et le point, hors nombres, reçoivent **systématiquement**
    /// leur espace après. Les nombres sont pris tels quels.
    #[test]
    fn la_virgule_et_le_point_se_suivent_d_une_espace() {
        let regles = Regles::francaise();

        assert_eq!(appliquer("fin.", &regles), "fin.");
        assert_eq!(appliquer("fin. La suite", &regles), "fin. La suite");
        assert_eq!(appliquer("fin.Suite", &regles), "fin. Suite");
        assert_eq!(appliquer("a,b", &regles), "a, b");
        assert_eq!(appliquer("a , b", &regles), "a, b");

        // Les nombres sont hors du compte.
        assert_eq!(appliquer("3.14", &regles), "3.14");
        assert_eq!(appliquer("1,2", &regles), "1,2");
    }

    /// Les guillemets se composent par niveau d'imbrication : `«`, `‹` puis
    /// `"` en français — la notation de la syntaxe reste `«` et `»`.
    #[test]
    fn les_guillemets_suivent_leur_niveau() {
        let regles = Regles::francaise();

        assert_eq!(
            appliquer("« un « deux » fin »", &regles),
            format!("«{INSECABLE}un ‹{INSECABLE}deux{INSECABLE}› fin{INSECABLE}»")
        );

        // La ponctuation qui suit ne change rien au guillemet explicite.
        assert_eq!(
            appliquer("« un ».", &regles),
            format!("«{INSECABLE}un{INSECABLE}».")
        );

        // La même notation compose autrement en anglais : le guillemet
        // colle au texte qu'il cite.
        let anglais = Regles::anglaise();
        assert_eq!(
            appliquer("« un « deux » fin »", &anglais),
            "\"un 'deux' fin\""
        );
    }

    /// Le guillemet droit du clavier compose lui aussi, sur le même
    /// empilement : ce qui s'ouvre devant un mot se ferme après lui. Mais
    /// **jamais d'espace** autour de lui : l'espace de composition
    /// n'appartient qu'aux `«»`, et un guillemet droit colle à son texte.
    #[test]
    fn le_guillemet_droit_du_clavier_compose() {
        let française = Regles::francaise();

        assert_eq!(
            appliquer("\"Double \"Insertion\" et quotes\"", &française),
            "«Double ‹Insertion› et quotes»"
        );

        let anglaise = Regles::anglaise();
        assert_eq!(
            appliquer("\"Double \"Insertion\" et quotes\"", &anglaise),
            "\"Double 'Insertion' et quotes\""
        );
    }

    #[test]
    fn les_quatre_guillemets_de_l_auteur() {
        let regles = Regles::francaise();
        let attendus = [
            "Citations «Niveau ‹un de la \"citation\"›»",
            "«test»",
            "«‹test›»",
            "«‹\"test\"›»",
        ];
        for (texte, attendu) in [
            "Citations \"Niveau \"un de la \"citation\"\"\"",
            "\"test\"",
            "\"\"test\"\"",
            "\"\"\"test\"\"\"",
        ]
        .into_iter()
        .zip(attendus)
        {
            assert_eq!(appliquer(texte, &regles), attendu, "{texte:?}");
        }

        // Le même essai avec les déclencheurs personnalisés de la norme
        // « Pers 1 » : les guillemets droits sont alors reconnus des deux
        // côtés, et les espaces sont celles du français.
        let mut personnalisees = regles.clone();
        personnalisees.guillemets_ouvrants = "«\"".chars().collect();
        personnalisees.guillemets_fermants = "»\"".chars().collect();
        assert_eq!(
            appliquer("\"test\"", &personnalisees),
            format!("«{INSECABLE}test{INSECABLE}»")
        );
        assert_eq!(
            appliquer("\"\"test\"\"", &personnalisees),
            format!("«{INSECABLE}‹{INSECABLE}test{INSECABLE}›{INSECABLE}»")
        );
        assert_eq!(
            appliquer("\"\"\"test\"\"\"", &personnalisees),
            format!(
                "«{INSECABLE}‹{INSECABLE}\"{INSECABLE}test{INSECABLE}\"{INSECABLE}›{INSECABLE}»"
            )
        );
    }

    /// Les guillemets droits se referment aussi quand ils terminent une
    /// phrase, et quand la phrase continue après eux.
    #[test]
    fn le_guillemet_droit_se_referme_en_fin_de_phrase() {
        let regles = Regles::francaise();

        assert_eq!(appliquer("\"Bonjour\"", &regles), "«Bonjour»");
        assert_eq!(
            appliquer("Il dit \"bonjour\" et part.", &regles),
            "Il dit «bonjour» et part."
        );
        assert_eq!(appliquer("\"Bonjour\".", &regles), "«Bonjour».");
        assert_eq!(
            appliquer("Il dit \"bonjour\", puis part.", &regles),
            "Il dit «bonjour», puis part."
        );
    }

    /// ÉVOLUTION-002 : les caractères de frappe se choisissent, et la paire
    /// produite reste celle des niveaux. Un même caractère des deux côtés
    /// se décide au contexte.
    #[test]
    fn des_declencheurs_personnalises_composent_la_paire() {
        let mut regles = Regles::francaise();
        regles.guillemets_ouvrants = vec!['§'];
        regles.guillemets_fermants = vec!['µ'];

        assert_eq!(
            appliquer("§un §deuxµ finµ", &regles),
            format!("«{INSECABLE}un ‹{INSECABLE}deux{INSECABLE}› fin{INSECABLE}»")
        );

        // Le même caractère des deux côtés : devant un blanc il ferme,
        // devant un mot il ouvre.
        regles.guillemets_ouvrants = vec!['°'];
        regles.guillemets_fermants = vec!['°'];
        assert_eq!(
            appliquer("°un °deux° fin°", &regles),
            format!("«{INSECABLE}un ‹{INSECABLE}deux{INSECABLE}› fin{INSECABLE}»")
        );

        // Même devant une ponctuation qui n'est pas un blanc, le caractère
        // contextuel ferme la citation ouverte : c'est le cas du guillemet
        // droit final dans « Bonjour ».
        assert_eq!(
            appliquer("°un texte°.", &regles),
            format!("«{INSECABLE}un texte{INSECABLE}».")
        );
    }

    /// Les règles se prennent une par une : changer l'espace des signes
    /// change la composition, sans toucher à la langue.
    #[test]
    fn les_regles_se_prennent_une_par_une() {
        let mut regles = Regles::francaise();
        regles.avant_signes = Espace::Aucune;

        assert_eq!(appliquer("a; b", &regles), "a; b");

        regles.apres_virgule_point = Espace::Insécable;
        assert_eq!(appliquer("a,b", &regles), format!("a,{INSECABLE}b"));
    }

    /// Sans règles, rien ne bouge.
    #[test]
    fn sans_regles_rien_ne_bouge() {
        let regles = Regles::aucune();
        assert_eq!(appliquer("a ;  b", &regles), "a ;  b");
        assert_eq!(appliquer("« x »", &regles), "« x »");
    }
}
