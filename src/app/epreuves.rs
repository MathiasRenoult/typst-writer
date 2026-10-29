//! Les épreuves : voir ce que l'écran ne dit pas.
//!
//! Une seule variable, `TYPST_WRITER_EPREUVES`, qui prend une liste de noms
//! séparés par des virgules. Non sollicités, ces instruments ne coûtent rien et
//! ne changent aucune décision de l'application.
//!
//! - `vue` — publie le défilement **vu par la vue**, et non celui de l'état :
//!   l'état est lu avant que l'arbre de widgets ne soit reconstruit, et un
//!   défilement perdu à la reconstruction ne s'y voit jamais.
//! - `defilement` — pilote l'éditeur tout seul : curseur en fin de document,
//!   ouverture et fermeture de la bulle, saut à une position. Le banc d'épreuve
//!   graphique n'arrive pas à lui faire avaler une frappe.
//! - `menu` — montre la bulle de suggestions même sans rien à proposer, avec
//!   trois lignes d'essai ; `defilement` la manœuvre.
//! - `memoire` — le journal de la mémoire.
//! - `lsp` — le journal d'assistance.
//!
//! `TYPST_WRITER_EPREUVE=<fichier>`, lui, n'est pas un interrupteur mais la
//! **donnée** d'entrée des épreuves de composition et de polices : il reste ce
//! qu'il est.

use crate::app::message::Message;
use crate::app::state::State;
use iced::Task;

/// Rejoue Ctrl, molette, relâchement dans la fenêtre réelle, sans frapper
/// dans un document. Le routage est celui de l'abonnement natif.
pub fn driver_zoom(state: &State) -> Option<Task<Message>> {
    use iced::{Event, event::Status, keyboard, mouse};
    use std::sync::atomic::{AtomicU32, Ordering};

    if !actif("zoom") || state.render.document.is_none() {
        return None;
    }
    static ETAPE: AtomicU32 = AtomicU32::new(0);
    let etape = ETAPE.fetch_add(1, Ordering::Relaxed);
    let evenement = match etape {
        2 => return Some(Task::done(Message::ModeBasculer)),
        4 => Event::Keyboard(keyboard::Event::ModifiersChanged(keyboard::Modifiers::CTRL)),
        6 => Event::Mouse(mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 },
        }),
        9 => Event::Mouse(mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Pixels { x: 0.0, y: -30.0 },
        }),
        12 => Event::Keyboard(keyboard::Event::ModifiersChanged(
            keyboard::Modifiers::default(),
        )),
        14 => Event::Mouse(mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 },
        }),
        5 | 8 | 11 | 16 => {
            let document = state.render.document.as_deref().expect("document rendu");
            let tailles = crate::ui::preview::sizes_of_with_zoom(
                document,
                state
                    .render
                    .viewport
                    .unwrap_or_else(crate::ui::preview::default_viewport),
                state.render.zoom,
            );
            println!(
                "[zoom] étape {etape} | Ctrl {} | {:?} | première page {:?}",
                state.controle_tenu,
                state.render.zoom,
                tailles.first()
            );
            return None;
        }
        _ => return None,
    };
    super::message_evenement(evenement, Status::Captured).map(Task::done)
}

/// L'instrument demandé est-il actif ?
///
/// La variable n'est lue qu'une fois, au premier appel : son contenu ne peut
/// pas changer en cours d'exécution.
pub fn actif(nom: &str) -> bool {
    static DEMANDES: std::sync::OnceLock<std::collections::HashSet<String>> =
        std::sync::OnceLock::new();

    DEMANDES
        .get_or_init(|| {
            std::env::var("TYPST_WRITER_EPREUVES")
                .unwrap_or_default()
                .split(',')
                .map(str::trim)
                .filter(|nom| !nom.is_empty())
                .map(str::to_string)
                .collect()
        })
        .contains(nom)
}

/// La bulle de l'épreuve : demandée, et ouverte par le pilote.
static MENU: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn menu_ouvert() -> bool {
    actif("menu") && MENU.load(std::sync::atomic::Ordering::Relaxed)
}

pub fn menu_basculer(ouvert: bool) {
    MENU.store(ouvert, std::sync::atomic::Ordering::Relaxed);
}

/// Le défilement vu par la vue — publié seulement au changement.
///
/// L'image passe soixante fois par seconde ; ce qu'on cherche est exactement la
/// **discontinuité**.
pub fn note_vue(defilement: f32, liste_ouverte: bool) {
    if !actif("vue") {
        return;
    }

    static DERNIERE: std::sync::Mutex<Option<(u32, bool)>> = std::sync::Mutex::new(None);

    let vue = (defilement.to_bits(), liste_ouverte);
    let mut derniere = DERNIERE.lock().unwrap();

    if *derniere != Some(vue) {
        println!(
            "[vue] défilement {defilement:.0} | liste {}",
            if liste_ouverte { "ouverte" } else { "fermée" }
        );

        *derniere = Some(vue);
    }
}

/// Pilote la fenêtre de mise en page : l'ouvrir, y changer des valeurs, et
/// les appliquer — pour voir, d'un bout à l'autre, qu'un champ devient une
/// page.
pub fn driver_reglages(state: &State) -> Option<Task<Message>> {
    use crate::mise_en_page::modele::Champ;

    if !actif("reglages") {
        return None;
    }

    static TRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

    let _ = state;

    match TRAME.fetch_add(1, std::sync::atomic::Ordering::Relaxed) {
        12 => {
            println!("[épreuve] la fenêtre de mise en page s'ouvre");

            Some(Task::done(Message::MiseEnPageOuvrir))
        }

        16 => {
            println!("[épreuve] le papier passe en A5");

            Some(Task::done(Message::MiseEnPageChamp(
                Champ::PapierFormat,
                "a5".to_string(),
            )))
        }

        20 => {
            println!("[épreuve] les marges gauches et droites à 4cm");

            Some(Task::done(Message::MiseEnPageChamp(
                Champ::MargeGauche,
                "4cm".to_string(),
            )))
        }

        60 => {
            println!("[épreuve] application de la mise en page");

            Some(Task::done(Message::MiseEnPageAppliquer))
        }

        _ => None,
    }
}

/// Pilote le renommage d'un symbole, un pas par tic.
///
/// Le document d'épreuve **finit** par une utilisation du symbole : le curseur
/// en fin de document est donc dessus, et c'est lui que le serveur renomme.
pub fn driver_renommage(state: &State) -> Option<Task<Message>> {
    if !actif("renommage") {
        return None;
    }

    static TRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

    match TRAME.fetch_add(1, std::sync::atomic::Ordering::Relaxed) {
        10 => {
            // Le symbole est le dernier mot du document : c'est là que le
            // curseur doit être pour que le serveur sache de quoi il parle.
            println!("[épreuve] le curseur part en fin de document");

            Some(Task::done(Message::Editor(
                iced_code_editor::Message::CtrlEnd,
            )))
        }

        12 => {
            println!("[épreuve] le renommage s'ouvre");

            Some(Task::done(Message::SymbolRenameStart))
        }

        14 => {
            println!("[épreuve] le nouveau nom : repere2");

            Some(Task::done(Message::SymbolRenameChanged(
                "repere2".to_string(),
            )))
        }

        16 => {
            println!("[épreuve] demande au serveur ce que cela changerait");

            Some(Task::done(Message::SymbolRenameAsk))
        }

        60 => {
            // L'aperçu : c'est lui que l'utilisateur verrait, et c'est lui
            // qui décide si l'application est permise.
            match state
                .renommage
                .as_ref()
                .and_then(|r| r.changements.as_ref())
            {
                Some(changements) => {
                    println!("[épreuve] aperçu reçu :");

                    for fichier in changements {
                        println!(
                            "[épreuve]   {} — {} modification(s)",
                            fichier.path.display(),
                            fichier.edits.len()
                        );
                    }

                    println!("[épreuve] application");

                    Some(Task::done(Message::SymbolRenameApply))
                }

                None => {
                    println!("[épreuve] aucun aperçu reçu — rien ne s'applique");

                    if let Some(message) = &state.export_status {
                        println!("[épreuve] la barre d'état dit : {message}");
                    }

                    if state.lsp.session.is_none() {
                        println!("[épreuve] la session du serveur n'est pas prête");
                    }

                    None
                }
            }
        }

        _ => None,
    }
}

/// Pilote le composeur de notes de références, un pas par tic.
///
/// La chronique va du `@` déclencheur à la note écrite — puis prouve la
/// promesse inverse : Échap ne laisse rien. Le document est lu à chaque
/// étape : c'est lui l'observable, pas le message envoyé.
pub fn driver_composeur(state: &State) -> Option<Task<Message>> {
    if !actif("composeur") {
        return None;
    }

    static TRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    static AVANT: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

    match TRAME.fetch_add(1, std::sync::atomic::Ordering::Relaxed) {
        8 => {
            // La main entre dans l'éditeur : c'est le clic qui la lui donne,
            // et sans elle la frappe est ignorée — le message du clic porte
            // les deux moitiés du focus, comme dans la vraie vie.
            println!("[épreuve] la main entre dans l'éditeur");
            Some(Task::done(Message::Editor(
                iced_code_editor::Message::MouseClick(iced::Point::ORIGIN),
            )))
        }
        12 => {
            println!("[épreuve] le `@` déclencheur est tapé dans le document");
            Some(Task::done(Message::Editor(
                iced_code_editor::Message::CharacterInput('@'),
            )))
        }
        20 => {
            match state.composeur.as_ref() {
                Some(composeur) => println!(
                    "[épreuve] composeur ouvert — ancre {}, filtre {:?}",
                    composeur.ancre, composeur.saisie
                ),
                None => println!("[épreuve] RIEN : le composeur ne s'est pas ouvert"),
            }

            None
        }
        24 => Some(Task::done(Message::ComposeurSaisie("dup".to_string()))),
        28 => {
            if let Some(composeur) = state.composeur.as_ref() {
                println!(
                    "[épreuve] filtre {:?} → {} clé(s), surlignée {:?}",
                    composeur.saisie,
                    composeur.visibles().len(),
                    composeur.choisie()
                );
                println!("[épreuve] aperçu : {}", composeur.apercu(true));
            }

            None
        }
        32 => {
            println!("[épreuve] Tab : prendre la référence, proposer la page");
            Some(Task::done(Message::ComposeurAvancer))
        }
        36 => Some(Task::done(Message::ComposeurSaisie("p. 45".to_string()))),
        40 => {
            if let Some(composeur) = state.composeur.as_ref() {
                println!(
                    "[épreuve] étape {:?} — aperçu : {}",
                    composeur.etape,
                    composeur.apercu(true)
                );
            }

            None
        }
        44 => {
            println!("[épreuve] Tab : valider la page, proposer une autre référence");
            Some(Task::done(Message::ComposeurAvancer))
        }
        48 => Some(Task::done(Message::ComposeurSaisie("mar".to_string()))),
        52 => {
            println!("[épreuve] Tab : prendre la seconde référence");
            Some(Task::done(Message::ComposeurAvancer))
        }
        56 => {
            println!("[épreuve] Entrée : insérer la note");
            Some(Task::done(Message::ComposeurTerminer))
        }
        60 => {
            let contenu = state.editor.content();

            println!("[épreuve] document après insertion : {contenu:?}");
            println!("[épreuve] curseur : {}", state.editor.cursor_offset());
            *AVANT.lock().unwrap() = Some(contenu);

            None
        }
        64 => {
            println!("[épreuve] un second `@`, puis Échap : rien ne doit bouger");
            Some(Task::done(Message::Editor(
                iced_code_editor::Message::CharacterInput('@'),
            )))
        }
        68 => {
            println!("[épreuve] Échap");

            Some(Task::done(Message::ComposeurFermer))
        }
        72 => {
            let avant = AVANT.lock().unwrap().clone();
            let maintenant = state.editor.content();

            if avant.as_deref() == Some(maintenant.as_str()) {
                println!("[épreuve] Échap a laissé le document intact : {maintenant:?}");
            } else {
                println!("[épreuve] ÉCHAP A ÉCRIT : {avant:?} devenu {maintenant:?}");
            }

            None
        }
        75 => {
            // Le curseur entre dans la citation — la note est finie depuis, et
            // c'est d'elle qu'on reprend la main.
            Some(Task::done(Message::Editor(
                iced_code_editor::Message::ArrowKey(iced_code_editor::ArrowDirection::Left, false),
            )))
        }
        76 => {
            println!("[épreuve] une frappe **dans** la citation : c'est son édition");
            Some(Task::done(Message::Editor(
                iced_code_editor::Message::CharacterInput('x'),
            )))
        }
        80 => match state.composeur.as_ref() {
            Some(composeur) => {
                println!(
                    "[épreuve] composeur d'édition — plage {:?}, clé {:?}, rang {:?}",
                    composeur.edition, composeur.saisie, composeur.rang
                );
                println!("[épreuve] aperçu : {}", composeur.apercu(true));

                None
            }
            None => {
                println!("[épreuve] RIEN : l'édition ne s'est pas ouverte");

                None
            }
        },
        82 => {
            println!("[épreuve] on choisit la **première** référence de la note");
            Some(Task::done(Message::ComposeurChoisir(0)))
        }
        83 => {
            if let Some(composeur) = state.composeur.as_ref() {
                println!(
                    "[épreuve] sous les doigts : {:?} (rang {:?})",
                    composeur.saisie, composeur.rang
                );
            }

            None
        }
        84 => {
            println!("[épreuve] Entrée : la citation est réécrite");
            Some(Task::done(Message::ComposeurTerminer))
        }
        88 => {
            let contenu = state.editor.content();

            if contenu.contains("martin202") {
                println!("[épreuve] la citation réécrite tient sa place : {contenu:?}");
            } else {
                println!("[épreuve] LA RÉÉCRITURE A PERDU LA CITATION : {contenu:?}");
            }

            None
        }
        92 => {
            println!("[épreuve] réouverture pour la vue — le composeur reste affiché");
            Some(Task::done(Message::Editor(
                iced_code_editor::Message::CharacterInput('@'),
            )))
        }
        96 => Some(Task::done(Message::ComposeurSaisie("dup".to_string()))),
        _ => None,
    }
}

/// Pilote l'éditeur tout seul, un pas par tic ; `None` quand rien à faire.
/// Pilote l'annulation : la suppression d'une note, puis Ctrl+Z, et ce que le
/// document devient — c'est la mesure du bug « le texte d'après disparaît ».
pub fn driver_annulation(state: &mut State) -> Option<Task<Message>> {
    if !actif("annulation") {
        return None;
    }

    static TRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

    match TRAME.fetch_add(1, std::sync::atomic::Ordering::Relaxed) {
        5 => {
            let contenu = state.editor.content();
            let notes = crate::langage::marques::notes(&contenu);

            println!("[annulation] départ : {contenu:?}");
            println!("[annulation] {} notes", notes.len());

            let note = notes.first().cloned()?;

            Some(
                state
                    .editor
                    .set_cursor_offset(note.fin)
                    .map(Message::Editor),
            )
        }
        15 => {
            println!("[annulation] une frappe sur le bord droit de la note");
            Some(Task::done(Message::Editor(
                iced_code_editor::Message::Backspace,
            )))
        }
        25 => {
            let contenu = state.editor.content();
            println!("[annulation] après suppression : {contenu:?}");
            Some(Task::done(Message::Editor(iced_code_editor::Message::Undo)))
        }
        35 => {
            let contenu = state.editor.content();
            let notes = crate::langage::marques::notes(&contenu);
            let plissees = notes
                .iter()
                .map(|n| (n.debut, n.fin, n.numero))
                .collect::<Vec<_>>();

            println!("[annulation] après annulation : {contenu:?}");
            println!(
                "[annulation] glyphes posés : {}",
                state.editor.nombre_de_glyphes()
            );
            println!("[annulation] glyphes (début, fin, n°) : {plissees:?}");
            None
        }
        _ => None,
    }
}

/// Pilote les glyphes : de vrais clics sur les signes, et ce que
/// l'application en fait — la note visée, la fenêtre qui s'ouvre, le pied de
/// page qui se déplie. C'est l'épreuve du **geste**, pas du rendu.
pub fn driver_glyphe(state: &mut State) -> Option<Task<Message>> {
    use crate::langage::marques::Nature;

    if !actif("glyphe") {
        return None;
    }

    static TRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

    match TRAME.fetch_add(1, std::sync::atomic::Ordering::Relaxed) {
        5 => {
            let contenu = state.editor.content();
            let notes = crate::langage::marques::notes(&contenu);

            println!("[glyphe] {} notes balayées", notes.len());
            for note in &notes {
                let extrait = &contenu[note.debut..note.fin.min(note.debut + 28)];
                println!(
                    "[glyphe]   n°{} {:?} : {extrait:?}",
                    note.numero, note.nature
                );
            }

            None
        }
        15 => {
            let contenu = state.editor.content();
            let notes = crate::langage::marques::notes(&contenu);
            let premier = notes.first()?;

            println!("[glyphe] le curseur va au premier glyphe");
            Some(
                state
                    .editor
                    .set_cursor_offset(premier.debut)
                    .map(Message::Editor),
            )
        }
        25 => {
            let point = state.editor.cursor_view_position();
            println!("[glyphe] clic réel au point {point:?}");
            Some(Task::done(Message::Editor(
                iced_code_editor::Message::MouseClick(point),
            )))
        }
        35 => {
            let contenu = state.editor.content();
            let notes = crate::langage::marques::notes(&contenu);
            let visee = state.editor.note_au_clic(&notes);

            println!(
                "[glyphe] note visée par le clic : {:?} — composeur {}",
                visee.map(|rang| notes[rang].numero),
                if state.composeur.is_some() {
                    "OUVERT"
                } else {
                    "fermé"
                }
            );

            Some(Task::done(Message::ComposeurFermer))
        }
        50 => {
            let contenu = state.editor.content();
            let notes = crate::langage::marques::notes(&contenu);
            let pied = notes.iter().find(|n| n.nature == Nature::PiedDePage)?;

            println!("[glyphe] le curseur va au glyphe du pied de page");
            Some(
                state
                    .editor
                    .set_cursor_offset(pied.debut)
                    .map(Message::Editor),
            )
        }
        60 => {
            let point = state.editor.cursor_view_position();
            println!("[glyphe] clic réel au point {point:?}");
            Some(Task::done(Message::Editor(
                iced_code_editor::Message::MouseClick(point),
            )))
        }
        70 => {
            let contenu = state.editor.content();
            let offset = state.editor.cursor_offset();
            let notes = crate::langage::marques::notes(&contenu);
            let pliees = notes
                .iter()
                .filter(|n| !(n.nature == Nature::PiedDePage && n.debut < offset && offset < n.fin))
                .count();

            println!(
                "[glyphe] curseur à l'octet {offset} — {pliees} glyphes pliés sur {}",
                notes.len()
            );
            None
        }
        _ => None,
    }
}

pub fn driver(state: &State) -> Option<Task<Message>> {
    if !actif("defilement") {
        return None;
    }

    static TRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

    match TRAME.fetch_add(1, std::sync::atomic::Ordering::Relaxed) {
        12 => {
            println!("[épreuve] le curseur part en fin de document");
            Some(Task::done(Message::Editor(
                iced_code_editor::Message::CtrlEnd,
            )))
        }
        30 => {
            println!(
                "[épreuve] défilement établi : {}",
                state.editor.editor.viewport_scroll()
            );
            None
        }
        40 => {
            println!("[épreuve] ouverture de la bulle");
            menu_basculer(true);
            None
        }
        50 => {
            println!(
                "[épreuve] bulle ouverte, défilement : {}",
                state.editor.editor.viewport_scroll()
            );
            None
        }
        60 => {
            println!("[épreuve] fermeture de la bulle");
            menu_basculer(false);
            None
        }
        70 => {
            println!(
                "[épreuve] bulle fermée, défilement : {}",
                state.editor.editor.viewport_scroll()
            );
            None
        }
        90 => {
            println!(
                "[épreuve] curseur avant le saut : {}",
                state.editor.cursor_offset()
            );
            Some(Task::done(Message::GotoError(5, 23)))
        }
        100 => {
            println!(
                "[épreuve] curseur après le saut : {}",
                state.editor.cursor_offset()
            );
            None
        }
        _ => None,
    }
}
