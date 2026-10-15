//! L'assistance de langage : la liste de complétion, le survol, la définition.
//!
//! Ce que l'application **demande** au serveur, et ce qu'elle en fait. La règle
//! qui traverse tout ce module : ce que le projet sait déjà — ses clés de
//! bibliographie, ses étiquettes — n'est jamais demandé à personne, car le
//! serveur ne les voit pas et l'aller-retour coûte plus cher qu'un parcours de
//! fichiers.

use super::state::{Completion, Hover};
use super::{Message, State, update};

use iced::Task;
use std::path::Path;

/// Les touches que la liste de complétion réserve pour elle.
///
/// Pendant qu'elle est ouverte, l'éditeur les **décline** au lieu de les traiter
/// (patch 4 du crate) : elles continuent donc leur chemin jusqu'ici, et les
/// flèches ne déplacent pas le curseur en plus de la sélection.
pub(super) fn reserved_keys() -> Vec<iced::keyboard::Key> {
    use iced::keyboard::{Key, key::Named};

    vec![
        Key::Named(Named::ArrowUp),
        Key::Named(Named::ArrowDown),
        Key::Named(Named::Enter),
        Key::Named(Named::Tab),
        Key::Named(Named::Escape),
    ]
}

/// Demande la liste au serveur, si le texte la demande à l'endroit du curseur.
///
/// C'est la **frappe** qui décide, et rien d'autre : après chaque message de
/// l'éditeur, on regarde le mot en cours. Le serveur répond plus tard, et la
/// liste s'ouvre à ce moment-là — une réponse est un événement comme un autre.
pub(super) fn request_completion(state: &mut State) -> Task<Message> {
    let content = state.editor.content();
    let offset = state.editor.cursor_offset();

    let Some(context) = crate::lsp::completion::context(&content, offset) else {
        // Plus de déclencheur sous le curseur : la liste n'a plus lieu d'être.
        close_completion(state);

        return Task::none();
    };

    // Rien de neuf à demander : même déclencheur, même mot déjà tapé.
    if let Some(open) = state.lsp.completion.as_ref()
        && open.trigger == context.trigger
        && open.typed == context.typed
    {
        return Task::none();
    }

    // `@` appelle une citation, et les citations sont **nos données** : le projet
    // connaît ses clés. Les proposer ici n'est pas un raccourci — c'est la seule
    // source qui les ait, tinymist n'en fournissant aucune pour un chapitre — et
    // c'est immédiat, puisqu'il n'y a personne à interroger.
    if context.trigger == crate::lsp::completion::CITATION {
        let items = crate::lsp::completion::citations(
            state
                .bibliography
                .iter()
                .map(|entry| (entry.key.as_str(), entry.kind.as_str())),
        );

        if !items.is_empty() {
            open_completion(state, items, &context);

            return Task::none();
        }
    }

    let Some(session) = state.lsp.session.as_mut() else {
        return Task::none();
    };

    // **Synchroniser avant de demander.** tinymist répond sur le document qu'il
    // connaît : demander la liste avant de lui avoir donné la frappe, c'est lui
    // demander de compléter un texte qui ne contient pas encore ce qu'on vient
    // d'écrire. Le serveur répondait alors pour l'ancien texte, et la liste ne
    // venait qu'à la demande suivante — d'où l'impression qu'il fallait une
    // touche de plus. Le coût est borné : on ne synchronise que lorsque le
    // curseur est dans un mot déclenché par `#` ou `@`.
    session.did_change(&content);

    state.lsp.tokens += 1;

    let token = state.lsp.tokens;
    let position = crate::lsp::messages::position_of(&content, offset);
    let anchor = state.editor.cursor_view_position();

    session.completion(
        position,
        token,
        crate::lsp::client::Trigger::Character(context.trigger),
    );

    state.lsp.awaiting = true;

    // La liste précédente n'a plus lieu d'être, mais on la garde à l'écran
    // jusqu'à la réponse : la faire disparaître à chaque lettre ferait
    // clignoter le texte.
    match state.lsp.completion.as_mut() {
        Some(open) => {
            open.typed = context.typed;
            open.trigger = context.trigger;
            open.start = context.start;
            open.anchor = anchor;
        }
        None => {
            state.lsp.completion = Some(Completion {
                items: Vec::new(),
                shown: Vec::new(),
                selected: 0,
                typed: context.typed,
                trigger: context.trigger,
                start: context.start,
                anchor,
            });

            state.editor.editor.set_reserved_keys(reserved_keys());
        }
    }

    Task::none()
}

/// Demande la définition de ce qui est sous le curseur.
///
/// La question porte sur le **curseur**, jamais sur la souris : le geste est une
/// touche, donc la cible est toujours celle qu'on est en train d'écrire.
pub(super) fn request_definition(state: &mut State) -> Task<Message> {
    let content = state.editor.content();
    let offset = state.editor.cursor_offset();

    // Une **référence** désigne une étiquette du projet : c'est notre affaire, et
    // tinymist ne la résout pas — mesuré : sa vue s'arrête au fichier ouvert,
    // alors qu'une étiquette vaut pour tout le document.
    if let Some(label) = crate::lsp::completion::reference(&content, offset)
        && let Some((path, line)) = crate::project::find_label(&state.project, &label)
    {
        // Toute réponse du serveur encore en vol devient périmée : elle ne doit
        // pas remplacer le saut qu'on vient de faire.
        state.lsp.attention += 1;

        return jump_to(state, &path, line, 0);
    }

    let Some(session) = state.lsp.session.as_mut() else {
        return Task::none();
    };

    state.lsp.attention += 1;

    session.definition(
        crate::lsp::messages::position_of(&content, offset),
        state.lsp.attention,
    );

    Task::none()
}

/// Demande ce que le serveur sait de ce qui est sous le curseur.
/// Demande au serveur ce que renommer le symbole sous le curseur changerait.
///
/// Le serveur répond par des modifications **réparties sur les fichiers** ;
/// rien ne s'écrit avant que l'utilisateur les ait vues.
pub(super) fn request_rename(state: &mut State, new_name: &str) -> Task<Message> {
    let content = state.editor.content();
    let offset = state.editor.cursor_offset();

    let Some(session) = state.lsp.session.as_mut() else {
        return Task::none();
    };

    state.lsp.attention += 1;

    let token = state.lsp.attention;
    let position = crate::lsp::messages::position_of(&content, offset);

    session.rename(position, new_name, token);

    Task::none()
}

pub(super) fn request_hover(state: &mut State) -> Task<Message> {
    let content = state.editor.content();
    let offset = state.editor.cursor_offset();

    let Some(session) = state.lsp.session.as_mut() else {
        return Task::none();
    };

    state.lsp.attention += 1;

    session.hover(
        crate::lsp::messages::position_of(&content, offset),
        state.lsp.attention,
    );

    Task::none()
}

/// Va d'un endroit à un autre : le même document, ou un autre chapitre.
///
/// Un aller à la définition peut traverser le projet. Le trajet se fait donc en
/// deux temps lorsque le fichier visé n'est pas celui qui est ouvert : on retient
/// l'endroit, on ouvre le chapitre, et c'est son ouverture qui y pose le curseur.
pub(super) fn jump_to(state: &mut State, path: &Path, line: usize, column: usize) -> Task<Message> {
    let current = state.editor.path.clone();

    if current.is_some_and(|open| crate::project::same_file(&open, path)) {
        state.lsp.hover = None;

        return state.editor.set_cursor(line, column).map(Message::Editor);
    }

    // Ailleurs : il faut que ce soit un chapitre du projet — sinon nous n'aurions
    // rien pour l'ouvrir.
    if !state.project.owns(path) {
        state.export_status = Some("La définition est hors du projet".to_string());

        return Task::none();
    }

    state.jump = Some((line, column));

    update(state, Message::OpenChapter(path.to_path_buf()))
}

/// Ouvre la liste, ou la remplit si elle attendait déjà.
///
/// **Une seule porte** : les clés du projet comme les éléments du serveur passent
/// par ici, ce qui évite que les deux chemins divergent. Et une liste qui ne
/// montrerait rien se ferme, plutôt que de retenir les touches pour rien.
pub(super) fn open_completion(
    state: &mut State,
    items: Vec<crate::lsp::messages::CompletionItem>,
    context: &crate::lsp::completion::Context,
) {
    let shown = crate::lsp::completion::matching(&items, &context.typed);

    if shown.is_empty() {
        close_completion(state);

        return;
    }

    let anchor = state.editor.cursor_view_position();

    match state.lsp.completion.as_mut() {
        Some(open) => {
            open.items = items;
            open.shown = shown;
            open.selected = 0;
            open.typed = context.typed.clone();
            open.trigger = context.trigger;
            open.start = context.start;
            open.anchor = anchor;
        }
        None => {
            state.lsp.completion = Some(Completion {
                items,
                shown,
                selected: 0,
                typed: context.typed.clone(),
                trigger: context.trigger,
                start: context.start,
                anchor,
            });

            state.editor.editor.set_reserved_keys(reserved_keys());
        }
    }
}

/// Ferme la liste, et rend les touches à l'éditeur.
pub(super) fn close_completion(state: &mut State) {
    if state.lsp.completion.take().is_some() {
        state.editor.editor.set_reserved_keys(Vec::new());
    }
}

/// Accepte l'élément sélectionné : il **remplace** ce qui a été tapé.
///
/// Remplacer, et non ajouter : `#fig` doit devenir `#figure(…)`, jamais
/// `#figfigure(…)`. La plage vient du serveur quand il la donne — c'est
/// précisément ce qu'une étiquette seule ne pouvait pas dire.
pub(super) fn accept_completion(state: &mut State) -> Task<Message> {
    let Some(open) = state.lsp.completion.as_ref() else {
        return Task::none();
    };

    let Some(item) = open.selected_item() else {
        close_completion(state);

        return Task::none();
    };

    let content = state.editor.content();

    // La plage du serveur est en unités UTF-16, et se compte depuis le début du
    // document ; ce qu'il faut effacer, c'est ce qui précède le curseur.
    let replacement = item.replacement().map(|range| {
        let start = crate::lsp::messages::offset_of(&content, range.start);
        let end = crate::lsp::messages::offset_of(&content, range.end);

        content[start..end].chars().count()
    });

    let insertion = crate::lsp::completion::insertion(item, &open.typed, replacement);

    close_completion(state);

    state
        .editor
        .replace_before(insertion.backspaces, &insertion.text)
        .map(Message::Editor)
}

/// Applique un événement du serveur de langage.
///
/// Le serveur est une source de vérité **en plus**, jamais *à la place* : ses
/// diagnostics ne remplacent pas ceux de notre propre compilation, qui restent
/// ceux de l'aperçu. Il arrive aussi qu'il parle d'un autre fichier que celui
/// qu'on écrit — ses diagnostics ne sont donc gardés que pour le document
/// ouvert.
pub(super) fn apply_lsp(state: &mut State, event: crate::lsp::client::Event) -> Task<Message> {
    use crate::lsp::client::Event;

    match event {
        Event::Ready { .. } => {
            state.lsp.ready = true;
            state.lsp.awaiting = false;

            Task::none()
        }

        Event::Diagnostics { uri, items } => {
            if state.lsp.document.as_deref() == Some(uri.as_str()) {
                // La traduction pour l'affichage se fait **ici**, une fois :
                // elle demande le texte du document, et la vue ne doit rien
                // cloner.
                state.lsp.reported = crate::lsp::messages::report(&state.editor.content(), &items);
                state.lsp.diagnostics = items;
            }

            Task::none()
        }

        Event::Gone { reason } => {
            // Le serveur est parti : l'application continue sans lui, et le
            // dit. Rien de ce qui suit ne dépend de lui.
            eprintln!("Le serveur de langage s'est arrêté : {reason}");

            state.lsp.session = None;
            state.lsp.unavailable = true;
            state.lsp.ready = false;
            state.lsp.awaiting = false;
            state.lsp.document = None;
            state.lsp.diagnostics.clear();
            state.lsp.reported.clear();

            Task::none()
        }

        Event::Completions { token, items } => {
            state.lsp.awaiting = false;

            // Une réponse à une demande périmée : la frappe a continué depuis, et
            // afficher cette liste-là montrerait ce qui n'est plus écrit.
            if token != state.lsp.tokens {
                return Task::none();
            }

            // Le mot en cours, tel qu'il est **maintenant** : entre la demande et
            // la réponse, la frappe a pu continuer.
            let Some(context) =
                state
                    .lsp
                    .completion
                    .as_ref()
                    .map(|open| crate::lsp::completion::Context {
                        trigger: open.trigger,
                        typed: open.typed.clone(),
                        start: open.start,
                    })
            else {
                return Task::none();
            };

            open_completion(state, items, &context);

            Task::none()
        }

        Event::CommandFinished { error, .. } => {
            if let Some(reason) = error {
                eprintln!("Le serveur n'a pas pu exécuter la commande demandée : {reason}");
            }

            Task::none()
        }

        // Les réponses aux questions qu'on n'a pas encore posées : la
        // navigation viendra après la liste.
        Event::Rename { token, changes } => {
            if token < state.lsp.attention {
                return Task::none();
            }

            // Le serveur répond par ce qu'il changerait. Rien ne s'écrit :
            // l'utilisateur verra les fichiers, les comptera, et confirmera.
            if changes.is_empty() {
                state.export_status = Some("Rien à renommer ici".to_string());
                state.renommage = None;

                return Task::none();
            }

            if let Some(renommage) = state.renommage.as_mut() {
                renommage.changements = Some(changes);
            }

            Task::none()
        }

        Event::Definitions { token, locations } => {
            if token < state.lsp.attention {
                return Task::none();
            }

            // La première suffit : dans un éditeur de texte, « où est défini ce
            //ci » n'a qu'une bonne réponse, et une liste n'aurait rien à montrer.
            let Some(location) = locations.first() else {
                state.export_status = Some("Rien à atteindre ici".to_string());

                return Task::none();
            };

            let Some(path) = crate::lsp::client::Session::path_from_uri(&location.uri) else {
                return Task::none();
            };

            let line = location.range.start.line as usize;
            let column = crate::lsp::messages::column_in_chars(
                &state.editor.content(),
                location.range.start,
            );

            jump_to(state, &path, line, column)
        }

        Event::Hover { token, text } => {
            if token < state.lsp.attention {
                return Task::none();
            }

            state.lsp.hover = text.map(|text| Hover {
                text,
                anchor: state.editor.cursor_view_position(),
            });

            Task::none()
        }
    }
}
