//! Le panneau d'un graphique primaviz : ses champs, son CSV, son insertion.
//!
//! Le panneau est une fenêtre à part : il ne s'ouvre que si l'add-on
//! « graphiques » est actif (la porte de [`Message::plugin`]) et vérifie les
//! données avant de poser la balise `{chart}` dans le document.

use crate::app::{Message, State};
use crate::langage::chart;
use iced::Task;

/// Traite les messages de ce domaine. La porte d'activation et la garde du
/// verrou sont appliquées par `update_interne` avant l'appel.
pub fn traiter(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::ChartOuvrir => {
            if state.chart.is_none() {
                state.chart = Some(crate::app::state::ChartDialogue::default());
            }
            iced::widget::operation::focus(crate::app::state::ChartDialogue::id_csv())
        }

        Message::ChartFermer => {
            state.chart = None;
            Task::none()
        }

        Message::ChartType(type_) => {
            if let Some(dialogue) = state.chart.as_mut() {
                dialogue.type_ = chart::normaliser_type(&type_).to_string();
            }
            Task::none()
        }

        Message::ChartTitre(titre) => {
            if let Some(dialogue) = state.chart.as_mut() {
                dialogue.titre = titre;
            }
            Task::none()
        }

        Message::ChartSousTitre(sous_titre) => {
            if let Some(dialogue) = state.chart.as_mut() {
                dialogue.sous_titre = sous_titre;
            }
            Task::none()
        }

        Message::ChartXLabel(label) => {
            if let Some(dialogue) = state.chart.as_mut() {
                dialogue.x_label = label;
            }
            Task::none()
        }

        Message::ChartYLabel(label) => {
            if let Some(dialogue) = state.chart.as_mut() {
                dialogue.y_label = label;
            }
            Task::none()
        }

        Message::ChartGrille(actif) => {
            if let Some(dialogue) = state.chart.as_mut() {
                dialogue.grille = actif;
            }
            Task::none()
        }

        Message::ChartGraduations(actif) => {
            if let Some(dialogue) = state.chart.as_mut() {
                dialogue.graduations = actif;
            }
            Task::none()
        }

        Message::ChartPoints(actif) => {
            if let Some(dialogue) = state.chart.as_mut() {
                dialogue.points = actif;
            }
            Task::none()
        }

        Message::ChartAire(actif) => {
            if let Some(dialogue) = state.chart.as_mut() {
                dialogue.aire = actif;
            }
            Task::none()
        }

        Message::ChartCourbe(courbe) => {
            if let Some(dialogue) = state.chart.as_mut() {
                dialogue.courbe = courbe;
            }
            Task::none()
        }

        Message::ChartDonut(actif) => {
            if let Some(dialogue) = state.chart.as_mut() {
                dialogue.donut = actif;
            }
            Task::none()
        }

        Message::ChartAnneau(anneau) => {
            if let Some(dialogue) = state.chart.as_mut() {
                dialogue.anneau = anneau;
            }
            Task::none()
        }

        Message::ChartHorizontal(actif) => {
            if let Some(dialogue) = state.chart.as_mut() {
                dialogue.horizontal = actif;
            }
            Task::none()
        }

        Message::ChartCsv(action) => {
            if let Some(dialogue) = state.chart.as_mut() {
                dialogue.csv.perform(action);
            }
            Task::none()
        }

        Message::ChartImporter => Task::perform(
            async {
                rfd::AsyncFileDialog::new()
                    .add_filter("CSV ou texte", &["csv", "txt"])
                    .pick_file()
                    .await
                    .and_then(|fichier| std::fs::read_to_string(fichier.path()).ok())
            },
            Message::ChartImporte,
        ),

        Message::ChartImporte(Some(texte)) => {
            if let Some(dialogue) = state.chart.as_mut() {
                dialogue.csv = iced::widget::text_editor::Content::with_text(&texte);
            }
            Task::none()
        }

        Message::ChartImporte(None) => Task::none(),

        Message::ChartLargeur(largeur) => {
            if let Some(dialogue) = state.chart.as_mut() {
                dialogue.largeur = largeur;
            }
            Task::none()
        }

        Message::ChartHauteur(hauteur) => {
            if let Some(dialogue) = state.chart.as_mut() {
                dialogue.hauteur = hauteur;
            }
            Task::none()
        }

        Message::ChartPlacement(placement) => {
            if let Some(dialogue) = state.chart.as_mut() {
                dialogue.placement = placement;
            }
            Task::none()
        }

        Message::ChartLegende(legende) => {
            if let Some(dialogue) = state.chart.as_mut() {
                dialogue.legende = legende;
            }
            Task::none()
        }

        Message::ChartValeurs(valeurs) => {
            if let Some(dialogue) = state.chart.as_mut() {
                dialogue.valeurs = valeurs;
            }
            Task::none()
        }

        Message::ChartInserer => {
            let Some(dialogue) = state.chart.take() else {
                return Task::none();
            };
            let graphe = dialogue.vers_schema();
            if let Err(erreur) = chart::valider(&graphe) {
                state.note = Some(crate::app::state::Note::erreur(format!(
                    "Le graphique n'a pas été inséré : {erreur}"
                )));
                state.chart = Some(dialogue);
                return Task::none();
            }

            let source = chart::source_canonique(&graphe);
            state.editor.insert_at_cursor(&source).map(Message::Editor)
        }

        _ => Task::none(),
    }
}
