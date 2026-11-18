//! Les références bibliques : le modèle, les textes, la lecture.
//!
//! Deux traductions sont **embarquées** (le produit reste autonome, sans
//! service extérieur) : Louis Segond 1910 (LSG) et J.N. Darby (DARBY), toutes
//! deux dans le domaine public. L'auteur peut ajouter d'autres versions
//! locales — voir [`crate::langage::versions`] — dont le nombre de livres
//! varie librement. Les textes embarqués sont rangés sous
//! `vendor/bible-fr/`, un chapitre par ligne.
//!
//! Une référence est un livre, un chapitre et une plage de versets ; un
//! groupe de références est une liste de références, à lire ou à afficher
//! ensemble.

use std::collections::HashMap;
use std::sync::OnceLock;

pub use crate::langage::references::{
    ElementCitation, FormeLivre, Reference, Traduction, abreviation_francaise, afficher,
    afficher_groupe, afficher_groupe_forme, contenu_de_balise, depuis_texte,
    lire_groupe_complet_de_balise, lire_groupe_de_balise,
};
use crate::langage::references::{nom_du_livre, nom_du_livre_dans};

/// Un livre de la Bible, tel que les textes le nomment.
#[derive(Clone, Debug, serde::Deserialize)]
pub struct LivreConnu {
    pub osis: String,
    pub nom: String,
    pub abreviation: String,
}

/// Une entrée de la liste déroulante cherchable du panneau biblique.
///
/// Le libellé suit la forme d'affichage choisie (la forme TOB montre ses
/// propres abréviations) ; le rang, lui, ne bouge jamais.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LivreChoix {
    /// Le rang du livre dans la liste complète, canon et hors canon compris.
    pub rang: usize,
    /// Le libellé affiché et comparé par le widget.
    pub libelle: String,
}

impl LivreChoix {
    pub fn depuis(rang: usize, livre: &LivreConnu, forme: Option<FormeLivre>) -> Self {
        let abreviation = match forme {
            Some(FormeLivre::Tob) => crate::langage::references::abreviation_tob(&livre.osis)
                .unwrap_or(&livre.abreviation),
            _ => abreviation_francaise(&livre.osis).unwrap_or(&livre.abreviation),
        };
        Self {
            rang,
            libelle: format!("{} — {}", livre.nom, abreviation),
        }
    }
}

impl std::fmt::Display for LivreChoix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.libelle)
    }
}

#[derive(serde::Deserialize)]
struct Chapitre {
    livre: String,
    chapitre: u32,
    versets: Vec<(u32, String)>,
}

type Table = HashMap<(String, u32), Chapitre>;

fn chapitres(t: Traduction) -> Option<&'static Table> {
    static LSG: OnceLock<Table> = OnceLock::new();
    static DARBY: OnceLock<Table> = OnceLock::new();
    let (cache, brut): (&OnceLock<Table>, &'static [u8]) = match t {
        Traduction::Lsg => (&LSG, include_bytes!("../../../vendor/bible-fr/LSG.jsonl")),
        Traduction::Darby => (
            &DARBY,
            include_bytes!("../../../vendor/bible-fr/DARBY.jsonl"),
        ),
        Traduction::Externe(_) => return None,
    };
    Some(cache.get_or_init(|| {
        let mut table = HashMap::new();
        for ligne in brut.split(|b| *b == b'\n') {
            if ligne.is_empty() {
                continue;
            }
            let chapitre: Chapitre = serde_json::from_slice(ligne).expect("un chapitre lisible");
            table.insert((chapitre.livre.clone(), chapitre.chapitre), chapitre);
        }
        table
    }))
}

/// Les versets d'un chapitre d'une version ajoutée, déjà filtrés.
fn versets_externes(reference: &Reference, index: u16) -> Vec<(u32, String)> {
    let mut sortie = Vec::new();
    if reference.chapitre_fin < reference.chapitre {
        return sortie;
    }

    for chapitre_numero in reference.chapitre..=reference.chapitre_fin {
        let versets = crate::langage::versions::chapitres(index, &reference.livre, chapitre_numero);
        let debut = if chapitre_numero == reference.chapitre {
            reference.verset_debut
        } else {
            1
        };
        let fin = if chapitre_numero == reference.chapitre_fin {
            reference.verset_fin
        } else {
            u32::MAX
        };
        sortie.extend(
            versets
                .into_iter()
                .filter(|(numero, _)| *numero >= debut && *numero <= fin),
        );
    }
    sortie
}

/// Les versets d'une référence, dans la traduction demandée.
///
/// Une référence qui couvre plusieurs chapitres rend les versets de chacun,
/// dans l'ordre : le premier chapitre à partir de `verset_debut`, les
/// suivants en entier, le dernier jusqu'à `verset_fin`.
pub fn versets(reference: &Reference, traduction: Traduction) -> Vec<(u32, String)> {
    let Some(table) = chapitres(traduction) else {
        return match traduction {
            Traduction::Externe(index) => versets_externes(reference, index),
            _ => Vec::new(),
        };
    };
    let mut sortie = Vec::new();
    if reference.chapitre_fin < reference.chapitre {
        return sortie;
    }

    for chapitre_numero in reference.chapitre..=reference.chapitre_fin {
        let Some(chapitre) = table.get(&(reference.livre.clone(), chapitre_numero)) else {
            continue;
        };
        let debut = if chapitre_numero == reference.chapitre {
            reference.verset_debut
        } else {
            1
        };
        let fin = if chapitre_numero == reference.chapitre_fin {
            reference.verset_fin
        } else {
            u32::MAX
        };
        sortie.extend(
            chapitre
                .versets
                .iter()
                .filter(|(n, _)| *n >= debut && *n <= fin)
                .cloned(),
        );
    }
    sortie
}

/// Les livres connus de la traduction (leurs noms et abréviations).
///
/// Les traductions embarquées ne contiennent que les 66 livres canoniques ;
/// les deutérocanoniques et les apocryphes sont ajoutés après elles pour
/// qu'on puisse les citer même sans texte consultable.
pub fn livres(traduction: Traduction) -> Vec<LivreConnu> {
    if let Traduction::Externe(index) = traduction {
        return crate::langage::versions::livres(index)
            .into_iter()
            .map(|livre| LivreConnu {
                osis: livre.osis,
                nom: livre.nom,
                abreviation: livre.abreviation,
            })
            .collect();
    }

    let brut: &'static [u8] = match traduction {
        Traduction::Lsg => include_bytes!("../../../vendor/bible-fr/LSG-livres.json"),
        Traduction::Darby => include_bytes!("../../../vendor/bible-fr/DARBY-livres.json"),
        Traduction::Externe(_) => unreachable!("les versions ajoutées sont traitées plus haut"),
    };
    let mut livres: Vec<LivreConnu> =
        serde_json::from_slice(brut).expect("les livres sont lisibles");

    for (osis, nom, abreviation) in crate::langage::references::livres_hors_canon() {
        if !livres.iter().any(|livre| livre.osis == osis) {
            livres.push(LivreConnu {
                osis: osis.to_string(),
                nom: nom.to_string(),
                abreviation: abreviation.to_string(),
            });
        }
    }

    livres
}

/// Filtre une liste de livres déjà chargée : le panneau n'a pas à charger
/// deux fois la table de la traduction. La casse et les accents sont pliés
/// de la même manière que dans la palette.
pub fn filtrer_livres(livres: &[LivreConnu], filtre: &str) -> Vec<LivreConnu> {
    let filtre = crate::base::texte::normaliser_recherche(filtre.trim());
    livres
        .iter()
        .filter(|livre| {
            if filtre.is_empty() {
                return true;
            }
            crate::base::texte::normaliser_recherche(&livre.nom).contains(&filtre)
                || crate::base::texte::normaliser_recherche(&livre.abreviation).contains(&filtre)
                || abreviation_francaise(&livre.osis).is_some_and(|abreviation| {
                    crate::base::texte::normaliser_recherche(abreviation).contains(&filtre)
                })
                || crate::langage::references::abreviation_tob(&livre.osis).is_some_and(
                    |abreviation| {
                        crate::base::texte::normaliser_recherche(abreviation).contains(&filtre)
                    },
                )
        })
        .cloned()
        .collect()
}

/// Les statistiques des livres cités (le système biblique) : ce qui est
/// cité, par livre et par testament. Un document, une étagère, tous les
/// projets — même mesure, à des échelles différentes.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Statistiques {
    /// Les citations par livre : « Jn » → 3.
    pub par_livre: std::collections::BTreeMap<String, usize>,
    /// Les références exactes citées : « Jn 3:16 » → 3. C'est la mémoire
    /// des versets du document, pas un classement des livres.
    pub par_reference: std::collections::BTreeMap<String, usize>,
    /// Les citations par testament : « Ancien » et « Nouveau ».
    pub par_testament: std::collections::BTreeMap<&'static str, usize>,
    /// Les références dans leur ordre de première apparition.
    pub ordre_apparition: Vec<String>,
    /// Le nombre total de citations.
    pub total: usize,
}

impl Statistiques {
    /// Ajoute les citations d'un texte, par occurrence : chaque membre d'un
    /// groupe compte pour lui.
    pub fn ajouter(&mut self, texte: &str) {
        // Le relevé est fait sur les formes écrites ; pour les livres, on
        // relit le texte lui-même, chaque membre de groupe pour soi.
        let mut curseur = 0;
        while let Some(ouverture) = texte[curseur..].find("{{") {
            let debut = curseur + ouverture;
            let Some((_, fin, references)) = reference_au(texte, debut) else {
                break;
            };
            for reference in references {
                let livre = nom_du_livre_dans(
                    &reference.livre,
                    FormeLivre::FrancaiseCourte,
                    reference.traduction,
                );
                *self.par_livre.entry(livre).or_insert(0) += 1;
                let cle = afficher(
                    &reference,
                    &crate::mise_en_page::modele::References::default(),
                );
                if !self.par_reference.contains_key(&cle) {
                    self.ordre_apparition.push(cle.clone());
                }
                *self.par_reference.entry(cle).or_insert(0) += 1;
                let testament = if reference.livre.starts_with(char::is_numeric)
                    || LIVRES_ANCIEN.contains(&reference.livre.as_str())
                {
                    "Ancien"
                } else {
                    "Nouveau"
                };
                *self.par_testament.entry(testament).or_insert(0) += 1;
                self.total += 1;
            }
            curseur = fin;
        }
    }

    /// Les statistiques d'un relevé écrit (le `references.json` d'un projet)
    /// : les comptes par livre et par testament, sans relire le texte.
    pub fn depuis_releve(releve: &std::collections::BTreeMap<String, usize>) -> Self {
        let mut stats = Self::default();
        for (forme, nombre) in releve {
            let Some(reference) = depuis_texte(forme) else {
                continue;
            };
            let livre = nom_du_livre_dans(
                &reference.livre,
                FormeLivre::FrancaiseCourte,
                reference.traduction,
            );
            *stats.par_livre.entry(livre).or_insert(0) += nombre;
            if !stats.par_reference.contains_key(forme) {
                stats.ordre_apparition.push(forme.clone());
            }
            *stats.par_reference.entry(forme.clone()).or_insert(0) += nombre;
            let testament = if LIVRES_ANCIEN.contains(&reference.livre.as_str()) {
                "Ancien"
            } else {
                "Nouveau"
            };
            *stats.par_testament.entry(testament).or_insert(0) += nombre;
            stats.total += nombre;
        }
        stats
    }

    /// Les livres les plus cités, du plus fréquent au moins.
    pub fn plus_cites(&self, limite: usize) -> Vec<(String, usize)> {
        let mut classement: Vec<(String, usize)> = self
            .par_livre
            .iter()
            .map(|(livre, nombre)| (livre.clone(), *nombre))
            .collect();
        classement.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        classement.truncate(limite);
        classement
    }

    /// Les références exactes les plus citées, du plus fréquent au moins.
    /// C'est ce que l'auteur veut retrouver : ses versets, pas un palmarès
    /// de livres.
    pub fn plus_citees(&self, limite: usize) -> Vec<(String, usize)> {
        let mut classement: Vec<(String, usize)> = self
            .par_reference
            .iter()
            .map(|(reference, nombre)| (reference.clone(), *nombre))
            .collect();
        classement.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        classement.truncate(limite);
        classement
    }
}

/// L'analyse intelligente des statistiques : ce que les comptes disent de
/// plus que des nombres — la couverture du texte, les livres jamais cités,
/// la comparaison des échelles.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Analyse {
    /// La couverture : le nombre de livres cités sur ceux qui sont proposés
    /// (canon, deutérocanoniques et apocryphes).
    pub livres_cites: usize,
    /// Les livres jamais cités, par testament.
    pub jamais_cites: Vec<String>,
    /// Le livre le plus cité.
    pub favori: Option<(String, usize)>,
    /// Le chapitre le plus cité (« Jn 3 »), si un relevé est fourni.
    pub chapitre_favori: Option<(String, usize)>,
    /// La concentration : la part des citations portées par les trois
    /// premiers livres, en pourcentage.
    pub concentration: u8,
}

impl Analyse {
    /// L'analyse d'un relevé écrit (le `references.json` d'un projet).
    pub fn depuis_releve(releve: &std::collections::BTreeMap<String, usize>) -> Self {
        let mut chapitres = std::collections::BTreeMap::new();
        let mut cites = std::collections::BTreeSet::new();
        for (forme, nombre) in releve {
            let Some(reference) = depuis_texte(forme) else {
                continue;
            };
            let livre = nom_du_livre_dans(
                &reference.livre,
                FormeLivre::FrancaiseCourte,
                reference.traduction,
            );
            cites.insert(reference.livre.clone());
            *chapitres
                .entry(format!("{livre} {}", reference.chapitre))
                .or_insert(0) += nombre;
        }
        let stats = Statistiques::depuis_releve(releve);
        let mut jamais_cites = Vec::new();
        for livre in livres(Traduction::Lsg) {
            if !cites.contains(&livre.osis) {
                jamais_cites.push(nom_du_livre(&livre.osis, FormeLivre::FrancaiseCourte));
            }
        }
        let mut top = stats.plus_cites(3);
        let trois: usize = top.iter().map(|(_, n)| n).sum();
        let concentration = (trois * 100).checked_div(stats.total).unwrap_or(0) as u8;
        let favori = top.first().cloned();
        let chapitre_favori = chapitres.into_iter().max_by_key(|(_, n)| *n);
        top.clear();
        Self {
            livres_cites: cites.len(),
            jamais_cites,
            favori,
            chapitre_favori,
            concentration,
        }
    }
}

/// Les livres de l'Ancien Testament, pour la répartition. Les
/// deutérocanoniques et les apocryphes y sont rangés avec l'Ancien.
const LIVRES_ANCIEN: &[&str] = &[
    "Gen", "Exod", "Lev", "Num", "Deut", "Josh", "Judg", "Ruth", "1Sam", "2Sam", "1Kgs", "2Kgs",
    "1Chr", "2Chr", "Ezra", "Neh", "Esth", "Job", "Ps", "Prov", "Eccl", "Song", "Isa", "Jer",
    "Lam", "Ezek", "Dan", "Hos", "Joel", "Amos", "Obad", "Jonah", "Mic", "Nah", "Hab", "Zeph",
    "Hag", "Zech", "Mal", "Tob", "Jdt", "AddEst", "1Macc", "2Macc", "Wis", "Sir", "Bar", "EpJer",
    "PrMan", "Ps151", "3Macc", "4Macc", "2Esd", "1Esd", "3Esd", "1En", "2En", "Jub", "TReu",
    "TSim", "TLevi", "TJud", "TIss", "TZeb", "TDan", "TNaph", "TGad", "TAsh", "TJos", "TBen",
    "2Bar", "3Bar", "ApAb", "ApZeph", "ApPet", "ApPaul", "ApMos",
];

/// La référence sous un décalage du texte : la plage de la forme `{{…}}`
/// (en octets, `debut` inclus, `fin` exclue) et ses références.
///
/// Le texte est balayé de gauche à droite ; la première forme `{{…}}` qui
/// contient `offset` (comparé à `debut..=fin`) et dont au moins un membre se
/// lit est rendue. Les membres sont séparés par `;`. `None` hors de toute
/// forme, ou si aucun membre n'est une référence valide.
pub fn reference_au(texte: &str, offset: usize) -> Option<(usize, usize, Vec<Reference>)> {
    let octets = texte.as_bytes();
    let mut debut = 0;
    while debut + 1 < octets.len() {
        if octets[debut] == b'{'
            && octets[debut + 1] == b'{'
            && let Some(fin) = texte[debut + 2..].find("}}")
        {
            let fin = debut + 2 + fin + 2;
            if offset >= debut && offset <= fin {
                let contenu = &texte[debut + 2..fin - 2];
                if let Some((references, _)) = lire_groupe_de_balise(contenu)
                    && !references.is_empty()
                {
                    return Some((debut, fin, references));
                }
            }
            debut = fin;
            continue;
        }
        debut += 1;
    }
    None
}

/// La référence précise sous le curseur, dans une balise qui peut en
/// contenir plusieurs. La position dans la liste décide — c'est ce qui fait
/// que le survol de `{{Jn 3:16, 17}}` dit Jean 3:17 au second verset.
pub fn reference_sous(texte: &str, offset: usize) -> Option<Reference> {
    let (debut, fin, references) = reference_au(texte, offset)?;
    let (contenu, prefixe) = contenu_de_balise(&texte[debut + 2..fin - 2]);
    let relatif = offset.saturating_sub(debut + 2 + prefixe);
    let positions = crate::langage::references::references_situees(contenu);

    if let Some((_, _, reference)) = positions
        .iter()
        .find(|(debut, fin, _)| relatif >= *debut && relatif < *fin)
    {
        return Some(reference.clone());
    }

    // Sur les accolades, un séparateur ou le suffixe de forme : la première
    // référence est celle du début de la balise, la dernière celle de sa fin.
    if positions
        .first()
        .is_some_and(|(debut, _, _)| relatif < *debut)
    {
        return positions.first().map(|(_, _, reference)| reference.clone());
    }
    positions
        .last()
        .map(|(_, _, reference)| reference.clone())
        .or_else(|| references.last().cloned())
}

/// La même construction quand le champ chapitre est libre : `3` ou `1-11`.
pub fn references_depuis_texte(livre: &str, chapitre: &str, versets: &str) -> Vec<Reference> {
    let chapitre = chapitre.trim();
    if chapitre.is_empty() {
        return Vec::new();
    }

    let (debut, fin) = if let Some((debut, fin)) = chapitre
        .split_once('-')
        .or_else(|| chapitre.split_once('–'))
    {
        let (Ok(debut), Ok(fin)) = (debut.trim().parse::<u32>(), fin.trim().parse::<u32>()) else {
            return Vec::new();
        };
        (debut, fin)
    } else {
        let Ok(chapitre) = chapitre.parse::<u32>() else {
            return Vec::new();
        };
        (chapitre, chapitre)
    };
    if debut == 0 {
        return Vec::new();
    }

    let versets = versets.trim();
    if versets.is_empty() {
        return vec![Reference::chapitres(livre, debut, fin)];
    }
    if fin > debut {
        // Une plage qui traverse des chapitres garde ses deux bornes de
        // versets : `1-2` avec `1-3` devient `1.1-2.3`.
        let (verset_debut, verset_fin) = if let Some((debut, fin)) =
            versets.split_once('-').or_else(|| versets.split_once('–'))
        {
            let (Ok(debut), Ok(fin)) = (debut.trim().parse::<u32>(), fin.trim().parse::<u32>())
            else {
                return Vec::new();
            };
            (debut, fin)
        } else {
            let Ok(verset) = versets.parse::<u32>() else {
                return Vec::new();
            };
            (verset, verset)
        };
        if verset_debut == 0 {
            return Vec::new();
        }
        return vec![Reference {
            livre: livre.to_string(),
            chapitre: debut,
            chapitre_fin: fin,
            verset_debut,
            verset_fin: verset_fin.max(verset_debut),
            traduction: None,
        }];
    }

    let mut references = Vec::new();
    for morceau in versets.split(',') {
        let morceau = morceau.trim();
        if morceau.is_empty() {
            continue;
        }
        if let Some((premier, dernier)) =
            morceau.split_once('-').or_else(|| morceau.split_once('–'))
        {
            if let (Ok(premier), Ok(dernier)) =
                (premier.trim().parse::<u32>(), dernier.trim().parse::<u32>())
            {
                references.push(Reference::plage(livre, debut, premier, dernier));
            }
        } else if let Ok(verset) = morceau.parse::<u32>() {
            references.push(Reference::verset(livre, debut, verset));
        }
    }
    references
}

/// La première balise qui porte exactement cette référence affichée.
pub fn position_reference(
    texte: &str,
    cible: &str,
    normes: &crate::mise_en_page::modele::References,
) -> Option<(usize, usize)> {
    let cible = cible.trim();
    let mut curseur = 0;
    while let Some(ouverture) = texte[curseur..].find("{{") {
        let debut = curseur + ouverture;
        let Some((_, fin, references)) = reference_au(texte, debut) else {
            curseur = debut + 2;
            continue;
        };
        for reference in references {
            if afficher(&reference, normes) == cible
                || depuis_texte(cible).as_ref() == Some(&reference)
            {
                return Some((debut, fin));
            }
        }
        curseur = fin;
    }
    None
}

/// L'extrait que le survol montre : les versets d'un groupe de références,
/// dans la traduction demandée, avec sa mention. C'est le contenu de la
/// bulle — au-delà de quelques lignes, elle tronque proprement.
pub fn extrait(references: &[Reference], traduction: Traduction) -> String {
    let mut sortie = String::new();
    let mut utilisees: Vec<Traduction> = Vec::new();
    for reference in references {
        // Une traduction écrite dans la citation gagne sur la consultation.
        let traduction = reference.traduction.unwrap_or(traduction);
        if !utilisees.contains(&traduction) {
            utilisees.push(traduction);
        }
        if reference.chapitre_fin > reference.chapitre {
            for chapitre in reference.chapitre..=reference.chapitre_fin {
                let mut portion = reference.clone();
                portion.chapitre = chapitre;
                portion.chapitre_fin = chapitre;
                for (numero, texte) in versets(&portion, traduction) {
                    if !sortie.is_empty() {
                        sortie.push(' ');
                    }
                    sortie.push_str(&format!("({chapitre}:{numero}) {texte}"));
                }
            }
        } else {
            for (numero, texte) in versets(reference, traduction) {
                if !sortie.is_empty() {
                    sortie.push(' ');
                }
                sortie.push_str(&format!("({numero}) {texte}"));
            }
        }
    }
    if sortie.is_empty() {
        sortie.push_str("(aucun verset)");
    }
    if !utilisees.is_empty() {
        sortie.push_str(" — ");
        sortie.push_str(
            &utilisees
                .iter()
                .map(|traduction| traduction.nom())
                .collect::<Vec<_>>()
                .join(" / "),
        );
    }
    sortie
}

/// Le relevé des références citées dans un texte : chaque référence et le
/// nombre de fois qu'elle est citée. C'est la trace du projet (le système de
/// références bibliques) — les groupes comptent pour chaque membre.
pub fn releve(texte: &str) -> std::collections::BTreeMap<String, usize> {
    let mut compte = std::collections::BTreeMap::new();
    let mut curseur = 0;
    while let Some(ouverture) = texte[curseur..].find("{{") {
        let debut = curseur + ouverture;
        let Some((_, fin, references)) = reference_au(texte, debut) else {
            break;
        };
        for reference in references {
            *compte
                .entry(afficher(
                    &reference,
                    &crate::mise_en_page::modele::References::default(),
                ))
                .or_insert(0) += 1;
        }
        curseur = fin;
    }
    compte
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::langage::references::afficher_groupe;

    /// La syntaxe choisie fait la référence (la fonctionnalité
    /// biblique) : l'abréviation du livre, la ponctuation.
    #[test]
    fn la_syntaxe_ecrit_la_reference() {
        let reference = Reference::plage("John", 3, 16, 18);
        // La forme par défaut : l'abréviation, le deux-points, le trait.
        assert_eq!(
            afficher(
                &reference,
                &crate::mise_en_page::modele::References::default()
            ),
            "Jn 3.16-18"
        );
        // Le nom complet, la virgule, le tiret cadratin.
        let complet = crate::mise_en_page::modele::References {
            livre: crate::mise_en_page::modele::SystemeAbreviation::FrancaiseLongue,
            chiffres: crate::mise_en_page::modele::Chiffres::Arabes,
            separateur_chapitre_verset: ",".to_string(),
            separateur_livre: " ; ".to_string(),
            separateur_plage: "–".to_string(),
            separateur_occurrence: ", ".to_string(),
            separateur_chapitres: " à ".to_string(),
            gabarit: "{livre} {nombres}".to_string(),
            ordre_canonique: false,
        };
        assert_eq!(afficher(&reference, &complet), "Jean 3,16–18");
        // Un seul verset : pas de plage.
        let seul = Reference::verset("Gen", 1, 1);
        assert_eq!(
            afficher(&seul, &crate::mise_en_page::modele::References::default()),
            "Gn 1.1"
        );

        // Le chapitre entier suit aussi la norme des chiffres.
        let romains = crate::mise_en_page::modele::References {
            chiffres: crate::mise_en_page::modele::Chiffres::Romains,
            ..Default::default()
        };
        let chapitre = Reference::plage("Gen", 3, 1, u32::MAX);
        assert_eq!(afficher(&chapitre, &romains), "Gn III");
    }

    /// Un groupe se sépare dun point-virgule.
    #[test]
    fn un_groupe_se_separe() {
        let groupe = vec![
            Reference::verset("John", 3, 16),
            Reference::plage("Gen", 1, 1, 3),
        ];
        assert_eq!(
            afficher_groupe(&groupe, &crate::mise_en_page::modele::References::default()),
            "Jn 3.16 ; Gn 1.1-3"
        );
    }

    /// Le même livre n'est nommé qu'une fois ; ses occurrences sont
    /// séparées par une virgule, et l'ordre canonique les range.
    #[test]
    fn un_livre_cite_plusieurs_fois_est_regroupe() {
        let references = vec![
            Reference::verset("2Chr", 5, 6),
            Reference::verset("Josh", 1, 9),
            Reference::plage("2Chr", 7, 6, 12),
        ];
        let normes = crate::mise_en_page::modele::References {
            livre: crate::mise_en_page::modele::SystemeAbreviation::FrancaiseLongue,
            ordre_canonique: true,
            ..Default::default()
        };
        assert_eq!(
            crate::langage::references::afficher_groupe(&references, &normes),
            "Josué 1.9 ; 2 Chroniques 5.6, 7.6-12",
            "l'exemple de l'auteur, dans l'ordre canonique"
        );

        // La même exigence, lue depuis la syntaxe exacte de l'auteur.
        let (lues, _) = crate::langage::references::lire_groupe("2 Ch 5.6 ; Js 1.9; 2 Ch 7.6-12")
            .expect("la balise de l'auteur se lit");
        assert_eq!(
            crate::langage::references::afficher_groupe(&lues, &normes),
            "Josué 1.9 ; 2 Chroniques 5.6, 7.6-12",
            "la forme longue et l'ordre canonique rendent l'exemple attendu"
        );

        let sans_ordre = crate::mise_en_page::modele::References {
            livre: crate::mise_en_page::modele::SystemeAbreviation::FrancaiseLongue,
            ..Default::default()
        };
        assert_eq!(
            crate::langage::references::afficher_groupe(&references, &sans_ordre),
            "2 Chroniques 5.6, 7.6-12 ; Josué 1.9",
            "sans l'ordre canonique, le premier livre rencontré reste en tête"
        );
    }

    /// Les chapitres entiers s'écrivent `1 à 11`, et `2 Ch` se lit comme
    /// `2Ch`.
    #[test]
    fn les_chapitres_entiers_se_lisent() {
        assert_eq!(
            depuis_texte("Gn 1-11"),
            Some(Reference::chapitres("Gen", 1, 11))
        );
        assert_eq!(
            afficher(
                &Reference::chapitres("Gen", 1, 11),
                &crate::mise_en_page::modele::References::default()
            ),
            "Gn 1 à 11"
        );
        assert_eq!(
            depuis_texte("2 Ch 5.6"),
            Some(Reference::verset("2Chr", 5, 6))
        );
        assert_eq!(
            depuis_texte("Gn 1.1-2.3"),
            Some(Reference {
                livre: "Gen".to_string(),
                chapitre: 1,
                chapitre_fin: 2,
                verset_debut: 1,
                verset_fin: 3,
                traduction: None,
            })
        );
        assert_eq!(
            references_depuis_texte("Gen", "1-11", ""),
            vec![Reference::chapitres("Gen", 1, 11)],
            "le panneau construit des chapitres entiers"
        );
        assert_eq!(
            references_depuis_texte("Gen", "1-2", "1-3"),
            vec![Reference {
                livre: "Gen".to_string(),
                chapitre: 1,
                chapitre_fin: 2,
                verset_debut: 1,
                verset_fin: 3,
                traduction: None,
            }],
            "une plage qui traverse deux chapitres garde ses versets"
        );
    }

    /// Le texte libre entre guillemets est conservé, sans devenir une
    /// référence, et les références autour gardent leur place.
    #[test]
    fn le_texte_libre_est_conserve() {
        let (elements, forme) =
            crate::langage::references::lire_groupe_complet("Jn 3.16 ; \"cf. aussi\" ; Gn 1.1")
                .expect("la balise se lit");
        assert_eq!(forme, None);
        assert_eq!(elements.len(), 3);
        assert_eq!(
            elements[1],
            crate::langage::references::ElementCitation::Texte("cf. aussi".to_string())
        );
        assert_eq!(
            crate::langage::references::afficher_elements(
                &elements,
                &crate::mise_en_page::modele::References::default()
            ),
            "Jn 3.16 ; cf. aussi ; Gn 1.1"
        );
        assert_eq!(
            crate::langage::references::afficher_elements_typst(
                &elements,
                &crate::mise_en_page::modele::References::default(),
                None
            ),
            "Jn 3.16 ; cf. aussi ; Gn 1.1",
            "le texte ordinaire ne change pas dans le rendu"
        );
        let avec_balisage = vec![crate::langage::references::ElementCitation::Texte(
            "cf. #aussi *fort*".to_string(),
        )];
        assert_eq!(
            crate::langage::references::afficher_elements_typst(
                &avec_balisage,
                &crate::mise_en_page::modele::References::default(),
                None
            ),
            "cf. \\#aussi \\*fort\\*",
            "le texte libre ne devient jamais du balisage Typst"
        );

        let (references, _) =
            crate::langage::references::lire_groupe("Jn 3.16 ; \"cf. aussi\" ; Gn 1.1")
                .expect("les références restent lisibles");
        assert_eq!(
            references.len(),
            2,
            "le texte libre n'est pas une référence"
        );
    }

    /// Une balise se sérialise et se relit sans perdre son texte libre.
    #[test]
    fn le_texte_libre_survit_a_l_aller_retour() {
        let texte = "{{Jn 3.16 ; \"cf. aussi\" ; Gn 1.1}}";
        let document = crate::langage::parse::parse(texte);
        let relu = crate::langage::serialiser::serialiser(&document);
        assert!(relu.contains("\"cf. aussi\""), "{relu:?}");
    }

    /// Une référence se lit d'un texte : le livre, le chapitre, la plage.
    #[test]
    fn une_reference_se_lit() {
        assert_eq!(
            depuis_texte("Jean 3:16"),
            Some(Reference::verset("John", 3, 16))
        );
        assert_eq!(
            depuis_texte("Jn 3,16-18"),
            Some(Reference::plage("John", 3, 16, 18))
        );
        assert_eq!(
            depuis_texte("Gn 1"),
            Some(Reference::plage("Gen", 1, 1, u32::MAX))
        );
        // Le code OSIS aussi.
        assert_eq!(
            depuis_texte("1Jn 1:9"),
            Some(Reference::verset("1John", 1, 9))
        );
        // Les livres numérotés, écrits en toutes lettres.
        assert_eq!(
            depuis_texte("1 Samuel 3:16"),
            Some(Reference::verset("1Sam", 3, 16))
        );
        assert_eq!(
            depuis_texte("2 Rois 5:1"),
            Some(Reference::verset("2Kgs", 5, 1))
        );
        assert_eq!(
            depuis_texte("1 Corinthiens 13:4"),
            Some(Reference::verset("1Cor", 13, 4))
        );
        // Une phrase qui n est pas une référence.
        assert_eq!(depuis_texte("bonjour le monde"), None);
    }

    /// Le gabarit des normes remplace ses jetons et laisse la référence
    /// lisible quand le gabarit est vide.
    #[test]
    fn le_gabarit_compose_la_reference() {
        let gabarit = crate::mise_en_page::modele::References {
            gabarit: "{livre} {chapitre}, {verset}".to_string(),
            ..Default::default()
        };
        assert_eq!(
            afficher(&Reference::verset("John", 3, 16), &gabarit),
            "Jn 3, 16"
        );
        let parentheses = crate::mise_en_page::modele::References {
            gabarit: "({reference})".to_string(),
            ..Default::default()
        };
        assert_eq!(
            afficher(&Reference::verset("John", 3, 16), &parentheses),
            "(Jn 3.16)"
        );
        let vide = crate::mise_en_page::modele::References {
            gabarit: String::new(),
            ..Default::default()
        };
        assert_eq!(
            afficher(&Reference::verset("John", 3, 16), &vide),
            "Jn 3.16",
            "un gabarit vide retombe sur le standard"
        );
    }

    /// Le survol : la référence sous la position est trouvée, avec sa
    /// plage et ses membres si c est un groupe.
    #[test]
    fn la_reference_sous_la_position_est_trouvee() {
        let texte = "Avant {{Jn 3:16 ; Gn 1:1-3}} après";
        // Au milieu du groupe.
        let (debut, fin, references) = reference_au(texte, 12).expect("le groupe est visé");
        assert_eq!(&texte[debut..fin], "{{Jn 3:16 ; Gn 1:1-3}}");
        assert_eq!(references.len(), 2, "un groupe de deux");
        // Hors de toute référence.
        assert!(reference_au(texte, 2).is_none(), "avant n est rien");

        // Chaque verset d une liste a sa propre position : survoler « 17 »
        // dit Jean 3:17, pas le premier membre de la balise.
        let contenu = "Jn 3:16, 17 ; Rm 8:1";
        let positions = crate::langage::references::references_situees(contenu);
        assert_eq!(positions.len(), 3, "trois versets : {positions:?}");
        assert_eq!(positions[1].2, Reference::verset("John", 3, 17));
        assert_eq!(positions[2].2, Reference::verset("Rom", 8, 1));

        let texte = ["{{", contenu, "}}"].concat();
        for (debut, _, reference) in positions {
            assert_eq!(
                reference_sous(&texte, debut + 2),
                Some(reference),
                "la position dit le bon verset"
            );
        }

        // Un point-virgule dans le texte libre ne coupe pas la citation :
        // la référence qui suit garde sa position.
        let avec_texte = "Jn 3.16 ; \"cf. a; b\" ; Gn 1.1";
        let positions = crate::langage::references::references_situees(avec_texte);
        assert_eq!(positions.len(), 2, "deux références : {positions:?}");
        assert_eq!(positions[1].2, Reference::verset("Gen", 1, 1));
        assert_eq!(
            &avec_texte[positions[1].0..positions[1].1],
            "Gn 1.1",
            "la position de la seconde référence est exacte"
        );
    }

    /// La bulle du survol : les versets du groupe, avec la traduction.
    #[test]
    fn la_bulle_dit_le_extrait() {
        let groupe = vec![
            Reference::verset("Gen", 1, 1),
            Reference::verset("John", 3, 16),
        ];
        let extrait = extrait(&groupe, Traduction::Lsg);
        assert!(extrait.contains("Au commencement"), "{extrait}");
        assert!(
            extrait.contains("(16)"),
            "le verset est numéroté : {extrait}"
        );
        assert!(
            extrait.contains("Louis Segond 1910"),
            "la traduction est dite : {extrait}"
        );
    }

    /// La trace du projet : chaque référence citée, et combien de fois.
    #[test]
    fn le_releve_compte_les_citations() {
        let texte = "{{Jn 3:16}} puis {{Jn 3:16 ; Gn 1:1-3}} et {{Jn 3:16}}.";
        let releve = releve(texte);
        assert_eq!(
            releve.get("Jn 3.16"),
            Some(&3),
            "trois citations : {releve:?}"
        );
        assert_eq!(
            releve.get("Gn 1.1-3"),
            Some(&1),
            "le groupe compte pour soi"
        );
    }

    /// Les statistiques des livres cités (le système biblique) : les
    /// comptes par livre et par testament.
    #[test]
    fn les_statistiques_comptent_les_livres() {
        let mut stats = Statistiques::default();
        stats.ajouter("{{Jn 3:16}} et {{Gn 1:1 ; Jn 3:16}}");
        assert_eq!(stats.total, 3, "trois citations");
        assert_eq!(stats.par_livre.get("Jn"), Some(&2), "Jn deux fois");
        assert_eq!(stats.par_livre.get("Gn"), Some(&1), "Gn une fois");
        assert_eq!(stats.par_testament.get("Nouveau"), Some(&2), "le Nouveau");
        assert_eq!(stats.par_testament.get("Ancien"), Some(&1), "l Ancien");
        let cites = stats.plus_cites(10);
        assert_eq!(cites[0], ("Jn".to_string(), 2), "le plus cité d abord");
    }

    /// Un relevé écrit devient des statistiques (l agrégation des
    /// échelles : document, étagère, tous les projets).
    #[test]
    fn un_releve_devient_des_statistiques() {
        let mut releve = std::collections::BTreeMap::new();
        releve.insert("Jn 3:16".to_string(), 3);
        releve.insert("Gn 1:1".to_string(), 2);
        let stats = Statistiques::depuis_releve(&releve);
        assert_eq!(stats.total, 5);
        assert_eq!(stats.par_livre.get("Jn"), Some(&3));
        assert_eq!(stats.par_testament.get("Ancien"), Some(&2));
    }

    /// L analyse intelligente : la couverture, les écarts, le favori.
    #[test]
    fn analyse_dit_plus_que_des_nombres() {
        let mut releve = std::collections::BTreeMap::new();
        releve.insert("Jn 3:16".to_string(), 5);
        releve.insert("Jn 3:17".to_string(), 2);
        releve.insert("Gn 1:1".to_string(), 1);
        let analyse = Analyse::depuis_releve(&releve);
        let total = livres(Traduction::Lsg).len();
        assert_eq!(analyse.livres_cites, 2, "deux livres cités");
        assert_eq!(
            analyse.jamais_cites.len(),
            total - 2,
            "les autres livres manquent"
        );
        assert_eq!(analyse.favori, Some(("Jn".to_string(), 7)));
        assert_eq!(analyse.chapitre_favori, Some(("Jn 3".to_string(), 7)));
        assert_eq!(analyse.concentration, 100, "les trois sommets portent tout");
    }

    /// Le texte embarqué est là : Genèse 1,1 en Segond 1910.
    #[test]
    fn le_texte_se_lit() {
        let reference = Reference::verset("Gen", 1, 1);
        let versets = versets(&reference, Traduction::Lsg);
        assert_eq!(versets.len(), 1);
        assert!(versets[0].1.contains("Au commencement"), "{:?}", versets);
    }

    /// Les deux traductions disent le même verset, chacune à sa façon.
    #[test]
    fn les_deux_traductions_disent_le_verset() {
        let reference = Reference::verset("John", 3, 16);
        let lsg = versets(&reference, Traduction::Lsg);
        let darby = versets(&reference, Traduction::Darby);
        assert!(!lsg.is_empty() && !darby.is_empty());
        assert_ne!(lsg[0].1, darby[0].1, "deux traductions, deux textes");
    }

    /// Une plage donne tous ses versets, dans l'ordre.
    #[test]
    fn une_plage_donne_ses_versets() {
        let reference = Reference::plage("Ps", 23, 1, 3);
        let versets = versets(&reference, Traduction::Lsg);
        assert_eq!(versets.len(), 3);
        assert_eq!(versets[0].0, 1);
        assert_eq!(versets[2].0, 3);
    }

    /// Une suite de chapitres entiers donne les versets de chacun.
    #[test]
    fn une_suite_de_chapitres_donne_les_versets_de_chacun() {
        let reference = Reference::chapitres("Gen", 1, 2);
        let versets = versets(&reference, Traduction::Lsg);
        assert!(versets.len() > 40, "les deux chapitres sont là");
        assert_eq!(
            versets.iter().filter(|(numero, _)| *numero == 1).count(),
            2,
            "le premier verset de chaque chapitre est présent"
        );
    }

    /// Une traduction explicite se lit, s'affiche et revient dans la source.
    #[test]
    fn une_traduction_explicite_se_lit_et_se_relit() {
        let reference = depuis_texte("Jn 3.16|lsg").expect("la référence");
        assert_eq!(reference.traduction, Some(Traduction::Lsg));
        assert_eq!(
            afficher(
                &reference,
                &crate::mise_en_page::modele::References::default()
            ),
            "Jn 3.16 (LSG)"
        );

        let (elements, forme) =
            crate::langage::references::lire_groupe_complet("Jn 3.16|darby ; 3.17|lsg")
                .expect("le groupe");
        assert_eq!(forme, None);
        assert_eq!(
            elements[0].reference().unwrap().traduction,
            Some(Traduction::Darby)
        );
        assert_eq!(
            elements[1].reference().unwrap().traduction,
            Some(Traduction::Lsg)
        );
        assert_eq!(
            crate::langage::references::afficher_elements(
                &elements,
                &crate::mise_en_page::modele::References::default()
            ),
            "Jn 3.16 (DARBY), 3.17 (LSG)"
        );
        assert_eq!(
            crate::langage::references::ecrire_elements(
                &elements,
                &crate::mise_en_page::modele::References::default()
            ),
            "Jn 3.16|darby ; Jn 3.17|lsg"
        );

        // La source reste canonique même quand l'affichage est un gabarit
        // libre : un `({reference})` ne doit pas casser une réécriture.
        let gabarit = crate::mise_en_page::modele::References {
            gabarit: "({reference})".to_string(),
            ..Default::default()
        };
        assert_eq!(
            crate::langage::references::ecrire_elements(&elements, &gabarit),
            "Jn 3.16|darby ; Jn 3.17|lsg"
        );

        let (avec_forme, forme) =
            crate::langage::references::lire_groupe_complet("Jn 3.16|lsg ; Gn 1.1|darby|long")
                .expect("le groupe");
        assert_eq!(forme, Some(FormeLivre::FrancaiseLongue));
        assert_eq!(avec_forme.len(), 2);
        assert_eq!(
            crate::langage::references::afficher_elements_forme(
                &avec_forme,
                &crate::mise_en_page::modele::References::default(),
                forme
            ),
            "Jean 3.16 (LSG) ; Genèse 1.1 (DARBY)"
        );
    }

    /// Les livres sont nommés et abrégés.
    #[test]
    fn les_livres_sont_nommes() {
        let livres = livres(Traduction::Lsg);
        assert_eq!(
            livres.len(),
            66 + crate::langage::references::livres_hors_canon().len()
        );
        let genese = livres.iter().find(|l| l.osis == "Gen").expect("la Genèse");
        assert_eq!(genese.nom, "Genèse");
        assert_eq!(genese.abreviation, "Gen");
    }

    #[test]
    fn la_recherche_ne_montre_que_les_livres_correspondants() {
        let tous = livres(Traduction::Lsg);
        let henoch = filtrer_livres(&tous, "henoch");
        assert!(henoch.iter().any(|livre| livre.osis == "1En"));
        assert!(!henoch.iter().any(|livre| livre.osis == "Gen"));

        let genese = filtrer_livres(&tous, "genese");
        assert!(genese.iter().any(|livre| livre.osis == "Gen"));
        assert!(!genese.iter().any(|livre| livre.osis == "1En"));

        assert!(
            filtrer_livres(&tous, "introuvable").is_empty(),
            "aucun livre ne correspond"
        );
    }
}
