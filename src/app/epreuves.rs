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
pub fn driver_renommage(_state: &State) -> Option<Task<Message>> {
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
