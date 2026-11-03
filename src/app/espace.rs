//! Documents partagés et vues indépendantes de l'espace de travail.
//!
//! Un document possède le texte et l'unique historique. Les éditeurs des vues
//! sont des projections synchronisées à la fin de chaque transaction ; leur
//! curseur, sélection, défilement et cache graphique restent locaux.

use super::state::{LspState, ModeVue, RenderState};
use iced::widget::pane_grid::{self, Axis, Configuration, Edge, Node, Pane};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

pub type VueId = u64;

/// Comment un document rejoint l'espace de travail.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Ouverture {
    /// Réutiliser l'onglet existant si le document est déjà ouvert, sinon
    /// nouvel onglet — l'ouverture d'un projet.
    /// Réutiliser si déjà ouvert, sinon remplacer le contenu de l'onglet
    /// actif — le clic simple sur un document.
    Remplacer,
    /// Toujours un nouvel onglet — shift+clic ou duplication.
    Nouvel,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Panneau {
    pub onglets: Vec<VueId>,
    pub actif: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Arbre {
    Panneau(Panneau),
    Scission {
        verticale: bool,
        ratio: f32,
        a: Box<Arbre>,
        b: Box<Arbre>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VueSauvee {
    pub id: VueId,
    pub projet: PathBuf,
    pub document: PathBuf,
    pub mode: ModeVue,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Disposition {
    pub arbre: Arbre,
    pub focus: VueId,
    #[serde(default)]
    pub vues: Vec<VueSauvee>,
}

/// Le contexte propre à une vue ; l'active est prêtée aux anciens domaines
/// de l'application pour conserver une migration incrémentale sûre.
pub struct Vue {
    pub editor: crate::editor::EditorState,
    pub render: RenderState,
    pub lsp: LspState,
    pub project: crate::project::Project,
    pub bibliography: Vec<crate::project::BibliographyEntry>,
    pub zotero: crate::zotero::Etat,
    pub mode: ModeVue,
    pub typst: String,
}

impl Vue {
    pub fn nouvelle(
        projet: crate::project::Project,
        chemin: PathBuf,
        contenu: String,
        mode: ModeVue,
        resolved: &crate::ui::theme::Resolved,
    ) -> Self {
        let mut editor = crate::editor::EditorState::new_file(chemin, contenu);
        editor.retheme(resolved);
        let bibliography = projet
            .bibliography
            .as_deref()
            .map(crate::project::parse_bibliography)
            .unwrap_or_default();
        let zotero = crate::zotero::Etat {
            lie: projet.zotero.is_some(),
            source: if projet.zotero.is_some() {
                crate::zotero::Source::Zotero
            } else {
                crate::zotero::Source::Locale
            },
        };
        Self {
            editor,
            render: RenderState::default(),
            lsp: LspState::default(),
            project: projet,
            bibliography,
            zotero,
            mode,
            typst: String::new(),
        }
    }

    pub fn echanger(&mut self, state: &mut super::State) {
        std::mem::swap(&mut self.editor, &mut state.editor);
        std::mem::swap(&mut self.render, &mut state.render);
        std::mem::swap(&mut self.lsp, &mut state.lsp);
        std::mem::swap(&mut self.project, &mut state.project);
        std::mem::swap(&mut self.bibliography, &mut state.bibliography);
        std::mem::swap(&mut self.zotero, &mut state.zotero);
        std::mem::swap(&mut self.mode, &mut state.mode);
        std::mem::swap(&mut self.typst, &mut state.typst_genere);
    }
}

pub struct Document {
    pub source: iced_code_editor::SourcePartagee,
}

pub struct Espace {
    pub grille: pane_grid::State<Panneau>,
    pub focus: Pane,
    pub active: VueId,
    pub prochain: VueId,
    pub vues: BTreeMap<VueId, Vue>,
    pub documents: BTreeMap<(PathBuf, PathBuf), Document>,
    /// Ce que chaque document avait sur disque la dernière fois que nous l'y
    /// avons lu ou écrit. Comparée au contenu présent, cette empreinte
    /// distingue notre propre sauvegarde d'un changement extérieur.
    pub disques: BTreeMap<(PathBuf, PathBuf), u64>,
    pub compilations: std::collections::BTreeSet<(PathBuf, PathBuf)>,
    pub verrous: BTreeMap<PathBuf, crate::project::Verrou>,
    pub glisse: Option<VueId>,
    pub origine_glisse: iced::Point,
    pub cible: Option<(Pane, Option<Edge>)>,
    pub initialise: bool,
    pub transaction: bool,
}

impl Default for Espace {
    fn default() -> Self {
        let (grille, focus) = pane_grid::State::new(Panneau::default());
        Self {
            grille,
            focus,
            active: 1,
            prochain: 2,
            vues: BTreeMap::new(),
            documents: BTreeMap::new(),
            disques: BTreeMap::new(),
            compilations: Default::default(),
            verrous: BTreeMap::new(),
            glisse: None,
            origine_glisse: iced::Point::ORIGIN,
            cible: None,
            initialise: false,
            transaction: false,
        }
    }
}

impl Espace {
    pub fn deplacer(&mut self, vue: VueId, cible: Pane, bord: Option<Edge>) {
        if self.grille.get(cible).is_none() {
            return;
        }
        let origine = self
            .grille
            .iter()
            .find_map(|(p, c)| c.onglets.contains(&vue).then_some(*p));
        let Some(origine) = origine else {
            return;
        };
        if origine == cible && bord.is_none() {
            return;
        }
        let destination = if let Some(bord) = bord {
            let axe = match bord {
                Edge::Left | Edge::Right => Axis::Vertical,
                _ => Axis::Horizontal,
            };
            let Some((nouveau, _)) = self.grille.split(axe, cible, Panneau::default()) else {
                return;
            };
            if matches!(bord, Edge::Left | Edge::Top) {
                self.grille.swap(cible, nouveau);
            }
            nouveau
        } else {
            cible
        };
        if let Some(p) = self.grille.get_mut(origine) {
            p.onglets.retain(|id| *id != vue);
            p.actif = p.actif.min(p.onglets.len().saturating_sub(1));
        }
        let p = self.grille.get_mut(destination).expect("panneau créé");
        p.onglets.push(vue);
        p.actif = p.onglets.len() - 1;
        self.focus = destination;
        if self
            .grille
            .get(origine)
            .is_some_and(|p| p.onglets.is_empty())
            && self.grille.len() > 1
        {
            self.grille.close(origine);
        }
    }

    pub fn disposition(&self) -> Disposition {
        fn convertir(n: &Node, grille: &pane_grid::State<Panneau>) -> Arbre {
            match n {
                Node::Pane(p) => Arbre::Panneau(grille.get(*p).expect("panneau existant").clone()),
                Node::Split {
                    axis, ratio, a, b, ..
                } => Arbre::Scission {
                    verticale: *axis == Axis::Vertical,
                    ratio: *ratio,
                    a: Box::new(convertir(a, grille)),
                    b: Box::new(convertir(b, grille)),
                },
            }
        }
        let focus = self
            .grille
            .get(self.focus)
            .and_then(|p| p.onglets.get(p.actif))
            .copied()
            .unwrap_or(0);
        Disposition {
            arbre: convertir(self.grille.layout(), &self.grille),
            focus,
            vues: Vec::new(),
        }
    }

    pub fn depuis_disposition(disposition: Disposition) -> Self {
        fn convertir(a: Arbre) -> Configuration<Panneau> {
            match a {
                Arbre::Panneau(mut p) => {
                    p.actif = p.actif.min(p.onglets.len().saturating_sub(1));
                    Configuration::Pane(p)
                }
                Arbre::Scission {
                    verticale,
                    ratio,
                    a,
                    b,
                } => Configuration::Split {
                    axis: if verticale {
                        Axis::Vertical
                    } else {
                        Axis::Horizontal
                    },
                    ratio: if ratio.is_finite() {
                        ratio.clamp(0.1, 0.9)
                    } else {
                        0.5
                    },
                    a: Box::new(convertir(*a)),
                    b: Box::new(convertir(*b)),
                },
            }
        }
        let grille = pane_grid::State::with_configuration(convertir(disposition.arbre));
        let focus = *grille
            .iter()
            .find(|(_, p)| p.onglets.contains(&disposition.focus))
            .or_else(|| grille.iter().next())
            .expect("au moins un panneau")
            .0;
        Self {
            grille,
            focus,
            ..Self::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deplacer_un_onglet_ne_deplace_pas_les_autres() {
        let mut espace = Espace::default();
        let premier = espace.focus;
        espace.grille.get_mut(premier).unwrap().onglets = vec![1, 2, 3];
        let (second, _) = espace
            .grille
            .split(
                iced::widget::pane_grid::Axis::Vertical,
                premier,
                Panneau::default(),
            )
            .unwrap();
        espace.deplacer(2, second, None);
        assert_eq!(espace.grille.get(premier).unwrap().onglets, vec![1, 3]);
        assert_eq!(espace.grille.get(second).unwrap().onglets, vec![2]);
    }

    #[test]
    fn scinder_sur_le_bord_gauche_conserve_tous_les_onglets() {
        let mut espace = Espace::default();
        let premier = espace.focus;
        espace.grille.get_mut(premier).unwrap().onglets = vec![1, 2];
        espace.deplacer(2, premier, Some(iced::widget::pane_grid::Edge::Left));
        assert_eq!(espace.grille.len(), 2);
        assert_eq!(
            espace
                .grille
                .iter()
                .map(|(_, p)| p.onglets.len())
                .sum::<usize>(),
            2
        );
        assert_ne!(espace.focus, premier);
    }

    #[test]
    fn la_session_restitue_les_scissions_les_modes_et_le_focus() {
        let mut espace = Espace::default();
        let premier = espace.focus;
        espace.grille.get_mut(premier).unwrap().onglets = vec![1, 2];
        espace.deplacer(2, premier, Some(iced::widget::pane_grid::Edge::Bottom));
        let sauvegarde = espace.disposition();
        let texte = serde_json::to_string(&sauvegarde).unwrap();
        let relue: Disposition = serde_json::from_str(&texte).unwrap();
        let restauree = Espace::depuis_disposition(relue);
        assert_eq!(restauree.disposition(), sauvegarde);
    }
}
