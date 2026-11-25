//! La saisie : la frappe, les listes, les formats, le composeur de
//! notes et la liste de complétion.
//!
//! Ses bras vivaient dans le conducteur (feuille de route, phase 1,
//! cartographie, geste 3) ; ils sont réunis par famille.

use super::Message;
use super::State;
use iced::Task;

/// Déplace la ligne du curseur d'un niveau de liste, sans passer par le
/// tampon de l'éditeur. `Tab` et `Maj+Tab` arrivent ici par deux chemins :
/// les messages historiques (`ListeIndenter` / `ListeDesindenter`) et les
/// messages que l'éditeur émet lui-même quand il a le focus.
fn decaler_la_liste(state: &mut State, indenter: bool) -> Task<Message> {
    let contenu = state.editor.content();
    let offset = state.editor.cursor_offset();
    let (debut, _, ligne) = super::ligne_du_curseur(&contenu, offset);

    // On ne remplace que le début de la ligne, jamais le document entier :
    // sélectionner tout le texte pour y insérer deux espaces faisait défiler
    // l'éditeur — et, par le suivi de l'aperçu, la page — à chaque Tab.
    let (tache, curseur) = if indenter {
        (state.editor.replace_span(debut, debut, "  "), offset + 2)
    } else {
        let retrait = (ligne.len() - ligne.trim_start().len()).min(2);

        if retrait == 0 {
            (Task::none(), offset)
        } else {
            (
                state.editor.replace_span(debut, debut + retrait, ""),
                offset.saturating_sub(retrait),
            )
        }
    };

    Task::batch([tache, state.editor.set_cursor_offset(curseur)]).map(Message::Editor)
}

/// L'éditeur a-t-il capté une tabulation qui appartient à une liste ?
///
/// Le widget de l'éditeur transforme `Tab` en `Message::Tab` et
/// `Maj+Tab` en `Message::FocusNavigationShiftTab` **avant** que les
/// raccourcis de l'application ne voient l'événement. Sans cette porte, la
/// tabulation insérait ses espaces et la liste ne changeait jamais de
/// niveau — le geste existait dans le plan de touches, mais jamais atteint.
fn tabulation_sur_liste(
    state: &mut State,
    message: &iced_code_editor::Message,
) -> Option<Task<Message>> {
    let indenter = match message {
        iced_code_editor::Message::Tab => true,
        iced_code_editor::Message::FocusNavigationShiftTab => false,
        _ => return None,
    };

    if super::pointeur_bloque(state) {
        return None;
    }

    let contenu = state.editor.content();
    let offset = state.editor.cursor_offset();
    if !super::est_sur_une_liste(&contenu, offset) {
        return None;
    }

    Some(decaler_la_liste(state, indenter))
}

/// Navigue dans un tableau et l'aligne avant de rendre la main.
///
/// `Tab` / `Maj+Tab` passent d'une cellule à l'autre ; `Entrée` /
/// `Maj+Entrée` passent à la même colonne de la ligne suivante ou précédente.
/// `Ctrl+Suppr` et `Maj+Suppr` retirent la ligne ou la colonne courante.
/// Une fois dans la grille, la touche ne doit jamais insérer ses espaces ou un
/// saut de ligne dans une cellule : au bord de la grille, elle ne fait rien.
fn navigation_tableau_message(
    state: &mut State,
    message: &iced_code_editor::Message,
) -> Option<Task<Message>> {
    use crate::langage::tableau::SensNavigation;

    if !state.editor.is_intermediate() || state.composeur.is_some() || super::pointeur_bloque(state)
    {
        return None;
    }

    let contenu = state.editor.content();
    let offset = state.editor.cursor_offset();
    if !crate::langage::tableau::dans_un_tableau(&contenu, offset) {
        return None;
    }

    // `Ctrl+Suppr` supprime la rangée courante ; `Maj+Suppr` la colonne.
    // Le widget transforme déjà ces gestes en `DeleteWord` et
    // `DeleteSelection` : on les intercepte avant qu'il ne touche au texte.
    if state.controle_tenu && matches!(message, iced_code_editor::Message::DeleteWord) {
        return Some(crate::app::routeur::tableaux::traiter(
            state,
            Message::TableauSupprimerLigne,
        ));
    }
    if state.maj_tenu && matches!(message, iced_code_editor::Message::DeleteSelection) {
        return Some(crate::app::routeur::tableaux::traiter(
            state,
            Message::TableauSupprimerColonne,
        ));
    }

    // `Ctrl+Maj+flèches` déplace la rangée ou la colonne. Le widget émet
    // `ArrowKey` pour ces flèches : le message remplace donc la touche avant
    // que l'éditeur ne la traite comme un simple déplacement du curseur.
    if state.controle_tenu
        && let iced_code_editor::Message::ArrowKey(direction, true) = message
    {
        use crate::langage::tableau::SensDeplacement;
        use iced_code_editor::ArrowDirection;
        let sens = match direction {
            ArrowDirection::Up => SensDeplacement::Haut,
            ArrowDirection::Down => SensDeplacement::Bas,
            ArrowDirection::Left => SensDeplacement::Gauche,
            ArrowDirection::Right => SensDeplacement::Droite,
        };
        return Some(crate::app::routeur::tableaux::traiter(
            state,
            Message::TableauDeplacer(sens),
        ));
    }

    // `Ctrl+Entrée` ajoute une ligne sous la ligne courante. Le widget
    // transforme `Entrée` en `Message::Enter` sans transporter les
    // modificateurs ; l'application suit `Ctrl` par `ModifiersChanged`.
    if matches!(message, iced_code_editor::Message::Enter) && state.controle_tenu {
        // Le remplacement est synchrone dans le routeur ; le rendre au
        // éditeur comme une tâche différée ferait attendre un tour de boucle
        // pour une ligne qui doit apparaître sous les doigts.
        return Some(crate::app::routeur::tableaux::traiter(
            state,
            Message::TableauInsererLigne,
        ));
    }

    let sens = match message {
        iced_code_editor::Message::Tab => SensNavigation::CelluleSuivante,
        iced_code_editor::Message::FocusNavigationShiftTab => SensNavigation::CellulePrecedente,
        iced_code_editor::Message::Enter => {
            if state.maj_tenu {
                SensNavigation::LignePrecedente
            } else {
                SensNavigation::LigneSuivante
            }
        }
        _ => return None,
    };

    let Some(formatage) = crate::langage::tableau::naviguer_et_formater(&contenu, offset, sens)
    else {
        // Au dernier rang ou à la dernière colonne : on garde la touche pour
        // elle-même, plutôt que de laisser le widget écrire un espace ou un
        // saut de ligne qui casserait la grille.
        return Some(Task::none());
    };

    let change = contenu[formatage.plage.clone()] != formatage.texte;
    if change {
        state.rafraichir_statistiques_bibliques();
        state.mots_a_recalculer();
    }
    let remplacement = if change {
        state
            .editor
            .replace_span(formatage.plage.start, formatage.plage.end, &formatage.texte)
            .map(Message::Editor)
    } else {
        Task::none()
    };

    Some(Task::batch([
        remplacement,
        state
            .editor
            .set_cursor_offset(formatage.curseur)
            .map(Message::Editor),
    ]))
}

/// Applique une mise en forme à la sélection, en tenant la couleur courante
/// et les compteurs qui dépendent du texte.
fn appliquer_le_format(
    state: &mut State,
    action: crate::app::message::FormatAction,
) -> Task<Message> {
    if !state.editor.is_intermediate() {
        return Task::none();
    }

    // TOPBAR-04-07 : toute couleur choisie devient la couleur courante.
    if let crate::app::message::FormatAction::Couleur(ref nom) = action {
        state.derniere_couleur = Some(nom.clone());
    }

    // Les statistiques des livres cités suivent le document (le panneau,
    // l add-on biblique).
    state.rafraichir_statistiques_bibliques();
    state.mots_a_recalculer();
    state.editor.apply_format(action).map(Message::Editor)
}

/// Ouvre, au point du clic droit, ce que la source désigne.
///
/// Une en-tête de colonne fait tourner son alignement ; une balise ouvre son
/// panneau d'insertion, rempli de sa forme actuelle et prêt à la remplacer.
/// Le reste du texte ouvre les actions ordinaires (couper, copier, coller),
/// sans écrire dans le document.
fn menu_contextuel(state: &mut State, point: iced::Point) -> Task<Message> {
    let Some(offset) = state.editor.offset_at_point(point) else {
        return Task::none();
    };
    let contenu = state.editor.content();

    if let Some(formatage) = crate::langage::tableau::aligner_colonne(&contenu, offset) {
        return crate::app::routeur::tableaux::appliquer_formatage(state, formatage);
    }

    if let Some(balise) = crate::langage::balise::balise_au(&contenu, offset) {
        // La référence biblique a son panneau riche : il doit être essayé
        // avant le formulaire de source générique.
        if matches!(balise.nom.as_str(), "b" | "bible" | "citation" | "ref")
            && let Some((debut, fin, references)) =
                crate::plugins::bible::reference_au(&contenu, offset)
        {
            crate::app::routeur::bible::ouvrir_le_panneau_sur(state, debut..fin, references);
            state.menu_contextuel = None;
            return Task::none();
        }

        // La personne a un panneau avec la liste de son registre : le clic
        // droit doit y entrer avec la clé et l'expression déjà remplies.
        if matches!(balise.nom.as_str(), "p" | "personne" | "personnes") {
            let reste = balise
                .interieur
                .split_once(|c: char| c.is_whitespace() || c == '|' || c == '=')
                .map(|(_, reste)| reste.trim())
                .unwrap_or("");
            let (cle, expression) = reste.split_once('|').unwrap_or((reste, ""));
            state.menu_contextuel = None;
            return crate::app::routeur::personnes::ouvrir_panneau_sur(
                state,
                balise.plage.clone(),
                cle.trim(),
                expression.trim(),
            );
        }

        if let Some(dialogue) = crate::app::palette::BaliseDialogue::depuis_balise(&balise) {
            state.balise_dialogue = Some(dialogue);
            state.menu_contextuel = None;
            state.editor.editor.lose_focus();
            return iced::widget::operation::focus(crate::app::palette::BaliseDialogue::id_champ(
                0,
            ));
        }
    }

    state.menu_contextuel = Some(crate::app::state::MenuContextuel { position: point });
    Task::none()
}

pub(super) fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::BlocDescendre => state.editor.deplacer_le_bloc(false).map(Message::Editor),

        Message::BlocMonter => state.editor.deplacer_le_bloc(true).map(Message::Editor),
        Message::ListeMonter | Message::ListeDescendre => {
            let haut = matches!(message, Message::ListeMonter);
            let contenu = state.editor.content();
            let offset = state.editor.cursor_offset();
            let (neuf, curseur) = super::deplacer_la_ligne(&contenu, offset, haut);

            Task::batch([
                state.editor.replace_span(0, contenu.len(), &neuf),
                state.editor.set_cursor_offset(curseur),
            ])
            .map(Message::Editor)
        }
        Message::ListeDesindenter => decaler_la_liste(state, false),
        Message::ListeIndenter => decaler_la_liste(state, true),
        Message::ComposeurSaisie(_)
        | Message::ComposeurAvancer
        | Message::ComposeurTerminer
        | Message::ComposeurFermer
        | Message::ComposeurDeplacer(_)
        | Message::ComposeurPrendre(_)
        | Message::ComposeurChoisir(_) => crate::app::composeur::composeur_update(state, message),

        Message::CompletionDismiss => {
            state.completion = None;
            Task::none()
        }

        Message::CompletionPick(index) => {
            if let Some(open) = state.completion.as_mut()
                && index < open.shown.len()
            {
                open.selected = index;
            }

            Task::none()
        }
        Message::CompletionChercher(t) => {
            // La recherche : la liste se filtre à la frappe.
            if let Some(open) = state.completion.as_mut() {
                open.typed = t;
                let cherche = open.typed.to_lowercase();
                open.shown = open
                    .items
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| e.filter().to_lowercase().contains(&cherche))
                    .map(|(i, _)| i)
                    .collect();
                open.selected = 0;
            }
            Task::none()
        }

        Message::CompletionAccept => {
            // L'insertion de l'élément choisi : `insert` porte la forme à
            // écrire, le libellé n'est que l'affichage de la liste.
            let insertion = {
                let Some(open) = state.completion.as_ref() else {
                    return Task::none();
                };
                let Some(element) = open.selected_item() else {
                    return Task::none();
                };
                crate::editor::completion::insertion(element, &open.typed, None)
            };
            let recul = if insertion.text.ends_with("}}") { 2 } else { 0 };

            let tache = state
                .editor
                .replace_before(insertion.backspaces, &insertion.text)
                .map(Message::Editor);
            let curseur = if recul > 0 {
                let position = state.editor.cursor_offset().saturating_sub(recul);
                state
                    .editor
                    .set_cursor_offset(position)
                    .map(Message::Editor)
            } else {
                Task::none()
            };
            state.completion = None;

            Task::batch([tache, curseur])
        }

        Message::CompletionMove(delta) => {
            if let Some(open) = state.completion.as_mut() {
                open.move_selection(delta);
            }

            Task::none()
        }
        Message::Format(action) => appliquer_le_format(state, action),
        Message::FormatDerniereCouleur => {
            let couleur = state
                .derniere_couleur
                .clone()
                .unwrap_or_else(|| "red".to_string());
            appliquer_le_format(state, crate::app::message::FormatAction::Couleur(couleur))
        }
        Message::TitreInserer(niveau) => {
            // L'insertion simple au curseur : `#` × n + espace. Pas de
            // reformatage de bloc — on tape, comme à la main.
            let marque = "#".repeat(niveau.clamp(1, 6) as usize);
            state
                .editor
                .insert_at_cursor(&format!("{marque} "))
                .map(Message::Editor)
        }
        Message::MenuContextuelExecuter(action) => {
            state.menu_contextuel = None;
            let message = match action {
                crate::app::message::MenuContextuelAction::Copier => {
                    iced_code_editor::Message::Copy
                }
                crate::app::message::MenuContextuelAction::Couper => iced_code_editor::Message::Cut,
                crate::app::message::MenuContextuelAction::Coller => {
                    iced_code_editor::Message::Paste(String::new())
                }
                crate::app::message::MenuContextuelAction::ToutSelectionner => {
                    iced_code_editor::Message::SelectAll
                }
                crate::app::message::MenuContextuelAction::Annuler => {
                    iced_code_editor::Message::Undo
                }
                crate::app::message::MenuContextuelAction::Retablir => {
                    iced_code_editor::Message::Redo
                }
            };
            state.editor.update(message).map(Message::Editor)
        }
        Message::MenuContextuelFermer => {
            state.menu_contextuel = None;
            Task::none()
        }

        Message::Editor(message) => {
            // Le clic droit appartient à l'application : le panneau d'une
            // balise ou l'action d'une colonne se place au point visé.
            if let iced_code_editor::Message::ContextMenuRequested(point) = &message {
                return menu_contextuel(state, *point);
            }

            // Un ouvrant tapé sur une sélection encadre cette sélection. Sans
            // sélection, la méthode rend `None` et la frappe garde son sens.
            if let iced_code_editor::Message::CharacterInput(ouvrant) = &message
                && let Some(fermant) = crate::editor::mots::fermeture_de(*ouvrant)
                && let Some(tache) = state.editor.envelopper_selection(*ouvrant, fermant)
            {
                state.rafraichir_statistiques_bibliques();
                state.mots_a_recalculer();
                return tache.map(Message::Editor);
            }

            if state.menu_contextuel.is_some()
                && matches!(
                    message,
                    iced_code_editor::Message::MouseClick(..)
                        | iced_code_editor::Message::DoubleClick(_)
                        | iced_code_editor::Message::TripleClick(_)
                        | iced_code_editor::Message::MouseDrag(_)
                )
            {
                state.menu_contextuel = None;
            }

            // Un clic dans le texte ferme le menu contextuel et rend la main
            // à l'éditeur : la bulle ne bloque ni la lecture ni le curseur.
            if state.composeur.is_some()
                && matches!(
                    message,
                    iced_code_editor::Message::MouseClick(..)
                        | iced_code_editor::Message::DoubleClick(_)
                        | iced_code_editor::Message::TripleClick(_)
                )
            {
                return Task::batch([
                    crate::app::composeur::fermer_composeur(state, None),
                    state.editor.update(message).map(Message::Editor),
                ]);
            }

            // L'éditeur capte `Tab`, `Maj+Tab` et `Entrée` avant les
            // raccourcis de l'application. Dans un tableau, ces touches
            // circulent dans la grille — même dans une cellule qui contient
            // une citation — : la cible est calculée sur la source, le
            // tableau est aligné, puis le curseur y est reposé.
            if let Some(tache) = navigation_tableau_message(state, &message) {
                return tache;
            }

            // `Ctrl` + flèches déplace d'un mot, et `Ctrl` + Retour arrière /
            // Suppr efface un mot. Le widget ne transmet que la direction et
            // Maj : l'état des modificateurs est suivi par l'application.
            if let iced_code_editor::Message::ArrowKey(direction, maj) = &message
                && state.controle_tenu
            {
                let vers_la_droite = matches!(direction, iced_code_editor::ArrowDirection::Right);
                return state
                    .editor
                    .deplacer_le_curseur_par_mot(vers_la_droite, *maj || state.maj_tenu)
                    .map(Message::Editor);
            }
            if state.controle_tenu && matches!(message, iced_code_editor::Message::Backspace) {
                state.rafraichir_statistiques_bibliques();
                state.mots_a_recalculer();
                return state.editor.supprimer_le_mot(true).map(Message::Editor);
            }
            if matches!(message, iced_code_editor::Message::DeleteWord) {
                state.rafraichir_statistiques_bibliques();
                state.mots_a_recalculer();
                return state.editor.supprimer_le_mot(false).map(Message::Editor);
            }

            // `Tab` sur une citation en cours d'écriture ouvre le menu
            // contextuel sans rien consommer ; ailleurs, la tabulation garde
            // son geste ordinaire.
            if matches!(message, iced_code_editor::Message::Tab)
                && let Some(tache) = crate::app::composeur::ouvrir_si_citation_au_curseur(state)
            {
                return tache;
            }

            // L'éditeur capte `Tab` et `Maj+Tab` pour ses propres gestes.
            // Sur une liste, c'est le document qui commande : le niveau
            // change, et la tabulation n'insère pas ses espaces.
            if let Some(tache) = tabulation_sur_liste(state, &message) {
                return tache;
            }

            let content_changed = crate::editor::EditorState::message_may_change_content(&message);
            // Le curseur a-t-il bougé ? Le défilement doit le suivre, même
            // quand la touche ou le clic a été traité ailleurs dans le widget :
            // le curseur ne doit jamais rester collé au bord haut ou bas de la
            // fenêtre. Un simple défilement de la molette ne change pas le
            // curseur et ne doit donc pas le ramener.
            let curseur_avant = state.editor.editor.cursor_position();
            let geste_de_souris = matches!(
                &message,
                iced_code_editor::Message::MouseClick(..)
                    | iced_code_editor::Message::DoubleClick(_)
                    | iced_code_editor::Message::TripleClick(_)
                    | iced_code_editor::Message::MouseDrag(_)
            );
            // Le texte a peut-être changé : le compte de mots est à refaire, au
            // prochain tic, et non à chaque image.
            // Le survol des références bibliques : ce qui est sous la souris
            // devient la cible de la bulle (le système de références).
            if let iced_code_editor::Message::MouseClick(_, offset) = message
                && state
                    .reglages
                    .fonctionnalites
                    .actif(crate::base::valeurs::CLE_REFERENCES_BIBLIQUES)
                && !super::pointeur_bloque(state)
            {
                // Le clic sur une référence : sa place, ses parties, son texte.
                let texte = state.editor.content();
                let nouvelle = crate::plugins::bible::reference_au(&texte, offset).and_then(
                    |(debut, fin, _)| {
                        let contenu = &texte[debut + 2..fin - 2];
                        crate::plugins::bible::lire_groupe_complet_de_balise(contenu).map(
                            |(elements, forme)| {
                                let references = elements
                                    .iter()
                                    .filter_map(|element| element.reference().cloned())
                                    .collect::<Vec<_>>();
                                let extrait = crate::plugins::bible::extrait(
                                    &references,
                                    state.plugins.bible.traduction,
                                );
                                crate::plugins::bible::ReferenceCliquee {
                                    intervalle: (debut, fin),
                                    elements,
                                    references,
                                    forme,
                                    extrait,
                                }
                            },
                        )
                    },
                );
                state.plugins.bible.reference_cliquee = nouvelle;
                state.plugins.bible.edition = None;
            }

            if let iced_code_editor::Message::MouseHover(point, offset) = message
                && state
                    .reglages
                    .fonctionnalites
                    .actif(crate::base::valeurs::CLE_REFERENCES_BIBLIQUES)
                && !super::pointeur_bloque(state)
                && state.composeur.is_none()
            {
                let texte = state.editor.content();
                if let Some(reference) = crate::plugins::bible::reference_sous(&texte, offset) {
                    let change = state
                        .plugins
                        .bible
                        .survol
                        .as_ref()
                        .is_some_and(|(_, actuel)| actuel != &reference);
                    if change {
                        state.plugins.bible.survol = None;
                    }
                    let deja_jeune = state
                        .plugins
                        .bible
                        .survol_candidat
                        .as_ref()
                        .is_some_and(|(_, actuel, _)| actuel == &reference);
                    if !deja_jeune {
                        state.plugins.bible.survol_candidat =
                            Some((point, reference, std::time::Instant::now()));
                    }
                } else {
                    state.plugins.bible.survol = None;
                    state.plugins.bible.survol_candidat = None;
                }
            }

            // Le texte défile sous le pointeur : la référence survolée n'est
            // plus celle qu'on voit, et sa bulle doit disparaître.
            if matches!(message, iced_code_editor::Message::Scrolled(_)) {
                state.plugins.bible.survol = None;
                state.plugins.bible.survol_candidat = None;
            }

            state.mots_a_recalculer();

            // L'aperçu suit ce qu'on lit — mais seulement quand le suivi est
            // demandé : le bouton de la barre d'outils en décide, et écrire
            // ailleurs dans l'assemblage ne doit pas arracher l'aperçu.
            let suivre = matches!(message, iced_code_editor::Message::Scrolled(_))
                && state.layout.follow_preview;
            let citation_tapee = matches!(&message, iced_code_editor::Message::CharacterInput('@'));
            let task = state.editor.update(message).map(Message::Editor);
            let defilement =
                if geste_de_souris || state.editor.editor.cursor_position() != curseur_avant {
                    state.editor.defiler_vers_curseur().map(Message::Editor)
                } else {
                    Task::none()
                };

            // La frappe vient peut-être de poser `,,` ou `/` : la palette
            // s'ouvre alors sans voler le texte, et son champ prend le relais.
            let palette = if content_changed {
                crate::app::palette::ouvrir_si_declencheur(state)
            } else {
                Task::none()
            };

            // `@` ouvre le composeur de citation dès la frappe. Le `@` d'une
            // adresse électronique ne passe pas la frontière, et le composeur
            // ne s'ouvre pas dans une bande de code.
            let citation = if citation_tapee {
                crate::app::composeur::ouvrir_si_citation_au_curseur(state)
            } else {
                None
            };

            let suivi = if suivre {
                super::follow_editor_scroll(state)
            } else {
                Task::none()
            };

            Task::batch(vec![
                task,
                defilement,
                palette,
                citation.unwrap_or_else(Task::none),
                suivi,
            ])
        }
        _ => Task::none(),
    }
}
