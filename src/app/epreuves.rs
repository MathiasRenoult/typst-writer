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

/// Les dimensions d'une bulle construite : ancre, place choisie, cadre.
pub fn note_bulle(ancre: iced::Point, haut: f32, panneau: f32, hauteur: f32) {
    if !actif("vue") {
        return;
    }

    static DERNIERE: std::sync::Mutex<Option<(u32, u32, u32, u32)>> = std::sync::Mutex::new(None);

    let vue = (
        ancre.x.to_bits(),
        haut.to_bits(),
        panneau.to_bits(),
        hauteur.to_bits(),
    );
    let mut derniere = DERNIERE.lock().unwrap();

    // Au changement : la bulle se reconstruit à chaque image, et ce qu'on
    // cherche est le moment où sa place change.
    if *derniere != Some(vue) {
        println!(
            "[vue] bulle : ancre x {:.0} y {:.0} | top {:.0} | panneau {:.0} | hauteur {:.0}",
            ancre.x, ancre.y, haut, panneau, hauteur
        );

        *derniere = Some(vue);
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

            Some(Task::done(Message::Editor(iced_code_editor::Message::CtrlEnd)))
        }

        12 => {
            println!("[épreuve] le renommage s'ouvre");

            Some(Task::done(Message::SymbolRenameStart))
        }

        14 => {
            println!("[épreuve] le nouveau nom : repere2");

            Some(Task::done(Message::SymbolRenameChanged("repere2".to_string())))
        }

        16 => {
            println!("[épreuve] demande au serveur ce que cela changerait");

            Some(Task::done(Message::SymbolRenameAsk))
        }

        60 => {
            // L'aperçu : c'est lui que l'utilisateur verrait, et c'est lui
            // qui décide si l'application est permise.
            match state.renommage.as_ref().and_then(|r| r.changements.as_ref()) {
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

/// Pilote l'éditeur tout seul, un pas par tic ; `None` quand rien à faire.
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
