//! Épreuves d'intégration exécutées uniquement dans un HOME jetable.
use super::*;

fn etat_isole() -> State {
    assert_eq!(std::env::var("TYPST_WRITER_TEST_ESPACE").as_deref(), Ok("1"), "cette épreuve exige un HOME jetable explicite");
    let home = PathBuf::from(std::env::var_os("HOME").unwrap());
    assert!(home.starts_with(std::env::temp_dir()));
    State::new()
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn les_documents_les_vues_et_la_session_restent_coherents() {
    let mut state = etat_isole();
    let premier = state.espace.active;
    let premier_path = state.editor.path.clone().unwrap();
    let premiere_source = state.editor.content();
    let _ = update(&mut state, Message::Editor(iced_code_editor::Message::MouseClick(iced::Point::ORIGIN)));
    let _ = update(&mut state, Message::Editor(iced_code_editor::Message::Paste("PARTAGE_UNIQUE ".into())));
    let source = state.editor.content();
    assert!(source.contains("PARTAGE_UNIQUE"));
    let _ = update(&mut state, Message::VueDupliquer);
    let double = state.espace.active;
    assert_ne!(premier, double);
    assert_eq!(state.editor.content(), source);
    assert_eq!(state.espace.vues[&premier].editor.content(), source);
    let _ = update(&mut state, Message::Editor(iced_code_editor::Message::MouseClick(iced::Point::ORIGIN)));
    let _ = update(&mut state, Message::Editor(iced_code_editor::Message::Undo));
    assert_eq!(state.editor.content(), premiere_source);
    assert_eq!(state.espace.vues[&premier].editor.content(), premiere_source);
    let _ = update(&mut state, Message::Editor(iced_code_editor::Message::Redo));
    assert_eq!(state.espace.vues[&premier].editor.content(), source);
    let _ = update(&mut state, Message::ModeBasculer);
    assert_eq!(state.mode, ModeVue::Rendu);
    assert_eq!(state.espace.vues[&premier].mode, ModeVue::Edition);
    let pane = state.espace.focus;
    state.espace.deplacer(double, pane, Some(iced::widget::pane_grid::Edge::Left));
    state.choisir_vue(double);
    assert_eq!(state.espace.grille.len(), 2);
    let _ = update(&mut state, Message::NewProject);
    let second_projet = state.project.dir.clone();
    let second = state.espace.active;
    assert_ne!(state.editor.path.as_ref(), Some(&premier_path));
    let deuxieme_source = state.editor.content();
    let _ = update(&mut state, Message::Editor(iced_code_editor::Message::MouseClick(iced::Point::ORIGIN)));
    let _ = update(&mut state, Message::Editor(iced_code_editor::Message::Paste("AUTRE_PROJET ".into())));
    assert!(state.editor.content().contains("AUTRE_PROJET"));
    assert_eq!(state.espace.vues[&premier].editor.content(), source);
    // Un retour asynchrone ne change ni le projet actif ni son texte.
    let _ = update(&mut state, Message::RetourVue(premier, Box::new(Message::ModeTypst)));
    assert_eq!(state.espace.active, second);
    assert_eq!(state.project.dir, second_projet);
    assert_eq!(state.espace.vues[&premier].mode, ModeVue::Typst);
    assert!(!state.espace.vues[&premier].typst.is_empty());
    let _ = update(&mut state, Message::SaveTick);
    assert_eq!(std::fs::read_to_string(&premier_path).unwrap(), source);
    assert!(std::fs::read_to_string(state.editor.path.as_ref().unwrap()).unwrap().contains("AUTRE_PROJET"));
    assert!(!deuxieme_source.contains("AUTRE_PROJET"));
    state.save_session();
    let disposition = state.disposition_complete().unwrap();
    let session = crate::project::load_session();
    assert_eq!(session.espace.as_ref(), Some(&disposition));
    drop(state);
    let mut restauree = State::new();
    let relue = restauree.disposition_complete().unwrap();
    assert_eq!(relue.arbre, disposition.arbre);
    assert_eq!(relue.focus, disposition.focus);
    assert_eq!(relue.vues.len(), 3);
    assert_eq!(restauree.project.dir, second_projet);
    let _ = update(&mut restauree, Message::VueChoisir(premier));
    assert_eq!(restauree.mode, ModeVue::Typst);
    assert_eq!(restauree.editor.content(), source);
    let _ = update(&mut restauree, Message::VueChoisir(double));
    assert_eq!(restauree.mode, ModeVue::Rendu);
    println!("[espace] 2 projets, 3 vues, 2 panneaux : source/undo partagés, modes indépendants, retour asynchrone routé, sauvegarde et restauration vérifiés");
}
