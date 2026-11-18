//! L'état de vue du plugin biblique : ce qui appartient à un onglet.
//!
//! Le texte, lui, appartient au document ; ici vivent le panneau de
//! statistiques, la référence cliquée, le survol et la portée choisie.

use super::domaine::{
    Analyse, ElementCitation, FormeLivre, LivreChoix, LivreConnu, Reference, Statistiques,
    Traduction,
};
use iced::widget::combo_box;

/// L'ordre des références dans le panneau.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TriReferences {
    /// Dans l'ordre où elles apparaissent dans le document.
    #[default]
    Apparition,
    /// Dans l'ordre canonique des livres, puis chapitre et verset.
    Canonique,
}

/// La construction d'une citation biblique, dans son panneau.
#[derive(Debug, Clone, Default)]
pub struct BiblePanneau {
    /// Le filtre de la liste des livres.
    pub recherche: String,
    /// La liste déroulante cherchable : un seul champ, et ses options.
    pub choix: combo_box::State<LivreChoix>,
    /// Le livre choisi, par son rang dans la liste.
    pub livre: Option<usize>,
    /// Le chapitre, en chiffres.
    pub chapitre: String,
    /// Les versets : `16`, `16-18`, `16, 17`.
    pub versets: String,
    /// La saisie a-t-elle changé depuis le dernier **Ajouter** ? C'est elle
    /// que l'insertion prend sans clic quand elle est encore en attente.
    pub saisie_modifiee: bool,
    /// Les références déjà ajoutées à la citation.
    pub references: Vec<Reference>,
    /// Un texte libre, ajouté entre guillemets à la fin de la citation.
    pub texte_libre: String,
    /// La forme choisie pour cette citation ; `None` hérite du document.
    pub forme: Option<FormeLivre>,
}

impl BiblePanneau {
    /// Le panneau neuf, avec sa liste déroulante déjà remplie.
    ///
    /// La liste est construite une fois à l'ouverture : le widget filtre
    /// ensuite ses propres options au fil de la frappe.
    pub fn avec_livres(livres: &[LivreConnu]) -> Self {
        let options = livres
            .iter()
            .enumerate()
            .map(|(rang, livre)| LivreChoix::depuis(rang, livre, None))
            .collect();
        Self {
            choix: combo_box::State::new(options),
            ..Self::default()
        }
    }

    /// Reconstruit les libellés de la liste quand la forme d'affichage change.
    ///
    /// La sélection reste la même ; seule la façon de nommer les livres suit
    /// la forme (la TOB montre ses abréviations).
    pub fn renouveler_choix(&mut self, livres: &[LivreConnu], forme: Option<FormeLivre>) {
        let options: Vec<LivreChoix> = livres
            .iter()
            .enumerate()
            .map(|(rang, livre)| LivreChoix::depuis(rang, livre, forme))
            .collect();
        let selection = self.livre.and_then(|index| {
            livres
                .get(index)
                .map(|livre| LivreChoix::depuis(index, livre, forme))
        });
        self.choix = combo_box::State::with_selection(options, selection.as_ref());
    }
}

/// La référence cliquée dans l'éditeur : sa place et ce qu'on en fait.
#[derive(Debug, Clone)]
pub struct ReferenceCliquee {
    /// L'intervalle de la forme `{{…}}` dans le document, en octets.
    pub intervalle: (usize, usize),
    /// Les parties de la balise, texte libre compris.
    pub elements: Vec<ElementCitation>,
    /// Les références seules, pour la consultation et la traduction.
    pub references: Vec<Reference>,
    /// La forme imposée par le suffixe, s'il y en a un.
    pub forme: Option<FormeLivre>,
    /// Son extrait biblique (le texte affiché).
    pub extrait: String,
}

/// L'édition d'une référence depuis le panneau de gauche.
#[derive(Debug, Clone, Default)]
pub struct ReferenceEdition {
    /// L'index dans `ReferenceCliquee::elements`.
    pub index: usize,
    /// Le livre choisi, par son rang dans la liste.
    pub livre: Option<usize>,
    /// Le champ chapitre : `3` ou `1-11`.
    pub chapitre: String,
    /// Le champ versets : `16`, `16-18`, vide pour le chapitre entier.
    pub versets: String,
    /// La traduction explicite de la référence d'origine, à conserver.
    pub traduction: Option<Traduction>,
}

/// Ce qu'une vue retient du plugin biblique.
#[derive(Debug, Clone, Default)]
pub struct BibleEtat {
    /// Le repli de la section statistiques.
    pub repliees: bool,
    /// La référence cliquée dans l'éditeur, et sa bulle.
    pub reference_cliquee: Option<ReferenceCliquee>,
    /// L'édition en cours d'une référence cliquée.
    pub edition: Option<ReferenceEdition>,
    /// Ce que le survol vise : la position du point et **la** référence
    /// biblique sous lui, choisie par la position du curseur dans la balise.
    pub survol: Option<(iced::Point, Reference)>,
    /// Le survol encore jeune : il devient une bulle après un court repos.
    pub survol_candidat: Option<(iced::Point, Reference, std::time::Instant)>,
    /// Les statistiques des livres cités du document (le panneau).
    pub stats: Statistiques,
    /// Le filtre de recherche du panneau des statistiques.
    pub recherche: String,
    /// La recherche est-elle ouverte ?
    pub recherche_active: bool,
    /// L'ordre choisi pour les références.
    pub tri: TriReferences,
    /// La traduction choisie pour la consultation.
    pub traduction: Traduction,
    /// Le panneau d'insertion, quand il est ouvert.
    pub panneau: Option<BiblePanneau>,
    /// L'analyse intelligente de ces statistiques.
    pub analyse: Analyse,
}
