//! Formatage et navigation dans les tableaux Markdown.
//!
//! Le module travaille directement sur la source `.tw` : il n'a pas besoin de
//! l'AST et ne modifie jamais autre chose que les lignes d'un tableau. Il
//! n'ajoute aucune dépendance : une bibliothèque de rendu de tableaux de
//! terminal (`comfy-table`, `prettytable`) produirait une grille à afficher,
//! pas la source Markdown à conserver dans le document.

use std::ops::Range;

/// Le déplacement demandé dans une grille.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensNavigation {
    /// `Tab` : cellule suivante dans l'ordre de lecture.
    CelluleSuivante,
    /// `Maj+Tab` : cellule précédente.
    CellulePrecedente,
    /// `Entrée` : même colonne, ligne suivante.
    LigneSuivante,
    /// `Maj+Entrée` : même colonne, ligne précédente.
    LignePrecedente,
}

/// Le sens d'un déplacement de rangée ou de colonne dans un tableau.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensDeplacement {
    Haut,
    Bas,
    Gauche,
    Droite,
}

/// Le remplacement à poser pour formater un tableau, et la position du
/// curseur dans la source **après** ce remplacement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatageTableau {
    /// La plage du bloc de tableau, en octets, sans les lignes voisines.
    pub plage: Range<usize>,

    /// Le bloc de tableau aligné.
    pub texte: String,

    /// Le début de la cellule cible dans la source après remplacement.
    pub curseur: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Alignement {
    Gauche,
    Centre,
    Droite,
}

#[derive(Debug)]
struct LigneTableau {
    debut: usize,
    fin: usize,
    cellules: Vec<(usize, usize)>,
    brutes: Vec<(usize, usize)>,
}

/// Une ligne de tableau, avec les bornes de chaque cellule dans la source.
fn cellule_de_ligne(source: &str, debut: usize, fin: usize) -> Option<LigneTableau> {
    let ligne = &source[debut..fin];
    let sans_retour = ligne.trim_end_matches(['\n', '\r']);
    let indentation = sans_retour.len() - sans_retour.trim_start().len();
    let contenu = &sans_retour[indentation..];
    if !contenu.starts_with('|') {
        return None;
    }

    let base = debut + indentation;
    let mut cellules = Vec::new();
    let mut brutes = Vec::new();
    let mut debut_cellule = base + 1;
    let mut antislashs = 0usize;

    for (index, caractere) in contenu.char_indices() {
        if index == 0 {
            continue;
        }
        if caractere == '\\' {
            antislashs += 1;
            continue;
        }
        if caractere == '|' && antislashs.is_multiple_of(2) {
            cellules.push(rogner(source, debut_cellule, base + index));
            brutes.push((debut_cellule, base + index));
            debut_cellule = base + index + 1;
        }
        antislashs = 0;
    }

    // Une ligne se termine normalement par `|`; le segment qui suit ce dernier
    // pipe est vide et ne doit pas devenir une cellule.
    if debut_cellule < base + contenu.len() {
        let fin_cellule = rogner(source, debut_cellule, base + contenu.len()).1;
        if fin_cellule > debut_cellule
            || !source[debut_cellule..base + contenu.len()]
                .trim()
                .is_empty()
        {
            cellules.push(rogner(source, debut_cellule, base + contenu.len()));
            brutes.push((debut_cellule, base + contenu.len()));
        }
    }

    Some(LigneTableau {
        debut,
        fin,
        cellules,
        brutes,
    })
}

fn rogner(source: &str, mut debut: usize, mut fin: usize) -> (usize, usize) {
    while debut < fin {
        let Some(caractere) = source[debut..fin].chars().next() else {
            break;
        };
        if !caractere.is_whitespace() {
            break;
        }
        debut += caractere.len_utf8();
    }
    while debut < fin {
        let Some(caractere) = source[..fin].chars().next_back() else {
            break;
        };
        if !caractere.is_whitespace() {
            break;
        }
        fin -= caractere.len_utf8();
    }
    (debut, fin)
}

fn est_ligne_tableau(source: &str, debut: usize, fin: usize) -> bool {
    let ligne = &source[debut..fin];
    ligne.trim_start().starts_with('|')
}

fn lignes_tableau(source: &str) -> Vec<(usize, usize)> {
    let mut lignes = Vec::new();
    let mut debut = 0;
    for (index, caractere) in source.char_indices() {
        if caractere == '\n' {
            lignes.push((debut, index + 1));
            debut = index + 1;
        }
    }
    if debut < source.len() {
        lignes.push((debut, source.len()));
    }
    lignes
}

fn lignes_du_bloc(source: &str, ligne_courante: usize) -> Option<(usize, usize)> {
    let lignes = lignes_tableau(source);
    if ligne_courante >= lignes.len() {
        return None;
    }
    let (debut, fin) = lignes[ligne_courante];
    if !est_ligne_tableau(source, debut, fin) {
        return None;
    }

    let mut premier = ligne_courante;
    while premier > 0 {
        let (d, f) = lignes[premier - 1];
        if !est_ligne_tableau(source, d, f) {
            break;
        }
        premier -= 1;
    }
    let mut dernier = ligne_courante;
    while dernier + 1 < lignes.len() {
        let (d, f) = lignes[dernier + 1];
        if !est_ligne_tableau(source, d, f) {
            break;
        }
        dernier += 1;
    }
    Some((premier, dernier))
}

fn index_ligne(source: &str, offset: usize) -> Option<usize> {
    let offset = offset.min(source.len());
    let lignes = lignes_tableau(source);

    let index = lignes
        .iter()
        .position(|(debut, fin)| offset >= *debut && offset < *fin);

    if index.is_some() {
        return index;
    }

    // Après le dernier saut de ligne, le curseur est sur une ligne vide qui
    // n'appartient pas au tableau. Sans cette distinction, `Entrée` resterait
    // prisonnière du tableau au lieu de laisser écrire le paragraphe suivant.
    if offset == source.len() && !source.ends_with('\n') && !lignes.is_empty() {
        return Some(lignes.len() - 1);
    }

    None
}

fn est_separateur(source: &str, ligne: &LigneTableau) -> bool {
    !ligne.cellules.is_empty()
        && ligne.cellules.iter().all(|(debut, fin)| {
            let texte = source[*debut..*fin].trim();
            !texte.is_empty()
                && texte.contains('-')
                && texte
                    .chars()
                    .all(|c| c == '-' || c == ':' || c.is_whitespace())
        })
}

/// La ligne est-elle dans un bloc de code clôturé par des fences ?
///
/// Une grille qui ressemble à un tableau dans un bloc ` ```typst ` n'en est
/// pas un : la navigation et le formatage ne doivent jamais y toucher.
fn est_dans_un_bloc_de_code(source: &str, ligne_cible: usize) -> bool {
    let mut dans_code = false;

    for (index, ligne) in source.lines().enumerate() {
        let debut = ligne.trim_start();
        let fence = debut.starts_with("```") || debut.starts_with("~~~");

        if index == ligne_cible {
            return dans_code;
        }
        if fence {
            dans_code = !dans_code;
        }
    }

    false
}

/// La cible d'un déplacement dans une grille, avant tout formatage.
///
/// `(rangée, colonne, début)` : la rangée est comptée dans le bloc de
/// tableau, séparateur compris ; le début est un octet dans la source.
fn cible_navigation(
    source: &str,
    offset: usize,
    sens: SensNavigation,
) -> Option<(usize, usize, usize)> {
    let ligne_courante = index_ligne(source, offset)?;
    if est_dans_un_bloc_de_code(source, ligne_courante) {
        return None;
    }
    let (premier, dernier) = lignes_du_bloc(source, ligne_courante)?;
    let lignes = lignes_tableau(source);

    let mut rangees = Vec::new();
    for (debut, fin) in &lignes[premier..=dernier] {
        let Some(ligne) = cellule_de_ligne(source, *debut, *fin) else {
            continue;
        };
        rangees.push(ligne);
    }

    let position = rangees
        .iter()
        .position(|ligne| offset >= ligne.debut && offset < ligne.fin)?;
    let colonne = rangees[position]
        .cellules
        .iter()
        .position(|(debut, fin)| offset >= *debut && offset <= *fin)
        .or_else(|| {
            // Le curseur est sur une barre verticale : il appartient à la
            // cellule qui la précède, ou à la première cellule de la ligne.
            rangees[position]
                .cellules
                .iter()
                .rposition(|(_, fin)| offset >= *fin)
                .or(Some(0))
        })?;

    match sens {
        SensNavigation::CelluleSuivante | SensNavigation::CellulePrecedente => {
            let mut plat = Vec::new();
            for (index, ligne) in rangees.iter().enumerate() {
                if est_separateur(source, ligne) {
                    continue;
                }
                for (colonne, (debut, _)) in ligne.cellules.iter().enumerate() {
                    plat.push((index, colonne, *debut));
                }
            }
            let position_plat = plat.iter().position(|(index, colonne_cellule, _)| {
                *index == position && *colonne_cellule == colonne
            });
            let cible = match sens {
                SensNavigation::CelluleSuivante => position_plat?.checked_add(1),
                SensNavigation::CellulePrecedente => position_plat?.checked_sub(1),
                _ => None,
            }?;
            plat.get(cible).copied()
        }
        SensNavigation::LigneSuivante | SensNavigation::LignePrecedente => {
            let pas = if sens == SensNavigation::LigneSuivante {
                1isize
            } else {
                -1
            };
            let mut index = position as isize + pas;
            while index >= 0 && (index as usize) < rangees.len() {
                let ligne = &rangees[index as usize];
                if !est_separateur(source, ligne) {
                    return ligne
                        .cellules
                        .get(colonne)
                        .map(|(debut, _)| (index as usize, colonne, *debut));
                }
                index += pas;
            }
            None
        }
    }
}

/// Déplace le curseur dans le tableau qui contient `offset`.
///
/// Renvoie `None` lorsque le curseur n'est pas dans une cellule, ou lorsqu'il
/// n'existe pas de cellule dans la direction demandée. L'appelant peut alors
/// laisser le geste habituel (focus, changement d'onglet) se produire.
#[allow(dead_code)] // API pure conservée pour les épreuves du module.
pub fn naviguer(source: &str, offset: usize, sens: SensNavigation) -> Option<usize> {
    cible_navigation(source, offset, sens).map(|(_, _, debut)| debut)
}

/// Le curseur est-il dans une ligne de tableau, même sans cellule cible ?
///
/// Sert à retenir `Entrée` dans une cellule au bord de la grille : la touche
/// ne doit pas insérer une ligne qui casserait le tableau.
pub fn dans_un_tableau(source: &str, offset: usize) -> bool {
    let Some(ligne) = index_ligne(source, offset) else {
        return false;
    };
    !est_dans_un_bloc_de_code(source, ligne) && lignes_du_bloc(source, ligne).is_some()
}

/// Le début d'une cellule dans un texte de tableau.
fn offset_de_cellule(source: &str, ligne: usize, colonne: usize) -> Option<usize> {
    let lignes = lignes_tableau(source);
    let (debut, fin) = *lignes.get(ligne)?;
    let ligne = cellule_de_ligne(source, debut, fin)?;
    let (debut_cellule, fin_cellule) = *ligne.cellules.get(colonne)?;

    // Une cellule vide n'a pas de contenu à viser : on place le curseur
    // après la marge du pipe, à l'intérieur de la case, plutôt qu'au bord
    // droit où il semblerait appartenir au pipe suivant.
    if debut_cellule == fin_cellule {
        let (brut_debut, brut_fin) = *ligne.brutes.get(colonne)?;
        return Some((brut_debut + 1).min(brut_fin));
    }

    Some(debut_cellule)
}

/// Navigue d'une cellule à l'autre **et** aligne le tableau avant de poser le
/// curseur.
///
/// Le formatage ne touche qu'au bloc de tableau courant ; le reste du document,
/// les autres tableaux et les titres canoniques restent intacts. La cible est
/// retrouvée après le formatage par ses coordonnées de rangée et de colonne,
/// jamais par un ancien décalage d'octets : les espaces ajoutés changent tous
/// les offsets de la ligne.
pub fn naviguer_et_formater(
    source: &str,
    offset: usize,
    sens: SensNavigation,
) -> Option<FormatageTableau> {
    let (rangee_cible, colonne_cible, _) = cible_navigation(source, offset, sens)?;
    let ligne_courante = index_ligne(source, offset)?;
    let (premier, dernier) = lignes_du_bloc(source, ligne_courante)?;
    let lignes = lignes_tableau(source);

    let texte_lignes: Vec<&str> = lignes[premier..=dernier]
        .iter()
        .map(|(debut, fin)| &source[*debut..*fin])
        .collect();
    let formate = formater_tableau(&texte_lignes);
    if formate.len() != texte_lignes.len() {
        return None;
    }

    let bloc = formate.join("");
    let debut_bloc = lignes[premier].0;
    let mut fin_bloc = lignes[dernier].1;
    // Le saut de ligne final reste dans la source : `replace_span` remplace
    // le contenu du bloc, pas la ligne qui le termine. Le garder ici ferait
    // une ligne vide de plus après chaque tabulation.
    let texte = if let Some(sans) = bloc.strip_suffix("\r\n") {
        fin_bloc = fin_bloc.saturating_sub(2);
        sans.to_string()
    } else if let Some(sans) = bloc.strip_suffix('\n') {
        fin_bloc = fin_bloc.saturating_sub(1);
        sans.to_string()
    } else {
        bloc
    };
    let curseur = offset_de_cellule(&texte, rangee_cible, colonne_cible)?;

    Some(FormatageTableau {
        plage: debut_bloc..fin_bloc,
        texte,
        curseur: debut_bloc + curseur,
    })
}

/// La situation du curseur dans le tableau courant : ses lignes brutes,
/// sa rangée et sa colonne, et l'indentation de la première ligne.
struct SituationTableau {
    premier: usize,
    dernier: usize,
    rangee: usize,
    colonne: usize,
    lignes: Vec<String>,
}

fn situation_tableau(source: &str, offset: usize) -> Option<SituationTableau> {
    let ligne_courante = index_ligne(source, offset)?;
    if est_dans_un_bloc_de_code(source, ligne_courante) {
        return None;
    }
    let (premier, dernier) = lignes_du_bloc(source, ligne_courante)?;
    let toutes = lignes_tableau(source);
    let mut rangees = Vec::new();
    for (debut, fin) in &toutes[premier..=dernier] {
        rangees.push(cellule_de_ligne(source, *debut, *fin)?);
    }

    let rangee = rangees
        .iter()
        .position(|ligne| offset >= ligne.debut && offset < ligne.fin)?;
    let colonne = rangees[rangee]
        .cellules
        .iter()
        .position(|(debut, fin)| offset >= *debut && offset <= *fin)
        .or_else(|| {
            rangees[rangee]
                .cellules
                .iter()
                .rposition(|(_, fin)| offset >= *fin)
        })
        .or(Some(0))?;

    Some(SituationTableau {
        premier,
        dernier,
        rangee,
        colonne,
        lignes: toutes[premier..=dernier]
            .iter()
            .map(|(debut, fin)| source[*debut..*fin].to_string())
            .collect(),
    })
}

fn lignes_depuis_cellules(
    indentation: &str,
    cellules: &[Vec<String>],
    derniere_sans_saut: bool,
) -> Vec<String> {
    let mut sortie = Vec::with_capacity(cellules.len());
    for (index, ligne) in cellules.iter().enumerate() {
        let saut = if index + 1 == cellules.len() && derniere_sans_saut {
            ""
        } else {
            "\n"
        };
        sortie.push(format!("{indentation}| {} |{saut}", ligne.join(" | ")));
    }
    sortie
}

fn cellules_du_contexte(situation: &SituationTableau) -> Vec<Vec<String>> {
    situation
        .lignes
        .iter()
        .map(|ligne| decouper_ligne(ligne).unwrap_or_default())
        .collect()
}

fn colonnes_du_contexte(situation: &SituationTableau) -> usize {
    cellules_du_contexte(situation)
        .iter()
        .map(Vec::len)
        .max()
        .unwrap_or(1)
        .max(1)
}

fn est_ligne_de_separation(cellules: &[String]) -> bool {
    !cellules.is_empty()
        && cellules.iter().all(|cellule| {
            let texte = cellule.trim();
            !texte.is_empty() && texte.contains('-') && texte.chars().all(|c| c == '-' || c == ':')
        })
}

fn formatage_depuis_cellules(
    source: &str,
    situation: &SituationTableau,
    cellules: Vec<Vec<String>>,
    rangee_cible: usize,
    colonne_cible: usize,
) -> Option<FormatageTableau> {
    let indentation = situation
        .lignes
        .first()
        .map(|ligne| &ligne[..ligne.len() - ligne.trim_start().len()])
        .unwrap_or("");
    let derniere_sans_saut = situation
        .lignes
        .last()
        .is_some_and(|ligne| !ligne.ends_with('\n'));
    let lignes = lignes_depuis_cellules(indentation, &cellules, derniere_sans_saut);
    let references: Vec<&str> = lignes.iter().map(String::as_str).collect();
    let formate = formater_tableau(&references);
    if formate.len() != lignes.len() {
        return None;
    }

    let bloc = formate.join("");
    let debut = lignes_tableau(source)[situation.premier].0;
    let mut fin = lignes_tableau(source)[situation.dernier].1;
    let texte = if let Some(sans) = bloc.strip_suffix("\r\n") {
        fin = fin.saturating_sub(2);
        sans.to_string()
    } else if let Some(sans) = bloc.strip_suffix('\n') {
        fin = fin.saturating_sub(1);
        sans.to_string()
    } else {
        bloc
    };
    let curseur = offset_de_cellule(&texte, rangee_cible, colonne_cible)?;

    Some(FormatageTableau {
        plage: debut..fin,
        texte,
        curseur: debut + curseur,
    })
}

/// Insère une colonne vide à droite de la colonne du curseur.
pub fn inserer_colonne(source: &str, offset: usize) -> Option<FormatageTableau> {
    let situation = situation_tableau(source, offset)?;
    let colonnes = colonnes_du_contexte(&situation);
    let nouvelle = (situation.colonne + 1).min(colonnes);
    let mut cellules = cellules_du_contexte(&situation);

    for (index, ligne) in cellules.iter_mut().enumerate() {
        while ligne.len() < colonnes {
            ligne.push(String::new());
        }
        let valeur = if index == 1 { "---" } else { "" };
        ligne.insert(nouvelle, valeur.to_string());
    }

    formatage_depuis_cellules(source, &situation, cellules, situation.rangee, nouvelle)
}

/// Insère une ligne vide sous la ligne du curseur.
pub fn inserer_ligne(source: &str, offset: usize) -> Option<FormatageTableau> {
    let situation = situation_tableau(source, offset)?;
    let colonnes = colonnes_du_contexte(&situation);
    let mut cellules = cellules_du_contexte(&situation);

    for ligne in &mut cellules {
        while ligne.len() < colonnes {
            ligne.push(String::new());
        }
    }

    let insertion = if situation.rangee <= 1 {
        if cellules.len() > 1 { 2 } else { 1 }
    } else {
        situation.rangee + 1
    };
    if insertion > cellules.len() {
        return None;
    }
    cellules.insert(insertion, vec![String::new(); colonnes]);

    formatage_depuis_cellules(
        source,
        &situation,
        cellules,
        insertion,
        situation.colonne.min(colonnes.saturating_sub(1)),
    )
}

/// Supprime la rangée de données du curseur.
///
/// L'en-tête et la ligne de séparation ne sont jamais supprimés : sans eux,
/// le tableau ne serait plus un tableau Markdown. Le curseur reste sur la
/// rangée suivante, ou sur l'en-tête quand la dernière rangée de données
/// vient de partir.
pub fn supprimer_ligne(source: &str, offset: usize) -> Option<FormatageTableau> {
    let situation = situation_tableau(source, offset)?;
    if situation.rangee < 2 {
        return None;
    }

    let colonnes = colonnes_du_contexte(&situation);
    let mut cellules = cellules_du_contexte(&situation);
    for ligne in &mut cellules {
        while ligne.len() < colonnes {
            ligne.push(String::new());
        }
    }
    if situation.rangee >= cellules.len() {
        return None;
    }
    cellules.remove(situation.rangee);

    let cible = if situation.rangee < cellules.len() {
        situation.rangee
    } else {
        cellules.len().saturating_sub(1)
    };
    let cible = if cible < 2 { 0 } else { cible };
    formatage_depuis_cellules(source, &situation, cellules, cible, situation.colonne)
}

/// Supprime la colonne du curseur, sur toutes les rangées du tableau.
///
/// La dernière colonne n'est pas supprimée : il doit rester au moins une
/// cellule par rangée pour que la grille reste valide.
pub fn supprimer_colonne(source: &str, offset: usize) -> Option<FormatageTableau> {
    let situation = situation_tableau(source, offset)?;
    let colonnes = colonnes_du_contexte(&situation);
    if colonnes <= 1 || situation.colonne >= colonnes {
        return None;
    }

    let mut cellules = cellules_du_contexte(&situation);
    for ligne in &mut cellules {
        while ligne.len() < colonnes {
            ligne.push(String::new());
        }
        ligne.remove(situation.colonne);
    }

    let cible = situation.colonne.min(colonnes - 2);
    formatage_depuis_cellules(source, &situation, cellules, situation.rangee, cible)
}

/// Déplace la rangée du curseur vers le haut ou vers le bas.
pub fn deplacer_ligne(
    source: &str,
    offset: usize,
    sens: SensDeplacement,
) -> Option<FormatageTableau> {
    let situation = situation_tableau(source, offset)?;
    if !matches!(sens, SensDeplacement::Haut | SensDeplacement::Bas) {
        return None;
    }
    if situation.rangee < 2 {
        return None;
    }

    let cible = match sens {
        SensDeplacement::Haut => situation.rangee.checked_sub(1)?,
        SensDeplacement::Bas => situation.rangee + 1,
        _ => return None,
    };
    if cible < 2 {
        return None;
    }

    let mut cellules = cellules_du_contexte(&situation);
    if cible >= cellules.len() {
        return None;
    }
    cellules.swap(situation.rangee, cible);

    formatage_depuis_cellules(source, &situation, cellules, cible, situation.colonne)
}

/// Déplace la colonne du curseur vers la gauche ou vers la droite.
pub fn deplacer_colonne(
    source: &str,
    offset: usize,
    sens: SensDeplacement,
) -> Option<FormatageTableau> {
    let situation = situation_tableau(source, offset)?;
    if !matches!(sens, SensDeplacement::Gauche | SensDeplacement::Droite) {
        return None;
    }
    let colonnes = colonnes_du_contexte(&situation);
    let cible = match sens {
        SensDeplacement::Gauche => situation.colonne.checked_sub(1)?,
        SensDeplacement::Droite => situation.colonne + 1,
        _ => return None,
    };
    if cible >= colonnes {
        return None;
    }

    let mut cellules = cellules_du_contexte(&situation);
    for ligne in &mut cellules {
        while ligne.len() < colonnes {
            ligne.push(String::new());
        }
        ligne.swap(situation.colonne, cible);
    }

    formatage_depuis_cellules(source, &situation, cellules, situation.rangee, cible)
}

/// Fait tourner l'alignement de la colonne sous le curseur.
///
/// Le clic droit n'a de sens que sur la ligne d'en-tête : ailleurs, le
/// tableau n'a pas de colonne à aligner.
pub fn aligner_colonne(source: &str, offset: usize) -> Option<FormatageTableau> {
    let situation = situation_tableau(source, offset)?;
    if situation.rangee != 0 {
        return None;
    }

    let mut cellules = cellules_du_contexte(&situation);
    let colonnes = colonnes_du_contexte(&situation);
    for ligne in &mut cellules {
        while ligne.len() < colonnes {
            ligne.push(String::new());
        }
    }
    let separation = cellules.get(1)?;
    if !est_ligne_de_separation(separation) {
        return None;
    }

    let suivant = match alignement(&separation[situation.colonne]) {
        Alignement::Gauche => Alignement::Centre,
        Alignement::Centre => Alignement::Droite,
        Alignement::Droite => Alignement::Gauche,
    };
    cellules[1][situation.colonne] = match suivant {
        Alignement::Gauche => "---".to_string(),
        Alignement::Centre => ":-:".to_string(),
        Alignement::Droite => "---:".to_string(),
    };

    formatage_depuis_cellules(source, &situation, cellules, 0, situation.colonne)
}

fn decouper_ligne(ligne: &str) -> Option<Vec<String>> {
    let sans_retour = ligne.trim_end_matches(['\n', '\r']);
    let contenu = sans_retour.trim_start();
    if !contenu.starts_with('|') {
        return None;
    }

    let mut cellules = Vec::new();
    let mut debut = 1;
    let mut antislashs = 0usize;
    for (index, caractere) in contenu.char_indices() {
        if index == 0 {
            continue;
        }
        if caractere == '\\' {
            antislashs += 1;
            continue;
        }
        if caractere == '|' && antislashs.is_multiple_of(2) {
            cellules.push(contenu[debut..index].trim().to_string());
            debut = index + 1;
        }
        antislashs = 0;
    }
    if debut < contenu.len() && !contenu[debut..].trim().is_empty() {
        cellules.push(contenu[debut..].trim().to_string());
    }
    Some(cellules)
}

fn alignement(cellule: &str) -> Alignement {
    let texte = cellule.trim();
    let gauche = texte.starts_with(':');
    let droite = texte.ends_with(':');
    match (gauche, droite) {
        (true, true) => Alignement::Centre,
        (false, true) => Alignement::Droite,
        _ => Alignement::Gauche,
    }
}

fn largeur(texte: &str) -> usize {
    texte.chars().count()
}

fn remplir(texte: &str, largeur_colonne: usize, alignement: Alignement) -> String {
    let manque = largeur_colonne.saturating_sub(largeur(texte));
    match alignement {
        Alignement::Gauche => format!("{texte}{}", " ".repeat(manque)),
        Alignement::Droite => format!("{}{texte}", " ".repeat(manque)),
        Alignement::Centre => {
            let gauche = manque / 2;
            format!(
                "{}{texte}{}",
                " ".repeat(gauche),
                " ".repeat(manque - gauche)
            )
        }
    }
}

fn tirets(largeur_colonne: usize, alignement: Alignement) -> String {
    let largeur_colonne = largeur_colonne.max(3);
    match alignement {
        Alignement::Gauche => format!(":{}", "-".repeat(largeur_colonne.saturating_sub(1))),
        Alignement::Droite => format!("{}:", "-".repeat(largeur_colonne.saturating_sub(1))),
        Alignement::Centre => format!(":{}:", "-".repeat(largeur_colonne.saturating_sub(2))),
    }
}

fn formater_tableau(lignes: &[&str]) -> Vec<String> {
    let cellules: Vec<Vec<String>> = lignes.iter().filter_map(|l| decouper_ligne(l)).collect();
    let colonnes = cellules.iter().map(Vec::len).max().unwrap_or(0);
    if colonnes == 0 {
        return lignes.iter().map(|l| (*l).to_string()).collect();
    }
    let indentation = lignes
        .first()
        .map(|ligne| &ligne[..ligne.len() - ligne.trim_start().len()])
        .unwrap_or("");

    let mut largeurs = vec![4usize; colonnes];
    for ligne in &cellules {
        for (index, cellule) in ligne.iter().enumerate() {
            largeurs[index] = largeurs[index].max(largeur(cellule));
        }
    }

    let alignements = cellules
        .get(1)
        .map(|ligne| {
            (0..colonnes)
                .map(|index| {
                    ligne
                        .get(index)
                        .map_or(Alignement::Gauche, |c| alignement(c))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| vec![Alignement::Gauche; colonnes]);

    let mut sortie = Vec::with_capacity(lignes.len());
    for (index, ligne) in cellules.iter().enumerate() {
        let est_separateur = index == 1
            && ligne.iter().all(|cellule| {
                let texte = cellule.trim();
                !texte.is_empty()
                    && texte.contains('-')
                    && texte.chars().all(|c| c == '-' || c == ':')
            });
        let mut morceaux = Vec::with_capacity(colonnes);
        for colonne in 0..colonnes {
            let texte = ligne.get(colonne).map(String::as_str).unwrap_or("");
            morceaux.push(if est_separateur {
                tirets(largeurs[colonne], alignements[colonne])
            } else {
                remplir(texte, largeurs[colonne], alignements[colonne])
            });
        }
        let retour = if lignes[index].ends_with('\n') {
            "\n"
        } else {
            ""
        };
        sortie.push(format!("{indentation}| {} |{retour}", morceaux.join(" | ")));
    }
    sortie
}

/// Aligne les colonnes de tous les tableaux du document.
///
/// Les titres canoniques (`{{t "…"}}`) ne sont pas touchés, pas plus que le
/// reste du texte. Une ligne de séparation est reconnue à sa deuxième ligne
/// et sert à conserver l'alignement gauche/centre/droite.
pub fn embellir(source: &str) -> String {
    let lignes: Vec<(usize, usize)> = lignes_tableau(source);
    let mut sortie = String::with_capacity(source.len());
    let mut index = 0;
    let mut dans_code = false;

    while index < lignes.len() {
        let (debut, fin) = lignes[index];
        let ligne = &source[debut..fin];
        let nu = ligne.trim_start();
        if nu.starts_with("```") || nu.starts_with("~~~") {
            dans_code = !dans_code;
            sortie.push_str(ligne);
            index += 1;
            continue;
        }
        if dans_code || !est_ligne_tableau(source, debut, fin) {
            sortie.push_str(ligne);
            index += 1;
            continue;
        }

        let premier = index;
        while index + 1 < lignes.len() {
            let (d, f) = lignes[index + 1];
            if !est_ligne_tableau(source, d, f) {
                break;
            }
            index += 1;
        }
        let texte_lignes: Vec<&str> = lignes[premier..=index]
            .iter()
            .map(|(d, f)| &source[*d..*f])
            .collect();
        sortie.push_str(&formater_tableau(&texte_lignes).join(""));
        index += 1;
    }

    sortie
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn les_colonnes_sont_aligneess_et_les_alignements_conserves() {
        let source =
            "Avant\n| Nom | Valeur |\n| :--- | ---: |\n| court | 1 |\n| plus long | 20 |\nAprès\n";
        let attendu = "Avant\n| Nom       | Valeur |\n| :-------- | -----: |\n| court     |      1 |\n| plus long |     20 |\nAprès\n";
        assert_eq!(embellir(source), attendu);
    }

    #[test]
    fn un_tableau_sans_alignement_reste_aligné_à_gauche() {
        let source = "| A | B |\n| --- | --- |\n| x | yy |\n";
        let attendu = "| A    | B    |\n| :--- | :--- |\n| x    | yy   |\n";
        assert_eq!(embellir(source), attendu);
    }

    #[test]
    fn tab_saute_les_cellules_et_la_ligne_de_separation() {
        let source = "| A | B |\n| --- | --- |\n| x | y |\n";
        let debut = source.find("A").unwrap();
        let b = source.find("B").unwrap();
        let x = source.find("x").unwrap();
        let y = source.find("y").unwrap();
        assert_eq!(
            naviguer(source, debut, SensNavigation::CelluleSuivante),
            Some(b)
        );
        assert_eq!(
            naviguer(source, b, SensNavigation::CelluleSuivante),
            Some(x)
        );
        assert_eq!(
            naviguer(source, y, SensNavigation::CellulePrecedente),
            Some(x)
        );
    }

    #[test]
    fn ctrl_tab_descend_dans_la_meme_colonne() {
        let source = "| A | B |\n| --- | --- |\n| x | y |\n";
        let a = source.find("A").unwrap();
        let x = source.find("x").unwrap();
        let b = source.find("B").unwrap();
        let y = source.find("y").unwrap();
        assert_eq!(naviguer(source, a, SensNavigation::LigneSuivante), Some(x));
        assert_eq!(naviguer(source, b, SensNavigation::LigneSuivante), Some(y));
        assert_eq!(
            naviguer(source, y, SensNavigation::LignePrecedente),
            Some(b)
        );
    }

    #[test]
    fn une_cellule_echappee_ne_coupe_pas_la_ligne() {
        let source = "| A | B |\n| --- | --- |\n| a \\| b | c |\n";
        let b_entete = source.find("B").unwrap();
        let a = source.find("a \\|").unwrap();
        assert_eq!(
            naviguer(source, b_entete, SensNavigation::CelluleSuivante),
            Some(a)
        );
        assert_eq!(
            naviguer(source, a, SensNavigation::CelluleSuivante),
            Some(source.find("c").unwrap())
        );
    }

    #[test]
    fn un_tableau_dans_un_bloc_de_code_n_est_pas_touche() {
        let source = "```\n| A | B |\n| --- | --- |\n```\n| C | D |\n| --- | --- |\n";
        let attendu = "```\n| A | B |\n| --- | --- |\n```\n| C    | D    |\n| :--- | :--- |\n";
        assert_eq!(embellir(source), attendu);
    }

    #[test]
    fn hors_tableau_aucune_navigation() {
        assert_eq!(
            naviguer("du texte\n", 0, SensNavigation::CelluleSuivante),
            None
        );
        assert!(!dans_un_tableau("du texte\n", 0));
        assert!(dans_un_tableau("| A | B |\n| --- | --- |\n", 2));
        assert!(
            !dans_un_tableau("| A | B |\n| --- | --- |\n", 24),
            "après le saut de ligne final, on est hors du tableau"
        );
        let code = "```typst\n| A | B |\n| --- | --- |\n```\n";
        assert!(!dans_un_tableau(code, code.find('A').unwrap()));
        assert_eq!(
            naviguer(
                code,
                code.find('A').unwrap(),
                SensNavigation::CelluleSuivante
            ),
            None
        );
    }

    fn appliquer(source: &str, formatage: &FormatageTableau) -> String {
        let mut resultat = source.to_string();
        resultat.replace_range(formatage.plage.clone(), &formatage.texte);
        resultat
    }

    #[test]
    fn tab_formate_le_tableau_et_repose_le_curseur_sur_la_cible() {
        let source = "Avant\n| A | B |\n| --- | --- |\n| x | y |\nAprès\n";
        let a = source.find("| A").unwrap() + 2;
        let formatage =
            naviguer_et_formater(source, a, SensNavigation::CelluleSuivante).expect("navigation");
        let resultat = appliquer(source, &formatage);

        assert_eq!(
            resultat,
            "Avant\n| A    | B    |\n| :--- | :--- |\n| x    | y    |\nAprès\n"
        );
        assert!(
            resultat[formatage.curseur..].starts_with('B'),
            "le curseur vise la cellule B, pas un ancien décalage : {}",
            &resultat[formatage.curseur..]
        );
    }

    #[test]
    fn une_cellule_vide_reste_une_cible() {
        let source = "| A | B |\n| --- | --- |\n|  | y |\n";
        let a = source.find('A').unwrap();
        let formatage =
            naviguer_et_formater(source, a, SensNavigation::LigneSuivante).expect("navigation");
        let resultat = appliquer(source, &formatage);

        assert_eq!(
            resultat,
            "| A    | B    |\n| :--- | :--- |\n|      | y    |\n"
        );
        assert_eq!(
            naviguer(
                &resultat,
                formatage.curseur,
                SensNavigation::CelluleSuivante
            ),
            Some(resultat.find('y').unwrap()),
            "la cellule vide reste une cible"
        );
    }

    #[test]
    fn entree_descend_dans_la_meme_colonne_et_formate_aussi() {
        let source = "| A | B |\n| --- | --- |\n| x | y |\n";
        let a = source.find('A').unwrap();
        let formatage =
            naviguer_et_formater(source, a, SensNavigation::LigneSuivante).expect("navigation");
        let resultat = appliquer(source, &formatage);

        assert_eq!(
            resultat,
            "| A    | B    |\n| :--- | :--- |\n| x    | y    |\n"
        );
        assert!(
            resultat[formatage.curseur..].starts_with('x'),
            "le curseur vise x : {}",
            &resultat[formatage.curseur..]
        );
    }

    #[test]
    fn le_formatage_ne_touche_qu_au_tableau_courant() {
        let source = "| A | B |\n| --- | --- |\n| x | y |\n\n| C | D |\n| --- | --- |\n| z | w |\n";
        let a = source.find('A').unwrap();
        let formatage =
            naviguer_et_formater(source, a, SensNavigation::CelluleSuivante).expect("navigation");
        let resultat = appliquer(source, &formatage);

        assert!(resultat.contains("| C | D |\n| --- | --- |\n| z | w |\n"));
    }

    #[test]
    fn ctrl_tab_insere_une_colonne_vide_a_droite() {
        let source = "| A | B |\n| --- | --- |\n| x | y |\n";
        let a = source.find('A').unwrap();
        let formatage = inserer_colonne(source, a).expect("colonne insérée");
        let resultat = appliquer(source, &formatage);

        assert_eq!(
            resultat,
            "| A    |      | B    |\n| :--- | :--- | :--- |\n| x    |      | y    |\n"
        );
        assert!(
            resultat[formatage.curseur..].starts_with(' '),
            "le curseur est dans la nouvelle cellule : {}",
            &resultat[formatage.curseur..]
        );
    }

    #[test]
    fn ctrl_enter_insere_une_ligne_vide_en_dessous() {
        let source = "| A | B |\n| --- | --- |\n| x | y |\n";
        let x = source.find('x').unwrap();
        let formatage = inserer_ligne(source, x).expect("ligne insérée");
        let resultat = appliquer(source, &formatage);

        assert_eq!(
            resultat,
            "| A    | B    |\n| :--- | :--- |\n| x    | y    |\n|      |      |\n"
        );
        assert!(
            resultat[formatage.curseur..].starts_with(' '),
            "le curseur est dans la nouvelle ligne : {}",
            &resultat[formatage.curseur..]
        );
    }

    #[test]
    fn ctrl_suppr_supprime_la_rangee_courante() {
        let source = "| A | B |\n| --- | --- |\n| x | y |\n| z | w |\n";
        let x = source.find('x').unwrap();
        let formatage = supprimer_ligne(source, x).expect("rangée supprimée");
        let resultat = appliquer(source, &formatage);
        assert_eq!(
            resultat,
            "| A    | B    |\n| :--- | :--- |\n| z    | w    |\n"
        );
        assert!(
            resultat[formatage.curseur..].starts_with('z'),
            "le curseur vise la rangée suivante : {}",
            &resultat[formatage.curseur..]
        );

        let z = source.find('z').unwrap();
        let formatage = supprimer_ligne(source, z).expect("dernière rangée supprimée");
        let resultat = appliquer(source, &formatage);
        assert_eq!(
            resultat,
            "| A    | B    |\n| :--- | :--- |\n| x    | y    |\n"
        );
        assert!(
            resultat[formatage.curseur..].starts_with('x'),
            "le curseur vise la rangée précédente"
        );

        let seule_rangee = "| A | B |\n| --- | --- |\n| x | y |\n";
        let formatage = supprimer_ligne(seule_rangee, seule_rangee.find('x').unwrap())
            .expect("dernière rangée supprimée");
        let resultat = appliquer(seule_rangee, &formatage);
        assert_eq!(resultat, "| A    | B    |\n| :--- | :--- |\n");
        assert!(
            resultat[formatage.curseur..].starts_with('A'),
            "le curseur revient dans l'en-tête"
        );

        assert!(
            supprimer_ligne(source, source.find('A').unwrap()).is_none(),
            "l'en-tête n'est pas une rangée supprimable"
        );
    }

    #[test]
    fn maj_suppr_supprime_la_colonne_courante() {
        let source = "| A | B |\n| --- | --- |\n| x | y |\n";
        let b = source.find('B').unwrap();
        let formatage = supprimer_colonne(source, b).expect("colonne supprimée");
        let resultat = appliquer(source, &formatage);
        assert_eq!(resultat, "| A    |\n| :--- |\n| x    |\n");

        assert!(
            supprimer_colonne(source, source.find('A').unwrap()).is_some(),
            "la première colonne peut partir s'il en reste une"
        );
        let seule = "| A |\n| --- |\n| x |\n";
        assert!(
            supprimer_colonne(seule, seule.find('A').unwrap()).is_none(),
            "la dernière colonne reste"
        );
    }

    #[test]
    fn ctrl_shift_haut_et_bas_deplacent_une_rangee() {
        let source = "| A | B |\n| --- | --- |\n| x | y |\n| z | w |\n";
        let x = source.find('x').unwrap();
        let haut = deplacer_ligne(source, x, SensDeplacement::Haut);
        assert!(haut.is_none(), "x est déjà la première rangée du corps");

        let bas = deplacer_ligne(source, x, SensDeplacement::Bas).expect("rangée déplacée");
        let resultat = appliquer(source, &bas);
        assert_eq!(
            resultat,
            "| A    | B    |\n| :--- | :--- |\n| z    | w    |\n| x    | y    |\n"
        );
        assert!(
            resultat[bas.curseur..].starts_with('x'),
            "le curseur suit x"
        );
    }

    #[test]
    fn ctrl_shift_gauche_et_droite_deplacent_une_colonne() {
        let source = "| A | B |\n| --- | --- |\n| x | y |\n";
        let b = source.find('B').unwrap();
        let gauche =
            deplacer_colonne(source, b, SensDeplacement::Gauche).expect("colonne déplacée");
        let resultat = appliquer(source, &gauche);
        assert_eq!(
            resultat,
            "| B    | A    |\n| :--- | :--- |\n| y    | x    |\n"
        );
        assert!(
            resultat[gauche.curseur..].starts_with('B'),
            "le curseur suit la colonne déplacée : {}",
            &resultat[gauche.curseur..]
        );

        let droite = deplacer_colonne(source, b, SensDeplacement::Droite);
        assert!(droite.is_none(), "B est déjà la dernière colonne");
    }

    #[test]
    fn un_clic_droit_sur_l_entete_alterne_l_alignement_de_sa_colonne() {
        let source = "| A | B |\n| --- | --- |\n| x | y |\n";
        let a = source.find('A').unwrap();
        let centre = aligner_colonne(source, a).expect("alignement changé");
        let resultat = appliquer(source, &centre);
        assert_eq!(
            resultat,
            "|  A   | B    |\n| :--: | :--- |\n|  x   | y    |\n"
        );

        let droite =
            aligner_colonne(&resultat, resultat.find('A').unwrap()).expect("alignement changé");
        let resultat = appliquer(&resultat, &droite);
        assert_eq!(
            resultat,
            "|    A | B    |\n| ---: | :--- |\n|    x | y    |\n"
        );

        let gauche =
            aligner_colonne(&resultat, resultat.find('A').unwrap()).expect("alignement changé");
        let resultat = appliquer(&resultat, &gauche);
        assert_eq!(
            resultat,
            "| A    | B    |\n| :--- | :--- |\n| x    | y    |\n"
        );

        assert!(
            aligner_colonne(source, source.find('x').unwrap()).is_none(),
            "le clic hors de l'en-tête ne touche pas au tableau"
        );
    }

    #[test]
    fn au_bord_de_la_grille_la_cible_est_absente_mais_le_tableau_est_reconnu() {
        let source = "| A | B |\n| --- | --- |\n| x | y |\n";
        let y = source.find('y').unwrap();
        assert!(dans_un_tableau(source, y));
        assert_eq!(naviguer(source, y, SensNavigation::CelluleSuivante), None);
        assert_eq!(naviguer(source, y, SensNavigation::LigneSuivante), None);
    }
}
