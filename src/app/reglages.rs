//! Les réglages du document : la fenêtre de mise en page, et ce qu'elle fait.
//!
//! Le dialogue montre une **copie de travail** : le projet n'est touché qu'à
//! l'application, et fermer la fenêtre ne change rien à personne. L'import et
//! l'export parlent de fichiers TOML — la mise en page est un objet qu'on se
//! prête, pas un réglage qu'on reconstitue à la main.

use crate::app::message::Message;
use crate::app::state::{DialogueMiseEnPage, State};
use crate::mise_en_page;
use iced::Task;

/// Traite les messages de mise en page. L'appelant a déjà vérifié le motif.
pub(super) fn update(state: &mut State, message: Message) -> Option<Task<Message>> {
    // Toute édition des réglages met la norme en cours de modification —
    // l'indication du panneau la suit (bug-report-9).
    if matches!(
        message,
        Message::MiseEnPageChamp(..)
            | Message::MiseEnPageMolette(..)
            | Message::MiseEnPageBascule(_)
            | Message::MiseEnPageTypographieDefaut(_)
            | Message::EntetePiedImage(_)
            | Message::EntetePiedImageChoisie(_, _)
            | Message::CalloutPresetChoisir(_)
            | Message::CalloutPresetChamp(_, _, _)
            | Message::CalloutPresetBascule(_, _, _)
            | Message::CalloutPresetAjouter
            | Message::CalloutPresetSupprimer(_)
    ) && let Some(dialogue) = state.mise_en_page.as_mut()
    {
        dialogue.modifie = true;
    }

    match message {
        Message::MiseEnPageOuvrir => ouvrir_panneau(state, false),
        Message::PluginsOuvrir => ouvrir_panneau(state, true),

        Message::MiseEnPageFermer => {
            state.mise_en_page = None;

            // Les mains reviennent à l'écriture.
            state.editor.editor.request_focus();

            Task::none()
        }

        // Les métadonnées : une paire par ligne, écrite dès qu'elle change
        // — des données, pas des réglages : elles ne s'annulent pas, elles
        // ne se perdent jamais.
        // Les profils de métadonnées (bug-report-10, 4.1–4.3).
        Message::MetaProfilNom(nom) => {
            if let Some(dialogue) = state.mise_en_page.as_mut() {
                dialogue.profil_nom = nom;
            }

            Task::none()
        }

        Message::MetaProfilActiver(nom) => {
            if let Some(dialogue) = state.mise_en_page.as_mut() {
                dialogue.metadonnees.activer_profil(&nom);
            }

            Task::none()
        }

        Message::MetaProfilEnregistrer(nom) => {
            let nom = nom.trim();

            if nom.is_empty() {
                state.note = Some(crate::app::state::Note::erreur(
                    "Un profil a un nom.".to_string(),
                ));

                return Some(Task::none());
            }

            if let Some(dialogue) = state.mise_en_page.as_mut() {
                dialogue.metadonnees.enregistrer_profil(nom);
                state.note = Some(crate::app::state::Note::info(format!(
                    "Le profil « {nom} » porte les valeurs courantes."
                )));
            }

            Task::none()
        }

        Message::MetaProfilSupprimer(nom) => {
            if let Some(dialogue) = state.mise_en_page.as_mut() {
                dialogue.metadonnees.supprimer_profil(&nom);
            }

            Task::none()
        }

        // Le remplacement des balises par leurs valeurs (4.4) — le texte
        // du tampon courant, et rien d'autre.
        Message::MetaRemplacer => {
            let Some(dialogue) = state.mise_en_page.as_ref() else {
                return Some(Task::none());
            };

            let avant = state.editor.content();
            let rempli = crate::project::metadonnees::remplacer_les_balises(
                &avant,
                &dialogue.metadonnees,
            );

            if rempli == avant {
                state.note = Some(crate::app::state::Note::info(
                    "Aucune balise de métadonnée dans le texte.".to_string(),
                ));

                return Some(Task::none());
            }

            let tache = state
                .editor
                .replace_span(0, avant.len(), &rempli)
                .map(Message::Editor);
            state.note = Some(crate::app::state::Note::info(
                "Les balises de métadonnées sont remplacées par leurs valeurs."
                    .to_string(),
            ));

            tache
        }

        Message::MetaChamp(ancienne, nouvelle, valeur) => {
            // Les clés système sont réservées (bug-report-10, 4.5) : une
            // métadonnée ne s'appelle ni `date` ni `glo` — sa balise
            // `{clé}` se confondrait avec une balise de la langue.
            if crate::project::metadonnees::cle_reservee(&nouvelle) {
                state.note = Some(crate::app::state::Note::erreur(format!(
                    "« {nouvelle} » est un nom réservé par le langage —                     choisissez-en un autre."
                )));

                return Some(Task::none());
            }

            if let Some(dialogue) = state.mise_en_page.as_mut() {
                dialogue.metadonnees.definir(&ancienne, nouvelle, valeur);
                let _ = dialogue.metadonnees.sauver(&state.project.dir);
            }
            Task::none()
        }

        Message::MetaAjouter => {
            if let Some(dialogue) = state.mise_en_page.as_mut() {
                dialogue.metadonnees.ajouter();
                let _ = dialogue.metadonnees.sauver(&state.project.dir);
            }
            Task::none()
        }

        Message::MetaSupprimer(cle) => {
            if let Some(dialogue) = state.mise_en_page.as_mut() {
                dialogue.metadonnees.supprimer(&cle);
                let _ = dialogue.metadonnees.sauver(&state.project.dir);
            }
            Task::none()
        }

        Message::MiseEnPageChamp(champ, valeur) => {
            // Une mesure : pas de négatif, et un nombre nu se lit en `em`.
            let valeur = if crate::mise_en_page::modele::MiseEnPage::est_mesure(champ) {
                crate::mise_en_page::modele::normaliser_mesure(&valeur)?
            } else {
                valeur
            };
            if let Some(dialogue) = state.mise_en_page.as_mut() {
                let concerne_le_style = matches!(
                    champ,
                    crate::mise_en_page::modele::Champ::BibliographieStyle
                );

                dialogue.reglages.modifier(champ, valeur);
                dialogue.modifie = true;

                // Le style change : l'aperçu se refait par le moteur même.
                if concerne_le_style {
                    let dir = state.project.dir.clone();
                    let style = dialogue
                        .reglages
                        .bibliographie
                        .style
                        .clone()
                        .unwrap_or_else(|| "chicago-notes".to_string());
                    dialogue.apercu_style = crate::typst::styles::apercu_du_style(&dir, &style);
                }
            }

            Task::none()
        }

        Message::MiseEnPageMolette(champ, pas) => {
            if let Some(dialogue) = state.mise_en_page.as_mut() {
                let actuel = dialogue.reglages.valeur(champ);
                let nouveau = incrémenter(&actuel, pas);
                // Une mesure ne descend jamais sous zéro.
                let nouveau = if crate::mise_en_page::modele::MiseEnPage::est_mesure(champ) {
                    crate::mise_en_page::modele::normaliser_mesure(&nouveau)?
                } else {
                    nouveau
                };
                dialogue.reglages.modifier(champ, nouveau);
                dialogue.modifie = true;
            }

            Task::none()
        }

        Message::MiseEnPageBascule(champ) => {
            if let Some(dialogue) = state.mise_en_page.as_mut() {
                dialogue.reglages.basculer(champ);
            }

            Task::none()
        }

        // Les préréglages de callouts : le panneau montre une copie de la
        // norme, et c'est seulement « Appliquer » qui l'écrit.
        Message::CalloutPresetChoisir(index) => {
            if let Some(dialogue) = state.mise_en_page.as_mut() {
                let nombre = dialogue.reglages.parametres_plugins.callouts.presets.len();
                dialogue.callout_preset = index.min(nombre.saturating_sub(1));
            }
            Task::none()
        }

        Message::CalloutPresetChamp(index, champ, valeur) => {
            if let Some(dialogue) = state.mise_en_page.as_mut()
                && let Some(preset) = dialogue
                    .reglages
                    .parametres_plugins
                    .callouts
                    .presets
                    .get_mut(index)
            {
                preset.modifier(champ, valeur);
            }
            Task::none()
        }

        Message::CalloutPresetBascule(index, champ, actif) => {
            if let Some(dialogue) = state.mise_en_page.as_mut()
                && let Some(preset) = dialogue
                    .reglages
                    .parametres_plugins
                    .callouts
                    .presets
                    .get_mut(index)
            {
                match champ {
                    crate::langage::callout::CalloutChamp::Gras => preset.gras = actif,
                    crate::langage::callout::CalloutChamp::Rupture => preset.rupture = actif,
                    _ => {}
                }
            }
            Task::none()
        }

        Message::CalloutPresetAjouter => {
            if let Some(dialogue) = state.mise_en_page.as_mut() {
                let callouts = &mut dialogue.reglages.parametres_plugins.callouts;
                let cle = callouts.cle_libre();
                let mut preset = crate::langage::callout::preset_defaut("note");
                preset.cle = cle;
                preset.nom = "Nouveau callout".to_string();
                preset.titre = String::new();
                callouts.presets.push(preset);
                dialogue.callout_preset = callouts.presets.len().saturating_sub(1);
            }
            Task::none()
        }

        Message::CalloutPresetSupprimer(index) => {
            if let Some(dialogue) = state.mise_en_page.as_mut() {
                let callouts = &mut dialogue.reglages.parametres_plugins.callouts;
                if callouts.presets.len() > 1 && index < callouts.presets.len() {
                    callouts.presets.remove(index);
                    dialogue.callout_preset = index.min(callouts.presets.len() - 1);
                } else if callouts.presets.len() <= 1 {
                    state.note = Some(crate::app::state::Note::info(
                        "Un callout prédéfini doit rester dans la norme.".to_string(),
                    ));
                }
            }
            Task::none()
        }

        Message::MiseEnPageAccrocher(accrocher) => {
            if let Some(dialogue) = state.mise_en_page.as_mut() {
                dialogue.accrocher = accrocher;
            }

            Task::none()
        }

        Message::MiseEnPagePanneau(panneau) => {
            if let Some(dialogue) = state.mise_en_page.as_mut() {
                dialogue.panneau = panneau;
            }

            Task::none()
        }

        Message::MiseEnPageTypographieDefaut(defaut) => {
            if let Some(dialogue) = state.mise_en_page.as_mut() {
                dialogue.reglages.typographie.charger_le_defaut(&defaut);
            }

            Task::none()
        }

        Message::MiseEnPageAppliquer => appliquer(state),

        // L'ajout d'image : le sélecteur natif, les formats que Typst
        // compose (bug-report-8, T4.2-T4.3).
        Message::EntetePiedImage(champ) => Task::future(async move {
            let Some(file) = rfd::AsyncFileDialog::new()
                .add_filter("Images", &["png", "jpg", "jpeg", "gif", "svg", "webp"])
                .pick_file()
                .await
            else {
                return Message::MiseEnPageNote("Ajout d'image annulé".to_string());
            };
            Message::EntetePiedImageChoisie(champ, file.path().to_string_lossy().into_owned())
        }),

        Message::EntetePiedImageChoisie(champ, chemin) => {
            let extension = std::path::Path::new(&chemin)
                .extension()
                .map(|extension| extension.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            let autorises = ["png", "jpg", "jpeg", "gif", "svg", "webp"];
            if !autorises.contains(&extension.as_str()) {
                state.note = Some(crate::app::state::Note::info(format!(
                    "Format d'image non pris en charge : {extension}. \
                     Les formats composables : png, jpg, gif, svg, webp."
                )));
                return None;
            }
            if !std::path::Path::new(&chemin).is_file() {
                state.note = Some(crate::app::state::Note::info(
                    "Cette image est illisible.".to_string(),
                ));
                return None;
            }
            if let Some(dialogue) = state.mise_en_page.as_mut() {
                let mut valeur = dialogue.reglages.valeur(champ);
                if !valeur.trim().is_empty() {
                    valeur.push(' ');
                }
                valeur.push_str(&format!("![  ]({chemin})"));
                dialogue.reglages.modifier(champ, valeur);
                dialogue.modifie = true;
            }
            Task::none()
        }

        Message::MiseEnPageImporter => Task::future(async {
            let Some(file) = rfd::AsyncFileDialog::new()
                .add_filter("Mise en page", &["toml"])
                .pick_file()
                .await
            else {
                return Message::MiseEnPageNote("Import annulé".to_string());
            };

            match std::fs::read_to_string(file.path()) {
                Ok(texte) => Message::MiseEnPageImportee(texte),
                Err(erreur) => Message::MiseEnPageNote(format!("Lire le fichier : {erreur}")),
            }
        }),

        Message::MiseEnPageImportee(texte) => {
            match mise_en_page::modele::MiseEnPage::lire(&texte) {
                Ok(reglages) => {
                    if let Some(dialogue) = state.mise_en_page.as_mut() {
                        dialogue.reglages = reglages;
                    }

                    state.export_status = Some("Mise en page importée".to_string());
                }

                Err(erreur) => {
                    state.export_status = Some(format!("Fichier illisible : {erreur}"));
                }
            }

            Task::none()
        }

        Message::MiseEnPageExporter => {
            let texte = match state
                .mise_en_page
                .as_ref()
                .map(|dialogue| dialogue.reglages.ecrire())
            {
                Some(Ok(texte)) => texte,
                Some(Err(erreur)) => {
                    return update(
                        state,
                        Message::MiseEnPageNote(format!("Export impossible : {erreur}")),
                    );
                }
                None => return None,
            };

            Task::future(async move {
                let Some(file) = rfd::AsyncFileDialog::new()
                    .add_filter("Mise en page", &["toml"])
                    .set_file_name("mise-en-page.toml")
                    .save_file()
                    .await
                else {
                    return Message::MiseEnPageNote("Export annulé".to_string());
                };

                Message::MiseEnPageDestination(file.path().to_path_buf(), texte)
            })
        }

        Message::MiseEnPageDestination(chemin, texte) => {
            let resultat = crate::app::fichiers::verifier_destination_export(state, &chemin)
                .and_then(|()| {
                    crate::services::stockage::ecrire_atomique(&chemin, &texte)
                        .map_err(|erreur| format!("Écrire le fichier : {erreur}"))
                });
            let note = match resultat {
                Ok(()) => format!("Exporté : {}", chemin.display()),
                Err(erreur) => erreur,
            };
            return update(state, Message::MiseEnPageNote(note));
        }

        Message::MiseEnPageNote(note) => {
            state.export_status = Some(note);

            Task::none()
        }

        _ => return None,
    }
    .into()
}

/// Ouvre le panneau de travail — les normes, ou les paramètres des plugins.
fn ouvrir_panneau(state: &mut State, plugins: bool) -> Task<Message> {
    let reglages = mise_en_page::charger(&state.project.dir);
    let accroche_manquante = mise_en_page::accroche_manquante(&state.project.dir);
    let apercu_style = crate::typst::styles::apercu_du_style(
        &state.project.dir,
        reglages
            .bibliographie
            .style
            .as_deref()
            .unwrap_or("chicago-notes"),
    );
    let styles_zotero = crate::zotero::data_dir()
        .map(|dir| crate::zotero::styles::installed(&dir.join("styles")))
        .unwrap_or_default();
    let norme_chargee = crate::project::norme_qui_correspond(&reglages).unwrap_or_default();

    state.mise_en_page = Some(DialogueMiseEnPage {
        metadonnees: crate::project::metadonnees::Metadonnees::charger(&state.project.dir),
        norme: norme_chargee.clone(),
        profil_nom: String::new(),
        norme_originale: norme_chargee,
        modifie: false,
        reglages,
        plugins,
        panneau: 0,
        callout_preset: 0,
        accrocher: accroche_manquante,
        accroche_manquante,
        regles_main: mise_en_page::regles_ecrites_a_la_main(&state.project.dir),
        apercu_style,
        styles_zotero,
    });

    state.editor.editor.lose_focus();
    Task::none()
}

/// Applique la copie de travail au projet.
///
/// Ce qui ne va pas n'est **jamais** écrit : les réglages fautifs se disent,
/// champ par champ, et la fenêtre reste ouverte pour les corriger.
fn appliquer(state: &mut State) -> Task<Message> {
    let Some(dialogue) = state.mise_en_page.as_ref() else {
        return Task::none();
    };

    let problemes = dialogue.reglages.problemes();

    if !problemes.is_empty() {
        state.export_status = Some(format!(
            "{} réglage(s) à corriger dans la fenêtre",
            problemes.len()
        ));

        return Task::none();
    }

    let reglages = dialogue.reglages.clone();
    let accrocher = dialogue.accrocher;
    // Les normes appliquées sont celles que l'interface lit pour
    // montrer ou cacher les fonctionnalités du projet, et celles qui
    // configurent les plugins à l'ouverture.
    state.reglages = reglages.clone();
    state.configurer_les_plugins_selon_les_normes();
    state.render.pending = true;
    state.rafraichir_statistiques_bibliques();

    match mise_en_page::appliquer(&state.project.dir, &reglages, accrocher) {
        Ok(mise_en_page::Applique::AccrocheAjoutee) => {
            state.mise_en_page = None;
            state.editor.editor.request_focus();
            state.export_status =
                Some("Mise en page appliquée — main.typ l'accueille maintenant".to_string());
        }

        Ok(mise_en_page::Applique::RienAjoute) => {
            state.mise_en_page = None;
            state.editor.editor.request_focus();
            state.export_status = Some("Mise en page appliquée".to_string());
        }

        Err(erreur) => {
            state.export_status = Some(erreur);
        }
    }

    Task::none()
}

/// Incrémente la valeur numérique d'un champ — la molette et les flèches.
///
/// `11pt` → `12pt`, `2cm` → `3cm`, `1.5` → `2.5`. L'unité est préservée.
fn incrémenter(valeur: &str, pas: i8) -> String {
    // Séparer le nombre et le suffixe (unité).
    let fin_nombre = valeur
        .find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-' && c != ',')
        .unwrap_or(valeur.len());
    let (nombre, suffixe) = valeur.split_at(fin_nombre);
    let nombre_normalise = nombre.replace(',', ".");
    let Ok(n) = nombre_normalise.parse::<f64>() else {
        return valeur.to_string();
    };
    let nouveau = n + pas as f64;
    // Pas de décimale inutile : 2.0 → 2, 2.5 → 2.5.
    if nouveau.fract() == 0.0 {
        format!("{}{suffixe}", nouveau as i64)
    } else {
        format!("{nouveau}{suffixe}")
    }
}
