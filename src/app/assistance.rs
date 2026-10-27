//! L'assistance de langage : la liste de complétion, le survol, la définition.
//!
//! Ce que l'application **demande** au serveur, et ce qu'elle en fait. La règle
//! qui traverse tout ce module : ce que le projet sait déjà — ses clés de
//! bibliographie, ses étiquettes — n'est jamais demandé à personne, car le
//! serveur ne les voit pas et l'aller-retour coûte plus cher qu'un parcours de
//! fichiers.

use super::composeur::Composeur;
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

/// Repose les glyphes de notes : le document change, les numéros suivent.
///
/// C'est ici que « dynamique » a lieu — à chaque frappe, à chaque déplacement
/// du curseur. La règle de pliage est **une**, dans l'éditeur
/// (`EditorState::marques_a_jour`) : elle suit aussi les remplacements faits
/// par le composeur, qui ne passent pas par la boucle des messages.
pub(super) fn actualiser_marques(state: &mut State) {
    state.editor.marques_a_jour();
}

/// Demande la liste au serveur, si le texte la demande à l'endroit du curseur.
///
/// La frappe ouvre et filtre la liste ; un déplacement du curseur sans édition
/// la ferme. Les événements de défilement et les autres messages sans effet sur
/// le curseur gardent la liste ancrée à l'endroit courant.
pub(super) fn request_completion(
    state: &mut State,
    content_changed: bool,
    declencheur_tape: bool,
) -> Task<Message> {
    // Les glyphes suivent le document : chaque frappe peut créer, défaire ou
    // déplacer une note, et ses rangs avec elle.
    actualiser_marques(state);

    // Le composeur de notes est ouvert : il tient le clavier, et rien de ce
    // qui arrive ici ne le concerne. C'est aussi ce qui l'empêche de se
    // refermer tout seul — son `@` déclencheur, une fois mangé, n'est plus
    // sous le curseur pour le justifier.
    if state.composeur.is_some() {
        return Task::none();
    }

    let content = state.editor.content();
    let offset = state.editor.cursor_offset();

    let cursor_moved = state
        .lsp
        .completion
        .as_ref()
        .is_some_and(|open| open.should_close_after(offset, content_changed));
    if cursor_moved {
        close_completion(state);

        return Task::none();
    }

    let Some(context) = crate::lsp::completion::context(&content, offset) else {
        // Plus de déclencheur sous le curseur : la liste n'a plus lieu d'être.
        close_completion(state);

        return Task::none();
    };

    // Rien de neuf à demander : même déclencheur, même mot déjà tapé.
    if let Some(open) = state.lsp.completion.as_mut()
        && open.trigger == context.trigger
        && open.typed == context.typed
    {
        // Un défilement ne change pas le curseur : garder la liste sans la
        // redemander, en rafraîchissant seulement sa position de référence.
        open.start = context.start;
        open.cursor_offset = offset;

        return Task::none();
    }

    // En Markdown, `#` est le marqueur saisi directement pour un titre, pas une
    // commande à compléter. Réserver l'assistance locale aux vraies extensions
    // du langage ; ainsi écrire `# ` ne fait pas clignoter une liste inutile.
    if state.editor.is_intermediate() && context.trigger != crate::lsp::completion::CITATION {
        close_completion(state);
        return Task::none();
    }

    // `@` appelle une citation, et les citations sont **nos données** : le projet
    // connaît ses clés. Deux portes, et elles ne se ressemblent pas.
    //
    // Un `@` qui vient d'être **tapé** ouvre la création : le déclencheur est
    // mangé, et le document ne revoit la note qu'à la fin. Pour une citation
    // déjà là, il n'y a plus qu'**une** porte : le clic sur son glyphe — la
    // frappe n'édite plus jamais une citation en cachette.
    if context.trigger == crate::lsp::completion::CITATION {
        if declencheur_tape {
            return ouvrir_composeur(state, &context);
        }

        return Task::none();
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

    session.completion(
        position,
        token,
        crate::lsp::client::Trigger::Character(context.trigger),
    );

    state.lsp.awaiting = true;
    state.lsp.depuis = Some(std::time::Instant::now());

    // La liste précédente n'a plus lieu d'être, mais on la garde à l'écran
    // jusqu'à la réponse : la faire disparaître à chaque lettre ferait
    // clignoter le texte.
    match state.lsp.completion.as_mut() {
        Some(open) => {
            open.typed = context.typed;
            open.trigger = context.trigger;
            open.start = context.start;
            open.cursor_offset = offset;
        }
        None => {
            state.lsp.completion = Some(Completion {
                items: Vec::new(),
                shown: Vec::new(),
                selected: 0,
                typed: context.typed,
                trigger: context.trigger,
                start: context.start,
                cursor_offset: offset,
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
    let reference = crate::lsp::completion::reference(&content, offset);

    if let Some(label) = reference.as_deref()
        && let Some((path, line)) = crate::project::find_label(&state.project, label)
    {
        // Toute réponse du serveur encore en vol devient périmée : elle ne doit
        // pas remplacer le saut qu'on vient de faire.
        state.lsp.attention += 1;

        return jump_to(state, &path, line, 0);
    }

    // Une **citation** `@clé` désigne une entrée de la bibliographie : le même
    // geste qu'une étiquette, mais vers `references.bib`. C'est ce qui rend la
    // référence atteignable — sans quoi la clé reste un nom qu'on ne peut jamais
    // voir ni corriger.
    if let Some(key) = reference.as_deref()
        && let Some((path, line)) = crate::project::find_bibliography_entry(&state.project, key)
    {
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
/// par ici, ce qui évite que les deux chemins divergent. Une recherche sans
/// résultat reste affichée avec son état vide tant que le curseur n'a pas bougé.
pub(super) fn open_completion(
    state: &mut State,
    items: Vec<crate::lsp::messages::CompletionItem>,
    context: &crate::lsp::completion::Context,
) {
    let shown = crate::lsp::completion::matching(&items, &context.typed);

    let cursor_offset = state.editor.cursor_offset();

    match state.lsp.completion.as_mut() {
        Some(open) => {
            open.items = items;
            open.shown = shown;
            open.selected = 0;
            open.typed = context.typed.clone();
            open.trigger = context.trigger;
            open.start = context.start;
            open.cursor_offset = cursor_offset;
        }
        None => {
            state.lsp.completion = Some(Completion {
                items,
                shown,
                selected: 0,
                typed: context.typed.clone(),
                trigger: context.trigger,
                start: context.start,
                cursor_offset,
            });

            state.editor.editor.set_reserved_keys(reserved_keys());
        }
    }
}

/// Ferme la liste, et rend les touches à l'éditeur.
pub(super) fn close_completion(state: &mut State) {
    if state.lsp.completion.take().is_some() {
        state.editor.editor.set_reserved_keys(Vec::new());
        state.lsp.cancel_completion_request();
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

/// Ouvre le composeur de notes, sur le `@` qui vient d'être tapé.
///
/// Le déclencheur **part** : le `@`, et ce qui a pu être tapé derrière, sont
/// mangés — leur place est dans le champ du panneau. Le document ne revoit
/// donc la note qu'à la fin, et Échap ne laisse rien. Le clavier part aux
/// champs ; l'éditeur le rend, et ne le reprend qu'à la fermeture.
pub(super) fn ouvrir_composeur(
    state: &mut State,
    context: &crate::lsp::completion::Context,
) -> Task<Message> {
    let cles = state
        .bibliography
        .iter()
        .map(|entry| (entry.key.clone(), entry.kind.clone()))
        .collect();

    // L'ancre est celle du `@` déclencheur : le mot tapé commence juste après.
    let ancre = context.start.saturating_sub(1);
    let manges = state.editor.cursor_offset() - ancre;

    // Toute réponse tinymist encore en vol n'a plus lieu d'être.
    close_completion(state);

    let manger = state.editor.replace_before(manges, "").map(Message::Editor);
    state.editor.editor.lose_focus();

    let mut composeur = Composeur::ouvert(cles, ancre);
    composeur.saisir(context.typed.clone());
    state.composeur = Some(composeur);

    Task::batch([manger, champ_composeur()])
}

/// Ouvre le composeur **sur une citation déjà écrite**, pour la réécrire.
///
/// Rien n'est mangé : la citation d'origine reste en place dans le document
/// tant qu'on ne l'a pas réécrite. Échap ne la touche donc pas — c'est la
/// même promesse qu'en création, tenue par un autre bout.
pub(super) fn ouvrir_edition(
    state: &mut State,
    references: Vec<crate::langage::schema::Reference>,
    debut: usize,
    fin: usize,
) -> Task<Message> {
    let cles = state
        .bibliography
        .iter()
        .map(|entry| (entry.key.clone(), entry.kind.clone()))
        .collect();

    // Toute réponse tinymist encore en vol n'a plus lieu d'être.
    close_completion(state);
    state.editor.editor.lose_focus();

    let mut composeur = Composeur::ouvert(cles, debut);
    composeur.charger(references);
    composeur.edition = Some(fin);
    state.composeur = Some(composeur);

    champ_composeur()
}

/// Le champ de saisie du composeur, prêt à recevoir la frappe.
fn champ_composeur() -> Task<Message> {
    iced::widget::operation::focus(Composeur::champ_id())
}

/// Les touches du composeur, quand il est ouvert.
///
/// Tab avance d'une étape, Entrée insère la note, Échap ne laisse rien, et
/// les flèches déplacent la sélection. Elles arrivent d'autant plus sûrement
/// qu'elles ne servent à rien dans le champ de saisie — qui, lui, prend la
/// frappe entière.
pub(super) fn composeur_touche(
    state: &mut State,
    key: &iced::keyboard::Key,
    modifiers: iced::keyboard::Modifiers,
) -> Option<Task<Message>> {
    use iced::keyboard::{Key, key::Named};

    let message = match key {
        Key::Named(Named::Tab) => Message::ComposeurAvancer,
        Key::Named(Named::Enter) => Message::ComposeurTerminer,
        Key::Named(Named::Escape) => Message::ComposeurFermer,
        Key::Named(Named::ArrowUp) => Message::ComposeurDeplacer(-1),
        Key::Named(Named::ArrowDown) => Message::ComposeurDeplacer(1),
        Key::Character(texte) if modifiers.control() && texte.as_str() == " " => {
            Message::ComposeurTerminer
        }
        _ => return None,
    };

    Some(update(state, message))
}

/// Les messages du composeur de notes.
pub(super) fn composeur_update(state: &mut State, message: Message) -> Task<Message> {
    let intermediaire = state.editor.is_intermediate();

    let Some(composeur) = state.composeur.as_mut() else {
        return Task::none();
    };

    match message {
        Message::ComposeurSaisie(texte) => {
            composeur.saisir(texte);

            Task::none()
        }
        Message::ComposeurDeplacer(delta) => {
            composeur.deplacer(delta);

            Task::none()
        }
        Message::ComposeurPrendre(index) => {
            let dernier = composeur.visibles().len().saturating_sub(1);
            composeur.selection = index.min(dernier);
            composeur.avancer();

            Task::none()
        }
        Message::ComposeurChoisir(index) => {
            composeur.choisir(index);

            Task::none()
        }
        Message::ComposeurAvancer => {
            composeur.avancer();

            Task::none()
        }
        Message::ComposeurTerminer => {
            let texte = composeur.terminer(intermediaire);

            fermer_composeur(state, texte)
        }
        Message::ComposeurFermer => fermer_composeur(state, None),
        _ => Task::none(),
    }
}

/// Ferme le composeur, et écrit la note s'il y en a une.
///
/// C'est le **seul** endroit où le document reçoit quelque chose du
/// composeur : la note entière, d'un seul tenant, à la place du `@`
/// déclencheur mangé. Quand il n'y a rien — Échap —, rien ne s'écrit : c'est
/// tout le sens de « Échap ne laisse rien ».
pub(super) fn fermer_composeur(state: &mut State, texte: Option<String>) -> Task<Message> {
    let Some(composeur) = state.composeur.take() else {
        return Task::none();
    };

    // Le clavier revient à l'éditeur.
    let refocus = state
        .editor
        .update(iced_code_editor::Message::CanvasFocusGained)
        .map(Message::Editor);

    let Some(texte) = texte else {
        return refocus;
    };

    // Ce qui précède le curseur depuis l'ancre : le `@` mangé, et rien d'autre
    // — sauf si la frappe a devancé le focus, auquel cas elle part avec. En
    // édition, c'est la plage de la citation d'origine qui est remplacée.
    state.render.words_pending = true;

    let edition = match composeur.edition {
        Some(fin) => {
            let texte = separer(state, fin, texte);

            state
                .editor
                .replace_span(composeur.ancre, fin, &texte)
                .map(Message::Editor)
        }
        None => {
            // En création, la plage va du `@` mangé (l'ancre) au curseur : ce
            // que la frappe a écrit par-dessus. Même remplacement en une
            // sélection — jamais de retours arrière à compter.
            let fin = state.editor.cursor_offset();
            let texte = separer(state, fin, texte);

            state.editor.replace_span(composeur.ancre, fin, &texte).map(Message::Editor)
        }
    };

    Task::batch([edition, refocus])
}

/// Un espace après la note, si ce qui suit viendrait s'y coller.
///
/// Une clé de citation court jusqu'au premier caractère qui n'en est pas un :
/// `@martin2020` suivi de « Un texte » donnerait la clé « martin2020Un ». Ce
/// qui suit est donc protégé d'un espace, comme la main l'aurait laissé.
fn separer(state: &State, offset: usize, mut texte: String) -> String {
    let content = state.editor.content();
    let suite = content.get(offset..).and_then(|reste| reste.chars().next());

    if suite.is_some_and(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        texte.push(' ');
    }

    texte
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
            // dit — à l'écran, pas dans un journal que personne ne lit.
            state.note = Some(crate::app::state::Note::erreur(format!(
                "L'assistance à la frappe s'est arrêtée : {reason}"
            )));

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
            // Une réponse à une demande périmée : la frappe a continué depuis, et
            // afficher cette liste-là montrerait ce qui n'est plus écrit. Elle ne
            // termine pas non plus l'attente d'une réponse plus récente.
            if !state.lsp.accept_completion_response(token) {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// La complétion reste tant que le curseur reste sur la même position ;
    /// une édition continue le mot, mais une navigation ferme la liste.
    #[test]
    fn completion_closes_on_cursor_move_but_not_while_typing() {
        let completion = Completion {
            items: Vec::new(),
            shown: Vec::new(),
            selected: 0,
            typed: "dupont".to_string(),
            trigger: '@',
            start: 4,
            cursor_offset: 10,
        };

        assert!(!completion.should_close_after(10, false));
        assert!(completion.should_close_after(9, false));
        assert!(!completion.should_close_after(11, true));
    }
}
