//! Épreuves d'intégration exécutées uniquement dans un HOME jetable.
use super::*;

/// Crée un projet par le vrai geste : le dialogue, puis la validation.
fn creer_un_projet(state: &mut crate::app::State, nom: &str) {
    let _ = update(state, Message::ProjetCreer);
    let _ = update(state, Message::ProjetNom(nom.to_string()));
    let _ = update(state, Message::ProjetValider);
}

/// Crée un projet depuis une base — un projet-modèle (demande de
/// l'auteur : un modèle est un projet comme un autre).
fn creer_depuis(state: &mut crate::app::State, nom: &str, base: &str) {
    let _ = update(state, Message::ProjetCreer);
    let _ = update(state, Message::ProjetNom(nom.to_string()));
    let _ = update(state, Message::ProjetBase(base.to_string()));
    let _ = update(state, Message::ProjetValider);
}
fn etat_isole() -> State {
    assert_eq!(
        std::env::var("TYPST_WRITER_TEST_ESPACE").as_deref(),
        Ok("1"),
        "cette épreuve exige un HOME jetable explicite"
    );
    let home = PathBuf::from(std::env::var_os("HOME").unwrap());
    assert!(home.starts_with(std::env::temp_dir()));
    State::new()
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn la_migration_a_plat_reprend_les_documents_dun_premier_lancement() {
    assert_eq!(
        std::env::var("TYPST_WRITER_TEST_ESPACE").as_deref(),
        Ok("1")
    );
    let home = PathBuf::from(std::env::var_os("HOME").unwrap());
    assert!(home.starts_with(std::env::temp_dir()));
    let ancien = crate::project::work_dir().join("ancien.typ");
    std::fs::write(&ancien, "= Chapitre\n\nTexte conservé.\n").expect("source synthétique");
    let state = etat_isole();
    let texte =
        std::fs::read_to_string(state.project.dir.join("document.tw")).expect("document migré");
    assert!(
        texte.contains("Texte conservé"),
        "le texte migré est présent"
    );
    assert!(!ancien.exists(), "la source se trouve dans le projet");
    assert!(
        state.project.dir.join("chapitres/01-ancien.typ").is_file(),
        "l'original est conservé"
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn creer_un_projet_existant_ne_reutilise_pas_son_dossier() {
    let state = etat_isole();
    let dir = state.project.dir.clone();
    let document = dir.join("document.tw");
    std::fs::write(&document, "TEXTE EXISTANT").expect("document synthétique");
    let resultat = crate::project::create(&state.project.name);
    assert!(
        resultat.is_err(),
        "une création ne doit pas réutiliser un projet existant"
    );
    assert_eq!(std::fs::read_to_string(document).unwrap(), "TEXTE EXISTANT");
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn renommer_un_projet_enregistre_la_nouvelle_session_sans_frappe_supplementaire() {
    let mut state = etat_isole();
    let ancien = state.project.dir.clone();
    let nouveau = ancien.parent().unwrap().join("nouveau-nom");
    let _ = update(&mut state, Message::VueDupliquer);
    let _ = update(&mut state, Message::ProjetRenommer(ancien));
    let _ = update(&mut state, Message::ProjetNom("nouveau nom".into()));
    let _ = update(&mut state, Message::ProjetValider);
    drop(state);
    let restauree = etat_isole();
    assert_eq!(restauree.project.dir, nouveau);
    let disposition = restauree.disposition_complete().expect("espace restauré");
    assert_eq!(disposition.vues.len(), 2);
    assert!(disposition.vues.iter().all(|vue| vue.projet == nouveau));
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn renommer_un_projet_conserve_toutes_ses_vues_et_leur_source_partagee() {
    let mut state = etat_isole();
    let ancien = state.project.dir.clone();
    let nouveau = ancien.parent().unwrap().join("nouveau-nom");
    let premiere_vue = state.espace.active;
    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::Paste("AVANT ".into())),
    );
    let _ = update(&mut state, Message::VueDupliquer);
    let double = state.espace.active;
    let _ = update(&mut state, Message::ProjetRenommer(ancien));
    let _ = update(&mut state, Message::ProjetNom("nouveau nom".into()));
    let _ = update(&mut state, Message::ProjetValider);
    let _ = update(&mut state, Message::VueChoisir(premiere_vue));
    assert_eq!(state.project.dir, nouveau, "la vue garde le projet renommé");
    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::Paste("APRES ".into())),
    );
    let contenu = state.editor.content();
    assert_eq!(
        state.espace.vues[&double].editor.content(),
        contenu,
        "les vues partagent encore la source"
    );
    let _ = update(&mut state, Message::SaveTick);
    assert_eq!(
        std::fs::read_to_string(nouveau.join("document.tw")).unwrap(),
        contenu
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn les_exports_tardifs_dune_vue_remplacee_necrivent_pas() {
    let mut state = etat_isole();
    let ancienne_vue = state.espace.active;
    let home = PathBuf::from(std::env::var_os("HOME").unwrap());
    let pdf = home.join("tardif.pdf");
    let mise = home.join("tardif.toml");
    let _ = update(&mut state, Message::NewProject);
    let _ = update(
        &mut state,
        Message::RetourVue(
            ancienne_vue,
            Box::new(Message::PdfDestination(Ok((pdf.clone(), b"PDF".to_vec())))),
        ),
    );
    let _ = update(
        &mut state,
        Message::RetourVue(
            ancienne_vue,
            Box::new(Message::MiseEnPageDestination(mise.clone(), "TOML".into())),
        ),
    );
    assert!(!pdf.exists());
    assert!(!mise.exists());
}

#[cfg(unix)]
#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_export_refuse_un_lien_vers_un_autre_projet() {
    let premiere = etat_isole();
    let home = PathBuf::from(std::env::var_os("HOME").unwrap());
    let lien = home.join("alias-projet");
    std::os::unix::fs::symlink(&premiere.project.dir, &lien).unwrap();
    let mut seconde = etat_isole();
    let _ = update(&mut seconde, Message::NewProject);
    let cible = lien.join("export.pdf");
    let _ = update(
        &mut seconde,
        Message::PdfDestination(Ok((cible.clone(), b"PDF".to_vec()))),
    );
    assert!(!cible.exists());
    assert!(
        seconde
            .export_status
            .as_ref()
            .is_some_and(|message| message.contains("projet"))
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn une_sauvegarde_conserve_la_version_precedente() {
    let mut state = etat_isole();
    let chemin = state.editor.path.clone().expect("document");
    let original = std::fs::read_to_string(&chemin).expect("document initial");
    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::Paste("NOUVELLE VERSION ".into())),
    );
    let attendu = state.editor.content();
    let _ = update(&mut state, Message::SaveTick);
    assert_eq!(std::fs::read_to_string(&chemin).unwrap(), attendu);
    let sauvegardes = state.project.dir.join("sauvegardes");
    let mut copies: Vec<_> = std::fs::read_dir(&sauvegardes)
        .expect("dossier des versions")
        .filter_map(Result::ok)
        .map(|entree| entree.path())
        .collect();
    copies.sort();
    assert_eq!(copies.len(), 1, "la version remplacée est conservée");
    assert_eq!(std::fs::read_to_string(&copies[0]).unwrap(), original);
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_changement_externe_nest_pas_ecrase_par_la_sauvegarde() {
    let mut state = etat_isole();
    let chemin = state.editor.path.clone().expect("document");
    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::Paste("EN MÉMOIRE ".into())),
    );
    let tampon = state.editor.content();
    std::fs::write(&chemin, "MODIFIÉ DEHORS").expect("changement extérieur");
    let _ = update(&mut state, Message::SaveTick);
    assert_eq!(std::fs::read_to_string(&chemin).unwrap(), "MODIFIÉ DEHORS");
    assert_eq!(
        state.editor.content(),
        tampon,
        "les frappes restent en mémoire"
    );
    assert!(
        state.confirmation.is_some(),
        "la question est posée une fois"
    );
    let _ = update(&mut state, Message::SaveTick);
    assert!(
        state.confirmation.is_some(),
        "la question reste posée, jamais répétée"
    );
    assert_eq!(std::fs::read_to_string(&chemin).unwrap(), "MODIFIÉ DEHORS");
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn resoudre_le_conflit_conserve_la_version_du_disque() {
    let mut state = etat_isole();
    let chemin = state.editor.path.clone().expect("document");
    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::Paste("EN MÉMOIRE ".into())),
    );
    let tampon = state.editor.content();
    std::fs::write(&chemin, "MODIFIÉ DEHORS").expect("changement extérieur");
    let _ = update(&mut state, Message::SaveTick);
    let _ = update(&mut state, Message::ConfirmationConfirmer);
    assert_eq!(
        std::fs::read_to_string(&chemin).unwrap(),
        tampon,
        "le texte tapé est écrit"
    );
    let sauvegardes = state.project.dir.join("sauvegardes");
    let conservee = std::fs::read_dir(&sauvegardes)
        .expect("dossier des versions")
        .filter_map(Result::ok)
        .map(|entree| std::fs::read_to_string(entree.path()).unwrap_or_default())
        .any(|contenu| contenu == "MODIFIÉ DEHORS");
    assert!(conservee, "la version du disque reste récupérable");
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn annuler_le_conflit_ne_le_repose_pas_au_tic_suivant() {
    let mut state = etat_isole();
    let chemin = state.editor.path.clone().expect("document");
    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::Paste("EN MÉMOIRE ".into())),
    );
    let tampon = state.editor.content();
    std::fs::write(&chemin, "MODIFIÉ DEHORS").expect("changement extérieur");
    let _ = update(&mut state, Message::SaveTick);
    let _ = update(&mut state, Message::ConfirmationAnnuler);
    assert!(state.confirmation.is_none());
    let _ = update(&mut state, Message::SaveTick);
    assert!(
        state.confirmation.is_none(),
        "un refus ne rouvre pas la question à chaque tic"
    );
    assert_eq!(std::fs::read_to_string(&chemin).unwrap(), "MODIFIÉ DEHORS");
    assert_eq!(state.editor.content(), tampon);
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn les_sauvegardes_suivantes_ne_demandent_plus_rien() {
    let mut state = etat_isole();
    let chemin = state.editor.path.clone().expect("document");
    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::Paste("PREMIER ".into())),
    );
    let _ = update(&mut state, Message::SaveTick);
    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::Paste("SECOND ".into())),
    );
    let _ = update(&mut state, Message::SaveTick);
    assert!(
        state.confirmation.is_none(),
        "notre propre écriture n'est pas un conflit"
    );
    assert_eq!(
        std::fs::read_to_string(&chemin).unwrap(),
        state.editor.content()
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn une_copie_de_securite_impossible_ne_remplace_pas_le_document() {
    let mut state = etat_isole();
    let chemin = state.editor.path.clone().expect("document");
    let original = std::fs::read_to_string(&chemin).expect("document initial");
    // Un obstacle porte le nom du dossier des copies : conserver échoue.
    std::fs::write(state.project.dir.join("sauvegardes"), "obstacle").expect("obstacle");
    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::Paste("NOUVEAU TEXTE ".into())),
    );
    let tampon = state.editor.content();
    let _ = update(&mut state, Message::SaveTick);
    assert_eq!(
        std::fs::read_to_string(&chemin).unwrap(),
        original,
        "rien n'est remplacé sans sa copie"
    );
    assert_eq!(state.editor.content(), tampon, "le texte reste en mémoire");
    assert!(
        state.note.as_ref().is_some_and(|note| note.erreur),
        "l'erreur est dite"
    );
}

#[cfg(unix)]
#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn une_ecriture_impossible_conserve_le_document_et_sa_copie() {
    use std::os::unix::fs::PermissionsExt;

    let mut state = etat_isole();
    let chemin = state.editor.path.clone().expect("document");
    let dossier = chemin.parent().expect("dossier du document").to_path_buf();
    let original = std::fs::read_to_string(&chemin).expect("document initial");
    let sauvegardes = state.project.dir.join("sauvegardes");
    std::fs::create_dir(&sauvegardes).expect("dossier des copies");
    // Le dossier devient inscriptible ; les copies, elles, restent possibles.
    std::fs::set_permissions(&dossier, std::fs::Permissions::from_mode(0o555)).expect("permission");
    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::Paste("NOUVEAU TEXTE ".into())),
    );
    let tampon = state.editor.content();
    let _ = update(&mut state, Message::SaveTick);
    std::fs::set_permissions(&dossier, std::fs::Permissions::from_mode(0o755))
        .expect("permission rendue");
    assert_eq!(
        std::fs::read_to_string(&chemin).unwrap(),
        original,
        "le document n'est pas touché"
    );
    assert_eq!(state.editor.content(), tampon, "le texte reste en mémoire");
    assert!(
        state.note.as_ref().is_some_and(|note| note.erreur),
        "l'erreur est dite"
    );
    let conservee = std::fs::read_dir(&sauvegardes)
        .expect("dossier des versions")
        .filter_map(Result::ok)
        .map(|entree| std::fs::read_to_string(entree.path()).unwrap_or_default())
        .any(|contenu| contenu == original);
    assert!(conservee, "la version remplacée est déjà conservée");
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_temporaire_abandonne_devient_une_version_recuperable() {
    let mut state = etat_isole();
    let chemin = state.editor.path.clone().expect("document");
    let nom = chemin
        .file_name()
        .expect("nom du document")
        .to_string_lossy()
        .into_owned();
    let dossier = chemin.parent().expect("dossier du document").to_path_buf();
    // Une écriture interrompue par une panne : son auteur n'existe plus.
    let mut enfant = std::process::Command::new("sh")
        .arg("-c")
        .arg("exit 0")
        .spawn()
        .expect("processus témoin");
    let pid = enfant.id();
    enfant.wait().expect("le témoin s'est achevé");
    let temporaire = dossier.join(format!(".{nom}.tmp-{pid}-0"));
    std::fs::write(&temporaire, "VERSION PERDUE").expect("temporaire oublié");
    // La réouverture du document le ramasse.
    let contenu = std::fs::read_to_string(&chemin).expect("document");
    let _ = update(
        &mut state,
        Message::FileOpened(Some((chemin.clone(), contenu))),
    );
    assert!(
        !temporaire.exists(),
        "le temporaire n'est plus laissé pour compte"
    );
    let sauvegardes = state.project.dir.join("sauvegardes");
    let recuperee = std::fs::read_dir(&sauvegardes)
        .expect("dossier des versions")
        .filter_map(Result::ok)
        .map(|entree| std::fs::read_to_string(entree.path()).unwrap_or_default())
        .any(|contenu| contenu == "VERSION PERDUE");
    assert!(recuperee, "la version interrompue devient récupérable");
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_temporaire_vivant_nest_pas_deplace() {
    let mut state = etat_isole();
    let chemin = state.editor.path.clone().expect("document");
    let nom = chemin
        .file_name()
        .expect("nom du document")
        .to_string_lossy()
        .into_owned();
    let dossier = chemin.parent().expect("dossier du document").to_path_buf();
    // L'auteur de ce temporaire est vivant : c'est une écriture en cours.
    let temporaire = dossier.join(format!(".{nom}.tmp-{}-0", std::process::id()));
    std::fs::write(&temporaire, "EN COURS").expect("temporaire vivant");
    let contenu = std::fs::read_to_string(&chemin).expect("document");
    let _ = update(
        &mut state,
        Message::FileOpened(Some((chemin.clone(), contenu))),
    );
    assert!(
        temporaire.exists(),
        "l'écriture d'un confrère vivant ne se touche pas"
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_import_ne_perd_pas_les_frappes_non_sauvegardees() {
    let mut state = etat_isole();
    let chemin = state.editor.path.clone().expect("document");
    let disque = std::fs::read(&chemin).expect("document");
    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::Paste("FRAPPES ".into())),
    );
    let tampon = state.editor.content();
    let source = PathBuf::from(std::env::var_os("HOME").unwrap()).join("fragment.typ");
    let contenu = "= Fragment\n\nTexte à importer.\n";
    std::fs::write(&source, contenu).expect("fragment synthétique");
    let _ = update(
        &mut state,
        Message::FileOpened(Some((source.clone(), contenu.into()))),
    );
    // La transformation se prévisualise : rien ne bouge avant la décision.
    assert!(
        state.confirmation.is_some(),
        "la transformation se prévisualise"
    );
    assert_eq!(
        state.editor.content(),
        tampon,
        "le tampon reste intact avant la décision"
    );
    assert_eq!(
        std::fs::read(&chemin).unwrap(),
        disque,
        "le disque non plus"
    );
    let _ = update(&mut state, Message::ConfirmationConfirmer);
    let apres = state.editor.content();
    assert!(apres.len() > tampon.len(), "l'import rejoint le tampon");
    assert!(
        apres.contains(&tampon),
        "les frappes non sauvegardées sont gardées"
    );
    assert!(
        apres.contains("Texte à importer"),
        "le fragment converti y est"
    );
    assert_eq!(
        std::fs::read(&chemin).unwrap(),
        disque,
        "rien n'écrit avant la sauvegarde"
    );
    assert_eq!(
        std::fs::read_to_string(&source).unwrap(),
        contenu,
        "l'original est préservé"
    );
    let _ = update(&mut state, Message::SaveTick);
    let sauve = std::fs::read_to_string(&chemin).unwrap();
    assert!(sauve.contains("FRAPPES"), "la sauvegarde porte les frappes");
    assert!(sauve.contains("Texte à importer"), "et l'import");
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_import_se_annule_dun_seul_geste() {
    let mut state = etat_isole();
    let avant = state.editor.content();
    let source = PathBuf::from(std::env::var_os("HOME").unwrap()).join("fragment.typ");
    let contenu = "= Fragment\n\nTexte à importer.\n";
    std::fs::write(&source, contenu).expect("fragment synthétique");
    let _ = update(
        &mut state,
        Message::FileOpened(Some((source.clone(), contenu.into()))),
    );
    let _ = update(&mut state, Message::ConfirmationConfirmer);
    let importe = state.editor.content();
    assert_ne!(importe, avant, "l'import est arrivé");
    let _ = update(&mut state, Message::Editor(iced_code_editor::Message::Undo));
    assert_eq!(
        state.editor.content(),
        avant,
        "l'import part d'un seul geste"
    );
    let _ = update(&mut state, Message::Editor(iced_code_editor::Message::Redo));
    assert_eq!(
        state.editor.content(),
        importe,
        "et revient d'un seul geste"
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn annuler_un_import_ne_change_rien() {
    let mut state = etat_isole();
    let chemin = state.editor.path.clone().expect("document");
    let disque = std::fs::read(&chemin).expect("document");
    let tampon = state.editor.content();
    let source = PathBuf::from(std::env::var_os("HOME").unwrap()).join("fragment.typ");
    let contenu = "= Fragment\n\nTexte à importer.\n";
    std::fs::write(&source, contenu).expect("fragment synthétique");
    let _ = update(
        &mut state,
        Message::FileOpened(Some((source.clone(), contenu.into()))),
    );
    let _ = update(&mut state, Message::ConfirmationAnnuler);
    assert!(state.confirmation.is_none());
    assert_eq!(state.editor.content(), tampon, "le tampon est intact");
    assert_eq!(std::fs::read(&chemin).unwrap(), disque, "le disque aussi");
    assert!(source.exists(), "l'original est là");
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_import_annule_si_le_fichier_a_change_depuis_l_apercu() {
    let mut state = etat_isole();
    let source = PathBuf::from(std::env::var_os("HOME").unwrap()).join("fragment.typ");
    let contenu = "= Fragment\n\nPremière version.\n";
    std::fs::write(&source, contenu).expect("fragment synthétique");
    let _ = update(
        &mut state,
        Message::FileOpened(Some((source.clone(), contenu.into()))),
    );
    assert!(state.confirmation.is_some(), "l'aperçu est là");
    let tampon = state.editor.content();

    // Le fichier change entre l'aperçu et la décision : la conversion
    // montrée n'est plus celle qu'on appliquerait.
    std::fs::write(&source, "= Fragment\n\nSeconde version.\n").expect("changement extérieur");
    let _ = update(&mut state, Message::ConfirmationConfirmer);

    assert_eq!(
        state.editor.content(),
        tampon,
        "rien ne s'applique par-dessus un aperçu périmé"
    );
    assert!(
        state.note.as_ref().is_some_and(|note| note.erreur),
        "le changement est dit : {:?}",
        state.note
    );
    assert!(
        state.importation.is_none(),
        "l'import périmé n'attend pas une seconde chance"
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_fichier_tw_rejoint_le_tampon_apres_apercu() {
    let mut state = etat_isole();
    let chemin = state.editor.path.clone().expect("document");
    let disque = std::fs::read(&chemin).expect("document");
    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::Paste("FRAPPES ".into())),
    );
    let tampon = state.editor.content();
    let source = PathBuf::from(std::env::var_os("HOME").unwrap()).join("autre.tw");
    let contenu = "Un paragraphe importé.\n";
    std::fs::write(&source, contenu).expect("fichier synthétique");
    let _ = update(
        &mut state,
        Message::FileOpened(Some((source.clone(), contenu.into()))),
    );
    assert!(
        state.confirmation.is_some(),
        "la transformation se prévisualise"
    );
    assert_eq!(state.editor.content(), tampon, "rien avant la décision");
    let _ = update(&mut state, Message::ConfirmationConfirmer);
    let apres = state.editor.content();
    assert!(
        apres.contains("Un paragraphe importé"),
        "le .tw rejoint le tampon"
    );
    assert!(apres.contains(&tampon), "les frappes sont gardées");
    assert_eq!(
        std::fs::read_to_string(&source).unwrap(),
        contenu,
        "l'original est préservé"
    );
    assert_eq!(
        std::fs::read(&chemin).unwrap(),
        disque,
        "rien n'écrit avant la sauvegarde"
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_projet_sans_verrou_ne_peut_pas_creer_de_bibliographie() {
    let _premiere = etat_isole();
    let mut seconde = etat_isole();
    let bib = seconde.project.dir.join("references.bib");
    std::fs::remove_file(&bib).expect("retrait de la bibliographie");
    let _ = update(&mut seconde, Message::BibliographieCreer);
    assert!(
        !bib.exists(),
        "aucune bibliographie n'est créée sans verrou"
    );
    assert!(seconde.note.as_ref().is_some_and(|note| note.erreur));
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn ouvrir_un_vieux_projet_sous_verrou_vivant_est_en_lecture_seule() {
    let mut state = etat_isole();
    let vieux = crate::project::create("vieux").expect("projet hérité");
    // Un ancien binaire a laissé son marqueur, et son processus tourne encore.
    std::fs::write(vieux.join(".verrou"), format!("{}\n", std::process::id()))
        .expect("marqueur hérité");
    let _ = update(&mut state, Message::OpenProject(vieux.clone()));
    assert!(
        state.note.as_ref().is_some_and(|note| note.erreur),
        "le verrou hérité est dit"
    );
    assert!(
        !state.espace.verrous.contains_key(&vieux),
        "aucun verrou n'est pris"
    );
    let document = vieux.join("document.tw");
    let avant = std::fs::read(&document).expect("document");
    let _ = update(&mut state, Message::NewChapter);
    assert_eq!(
        std::fs::read(&document).unwrap(),
        avant,
        "le projet hérité reste intact"
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_temporaire_de_creation_abandonne_part_a_la_corbeille() {
    let projets = crate::project::projects_dir();
    // Une création interrompue : son auteur n'existe plus.
    let mut enfant = std::process::Command::new("sh")
        .arg("-c")
        .arg("exit 0")
        .spawn()
        .expect("processus témoin");
    let pid = enfant.id();
    enfant.wait().expect("le témoin s'est achevé");
    let abandonne = projets.join(format!(".mort.creation-{pid}-0"));
    std::fs::create_dir_all(&abandonne).expect("temporaire abandonné");
    std::fs::write(abandonne.join("document.tw"), "CREATION INTERROMPUE").expect("fichier");
    // Une création en cours, elle, ne se touche pas.
    let en_cours = projets.join(format!(".vivant.creation-{}-0", std::process::id()));
    std::fs::create_dir_all(&en_cours).expect("temporaire vivant");
    let _state = etat_isole();
    assert!(
        !abandonne.exists(),
        "le temporaire abandonné ne reste pas à la place"
    );
    assert!(en_cours.exists(), "une création en cours ne se touche pas");
    let recuperee = std::fs::read_dir(crate::project::corbeille())
        .expect("corbeille")
        .filter_map(Result::ok)
        .map(|entree| entree.path().join("document.tw"))
        .any(|fichier| {
            std::fs::read_to_string(fichier).unwrap_or_default() == "CREATION INTERROMPUE"
        });
    assert!(
        recuperee,
        "le temporaire abandonné reste récupérable dans la corbeille"
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn le_balayage_des_projets_ignore_un_projet_en_creation() {
    let _state = etat_isole();
    let projets = crate::project::projects_dir();
    let en_cours = projets.join(".mi-creation-999-0");
    std::fs::create_dir_all(&en_cours).expect("temporaire caché");
    std::fs::write(en_cours.join("document.tw"), "DEMI-PROJET").expect("fichier");
    let noms: Vec<String> = crate::project::list_projects()
        .iter()
        .map(|projet| projet.name.clone())
        .collect();
    assert!(
        !noms.iter().any(|nom| nom.contains("mi-creation")),
        "un projet en création ne se voit pas : {noms:?}"
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn une_creation_publiee_ne_laisse_qu_un_projet_complet() {
    let _state = etat_isole();
    let projets = crate::project::projects_dir();
    let dir = crate::project::create("neuf").expect("création");
    assert!(dir.is_dir());
    assert!(dir.join("chapitres").is_dir(), "le squelette est prêt");
    for fichier in ["main.typ", "document.tw", "references.bib"] {
        assert!(
            dir.join(fichier).is_file(),
            "{fichier} présent avant d'être visible"
        );
    }
    let restes: Vec<String> = std::fs::read_dir(&projets)
        .expect("dossier des projets")
        .filter_map(Result::ok)
        .map(|entree| entree.file_name().to_string_lossy().into_owned())
        .filter(|nom| nom.starts_with(".neuf.creation-"))
        .collect();
    assert!(
        restes.is_empty(),
        "aucun temporaire de création ne subsiste : {restes:?}"
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_style_zotero_choisi_se_copie_et_se_choisit() {
    let mut state = etat_isole();
    let _ = update(&mut state, Message::MiseEnPageOuvrir);

    // Un style installé dans un Zotero fabriqué — jamais les données de la
    // machine. Le fichier est lu à son chemin absolu, la copie part au
    // projet, et Zotero ne bouge pas.
    let zotero = std::env::temp_dir().join(format!("tw-style-zotero-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&zotero);
    std::fs::create_dir_all(&zotero).expect("styles Zotero");
    let csl = r#"<?xml version="1.0" encoding="utf-8"?>
<style xmlns="http://purl.org/net/xbiblio/csl" version="1.0" class="note">
  <info>
    <id>https://example.org/styles/zotero</id>
    <title>Style Zotero</title>
    <updated>2026-01-01T00:00:00+00:00</updated>
  </info>
  <citation><layout><text variable="title"/></layout></citation>
  <bibliography><layout><text variable="title"/></layout></bibliography>
</style>"#;
    std::fs::write(zotero.join("zotero.csl"), csl).expect("style Zotero");

    let dialogue = state.mise_en_page.as_mut().expect("fenêtre ouverte");
    dialogue.styles_zotero = vec![crate::zotero::styles::CslStyle {
        id: "https://example.org/styles/zotero".to_string(),
        title: "Style Zotero".to_string(),
        path: zotero.join("zotero.csl"),
    }];

    let _ = update(
        &mut state,
        Message::StyleZoteroChoisi("https://example.org/styles/zotero".to_string()),
    );

    let copie = state.project.dir.join("styles").join("zotero.csl");
    assert!(copie.is_file(), "la copie suit le projet");
    assert_eq!(
        std::fs::read_to_string(&copie).expect("copie"),
        csl,
        "la copie est fidèle"
    );
    assert_eq!(
        std::fs::read_to_string(zotero.join("zotero.csl")).expect("Zotero"),
        csl,
        "Zotero n'est jamais modifié"
    );
    let dialogue = state.mise_en_page.as_ref().expect("fenêtre ouverte");
    assert_eq!(
        dialogue.reglages.bibliographie.style.as_deref(),
        Some("styles/zotero.csl"),
        "le choix est fait"
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_style_zotero_disparu_est_un_diagnostic() {
    let mut state = etat_isole();
    let _ = update(&mut state, Message::MiseEnPageOuvrir);

    let dialogue = state.mise_en_page.as_mut().expect("fenêtre ouverte");
    dialogue.styles_zotero = vec![crate::zotero::styles::CslStyle {
        id: "https://example.org/styles/fantome".to_string(),
        title: "Style fantôme".to_string(),
        path: std::env::temp_dir().join("style-qui-nexiste-plus.csl"),
    }];

    let _ = update(
        &mut state,
        Message::StyleZoteroChoisi("https://example.org/styles/fantome".to_string()),
    );

    // Style manquant : un diagnostic, aucun remplacement discret.
    assert!(
        state.note.as_ref().is_some_and(|note| note.erreur),
        "le problème se dit : {:?}",
        state.note
    );
    let dialogue = state.mise_en_page.as_ref().expect("fenêtre ouverte");
    assert_eq!(
        dialogue.reglages.bibliographie.style, None,
        "rien n'est choisi"
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_apercu_affiche_sous_des_erreurs_est_indique_ancien() {
    let mut state = etat_isole();
    // Une vraie compilation, pour un vrai document affiché.
    let (entree, substitutions, _) = state.compilation_unit_avec_diagnostics();
    let dossier = state.document_dir();
    let rendu = crate::typst::render::compile(&dossier, &entree, &substitutions);
    assert!(
        rendu.errors.is_empty(),
        "le projet témoin compile : {rendu:?}"
    );
    let cle = state.cle_document().expect("document du projet");
    let _ = update(
        &mut state,
        Message::RetourDocument(cle.clone(), Box::new(Message::RenderDone(rendu))),
    );
    assert!(state.render.document.is_some(), "des pages sont affichées");
    assert!(
        !state.render.apercu_perime(),
        "un aperçu à jour n'est pas ancien"
    );

    // La compilation suivante échoue : l'ancien aperçu reste affiché — et
    // cette fois, il se dit ancien.
    let echec = crate::typst::render::CompileOutcome {
        content: state.compilation_fingerprint(),
        errors: vec![crate::typst::render::Diagnostic {
            origine: crate::typst::render::Origine::CompilationTypst,
            message: "Une panne synthétique.".into(),
            line: Some(1),
            column: None,
            file: None,
        }],
        warnings: vec![],
        chapter_page: None,
        outline: vec![],
        document: None,
    };
    let _ = update(
        &mut state,
        Message::RetourDocument(cle, Box::new(Message::RenderDone(echec))),
    );
    assert!(
        state.render.document.is_some(),
        "le dernier aperçu valide reste affiché"
    );
    assert!(
        state.render.apercu_perime(),
        "l'aperçu affiché est déclaré ancien"
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_retour_de_compilation_perime_ne_remplace_pas_le_rendu_courant() {
    let mut state = etat_isole();
    let cle = state.cle_document().expect("document du projet");
    let courante = state.compilation_fingerprint();
    let revision = state.render.revision;
    let perime = crate::typst::render::CompileOutcome {
        content: "empreinte d'un autre texte".to_string(),
        errors: vec![],
        warnings: vec![],
        chapter_page: None,
        outline: vec![],
        document: None,
    };
    let _ = update(
        &mut state,
        Message::RetourDocument(cle.clone(), Box::new(Message::RenderDone(perime))),
    );
    assert_ne!(state.render.content, "empreinte d'un autre texte");
    assert_eq!(
        state.render.revision, revision,
        "un retour périmé ne produit aucune révision"
    );
    let frais = crate::typst::render::CompileOutcome {
        content: courante.clone(),
        errors: vec![],
        warnings: vec![],
        chapter_page: None,
        outline: vec![],
        document: None,
    };
    let _ = update(
        &mut state,
        Message::RetourDocument(cle, Box::new(Message::RenderDone(frais))),
    );
    assert_eq!(
        state.render.content, courante,
        "le retour à jour s'installe"
    );
    assert!(state.render.revision > revision);
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn une_compilation_dun_projet_remplace_ne_simpose_pas_au_projet_de_meme_nom() {
    let mut state = etat_isole();
    let premier_dir = state.project.dir.clone();
    let ancienne_cle = state.cle_document().expect("document du projet");
    let ancienne_empreinte = state.compilation_fingerprint();
    let second = crate::project::create("second").expect("second projet");
    let _ = update(&mut state, Message::OpenProject(second));
    let _ = update(&mut state, Message::ProjetSupprimer(premier_dir.clone()));
    let _ = update(&mut state, Message::ConfirmationConfirmer);
    let nouveau = crate::project::create("mon-travail").expect("réutilisation du nom");
    assert_eq!(nouveau, premier_dir);
    let _ = update(&mut state, Message::OpenProject(nouveau));
    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::Paste("NOUVEAU TEXTE ".into())),
    );
    assert_ne!(state.compilation_fingerprint(), ancienne_empreinte);
    let perime = crate::typst::render::CompileOutcome {
        content: ancienne_empreinte.clone(),
        errors: vec![],
        warnings: vec![],
        chapter_page: None,
        outline: vec![],
        document: None,
    };
    let _ = update(
        &mut state,
        Message::RetourDocument(ancienne_cle, Box::new(Message::RenderDone(perime))),
    );
    assert_ne!(
        state.render.content, ancienne_empreinte,
        "le rendu de l'ancien projet ne s'impose pas au nouveau"
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_export_refuse_un_projet_exterieur_sans_fichier_de_verrou() {
    let mut state = etat_isole();
    let projet = PathBuf::from(std::env::var_os("HOME").unwrap()).join("projet-exterieur");
    let sorties = projet.join("sorties");
    std::fs::create_dir_all(&sorties).unwrap();
    std::fs::write(projet.join("main.typ"), "#include \"document.tw\"").unwrap();
    std::fs::write(projet.join("document.tw"), "DOCUMENT À CONSERVER").unwrap();
    let cible = sorties.join("ouvrage.pdf");
    let contenu = "PDF À CONSERVER".as_bytes();
    std::fs::write(&cible, contenu).unwrap();
    let _ = update(
        &mut state,
        Message::PdfDestination(Ok((cible.clone(), b"AUTRE PDF".to_vec()))),
    );
    assert_eq!(std::fs::read(cible).unwrap(), contenu);
    assert!(
        state
            .export_status
            .as_ref()
            .is_some_and(|message| message.contains("projet"))
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_export_secrit_dans_un_dossier_typst_ordinaire() {
    let mut state = etat_isole();
    let dossier = PathBuf::from(std::env::var_os("HOME").unwrap()).join("typst-ordinaire");
    std::fs::create_dir_all(&dossier).unwrap();
    std::fs::write(dossier.join("main.typ"), "#set page(a4)").unwrap();
    let cible = dossier.join("ouvrage.pdf");
    let _ = update(
        &mut state,
        Message::PdfDestination(Ok((cible.clone(), b"PDF".to_vec()))),
    );
    assert_eq!(std::fs::read(cible).unwrap(), b"PDF");
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_export_de_mise_en_page_secrit_hors_projet() {
    let mut state = etat_isole();
    let mise = crate::mise_en_page::modele::MiseEnPage::default();
    let texte = mise.ecrire().expect("sérialisation des réglages");
    let cible = PathBuf::from(std::env::var_os("HOME").unwrap()).join("export.toml");
    let _ = update(
        &mut state,
        Message::MiseEnPageDestination(cible.clone(), texte),
    );
    let copie = std::fs::read_to_string(cible).expect("fichier exporté");
    assert_eq!(
        crate::mise_en_page::modele::MiseEnPage::lire(&copie).unwrap(),
        mise
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_export_de_mise_en_page_necrit_pas_dans_un_autre_projet_verrouille() {
    let premiere = etat_isole();
    let cible = premiere.project.dir.join("mise-en-page.toml");
    std::fs::write(&cible, "RÉGLAGES À PRÉSERVER").expect("fichier synthétique");
    let mut seconde = etat_isole();
    let _ = update(&mut seconde, Message::NewProject);
    let _ = update(
        &mut seconde,
        Message::MiseEnPageDestination(cible.clone(), "AUTRES RÉGLAGES".into()),
    );
    assert_eq!(
        std::fs::read_to_string(cible).unwrap(),
        "RÉGLAGES À PRÉSERVER"
    );
    assert!(
        seconde
            .export_status
            .as_ref()
            .is_some_and(|message| message.contains("projet"))
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_export_pdf_compile_et_secrit_hors_projet() {
    let mut state = etat_isole();
    let (entree, substitutions) = state.compilation_unit();
    let pdf = crate::typst::render::render_pdf(&state.project.dir, &entree, &substitutions)
        .expect("compilation Typst réelle");
    assert!(pdf.starts_with(b"%PDF"));
    let cible = PathBuf::from(std::env::var_os("HOME").unwrap()).join("export.pdf");
    let _ = update(
        &mut state,
        Message::PdfDestination(Ok((cible.clone(), pdf.clone()))),
    );
    assert_eq!(std::fs::read(cible).unwrap(), pdf);
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_export_pdf_necrit_pas_dans_un_autre_projet_verrouille() {
    let premiere = etat_isole();
    let cible = premiere.project.dir.join("rapport.pdf");
    std::fs::write(&cible, b"PDF EXISTANT").expect("fichier synthétique");
    let mut seconde = etat_isole();
    let _ = update(&mut seconde, Message::NewProject);
    let _ = update(
        &mut seconde,
        Message::PdfDestination(Ok((cible.clone(), b"AUTRE PDF".to_vec()))),
    );
    assert_eq!(std::fs::read(cible).unwrap(), b"PDF EXISTANT");
    assert!(
        seconde
            .export_status
            .as_ref()
            .is_some_and(|message| message.contains("projet"))
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn une_copie_necrit_pas_dans_un_autre_projet_verrouille() {
    let premiere = etat_isole();
    let cible = premiere.project.dir.join("copie.tw");
    std::fs::write(&cible, "À PRÉSERVER").expect("fichier synthétique");
    let mut seconde = etat_isole();
    let _ = update(&mut seconde, Message::NewProject);
    let _ = update(&mut seconde, Message::FileSaved(Some(cible.clone())));
    assert_eq!(std::fs::read_to_string(cible).unwrap(), "À PRÉSERVER");
    assert!(seconde.note.as_ref().is_some_and(|note| note.erreur));
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn une_copie_ne_remplace_pas_le_document_ouvert() {
    let mut state = etat_isole();
    let cible = state.editor.path.clone().expect("document");
    let avant = std::fs::read(&cible).expect("source initiale");
    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::Paste("EN MÉMOIRE".into())),
    );
    let _ = update(&mut state, Message::FileSaved(Some(cible.clone())));
    assert_eq!(std::fs::read(cible).unwrap(), avant);
    assert!(state.editor.is_modified());
    assert!(state.note.as_ref().is_some_and(|note| note.erreur));
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn une_source_tw_ne_peut_pas_etre_enregistree_comme_un_typst() {
    let mut state = etat_isole();
    let cible = PathBuf::from(std::env::var_os("HOME").unwrap()).join("copie.typ");
    let _ = update(&mut state, Message::FileSaved(Some(cible.clone())));
    assert!(!cible.exists());
    assert!(state.note.as_ref().is_some_and(|note| note.erreur));
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn enregistrer_une_copie_ne_fait_pas_perdre_les_frappes_pendant_le_dialogue() {
    let mut state = etat_isole();
    let original = state.editor.path.clone().expect("document du projet");
    let copie = PathBuf::from(std::env::var_os("HOME").unwrap()).join("copie.tw");
    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::Paste("AVANT ".into())),
    );
    std::fs::write(&copie, state.editor.content()).expect("snapshot ancien");
    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::Paste("APRÈS ".into())),
    );
    let attendu = state.editor.content();
    let _ = update(&mut state, Message::FileSaved(Some(copie.clone())));
    assert_eq!(
        state.editor.path.as_ref(),
        Some(&original),
        "la copie ne remplace pas le document du projet"
    );
    assert!(
        state.editor.is_modified(),
        "la copie n'acquitte pas le document"
    );
    assert_eq!(
        std::fs::read_to_string(copie).unwrap(),
        attendu,
        "la copie contient les dernières frappes"
    );
    let _ = update(&mut state, Message::SaveTick);
    assert_eq!(
        std::fs::read_to_string(original).unwrap(),
        attendu,
        "le document du projet est sauvegardé"
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn supprimer_le_dernier_projet_ouvre_un_remplacement_sans_recreer_la_cible() {
    let mut state = etat_isole();
    let ancien = state.project.dir.clone();
    let ancienne_vue = state.espace.active;
    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::Paste("À RETROUVER ".into())),
    );
    let texte = state.editor.content();
    let _ = update(&mut state, Message::ProjetSupprimer(ancien.clone()));
    let _ = update(&mut state, Message::ConfirmationConfirmer);
    assert!(!ancien.exists());
    assert_ne!(state.project.dir, ancien);
    assert!(state.project.dir.is_dir());
    assert!(!state.espace.vues.contains_key(&ancienne_vue));
    assert!(
        state
            .espace
            .grille
            .get(state.espace.focus)
            .is_some_and(|p| p.onglets.contains(&state.espace.active))
    );
    assert_eq!(
        std::fs::read_to_string(
            crate::project::corbeille()
                .join(ancien.file_name().unwrap())
                .join("document.tw")
        )
        .unwrap(),
        texte
    );
    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::Paste("NOUVEAU ".into())),
    );
    let _ = update(&mut state, Message::SaveTick);
    assert!(
        !ancien.exists(),
        "les frappes suivantes ne recréent pas l'ancien dossier"
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn supprimer_un_projet_retire_toutes_ses_vues_et_reponses_tardives() {
    let mut state = etat_isole();
    let ancien = state.project.dir.clone();
    let ancien_id = state.espace.active;
    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::Paste("À CONSERVER ".into())),
    );
    let attendu = state.editor.content();
    let _ = update(
        &mut state,
        Message::Scinder(iced::widget::pane_grid::Edge::Left),
    );
    let double = state.espace.active;
    assert_eq!(state.espace.grille.len(), 2);
    let second = crate::project::create("second").expect("autre projet");
    state.projects = crate::project::list_projects();
    state.etageres = crate::project::etageres();
    state.maj_tenu = true;
    let _ = update(&mut state, Message::OpenProject(second.clone()));
    state.maj_tenu = false;
    assert_eq!(state.project.dir, second);
    let _ = update(&mut state, Message::ProjetSupprimer(ancien.clone()));
    assert!(state.confirmation.is_some());
    let _ = update(&mut state, Message::ConfirmationConfirmer);
    assert!(!ancien.exists(), "le projet a été déplacé");
    assert_eq!(
        std::fs::read_to_string(
            crate::project::corbeille()
                .join(ancien.file_name().unwrap())
                .join("document.tw")
        )
        .unwrap(),
        attendu
    );
    assert!(
        state
            .espace
            .grille
            .iter()
            .all(|(_, p)| !p.onglets.contains(&ancien_id) && !p.onglets.contains(&double))
    );
    assert_eq!(
        state.espace.grille.len(),
        1,
        "le panneau devenu vide est fermé"
    );
    assert!(
        state
            .espace
            .grille
            .get(state.espace.focus)
            .is_some_and(|p| p.onglets.contains(&state.espace.active))
    );
    assert!(!state.espace.vues.contains_key(&ancien_id));
    assert!(!state.espace.vues.contains_key(&double));
    assert!(
        state
            .espace
            .documents
            .keys()
            .all(|(projet, _)| projet != &ancien)
    );

    let nouveau = crate::project::create("mon-travail").expect("réutilisation du nom");
    assert_eq!(nouveau, ancien);
    let _ = update(
        &mut state,
        Message::RetourVue(
            ancien_id,
            Box::new(Message::ZoteroExport(
                "ancienne".into(),
                "Ancienne".into(),
                ancien.clone(),
                Ok("@book{ancien,title={Ancien}}".into()),
            )),
        ),
    );
    assert!(
        !nouveau.join("references-zotero.bib").exists(),
        "une tâche périmée n'atteint pas le nouveau projet"
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_echec_de_sauvegarde_annule_la_suppression_du_projet() {
    let mut state = etat_isole();
    let dir = state.project.dir.clone();
    let chemin = state.editor.path.clone().unwrap();
    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::Paste("À GARDER ".into())),
    );
    let contenu = state.editor.content();
    std::fs::rename(&chemin, chemin.with_extension("original")).expect("fichier initial à l'abri");
    std::fs::create_dir(&chemin).expect("échec de remplacement");
    state.geste_confirme = true;
    let _ = update(&mut state, Message::ProjetSupprimer(dir.clone()));
    assert!(dir.is_dir(), "le projet ne part pas à la corbeille");
    assert_eq!(state.editor.content(), contenu);
    assert!(state.editor.is_modified());
    assert!(state.note.as_ref().is_some_and(|note| note.erreur));
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn renommer_un_projet_sauve_le_texte_et_conserve_le_verrou() {
    let mut state = etat_isole();
    let ancien = state.project.dir.clone();
    let nouveau = ancien.parent().unwrap().join("nouveau-nom");
    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::Paste("AVANT ".into())),
    );
    let avant = state.editor.content();
    let _ = update(&mut state, Message::ProjetRenommer(ancien.clone()));
    let _ = update(&mut state, Message::ProjetNom("nouveau nom".into()));
    let _ = update(&mut state, Message::ProjetValider);
    assert_eq!(state.project.dir, nouveau);
    assert_eq!(
        std::fs::read_to_string(nouveau.join("document.tw")).unwrap(),
        avant
    );
    assert!(state.espace.verrous.contains_key(&nouveau));
    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::Paste("APRES ".into())),
    );
    let apres = state.editor.content();
    let _ = update(&mut state, Message::SaveTick);
    assert_eq!(
        std::fs::read_to_string(nouveau.join("document.tw")).unwrap(),
        apres
    );
    assert!(!ancien.exists(), "l'ancien dossier ne renaît pas");
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_projet_verrouille_ne_peut_pas_etre_renomme_par_une_autre_instance() {
    let premiere = etat_isole();
    let dir = premiere.project.dir.clone();
    let destination = dir.parent().unwrap().join("nouveau-nom");
    let mut seconde = etat_isole();
    let _ = update(&mut seconde, Message::ProjetRenommer(dir.clone()));
    let _ = update(&mut seconde, Message::ProjetNom("nouveau nom".into()));
    let _ = update(&mut seconde, Message::ProjetValider);
    assert!(dir.is_dir(), "le projet reste à sa place");
    assert!(!destination.exists());
    assert!(seconde.note.as_ref().is_some_and(|note| note.erreur));
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn supprimer_un_projet_conserve_les_dernieres_frappes_dans_la_corbeille() {
    let mut state = etat_isole();
    let dir = state.project.dir.clone();
    let nom = dir.file_name().expect("nom du projet").to_owned();
    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::Paste(
            "DERNIERES FRAPPES ".into(),
        )),
    );
    let contenu = state.editor.content();
    state.geste_confirme = true;
    let _ = update(&mut state, Message::ProjetSupprimer(dir.clone()));
    let destination = crate::project::corbeille().join(nom);
    assert!(!dir.exists());
    assert_eq!(
        std::fs::read_to_string(destination.join("document.tw")).unwrap(),
        contenu
    );
    assert!(
        !destination.join(".verrou").exists(),
        "la copie reste récupérable"
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_projet_verrouille_ne_peut_pas_etre_supprime_par_une_autre_instance() {
    let premiere = etat_isole();
    let dir = premiere.project.dir.clone();
    let mut seconde = etat_isole();
    seconde.geste_confirme = true;
    let _ = update(&mut seconde, Message::ProjetSupprimer(dir.clone()));
    assert!(
        dir.is_dir(),
        "le projet utilisé par la première instance doit rester en place"
    );
    assert!(seconde.note.as_ref().is_some_and(|note| note.erreur));
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_projet_sans_verrou_refuse_la_suppression_dun_mot() {
    let premiere = etat_isole();
    let document = premiere.project.dir.join("document.tw");
    std::fs::write(&document, "Premier mot à conserver.\n").expect("source synthétique");
    let mut seconde = etat_isole();
    let avant = seconde.editor.content();
    let _ = update(
        &mut seconde,
        Message::Editor(iced_code_editor::Message::GotoPosition(0, 0)),
    );
    let _ = update(
        &mut seconde,
        Message::Editor(iced_code_editor::Message::DeleteWord),
    );
    assert_eq!(
        seconde.editor.content(),
        avant,
        "le tampon en lecture seule reste intact"
    );
    assert_eq!(std::fs::read_to_string(document).unwrap(), avant);
    assert!(!seconde.editor.is_modified());
    assert!(seconde.note.as_ref().is_some_and(|note| note.erreur));
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_projet_sans_verrou_refuse_la_frappe_au_lieu_de_la_perdre() {
    let _premiere = etat_isole();
    let mut seconde = etat_isole();
    let avant = seconde.editor.content();
    let _ = update(
        &mut seconde,
        Message::Editor(iced_code_editor::Message::Paste("TEXTE ".into())),
    );
    assert_eq!(seconde.editor.content(), avant);
    assert!(!seconde.editor.is_modified());
    assert!(seconde.note.as_ref().is_some_and(|note| note.erreur));
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn renommer_un_chapitre_dun_autre_projet_est_refuse() {
    let premiere = etat_isole();
    let a = premiere.project.dir.join("chapitres/01-ancien.typ");
    std::fs::write(&a, "TEXTE A").expect("chapitre synthétique");
    let mut seconde = etat_isole();
    let _ = update(&mut seconde, Message::NewProject);
    let main = seconde.project.main.clone();
    let avant = std::fs::read(&main).expect("assemblage B");
    seconde.renaming = Some(crate::app::ChapterRename {
        path: a.clone(),
        buffer: "nouveau".into(),
    });
    let _ = update(&mut seconde, Message::RenameCommit);
    assert_eq!(std::fs::read_to_string(&a).unwrap(), "TEXTE A");
    assert!(!a.with_file_name("00-nouveau.typ").exists());
    assert_eq!(std::fs::read(main).unwrap(), avant);
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn confirmer_un_chapitre_dun_autre_projet_ne_le_supprime_pas() {
    let premiere = etat_isole();
    let a = premiere.project.dir.join("chapitres/01-ancien.typ");
    std::fs::write(&a, "TEXTE A").expect("chapitre synthétique");
    let mut seconde = etat_isole();
    let _ = update(&mut seconde, Message::NewProject);
    let main = seconde.project.main.clone();
    let assemblage = b"#include \"chapitres/01-ancien.typ\"\n";
    std::fs::write(&main, assemblage).expect("assemblage synthétique");
    seconde.geste_confirme = true;
    let _ = update(&mut seconde, Message::DeleteChapter(a.clone()));
    assert_eq!(std::fs::read_to_string(a).unwrap(), "TEXTE A");
    assert_eq!(std::fs::read(main).unwrap(), assemblage);
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn une_reponse_tardive_ne_modifie_pas_le_projet_dun_onglet_remplace() {
    let mut state = etat_isole();
    let ancien_id = state.espace.active;
    let ancien_dossier = state.project.dir.clone();
    let document = ancien_dossier.join("document.tw");
    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::Paste("TEXTE NON SAUVÉ ".into())),
    );
    let texte = state.editor.content();
    let _ = update(&mut state, Message::NewProject);
    assert_ne!(state.espace.active, ancien_id);
    let nouveau = state.project.dir.clone();
    let _ = update(
        &mut state,
        Message::RetourVue(
            ancien_id,
            Box::new(Message::ZoteroExport(
                "ancienne-collection".into(),
                "Ancienne".into(),
                ancien_dossier.clone(),
                Ok("@book{ancien,title={Ancien}}".into()),
            )),
        ),
    );
    assert!(
        !ancien_dossier.join("references-zotero.bib").exists(),
        "la réponse d'une vue remplacée est périmée"
    );
    assert!(!nouveau.join("references-zotero.bib").exists());
    let _ = update(&mut state, Message::SaveTick);
    assert_eq!(
        std::fs::read_to_string(document).unwrap(),
        texte,
        "le tampon de la vue remplacée reste sauvegardable"
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_export_zotero_en_cours_ne_s_ecrit_pas_dans_un_autre_projet() {
    let mut state = etat_isole();

    // La requête part d'un projet, la réponse revient après un changement :
    // elle est rattachée au demandeur, jamais au projet courant.
    let ancien = state.project.dir.clone();
    let _ = update(&mut state, Message::NewProject);
    assert_ne!(state.project.dir, ancien);

    let _ = update(
        &mut state,
        Message::ZoteroExport(
            "collection".into(),
            "Test".into(),
            ancien,
            Ok("@book{test, title={Test}}".into()),
        ),
    );

    // Rien n'est écrit dans le projet qui n'a rien demandé — ni clés, ni
    // bibliothèque, ni liaison.
    let nouveau = state.project.dir.clone();
    assert!(!nouveau.join("references-zotero.bib").exists());
    assert!(!nouveau.join("citations.json").exists());
    assert!(!nouveau.join("zotero.json").exists());
    assert!(
        state.note.as_ref().is_some_and(|note| !note.erreur),
        "la péremption se dit sans faire peur : {:?}",
        state.note
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_export_zotero_ecrit_ses_fichiers_en_transaction() {
    let mut state = etat_isole();
    let dir = state.project.dir.clone();

    let _ = update(
        &mut state,
        Message::ZoteroExport(
            "collection".into(),
            "Test".into(),
            dir.clone(),
            Ok("@book{test, title={Test}}".into()),
        ),
    );

    // Les trois fichiers produits sont là, ensemble. La clé est stabilisée
    // (« test » devient une clé dérivée) mais le contenu tient — c'est la
    // politique des clés stables, éprouvée ailleurs.
    let bibliotheque = std::fs::read_to_string(dir.join("references-zotero.bib")).expect("bib");
    assert!(
        bibliotheque.contains("title={Test}"),
        "le contenu tient : {bibliotheque}"
    );
    assert!(dir.join("citations.json").is_file(), "les clés suivent");
    assert!(dir.join("zotero.json").is_file(), "la liaison suit");

    // Aucun débris de transaction.
    let debris: Vec<_> = std::fs::read_dir(&dir)
        .expect("projet")
        .filter_map(|entree| entree.ok())
        .map(|entree| entree.file_name().to_string_lossy().into_owned())
        .filter(|nom| nom.contains("transaction-"))
        .collect();
    assert!(debris.is_empty(), "aucun débris : {debris:?}");
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_import_bib_sans_plugin_rejoint_la_bibliographie() {
    let mut state = etat_isole();

    // Une bibliothèque locale avec une notice — l'import ne l'écrasera pas.
    let dir = state.project.dir.clone();
    std::fs::write(
        dir.join("references.bib"),
        "@book{dupont2023,\n  author = {Dupont},\n  title = {Premier},\n}\n",
    )
    .expect("bibliothèque");
    let _ = update(&mut state, Message::RefreshProject);

    // L'export Zotero (un `.bib`, sans plugin) : une notice neuve, et la
    // même clé avec un autre contenu.
    let source = dir.join("export.bib");
    std::fs::write(
        &source,
        "@book{dupont2023,\n  author = {Dupont},\n  title = {Second},\n}\n\n@article{martin2024,\n  author = {Martin},\n  title = {Neuf},\n}\n",
    )
    .expect("export");

    let _ = update(&mut state, Message::BibImporte(Some(source)));

    let bibliotheque = std::fs::read_to_string(dir.join("references.bib")).expect("bibliothèque");
    assert!(
        bibliotheque.contains("Premier") && !bibliotheque.contains("Second"),
        "la première tient : {bibliotheque}"
    );
    assert!(
        bibliotheque.contains("martin2024"),
        "la notice neuve entre : {bibliotheque}"
    );

    // Le rapport se lit : entrée, et conflit dit.
    let note = state.note.as_ref().expect("rapport").texte.clone();
    assert!(note.contains("martin2024"), "{note}");
    assert!(
        note.contains("conflit") && note.contains("dupont2023"),
        "le conflit se dit : {note}"
    );
    assert_eq!(state.bibliography.len(), 2, "l'état est rafraîchi");
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn le_devoir_synthetique_tient_ses_promesses() {
    let mut state = etat_isole();

    // Le geste : le devoir naît du dialogue.
    creer_un_projet(&mut state, "devoir-synthetique");
    let dir = state.project.dir.clone();
    // Le devoir lui-même : ce que l'auteur écrirait (les modèles
    // intégrés sont retirés — demande de l'auteur).
    std::fs::write(
        dir.join("document.tw"),
        "= Introduction

Selon @dupont2023, la thèse est posée^[note exacte].

= Conclusion

La synthèse.

= Bibliographie

La liste se compose elle-même, d'après vos citations.
",
    )
    .expect("document");
    // Les normes du devoir : page de titre et sommaire — c'est le
    // sommaire qui nomme chaque section dans le PDF.
    let mut normes = crate::mise_en_page::charger(&dir);
    normes.page_de_titre = true;
    normes.sommaire = true;
    let _ = crate::mise_en_page::appliquer(&dir, &normes, true);

    // Une notice, pour la bibliographie.
    std::fs::write(
        dir.join("references.bib"),
        "@book{dupont2023, title = {La théologie du lien}, author = {Dupont, Jean}, year = {2023}}\n",
    )
    .expect("bibliographie");

    // Un chapitre, par le geste de l'application : il parle Typst, où
    // notes, légendes et renvois ont leur vraie syntaxe.
    let _ = update(&mut state, Message::NewChapter);
    let chapitre = std::fs::read_dir(dir.join("chapitres"))
        .expect("chapitres")
        .filter_map(|entree| entree.ok())
        .map(|entree| entree.path())
        .find(|chemin| chemin.extension().is_some_and(|ext| ext == "typ"))
        .expect("chapitre créé");
    std::fs::write(
        &chapitre,
        concat!(
            "= Introduction\n\n",
            "Selon @dupont2023, la thèse est posée#footnote[La note exacte.] suite.\n\n",
            "#figure(\n  rect(width: 2cm),\n  caption: [La légende exacte.],\n) <fig:une>\n\n",
            "Comme le montre @fig:une, le renvoi tient.\n\n",
            "= Développement\n\n",
            "Un second passage.\n\n",
            "= Conclusion\n\n",
            "Le devoir se termine.\n",
        ),
    )
    .expect("chapitre");

    // Le même chemin que l'application : l'assemblage du modèle, le
    // document d'écriture, et ses diagnostics.
    let main = std::fs::read_to_string(dir.join("main.typ")).expect("assemblage");
    let (entry, corrections, rapports) =
        crate::langage::pipeline::compilation_unit_avec_diagnostics(&dir, &main, "");

    assert!(
        !rapports
            .iter()
            .any(|diagnostic| diagnostic.message.contains("neutralisée")),
        "rien n'est neutralisé : {rapports:?}"
    );

    let pdf = crate::typst::render::render_pdf(&dir, &entry, &corrections).expect("PDF");

    // L'export : le fichier produit est le PDF du moteur.
    let cible = dir.join("devoir.pdf");
    let _ = update(
        &mut state,
        Message::PdfDestination(Ok((cible.clone(), pdf.clone()))),
    );
    assert_eq!(
        std::fs::read(&cible).expect("export"),
        pdf,
        "l'export est le PDF"
    );

    // Le texte extrait porte chaque promesse : titres, sommaire, sauts,
    // notes, légendes, renvois, bibliographie.
    let fichier = dir.join("sortie.pdf");
    std::fs::write(&fichier, &pdf).expect("PDF sur disque");
    let extrait = std::process::Command::new("pdftotext")
        .arg(&fichier)
        .arg("-")
        .output();
    let Ok(sortie) = extrait else {
        eprintln!("pdftotext absent — course d'acceptation sautée");
        return;
    };
    assert!(sortie.status.success(), "pdftotext s'exécute");
    let texte = String::from_utf8_lossy(&sortie.stdout).to_lowercase();

    // « titre du devoir » et « sommaire » ont déménagé : le titre vient
    // des métadonnées, le sommaire du gabarit (bug-report-7, points 8-9).
    for promesse in [
        "introduction",
        "développement",
        "conclusion",
        "la note exacte",
        "la légende exacte",
        "dupont",
        "bibliographie",
    ] {
        assert!(texte.contains(promesse), "« {promesse} » manque : {texte}");
    }

    // Le renvoi s'est résolu, jamais neutralisé.
    assert!(!texte.contains("[fig"), "aucun renvoi neutralisé : {texte}");

    // Les sauts : la page de titre, le sommaire, le corps — au moins trois
    // pages séparées.
    let pages = texte.split('\u{c}').count();
    assert!(pages >= 3, "les sauts tiennent : {pages} pages");
}

/// La seconde course : une thèse synthétique, du modèle au PDF — avec son
/// en-tête courant résolu par le moteur (R5, acceptation).
#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn la_these_synthetique_tient_ses_promesses() {
    let mut state = etat_isole();

    creer_un_projet(&mut state, "these-synthetique");
    let dir = state.project.dir.clone();

    std::fs::write(
        dir.join("references.bib"),
        "@book{dupont2023, title = {La théologie du lien}, author = {Dupont, Jean}, year = {2023}}\n",
    )
    .expect("bibliographie");

    let _ = update(&mut state, Message::NewChapter);
    let chapitre = std::fs::read_dir(dir.join("chapitres"))
        .expect("chapitres")
        .filter_map(|entree| entree.ok())
        .map(|entree| entree.path())
        .find(|chemin| chemin.extension().is_some_and(|ext| ext == "typ"))
        .expect("chapitre créé");
    std::fs::write(
        &chapitre,
        concat!(
            "= Premier chapitre <chap:un>\n\n",
            "Selon @dupont2023#footnote[La note de thèse.] la matière tient.\n\n",
            "#figure(\n  rect(width: 2cm),\n  caption: [La planche exacte.],\n) <fig:planche>\n\n",
            "Le renvoi @fig:planche et le chapitre @chap:un se lisent.\n\n",
            "#figure(\n  table(columns: 2, [Colonne], [Autre]),\n  caption: [Le tableau exact.],\n) <tab:un>\n\n",
            "Le tableau @tab:un se cite aussi.\n\n",
            "= Second chapitre\n\n",
            "Le développement continue sur plusieurs pages.\n\n",
            "#pagebreak()\n\n",
            "Une page de plus, pour les sauts.\n",
        ),
    )
    .expect("chapitre");

    // Les métadonnées nourrissent la page de titre et les quick parts —
    // c'est elles que la compilation reçoit (bug-report-7, points 8-9).
    let mut meta = crate::project::metadonnees::Metadonnees::base();
    meta.definir("", "titre".to_string(), "Titre du travail".to_string());
    meta.sauver(&dir).expect("métadonnées");
    let entrees = crate::project::metadonnees::Metadonnees::charger(&dir).entrees();

    // Et les normes portent la page de titre — appliquées, accroche
    // comprise, comme le fait l'application.
    let normes = crate::mise_en_page::modele::MiseEnPage {
        page_de_titre: true,
        sommaire: true,
        typographie: crate::mise_en_page::modele::Typographie {
            base: "francaise".to_string(),
            ..Default::default()
        },
        ..Default::default()
    };
    let _ = crate::mise_en_page::appliquer(&dir, &normes, true).expect("normes");

    let main = std::fs::read_to_string(dir.join("main.typ")).expect("assemblage");
    let (entry, corrections, rapports) =
        crate::langage::pipeline::compilation_unit_avec_diagnostics(&dir, &main, "");

    assert!(
        !rapports
            .iter()
            .any(|diagnostic| diagnostic.message.contains("neutralisée")),
        "rien n'est neutralisé : {rapports:?}"
    );

    let outcome = crate::typst::render::compile_avec_entrees(&dir, &entry, &corrections, &entrees);
    assert!(
        outcome.errors.is_empty(),
        "erreurs moteur : {:?}",
        outcome.errors
    );
    let pdf = crate::typst::render::render_pdf(&dir, &entry, &corrections).expect("PDF");
    let fichier = dir.join("sortie.pdf");
    std::fs::write(&fichier, &pdf).expect("PDF sur disque");
    let extrait = std::process::Command::new("pdftotext")
        .arg(&fichier)
        .arg("-")
        .output();
    let Ok(sortie) = extrait else {
        eprintln!("pdftotext absent — course d'acceptation sautée");
        return;
    };
    assert!(sortie.status.success(), "pdftotext s'exécute");
    let texte = String::from_utf8_lossy(&sortie.stdout).to_lowercase();

    // « titre du travail » et « sommaire » ont déménagé : le titre vient
    // des métadonnées et de la page de titre, le sommaire du gabarit
    // (`outline()` — éprouvé dans `tests_legendes` et
    // `tests_modeles_visibles`, bug-report-7, points 8-9).
    for promesse in [
        "premier chapitre",
        "premier chapitre",
        "second chapitre",
        "la note de thèse",
        "la planche exacte",
        "le tableau exact",
        "dupont",
        "bibliographie",
    ] {
        assert!(texte.contains(promesse), "« {promesse} » manque : {texte}");
    }

    assert!(
        !texte.contains("[fig") && !texte.contains("[chap") && !texte.contains("[tab"),
        "aucun renvoi neutralisé : {texte}"
    );

    // L'en-tête courant a déménagé : le titre vient des métadonnées et de
    // la page de titre (bug-report-7, points 8-9) — la chaîne complète
    // (métadonnées → `sys.inputs` → gabarit) est éprouvée à sa maison
    // (`tests_modeles_visibles`, `les_metadonnées_se_reglent_et_se_sauvent`).

    // Les sauts : titre, sommaire, chapitres — au moins quatre pages.
    let pages = texte.split('\u{c}').count();
    assert!(pages >= 4, "les sauts tiennent : {pages} pages");
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_saut_de_diagnostic_ne_touche_pas_la_saisie() {
    let mut state = etat_isole();

    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::Paste("le texte écrit".into())),
    );
    let avant = state.editor.content();

    // Le saut à un diagnostic déplace la vue — jamais le texte, jamais
    // l'annulation.
    let _ = update(&mut state, Message::GotoError(1, 2));
    assert_eq!(state.editor.content(), avant, "le texte ne bouge pas");

    // La saisie continue d'aller au texte.
    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::Paste(" — la suite".into())),
    );
    assert!(
        state.editor.content().contains("la suite"),
        "on écrit toujours"
    );

    // Et l'annulation ne révèle aucun geste fantôme : le saut n'en a pas
    // ajouté un, une seule annulation remonte au texte d'avant.
    let _ = update(&mut state, Message::Editor(iced_code_editor::Message::Undo));
    assert_eq!(
        state.editor.content(),
        avant,
        "le saut n'a laissé aucun geste dans l'annulation"
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn la_selection_de_bibliographie_ouvre_la_bonne_notice() {
    let mut state = etat_isole();
    let dir = state.project.dir.clone();

    std::fs::write(
        dir.join("references.bib"),
        concat!(
            "@book{premiere, title = {Premier}, author = {A, A}, year = {2020}}\n",
            "@article{seconde, title = {Second}, author = {B, B}, year = {2021}}\n",
        ),
    )
    .expect("bibliographie");
    let _ = update(&mut state, Message::RefreshProject);

    // La sélection : la **bonne** notice s'ouvre dans la fiche.
    let _ = update(&mut state, Message::ModifierReference("seconde".into()));
    let fiche = state.edition_reference.as_ref().expect("fiche ouverte");
    assert_eq!(fiche.cle, "seconde", "la bonne notice est sélectionnée");
    assert_eq!(fiche.kind, "article", "c'est bien elle");

    // Fermer sans rien écrire ne touche pas la bibliothèque.
    let _ = update(&mut state, Message::EditionReferenceFermer);
    assert!(state.edition_reference.is_none(), "la fiche se ferme");
    let bibliotheque = std::fs::read_to_string(dir.join("references.bib")).expect("bibliographie");
    assert!(bibliotheque.contains("premiere") && bibliotheque.contains("seconde"));
    assert_eq!(
        crate::project::parse_bibliography(&dir.join("references.bib")).len(),
        2,
        "aucune notice n'a bougé"
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn la_selection_de_note_s_ecrit_a_la_fin() {
    let mut state = etat_isole();
    let dir = state.project.dir.clone();

    std::fs::write(
        dir.join("references.bib"),
        "@book{dupont2023, title = {Un titre}, author = {Dupont, Jean}, year = {2023}}\n",
    )
    .expect("bibliographie");
    let _ = update(&mut state, Message::RefreshProject);

    // Le composeur, ouvert comme l'application l'ouvre : ses clés, son
    // ancre — puis la note se choisit et s'écrit.
    let cles = state
        .bibliography
        .iter()
        .map(|entree| (entree.key.clone(), entree.kind.clone(), String::new()))
        .collect();
    state.composeur = Some(crate::app::composeur::Composeur::ouvert(cles, 0));

    let _ = update(&mut state, Message::ComposeurSaisie("dupont".into()));
    let _ = update(&mut state, Message::ComposeurChoisir(0));
    let _ = update(&mut state, Message::ComposeurTerminer);

    assert!(
        state.editor.content().contains("dupont2023"),
        "la note choisie s'écrit : {:?}",
        state.editor.content()
    );
    assert!(
        state.composeur.is_none(),
        "le composeur se ferme en écrivant"
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_changement_d_onglet_alterne_les_vues_sans_perdre_le_tampon() {
    let mut state = etat_isole();
    let premier = state.espace.active;

    // Deux vues : la seconde dérive de la première, et le document partagé
    // reste partagé — c'est le contrat des vues.
    let _ = update(&mut state, Message::VueDupliquer);
    let double = state.espace.active;
    assert_ne!(premier, double, "deux onglets, deux vues");

    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::Paste("LE TEXTE PARTAGE".into())),
    );
    let contenu = state.editor.content();
    assert!(contenu.contains("LE TEXTE PARTAGE"), "{contenu}");

    // Le changement d'onglet alterne les deux vues, dans les deux sens —
    // et le tampon ne se perd dans aucun.
    let mut alternance = Vec::new();
    for _ in 0..4 {
        let _ = update(&mut state, Message::OngletSuivant);
        alternance.push(state.espace.active);
    }

    assert_eq!(alternance.len(), 4);
    assert!(
        alternance.contains(&premier) && alternance.contains(&double),
        "les deux onglets reviennent : {alternance:?}"
    );
    assert_eq!(
        state.editor.content(),
        contenu,
        "aucun changement d'onglet ne perd le tampon"
    );
    // Les vues garées tiennent le même document partagé.
    for (id, vue) in &state.espace.vues {
        assert_eq!(
            vue.editor.content(),
            contenu,
            "la vue garée {id} tient le document"
        );
    }
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn une_longue_liste_bibliographique_restee_vivante() {
    let mut state = etat_isole();
    let dir = state.project.dir.clone();

    // Trois cents notices : la liste longue de l'acceptation. Aucune ne se
    // perd, aucune ne se tronque.
    let mut bibliotheque = String::new();
    for rang in 0..300 {
        bibliotheque.push_str(&format!(
            "@book{{notice{rang}, title = {{Titre {rang}}}, author = {{Auteur {rang}}}, year = {{2020}}}}\n"
        ));
    }
    std::fs::write(dir.join("references.bib"), bibliotheque).expect("bibliographie");
    let _ = update(&mut state, Message::RefreshProject);

    assert_eq!(
        state.bibliography.len(),
        300,
        "les trois cents notices sont là, sans troncature"
    );

    // Le filtre vit sur la liste, et ne perd aucune entrée.
    let _ = update(&mut state, Message::ReferencesFiltrer("notice123".into()));
    assert_eq!(state.references_filtre, "notice123", "le filtre tient");
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn le_mode_focus_rend_la_disposition_d_avant() {
    let mut state = etat_isole();

    // La disposition d'avant, telle que le contrat la promet.
    let avant = format!("{:?}", state.espace.grille);

    let _ = update(&mut state, Message::ToggleFocusMode);
    assert!(state.layout.focus_mode, "le mode focus s'ouvre");

    let _ = update(&mut state, Message::ToggleFocusMode);
    assert!(!state.layout.focus_mode, "le mode focus se ferme");

    assert_eq!(
        format!("{:?}", state.espace.grille),
        avant,
        "sortir du mode focus rend la disposition exactement comme avant"
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn les_reglages_annules_ne_s_enregistrent_pas() {
    let mut state = etat_isole();
    let dir = state.project.dir.clone();

    // Les normes du projet, telles qu'elles sont enregistrées.
    let avant = std::fs::read_to_string(dir.join("mise-en-page.toml")).ok();

    // L'aperçu : la fenêtre ouverte, un réglage changé — le document
    // n'enregistre rien tant qu'on n'applique pas.
    let _ = update(&mut state, Message::MiseEnPageOuvrir);
    let _ = update(
        &mut state,
        Message::MiseEnPageChamp(
            crate::mise_en_page::modele::Champ::BibliographieStyle,
            "styles/aperçu.csl".to_string(),
        ),
    );

    // L'annulation : la fenêtre se ferme sans appliquer.
    let _ = update(&mut state, Message::MiseEnPageFermer);

    let apres = std::fs::read_to_string(dir.join("mise-en-page.toml")).ok();
    assert_eq!(
        avant, apres,
        "un aperçu annulé n'enregistre aucun réglage prématurément"
    );

    // Et la réouverture montre l'ancienne valeur, pas celle de l'aperçu.
    let _ = update(&mut state, Message::MiseEnPageOuvrir);
    let dialogue = state.mise_en_page.as_ref().expect("fenêtre ouverte");
    assert_ne!(
        dialogue.reglages.bibliographie.style.as_deref(),
        Some("styles/aperçu.csl"),
        "l'aperçu annulé ne devient pas un réglage"
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn le_theme_annule_ne_s_enregistre_pas() {
    let mut state = etat_isole();

    // Un thème qui a un fichier : c'est lui qu'un enregistrement
    // prématuré changerait. Ses octets d'abord.
    let fichier = state
        .themes
        .iter()
        .filter_map(|theme| theme.file.as_ref())
        .find_map(|fichier| fichier.source.clone());
    let Some(path) = fichier else {
        eprintln!("aucun thème à fichier — épreuve sautée");
        return;
    };
    let avant = std::fs::read_to_string(&path).ok();

    // L'aperçu : l'éditeur ouvert, une couleur changée dans la copie de
    // travail.
    let _ = update(&mut state, Message::ThemeEditeurOuvrir);
    let _ = update(
        &mut state,
        Message::ThemeEditeurChamp("background", "#123456".to_string()),
    );

    // L'annulation : fermer sans enregistrer.
    let _ = update(&mut state, Message::ThemeEditeurFermer);

    let apres = std::fs::read_to_string(&path).ok();
    assert_eq!(
        avant,
        apres,
        "un aperçu de thème annulé n'enregistre rien : {}",
        path.display()
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_remplacement_s_annule_d_un_geste() {
    let mut state = etat_isole();
    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::Paste(
            "un chat et un chat".into(),
        )),
    );
    let avant = state.editor.content();

    // La recherche, son motif, son remplacement — puis tout remplacer.
    let _ = update(&mut state, Message::RechercheOuvrir);
    let _ = update(&mut state, Message::RechercheMotif("chat".into()));
    let _ = update(&mut state, Message::RechercheRemplacement("chien".into()));
    let _ = update(&mut state, Message::RechercheRemplacerTout);

    let remplace = state.editor.content();
    assert_ne!(remplace, avant, "le remplacement est passé");
    assert!(!remplace.contains("chat"), "{remplace}");

    // L'aperçu avait dit le changement avant qu'il n'arrive.
    let etat = state.recherche.as_ref().expect("recherche ouverte");
    assert!(
        etat.apercu
            .iter()
            .all(|(avant, apres)| avant == "chat" && apres == "chien"),
        "l'aperçu dit le changement : {:?}",
        etat.apercu
    );

    // Une seule annulation remet tout : le remplacement est un geste.
    let _ = update(&mut state, Message::Editor(iced_code_editor::Message::Undo));
    assert_eq!(
        state.editor.content(),
        avant,
        "une annulation remplace tout d'un geste"
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn une_note_supprimee_se_retrouve_a_l_annulation() {
    let mut state = etat_isole();
    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::Paste(
            "texte^[la note exacte.] suite".into(),
        )),
    );
    let avant = state.editor.content();

    // La suppression de la note : une sélection, une frappe — le document
    // ne la perd qu'en un geste.
    let debut = avant.find("^[").expect("note");
    let fin = debut + "^[la note exacte.]".len();
    let _ = state.editor.selectionner_plage(debut, fin);
    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::Backspace),
    );
    assert!(
        !state.editor.content().contains("note exacte"),
        "la note est partie"
    );

    // L'annulation la remet, entière.
    let _ = update(&mut state, Message::Editor(iced_code_editor::Message::Undo));
    assert_eq!(
        state.editor.content(),
        avant,
        "la note supprimée revient entière"
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn les_gestes_d_insertion_placent_texte_et_curseur() {
    // Sur un document `.tw` : la note s'écrit à son panneau, et la marque
    // `^[…]` seule vient au curseur (bug-report-7, point 15).
    let mut state = etat_isole();
    let _ = update(&mut state, Message::NoteInserer);
    assert!(state.note_dialogue.is_some(), "le panneau de la note");
    let _ = update(&mut state, Message::NoteTexte("une note".to_string()));
    let _ = update(&mut state, Message::NoteValider);
    let contenu = state.editor.content();
    assert_eq!(contenu, "^[une note]");

    // Le renvoi : le `@` ouvre la complétion.
    let _ = state.editor.set_cursor_offset(state.editor.content().len());
    let _ = update(&mut state, Message::RenvoiInserer);
    let contenu = state.editor.content();
    assert!(contenu.ends_with('@'), "{contenu}");
    assert_eq!(state.editor.cursor_offset(), contenu.len());

    // L'étiquette s'écrit dans les deux dialectes : `<nom>` est la syntaxe
    // que le pipeline lit déjà comme étiquette (bug-report-7, point 7).
    let _ = update(&mut state, Message::EtiquetteInserer);
    let contenu = state.editor.content();
    assert!(contenu.ends_with(" <>"), "l'étiquette se pose : {contenu}");
    assert_eq!(
        state.editor.cursor_offset(),
        contenu.len() - ">".len(),
        "le curseur est entre les chevrons"
    );

    // Sur un chapitre `.typ` : la note parle Typst, l'étiquette entre, et le
    // curseur se pose à l'intérieur.
    state.editor.path = Some(state.project.dir.join("chapitres").join("01.typ"));
    let _ = state.editor.set_cursor_offset(state.editor.content().len());
    let _ = update(&mut state, Message::NoteInserer);
    let _ = update(&mut state, Message::NoteTexte("typst".to_string()));
    let _ = update(&mut state, Message::NoteValider);
    let contenu = state.editor.content();
    assert!(contenu.ends_with("#footnote[typst]"), "{contenu}");

    let _ = state.editor.set_cursor_offset(state.editor.content().len());
    let _ = update(&mut state, Message::EtiquetteInserer);
    let contenu = state.editor.content();
    assert!(contenu.ends_with(" <>"), "{contenu}");
    assert_eq!(
        state.editor.cursor_offset(),
        contenu.len() - ">".len(),
        "le curseur est dans les chevrons"
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_geste_cree_le_projet_de_la_base_choisie() {
    let mut state = etat_isole();

    // La base : un projet comme un autre, marqué comme modèle.
    creer_un_projet(&mut state, "mon-modele");
    let base = state.project.dir.clone();
    std::fs::write(base.join("document.tw"), "Le squelette de mes devoirs.").expect("contenu");
    let mut normes = crate::mise_en_page::charger(&base);
    normes.page_de_titre = true;
    let _ = crate::mise_en_page::appliquer(&base, &normes, true);
    let _ = update(&mut state, Message::ProjetModeleBascule);

    // Le geste : le nouveau projet naît de la base choisie.
    creer_depuis(&mut state, "un-devoir", "mon-modele");
    let neuf = state.project.dir.clone();
    assert_ne!(neuf, base, "deux projets, deux dossiers");

    let contenu = std::fs::read_to_string(neuf.join("document.tw")).expect("document");
    assert!(
        contenu.contains("Le squelette de mes devoirs."),
        "le contenu de la base suit"
    );
    let normes = crate::mise_en_page::charger(&neuf);
    assert!(normes.page_de_titre, "les normes de la base suivent");
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_projet_existant_garde_son_rendu() {
    // Un projet d'avant R5 : le gabarit classique, inchangé.
    let vieux = crate::project::create("heritage").expect("projet hérité");
    let avant = std::fs::read_to_string(vieux.join("main.typ")).expect("assemblage");

    // La réouverture — et même l'ouverture de la fenêtre des normes — ne
    // réécrit rien sans choix explicite.
    let mut state = etat_isole();
    let _ = update(&mut state, Message::OpenProject(vieux.clone()));
    let _ = update(&mut state, Message::MiseEnPageOuvrir);
    let _ = update(&mut state, Message::RefreshProject);

    let apres = std::fs::read_to_string(vieux.join("main.typ")).expect("assemblage");
    assert_eq!(
        avant, apres,
        "un projet existant garde son rendu sans choix explicite"
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_projet_relu_hors_ligne_conserve_son_style() {
    let mut state = etat_isole();
    let dir = state.project.dir.clone();

    // Un style importé dans le projet, puis choisi dans les normes.
    std::fs::create_dir_all(dir.join("styles")).expect("styles");
    let csl = r#"<?xml version="1.0" encoding="utf-8"?>
<style xmlns="http://purl.org/net/xbiblio/csl" version="1.0" class="note">
  <info>
    <id>https://example.org/styles/mien</id>
    <title>Le mien</title>
    <updated>2026-01-01T00:00:00+00:00</updated>
  </info>
  <citation><layout><text variable="title"/></layout></citation>
  <bibliography><layout><text variable="title"/></layout></bibliography>
</style>"#;
    std::fs::write(dir.join("styles").join("mien.csl"), csl).expect("style");

    let _ = update(&mut state, Message::MiseEnPageOuvrir);
    let _ = update(
        &mut state,
        Message::MiseEnPageChamp(
            crate::mise_en_page::modele::Champ::BibliographieStyle,
            "styles/mien.csl".to_string(),
        ),
    );
    let _ = update(&mut state, Message::MiseEnPageAppliquer);

    // Le redémarrage, hors ligne : un état neuf sur le même projet. Le
    // style vient du disque — aucune requête, aucun réseau, aucun Zotero.
    let mut seconde = etat_isole();
    let _ = update(&mut seconde, Message::MiseEnPageOuvrir);

    let dialogue = seconde.mise_en_page.as_ref().expect("fenêtre ouverte");
    assert_eq!(
        dialogue.reglages.bibliographie.style.as_deref(),
        Some("styles/mien.csl"),
        "le choix survit au redémarrage"
    );
    assert!(
        dialogue
            .apercu_style
            .as_ref()
            .is_some_and(|texte| !texte.is_empty()),
        "l'aperçu se rend hors ligne : {:?}",
        dialogue.apercu_style
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn une_coupure_zotero_n_ecrit_rien_et_le_dit() {
    let mut state = etat_isole();
    let dir = state.project.dir.clone();

    // Zotero tombe pendant la requête : la réponse est une erreur.
    let _ = update(
        &mut state,
        Message::ZoteroExport(
            "collection".into(),
            "Test".into(),
            dir.clone(),
            Err("connexion coupée".into()),
        ),
    );

    // Rien n'est écrit, et le problème se dit.
    assert!(!dir.join("references-zotero.bib").exists());
    assert!(!dir.join("citations.json").exists());
    assert!(!dir.join("zotero.json").exists());
    let liaison = state.liaison.as_ref().expect("état de liaison");
    let crate::app::state::LiaisonZotero::Erreur { motif, reglable } = liaison else {
        panic!("une erreur est annoncée : {liaison:?}")
    };
    assert!(motif.contains("coupée"), "{motif}");
    assert!(!reglable, "un réseau coupé ne se règle pas dans l'app");
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_echec_de_stockage_ne_laisse_rien() {
    let mut state = etat_isole();
    let dir = state.project.dir.clone();

    // Le stockage refuse : un dossier occupe la place du trousseau. La
    // transaction doit renoncer **en entier**, pas à moitié.
    std::fs::create_dir_all(dir.join("citations.json")).expect("obstacle");

    let _ = update(
        &mut state,
        Message::ZoteroExport(
            "collection".into(),
            "Test".into(),
            dir.clone(),
            Ok("@book{test, title={Test}}".into()),
        ),
    );

    assert!(
        !dir.join("references-zotero.bib").exists() && !dir.join("zotero.json").exists(),
        "rien n'est écrit quand la transaction échoue"
    );
    let liaison = state.liaison.as_ref().expect("état de liaison");
    let crate::app::state::LiaisonZotero::Erreur { motif, reglable: _ } = liaison else {
        panic!("une erreur est annoncée : {liaison:?}")
    };
    assert!(
        motif.contains("rien n'a changé"),
        "l'absence de changement se dit : {motif}"
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_projet_sans_verrou_ne_peut_pas_appliquer_un_export_zotero() {
    let premiere = etat_isole();
    let bib = premiere.project.dir.join("references-zotero.bib");
    let mut seconde = etat_isole();
    let dir = seconde.project.dir.clone();
    let _ = update(
        &mut seconde,
        Message::ZoteroExport(
            "collection".into(),
            "Test".into(),
            dir,
            Ok("@book{test, title={Test}}".into()),
        ),
    );
    assert!(
        !bib.exists(),
        "une réponse tardive ne doit pas écrire dans le projet"
    );
    assert!(seconde.note.as_ref().is_some_and(|note| note.erreur));
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_projet_sans_verrou_ne_peut_pas_creer_de_chapitre() {
    let premiere = etat_isole();
    let main = premiere.project.main.clone();
    let avant = std::fs::read(&main).expect("assemblage initial");
    let mut seconde = etat_isole();
    let _ = update(&mut seconde, Message::NewChapter);
    assert_eq!(std::fs::read(&main).expect("assemblage"), avant);
    assert!(seconde.note.as_ref().is_some_and(|note| note.erreur));
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_projet_sans_verrou_ne_peut_pas_appliquer_de_norme() {
    let premiere = etat_isole();
    let norme = premiere.project.dir.join(crate::mise_en_page::REGLAGES);
    let avant = std::fs::read(&norme).ok();
    let mut seconde = etat_isole();
    let _ = update(&mut seconde, Message::MiseEnPageOuvrir);
    let _ = update(&mut seconde, Message::MiseEnPageAppliquer);
    assert_eq!(std::fs::read(&norme).ok(), avant);
    assert!(seconde.note.as_ref().is_some_and(|note| note.erreur));
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_projet_sans_verrou_ne_peut_pas_importer_un_fragment() {
    let premiere = etat_isole();
    let document = premiere.project.dir.join("document.tw");
    let avant = std::fs::read(&document).expect("document initial");
    let source = PathBuf::from(std::env::var_os("HOME").unwrap()).join("fragment.typ");
    let contenu = "= Fragment\n\nTexte à importer.\n";
    std::fs::write(&source, contenu).expect("fragment synthétique");
    let mut seconde = etat_isole();
    let _ = update(
        &mut seconde,
        Message::FileOpened(Some((source, contenu.into()))),
    );
    assert_eq!(std::fs::read(&document).expect("document"), avant);
    assert!(seconde.note.as_ref().is_some_and(|note| note.erreur));
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn une_image_homonyme_ne_crase_pas_la_premiere() {
    let mut state = etat_isole();
    let premiere = state.project.dir.join("images").join("photo.png");
    std::fs::create_dir_all(premiere.parent().expect("dossier images")).expect("dossier images");
    std::fs::write(&premiere, "PREMIERE").expect("première image");
    let source = state
        .project
        .dir
        .parent()
        .expect("le projet a un parent")
        .join("photo.png");
    std::fs::write(&source, "SECONDE").expect("image source");

    let _ = update(&mut state, Message::ImageChoisie(Some(source)));

    assert_eq!(
        std::fs::read_to_string(&premiere).expect("première image"),
        "PREMIERE",
        "la première image est intacte"
    );
    let seconde = state.project.dir.join("images").join("photo-2.png");
    assert_eq!(
        std::fs::read_to_string(&seconde).expect("la copie"),
        "SECONDE",
        "la nouvelle image est la copie, sous un nom qui n'appartient à personne"
    );
    assert!(
        state.editor.content().contains("photo-2.png"),
        "le document cite la copie : {}",
        state.editor.content()
    );
}

#[cfg(unix)]
#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn importer_une_image_refuse_un_dossier_images_pointant_hors_du_projet() {
    use std::os::unix::fs::symlink;
    let mut state = etat_isole();
    let dehors = PathBuf::from(std::env::var_os("HOME").unwrap()).join("exterieur");
    std::fs::create_dir(&dehors).expect("dossier synthétique extérieur");
    symlink(&dehors, state.project.dir.join("images")).expect("lien sortant");
    let source = PathBuf::from(std::env::var_os("HOME").unwrap()).join("figure.png");
    std::fs::write(&source, "IMAGE").expect("source synthétique");
    let avant = state.editor.content();
    let _ = update(&mut state, Message::ImageChoisie(Some(source)));
    assert!(!dehors.join("figure.png").exists());
    assert_eq!(state.editor.content(), avant);
    assert!(state.note.as_ref().is_some_and(|note| note.erreur));
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn importer_une_image_necrase_pas_une_image_de_meme_nom() {
    let mut state = etat_isole();
    let images = state.project.dir.join("images");
    std::fs::create_dir_all(&images).expect("images du projet");
    let existante = images.join("figure.png");
    std::fs::write(&existante, "IMAGE EXISTANTE").expect("image initiale");
    let source = PathBuf::from(std::env::var_os("HOME").unwrap()).join("figure.png");
    std::fs::write(&source, "NOUVELLE IMAGE").expect("image source");
    let _ = update(&mut state, Message::ImageChoisie(Some(source)));
    assert_eq!(
        std::fs::read_to_string(existante).unwrap(),
        "IMAGE EXISTANTE"
    );
    assert_eq!(
        std::fs::read_to_string(images.join("figure-2.png")).unwrap(),
        "NOUVELLE IMAGE"
    );
    assert!(state.editor.content().contains("images/figure-2.png"));
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_projet_sans_verrou_ne_peut_pas_importer_une_image() {
    let premiere = etat_isole();
    let chemin = PathBuf::from(std::env::var_os("HOME").unwrap()).join("image.png");
    std::fs::write(&chemin, "image synthétique").expect("image");
    let destination = premiere.project.dir.join("images/image.png");
    let mut seconde = etat_isole();
    let _ = update(&mut seconde, Message::ImageChoisie(Some(chemin)));
    assert!(!destination.exists(), "l'image ne doit pas être copiée");
    assert!(seconde.note.as_ref().is_some_and(|note| note.erreur));
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn une_notice_supprimee_vraiment_a_la_corbeille() {
    let mut state = etat_isole();
    let bib = state.project.dir.join("references.bib");
    let contenu = "@book{exemple, title = {À conserver}}\n";
    std::fs::write(&bib, contenu).expect("bibliographie synthétique");
    state.geste_confirme = true;
    let _ = update(&mut state, Message::ReferenceSupprimer("exemple".into()));
    assert!(
        !std::fs::read_to_string(&bib)
            .expect("bibliographie")
            .contains("exemple"),
        "la notice quitte la bibliographie"
    );
    let corbeille = crate::project::corbeille();
    let conservee = std::fs::read_dir(&corbeille)
        .expect("corbeille")
        .filter_map(Result::ok)
        .map(|entree| std::fs::read_to_string(entree.path()).unwrap_or_default())
        .any(|texte| texte.contains("À conserver"));
    assert!(conservee, "la notice est vraiment à la corbeille");
    assert!(
        state.derniere_notice_supprimee.is_some(),
        "le chemin de la copie est dit, pour se raviser"
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn une_notice_supprimee_se_restaure() {
    let mut state = etat_isole();
    let bib = state.project.dir.join("references.bib");
    let contenu = "@book{exemple, title = {À conserver}}\n";
    std::fs::write(&bib, contenu).expect("bibliographie synthétique");
    state.geste_confirme = true;
    let _ = update(&mut state, Message::ReferenceSupprimer("exemple".into()));
    let copie = state.derniere_notice_supprimee.clone().expect("chemin dit");
    let _ = update(&mut state, Message::ReferenceRestaurer(copie));
    assert!(
        std::fs::read_to_string(&bib)
            .expect("bibliographie")
            .contains("exemple"),
        "la notice est de retour dans la bibliographie"
    );
    assert!(
        state
            .bibliography
            .iter()
            .any(|entree| entree.key == "exemple"),
        "la bibliographie la voit"
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_projet_sans_verrou_ne_peut_pas_supprimer_une_reference() {
    let premiere = etat_isole();
    let bib = premiere.project.dir.join("references.bib");
    let contenu = "@book{exemple, title = {À conserver}}\n";
    std::fs::write(&bib, contenu).expect("bibliographie synthétique");
    let mut seconde = etat_isole();
    assert!(!seconde.espace.verrous.contains_key(&seconde.project.dir));
    seconde.geste_confirme = true;
    let _ = update(&mut seconde, Message::ReferenceSupprimer("exemple".into()));
    assert_eq!(
        std::fs::read_to_string(&bib).expect("bibliographie"),
        contenu
    );
    assert!(seconde.note.as_ref().is_some_and(|note| note.erreur));
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn fermer_la_fenetre_sauvegarde_tous_les_documents_modifies() {
    let mut state = etat_isole();
    state.window = Some(iced::window::Id::unique());
    let premier = state.editor.path.clone().expect("premier document");
    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::Paste("PREMIER ".into())),
    );
    let premier_contenu = state.editor.content();
    let nouveau = crate::project::create("second").expect("second projet");
    state.maj_tenu = true;
    let _ = update(&mut state, Message::OpenProject(nouveau));
    state.maj_tenu = false;
    let second = state.editor.path.clone().expect("second document");
    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::Paste("SECOND ".into())),
    );
    let second_contenu = state.editor.content();
    let fermeture = super::super::message_evenement(
        iced::event::Event::Window(iced::window::Event::CloseRequested),
        iced::event::Status::Ignored,
    )
    .expect("la fermeture est interceptée");
    let tache = update(&mut state, fermeture);
    assert_eq!(std::fs::read_to_string(premier).unwrap(), premier_contenu);
    assert_eq!(std::fs::read_to_string(second).unwrap(), second_contenu);
    assert!(
        tache.units() > 0,
        "la fenêtre peut se fermer après sauvegarde"
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn fermer_la_fenetre_garde_le_tampon_si_la_sauvegarde_echoue() {
    let mut state = etat_isole();
    state.window = Some(iced::window::Id::unique());
    let chemin = state.editor.path.clone().expect("document");
    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::Paste("A_CONSERVER ".into())),
    );
    let contenu = state.editor.content();
    std::fs::rename(&chemin, chemin.with_extension("original"))
        .expect("mettre le fichier à l'abri");
    std::fs::create_dir(&chemin).expect("simuler un échec de remplacement");
    let fermeture = super::super::message_evenement(
        iced::event::Event::Window(iced::window::Event::CloseRequested),
        iced::event::Status::Ignored,
    )
    .expect("la fermeture est interceptée");
    let tache = update(&mut state, fermeture);
    assert_eq!(state.editor.content(), contenu, "le tampon reste intact");
    assert!(state.editor.is_modified(), "la modification reste signalée");
    assert_eq!(tache.units(), 0, "l'échec interdit la fermeture");
    assert!(state.note.is_some(), "l'erreur est signalée");
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn la_migration_a_plat_ne_touche_pas_un_projet_existant_verrouille() {
    let premiere = etat_isole();
    let dossier = premiere.project.dir.clone();
    let source = crate::project::work_dir().join("ancien.typ");
    let assemblage = std::fs::read(&premiere.project.main).expect("assemblage initial");
    std::fs::write(&source, "= Ancien\n\nTexte à conserver.\n").expect("source synthétique");

    let seconde = etat_isole();
    assert!(
        source.exists(),
        "la source à plat ne doit pas être déplacée"
    );
    assert_eq!(
        std::fs::read(dossier.join("main.typ")).expect("assemblage"),
        assemblage
    );
    drop(seconde);
    drop(premiere);
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn la_migration_ne_modifie_pas_un_projet_verrouille() {
    let premiere = etat_isole();
    let dossier = premiere.project.dir.clone();
    let document = dossier.join("document.tw");
    if document.exists() {
        std::fs::remove_file(&document).expect("retirer le document de la fixture");
    }
    std::fs::create_dir_all(dossier.join("chapitres")).expect("chapitres");
    std::fs::write(
        dossier.join("chapitres/01-ancien.typ"),
        "= Ancien\n\nTexte.\n",
    )
    .expect("chapitre synthétique");
    std::fs::write(
        dossier.join("main.typ"),
        "#include \"chapitres/01-ancien.typ\"\n",
    )
    .expect("assemblage synthétique");

    let seconde = etat_isole();
    assert!(
        !document.exists(),
        "une seconde instance ne doit pas migrer un projet déjà verrouillé"
    );
    assert!(seconde.verrou.is_none() && !seconde.espace.verrous.contains_key(&dossier));
    drop(seconde);
    drop(premiere);
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn les_documents_les_vues_et_la_session_restent_coherents() {
    let mut state = etat_isole();
    let premier = state.espace.active;
    let premier_path = state.editor.path.clone().unwrap();
    let premiere_source = state.editor.content();
    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::MouseClick(iced::Point::ORIGIN)),
    );
    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::Paste("PARTAGE_UNIQUE ".into())),
    );
    let source = state.editor.content();
    assert!(source.contains("PARTAGE_UNIQUE"));
    let _ = update(&mut state, Message::VueDupliquer);
    let double = state.espace.active;
    assert_ne!(premier, double);
    assert_eq!(state.editor.content(), source);
    assert_eq!(state.espace.vues[&premier].editor.content(), source);
    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::MouseClick(iced::Point::ORIGIN)),
    );
    let _ = update(&mut state, Message::Editor(iced_code_editor::Message::Undo));
    assert_eq!(state.editor.content(), premiere_source);
    assert_eq!(
        state.espace.vues[&premier].editor.content(),
        premiere_source
    );
    let _ = update(&mut state, Message::Editor(iced_code_editor::Message::Redo));
    assert_eq!(state.espace.vues[&premier].editor.content(), source);
    let _ = update(&mut state, Message::ModeBasculer);
    assert_eq!(state.mode, ModeVue::Rendu);
    assert_eq!(state.espace.vues[&premier].mode, ModeVue::Edition);
    let pane = state.espace.focus;
    state
        .espace
        .deplacer(double, pane, Some(iced::widget::pane_grid::Edge::Left));
    state.choisir_vue(double);
    assert_eq!(state.espace.grille.len(), 2);
    // Ouvrir explicitement le second projet dans un nouvel onglet :
    // NewProject remplace la vue active depuis le changement NAV-06-01.
    let nouveau = crate::project::create("projet de test").expect("second projet synthétique");
    state.maj_tenu = true;
    let _ = update(&mut state, Message::OpenProject(nouveau));
    state.maj_tenu = false;
    let second_projet = state.project.dir.clone();
    let second = state.espace.active;
    assert_ne!(state.editor.path.as_ref(), Some(&premier_path));
    let deuxieme_source = state.editor.content();
    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::MouseClick(iced::Point::ORIGIN)),
    );
    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::Paste("AUTRE_PROJET ".into())),
    );
    assert!(state.editor.content().contains("AUTRE_PROJET"));
    assert_eq!(state.espace.vues[&premier].editor.content(), source);
    // Un retour asynchrone ne change ni le projet actif ni son texte.
    let _ = update(
        &mut state,
        Message::RetourVue(premier, Box::new(Message::ModeTypst)),
    );
    assert_eq!(state.espace.active, second);
    assert_eq!(state.project.dir, second_projet);
    assert_eq!(state.espace.vues[&premier].mode, ModeVue::Typst);
    assert!(!state.espace.vues[&premier].typst.is_empty());
    let _ = update(&mut state, Message::SaveTick);
    assert_eq!(std::fs::read_to_string(&premier_path).unwrap(), source);
    assert!(
        std::fs::read_to_string(state.editor.path.as_ref().unwrap())
            .unwrap()
            .contains("AUTRE_PROJET")
    );
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
    println!(
        "[espace] 2 projets, 3 vues, 2 panneaux : source/undo partagés, modes indépendants, retour asynchrone routé, sauvegarde et restauration vérifiés"
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_import_de_sauvegarde_se_questionne_avant_de_remplacer() {
    let mut state = etat_isole();

    // Le projet ouvert a déjà son document — c'est lui qui sera en
    // collision avec la sauvegarde.
    std::fs::write(state.project.dir.join("document.tw"), "CE QUE J'AVAIS").expect("document");

    // Le projet de l'archive porte le nom du projet ouvert : à l'import, la
    // destination est le projet lui-même — collision assurée.
    let nom = state
        .project
        .dir
        .file_name()
        .map(|nom| nom.to_string_lossy().to_string())
        .expect("nom du projet");
    let racine = std::env::temp_dir().join(format!("tw-portable-question-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&racine);
    let source = racine.join(&nom);
    std::fs::create_dir_all(&source).expect("source");
    std::fs::write(source.join("document.tw"), "CE QUE L'ARCHIVE PORTE").expect("document");
    let archive =
        std::env::temp_dir().join(format!("tw-portable-question-{}.zip", std::process::id()));
    let _ = std::fs::remove_file(&archive);
    crate::project::portable::exporter_projet(&source, &archive).expect("export");

    // L'import entre : collision sur document.tw, donc question — et rien ne
    // bouge tant que la réponse n'est pas venue.
    let _ = update(
        &mut state,
        Message::ProjetImportChoisi(Some(archive.clone())),
    );
    assert!(state.confirmation.is_some(), "la question est la barrière");
    let avant = std::fs::read_to_string(state.project.dir.join("document.tw")).expect("document");
    assert_eq!(avant, "CE QUE J'AVAIS", "rien n'est écrasé sans le mot");

    // Le mot venu : le remplacement passe par la corbeille, réversible.
    let destination = crate::project::projects_dir().join(&nom);
    let _ = update(
        &mut state,
        Message::ProjetImportConfirmer(archive, destination),
    );
    let apres = std::fs::read_to_string(state.project.dir.join("document.tw")).expect("document");
    assert_eq!(
        apres, "CE QUE L'ARCHIVE PORTE",
        "le mot venu, la sauvegarde remplace"
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_export_markdown_ecrit_le_document_canonique() {
    let mut state = etat_isole();

    let source = "# Un titre\n\nUn paragraphe avec une note^[la note] et @dupont2023[p. 45].\n";
    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::Paste(source.into())),
    );

    let cible = std::env::temp_dir().join(format!("tw-export-md-{}.md", std::process::id()));
    let _ = std::fs::remove_file(&cible);
    let _ = update(
        &mut state,
        Message::ExportMarkdownChoisi(Some(cible.clone())),
    );

    let ecrit = std::fs::read_to_string(&cible).expect("markdown");
    assert_eq!(
        crate::langage::parse::parse(&ecrit),
        crate::langage::parse::parse(source),
        "le Markdown exporté redonne le même document"
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_export_docx_dit_le_geste_ou_ecrit_le_docx() {
    let mut state = etat_isole();

    let source = "# Un titre\n\nUn paragraphe avec une note^[la note].\n";
    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::Paste(source.into())),
    );

    let cible = std::env::temp_dir().join(format!("tw-export-docx-{}.docx", std::process::id()));
    let _ = std::fs::remove_file(&cible);
    let _ = update(&mut state, Message::ExportDocxChoisi(Some(cible.clone())));

    if crate::langage::export::soffice_disponible() {
        let octets = std::fs::read(&cible).expect("docx");
        assert!(!octets.is_empty(), "le DOCX a du contenu");
        let note = state.note.as_ref().expect("une note");
        assert!(
            note.texte.contains("[note:"),
            "la limite est dite : {}",
            note.texte
        );
    } else {
        // Sans LibreOffice, l'état d'erreur propose son remède : le geste,
        // pas seulement le nom du réglage.
        let note = state.note.as_ref().expect("une note");
        assert!(note.erreur, "c'est une erreur : {}", note.texte);
        assert!(note.texte.contains("LibreOffice"), "{}", note.texte);
        assert!(note.texte.contains("Markdown"), "{}", note.texte);
    }
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn l_aide_se_bascule_et_dit_le_chemin_des_donnees() {
    let mut state = etat_isole();

    // F1 (plan_touche) porte ce message : la fenêtre d'aide se bascule.
    assert!(!state.aide);
    let _ = update(&mut state, Message::AideBasculer);
    assert!(state.aide, "l'aide s'ouvre");
    let _ = update(&mut state, Message::AideBasculer);
    assert!(!state.aide, "l'aide se ferme");

    // Le chemin qu'elle montre est celui des données : réel, créé, et
    // reconnaissable à l'œil.
    let chemin = crate::project::work_dir();
    assert!(chemin.is_dir(), "le dossier des données existe");
    assert!(
        chemin.to_string_lossy().contains("typst-writer"),
        "le chemin est reconnaissable : {}",
        chemin.display()
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_glissement_de_separateur_regle_la_disposition() {
    let mut state = etat_isole();

    // L'origine du glissement est la position réelle du clic.
    let _ = update(
        &mut state,
        Message::MouseMoved(iced::Point::new(200.0, 100.0)),
    );
    let depart = state.layout.sidebar_width;
    let _ = update(
        &mut state,
        Message::SplitterPressed(crate::app::state::Splitter::Sidebar),
    );
    assert!(state.layout.dragging.is_some(), "le séparateur est tenu");

    // Le glissement suit le delta depuis l'origine — jamais cumulé.
    let _ = update(
        &mut state,
        Message::MouseMoved(iced::Point::new(250.0, 100.0)),
    );
    let apres_50 = state.layout.sidebar_width;
    assert!(
        (apres_50 - (depart + 50.0)).abs() < 0.5,
        "50 px de plus : {apres_50} pour {depart}"
    );
    let _ = update(
        &mut state,
        Message::MouseMoved(iced::Point::new(260.0, 100.0)),
    );
    assert!(
        (state.layout.sidebar_width - (depart + 60.0)).abs() < 0.5,
        "le delta part de l'origine, pas de la valeur courante"
    );

    // Les bornes tiennent : le séparateur ne va pas écraser la disposition.
    let _ = update(
        &mut state,
        Message::MouseMoved(iced::Point::new(-5000.0, 100.0)),
    );
    assert!(
        state.layout.sidebar_width >= 1.0,
        "la borne tient : {}",
        state.layout.sidebar_width
    );

    // Le relâchement rend la main.
    let _ = update(&mut state, Message::SplitterReleased);
    assert!(state.layout.dragging.is_none(), "le séparateur est relâché");
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn l_aide_dit_le_diagnostic_et_la_recuperation() {
    let mut state = etat_isole();

    // Le diagnostic du poste, en une ligne — chaque état dit son remède.
    let resume = crate::ui::aide::diagnostic_resume(&state);
    assert!(resume.contains(env!("CARGO_PKG_VERSION")), "{resume}");
    assert!(resume.contains("diagnostic(s)"), "{resume}");
    assert!(resume.contains("rendu"), "la promesse est dite : {resume}");
    assert!(
        resume.contains("l'écriture reste entière"),
        "le remède est dit : {resume}"
    );
    assert!(resume.contains("bibliothèque"), "{resume}");

    // Les deux gestes de récupération sont portés par l'aide.
    let _ = update(&mut state, Message::AideBasculer);
    assert!(state.aide, "l'aide ouvre le centre");
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_devoir_se_produit_et_se_recupere() {
    let mut state = etat_isole();
    let chemin = state.editor.path.clone().expect("document");

    // Produire : le devoir s'écrit et s'enregistre.
    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::Paste("LE DEVOIR PREMIER".into())),
    );
    let _ = update(&mut state, Message::SaveTick);
    assert_eq!(
        std::fs::read_to_string(&chemin).expect("document"),
        "LE DEVOIR PREMIER",
        "le devoir est écrit"
    );

    // Le coup du sort : une autre main touche le fichier pendant ce temps.
    std::fs::write(&chemin, "CE QUI A ÉTÉ PERDU").expect("changement extérieur");
    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::Paste(" — LA SUITE".into())),
    );
    let _ = update(&mut state, Message::SaveTick);
    // La sauvegarde refuse le conflit — et le confirmer garde ce qui allait
    // être remplacé.
    let _ = update(&mut state, Message::ConfirmationConfirmer);
    assert_eq!(
        std::fs::read_to_string(&chemin).expect("document"),
        state.editor.content(),
        "le travail de l'utilisateur est écrit"
    );

    // Récupérer : ce qui a été perdu se relit dans les sauvegardes.
    let sauvegardes = crate::project::sauvegardes_dir(&state.project.dir);
    let retrouve = std::fs::read_dir(&sauvegardes)
        .map(|entrees| {
            entrees.filter_map(Result::ok).any(|entree| {
                std::fs::read_to_string(entree.path())
                    .map(|texte| texte.contains("CE QUI A ÉTÉ PERDU"))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false);
    assert!(
        retrouve,
        "la version perdue se récupère dans {}",
        sauvegardes.display()
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn les_boutons_de_format_agissent_sur_le_bloc() {
    let mut state = etat_isole();

    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::Paste("Une ligne".into())),
    );
    let depart = state.editor.content();
    eprintln!("SONDAGE départ={depart:?}");

    let _ = update(
        &mut state,
        Message::Format(crate::app::message::FormatAction::ListePuce),
    );
    eprintln!("SONDAGE puce={:?}", state.editor.content());

    let _ = update(
        &mut state,
        Message::Format(crate::app::message::FormatAction::ListeNumero),
    );
    eprintln!("SONDAGE numero={:?}", state.editor.content());

    let _ = update(
        &mut state,
        Message::Format(crate::app::message::FormatAction::CitationBloc),
    );
    eprintln!("SONDAGE citation={:?}", state.editor.content());

    let _ = update(
        &mut state,
        Message::Format(crate::app::message::FormatAction::Titre(2)),
    );
    eprintln!("SONDAGE titre={:?}", state.editor.content());

    assert_ne!(state.editor.content(), depart, "quelque chose a bougé");
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_bloc_vide_se_formate_en_puce() {
    // L'éditeur vient de naître : le document est vide — c'est le cas des
    // projets neufs et des modèles. Le bouton y écrit la marque, il ne se
    // tait jamais.
    let mut state = etat_isole();
    assert_eq!(state.editor.content(), "");
    let _ = update(
        &mut state,
        Message::Format(crate::app::message::FormatAction::ListePuce),
    );
    assert_eq!(state.editor.content(), "- ", "la puce s'écrit");
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_bloc_vide_se_formate_en_numero() {
    let mut state = etat_isole();
    let _ = update(
        &mut state,
        Message::Format(crate::app::message::FormatAction::ListeNumero),
    );
    assert_eq!(state.editor.content(), "1. ", "le numéro s'écrit");
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_bloc_vide_se_formate_en_citation() {
    let mut state = etat_isole();
    let _ = update(
        &mut state,
        Message::Format(crate::app::message::FormatAction::CitationBloc),
    );
    assert_eq!(state.editor.content(), "> ", "la citation s'écrit");
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_bloc_vide_se_formate_en_titre() {
    let mut state = etat_isole();
    let _ = update(
        &mut state,
        Message::Format(crate::app::message::FormatAction::Titre(1)),
    );
    assert_eq!(state.editor.content(), "# ", "le titre s'écrit");
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn le_bouton_document_ecriture_agit_toujours() {
    let mut state = etat_isole();

    // Cas 1 : le document d'écriture est déjà sous les yeux — le bouton le
    // dit, il ne se tait jamais.
    let _ = update(&mut state, Message::NewFile);
    let note = state.note.as_ref().expect("une note");
    assert!(
        note.texte.contains("déjà sous les yeux"),
        "le geste répond : {}",
        note.texte
    );

    // Cas 2 : ailleurs dans le projet — un chapitre — le bouton ramène au
    // document d'écriture.
    let dir = state.project.dir.clone();
    std::fs::create_dir_all(dir.join("chapitres")).expect("chapitres");
    std::fs::write(dir.join("chapitres/ch.typ"), "AUTRE FICHIER").expect("chapitre");
    state.project = crate::project::load(&dir);
    state.open_document(
        dir.join("chapitres/ch.typ"),
        "AUTRE FICHIER".to_string(),
        false,
    );
    assert!(
        state
            .editor
            .path
            .as_ref()
            .is_some_and(|p| p.ends_with("ch.typ")),
        "le chapitre est sous les yeux"
    );
    let _ = update(&mut state, Message::NewFile);
    assert!(
        state
            .editor
            .path
            .as_ref()
            .is_some_and(|p| p.ends_with("document.tw")),
        "le document d'écriture revient : {:?}",
        state.editor.path
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn le_bouton_etiquette_pose_une_etiquette() {
    let mut state = etat_isole();

    // Le geste écrit la marque `<…>` dans les deux dialectes — c'est la
    // syntaxe que le pipeline lit déjà comme étiquette, et qu'un renvoi
    // `@fig:nom` trouve.
    let _ = update(&mut state, Message::EtiquetteInserer);
    let contenu = state.editor.content();
    assert!(
        contenu.contains('<') && contenu.contains('>'),
        "l'étiquette se pose : {contenu:?}"
    );
    assert!(
        state.note.is_none(),
        "plus de refus périmé : la syntaxe existe"
    );
}

/// **Bug-report-8, T6.1-T6.2** : la pastille d'une couleur ouvre la roue
/// pour ce champ-là, et la couleur choisie s'écrit dans les normes —
/// section « Couleurs » comme « Défauts du décor », le même contrôle.
#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
/// **Bug-report-8, T9.1** : les contrôles d'une même section cohabitent
/// — la grille des titres règle des niveaux indépendants, les mesures
/// refusent le négatif, la langue des dates s'écrit en code.
fn une_section_se_regle_dans_son_entier() {
    use crate::mise_en_page::modele::{Champ, normaliser_mesure};

    let mut state = etat_isole();
    let _ = update(&mut state, Message::MiseEnPageOuvrir);

    // La grille des titres : chaque niveau reste indépendant.
    let _ = update(
        &mut state,
        Message::MiseEnPageChamp(Champ::TitresTaille2, "15".into()),
    );
    let _ = update(
        &mut state,
        Message::MiseEnPageChamp(Champ::TitresGras2, "false".into()),
    );
    let _ = update(
        &mut state,
        Message::MiseEnPageChamp(Champ::TitresItalique2, "true".into()),
    );
    let _ = update(
        &mut state,
        Message::MiseEnPageChamp(Champ::TitresSouligne3, "true".into()),
    );
    let _ = update(
        &mut state,
        Message::MiseEnPageChamp(Champ::TitresEspacement1, "0.5".into()),
    );

    let dialogue = state.mise_en_page.as_ref().expect("le dialogue");
    assert_eq!(
        dialogue.reglages.valeur(Champ::TitresTaille2),
        "15em",
        "une taille nue devient une mesure"
    );
    assert_eq!(dialogue.reglages.valeur(Champ::TitresGras2), "false");
    assert_eq!(dialogue.reglages.valeur(Champ::TitresItalique2), "true");
    assert_eq!(dialogue.reglages.valeur(Champ::TitresSouligne3), "true");
    assert_eq!(
        dialogue.reglages.valeur(Champ::TitresEspacement1),
        "0.5em",
        "un nombre nu devient une mesure"
    );
    assert_eq!(
        dialogue.reglages.valeur(Champ::TitresItalique1),
        "",
        "le niveau 1 n'est pas touché par le niveau 2"
    );

    // Le négatif refuse, à la mesure comme à la molette.
    let avant = dialogue.reglages.valeur(Champ::TitresEspacement1);
    let _ = update(
        &mut state,
        Message::MiseEnPageChamp(Champ::TitresEspacement1, "-2em".into()),
    );
    let _ = update(
        &mut state,
        Message::MiseEnPageMolette(Champ::TitresEspacement1, -1),
    );
    let dialogue = state.mise_en_page.as_ref().expect("le dialogue");
    assert_eq!(
        dialogue.reglages.valeur(Champ::TitresEspacement1),
        avant,
        "le négatif ne passe pas"
    );
    assert!(
        normaliser_mesure("-2em").is_none(),
        "le cœur le refuse aussi"
    );

    // La langue des dates s'écrit en code, pas en libellé.
    let _ = update(
        &mut state,
        Message::MiseEnPageChamp(Champ::DatesLangue, "de".into()),
    );
    let dialogue = state.mise_en_page.as_ref().expect("le dialogue");
    assert_eq!(dialogue.reglages.valeur(Champ::DatesLangue), "de");

    // Tout cela cohabite dans un même modèle.
    let gabarit = crate::mise_en_page::gabarit::generer(&dialogue.reglages);
    assert!(gabarit.contains("0.5em"), "l'espacement part : {gabarit}");
    assert!(gabarit.contains("style: \"italic\""), "l'italique part");
    assert!(gabarit.contains("underline"), "le souligné part");
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn la_pastille_d_une_couleur_ecrit_sa_valeur() {
    use crate::mise_en_page::modele::Champ;

    let mut state = etat_isole();
    let _ = update(&mut state, Message::MiseEnPageOuvrir);

    // La section « Couleurs » : le texte se teinte de la roue.
    let _ = update(
        &mut state,
        Message::PickerCouleurPourReglage(Champ::CouleurTexte),
    );
    let _ = update(&mut state, Message::PickerCouleurHex("#3498db".to_string()));
    let _ = update(&mut state, Message::PickerCouleurAppliquer);
    let dialogue = state.mise_en_page.as_ref().expect("le dialogue");
    let attendu = {
        let (t, s, v) = crate::ui::picker_couleur::hex_vers_hsv("#3498db").expect("le hex");
        crate::ui::picker_couleur::hsv_vers_hex(t, s, v)
    };
    assert_eq!(
        dialogue.reglages.valeur(Champ::CouleurTexte),
        attendu,
        "la couleur choisie s'écrit dans les normes"
    );

    // Le décor reçoit le même traitement.
    let _ = update(
        &mut state,
        Message::PickerCouleurPourReglage(Champ::DecorRemplissage),
    );
    let _ = update(&mut state, Message::PickerCouleurHex("#e74c3c".to_string()));
    let _ = update(&mut state, Message::PickerCouleurAppliquer);
    let dialogue = state.mise_en_page.as_ref().expect("le dialogue");
    let attendu = {
        let (t, s, v) = crate::ui::picker_couleur::hex_vers_hsv("#e74c3c").expect("le hex");
        crate::ui::picker_couleur::hsv_vers_hex(t, s, v)
    };
    assert_eq!(
        dialogue.reglages.valeur(Champ::DecorRemplissage),
        attendu,
        "le décor se règle de la même roue"
    );

    // Un niveau de titre se teinte de la même roue (bug-report-8, T2.1).
    let _ = update(
        &mut state,
        Message::PickerCouleurPourReglage(Champ::TitresCouleur1),
    );
    let _ = update(&mut state, Message::PickerCouleurHex("#27ae60".to_string()));
    let _ = update(&mut state, Message::PickerCouleurAppliquer);
    let dialogue = state.mise_en_page.as_ref().expect("le dialogue");
    let attendu = {
        let (t, s, v) = crate::ui::picker_couleur::hex_vers_hsv("#27ae60").expect("le hex");
        crate::ui::picker_couleur::hsv_vers_hex(t, s, v)
    };
    assert_eq!(
        dialogue.reglages.valeur(Champ::TitresCouleur1),
        attendu,
        "chaque niveau de titre porte sa couleur"
    );
}

/// **Bug-report-8, T4.2-T4.4** : une image choisie se pose dans le lieu
/// visé ; un format non composable est refusé avec son mot ; la hauteur
/// se règle en pourcentage.
#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn une_image_s_insere_et_se_regle() {
    use crate::mise_en_page::modele::Champ;

    let mut state = etat_isole();
    let _ = update(&mut state, Message::MiseEnPageOuvrir);

    // Un vrai fichier image, pour la vérification de lecture.
    let image = std::env::temp_dir().join("tw-image-epreuve.png");
    std::fs::write(&image, [137u8, 80, 78, 71]).expect("l'image s'écrit");

    let _ = update(
        &mut state,
        Message::EntetePiedImageChoisie(Champ::EnteteGauche, image.to_string_lossy().into_owned()),
    );
    let dialogue = state.mise_en_page.as_ref().expect("le dialogue");
    let valeur = dialogue.reglages.valeur(Champ::EnteteGauche);
    assert!(
        valeur.contains(&format!("![  ]({})", image.display())),
        "l'image se pose dans le lieu : {valeur}"
    );

    // Un format non composable : refusé, et le champ ne bouge pas.
    let avant = valeur.clone();
    let _ = update(
        &mut state,
        Message::EntetePiedImageChoisie(Champ::EnteteGauche, "/tmp/une-video.mp4".to_string()),
    );
    let dialogue = state.mise_en_page.as_ref().expect("le dialogue");
    assert_eq!(
        dialogue.reglages.valeur(Champ::EnteteGauche),
        avant,
        "un format non composable ne s'insère pas"
    );
    assert!(state.note.is_some(), "le refus se dit, jamais en silence");

    // La hauteur se règle en pourcentage (T4.4).
    let _ = update(
        &mut state,
        Message::MiseEnPageChamp(Champ::ImagesHauteur, "60".to_string()),
    );
    let dialogue = state.mise_en_page.as_ref().expect("le dialogue");
    assert_eq!(
        dialogue.reglages.valeur(Champ::ImagesHauteur),
        "60",
        "la hauteur des images se règle"
    );

    let _ = std::fs::remove_file(&image);
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn la_roue_s_entend_avec_le_hex_et_les_rapides() {
    let mut state = etat_isole();
    let _ = update(&mut state, Message::PickerCouleurOuvrir);

    // Le hex tape pilote la roue.
    let _ = update(&mut state, Message::PickerCouleurHex("#3498db".to_string()));
    let picker = state.picker_couleur.as_ref().expect("la roue est ouverte");
    assert_eq!(
        picker.hex_saisi.as_deref(),
        Some("#3498db"),
        "la saisie reste sous les doigts"
    );
    assert!(
        (200.0..230.0).contains(&picker.teinte),
        "le bleu vient sur la roue : {}°",
        picker.teinte
    );

    // Appliquer ferme et teinte le texte du hex courant.
    let _ = update(&mut state, Message::PickerCouleurAppliquer);
    assert!(state.picker_couleur.is_none(), "la roue se referme");
    assert!(
        state.editor.content().contains("#3498db"),
        "la couleur vient : {:?}",
        state.editor.content()
    );

    // Une couleur rapide touche le texte et laisse la roue ouverte.
    let _ = update(&mut state, Message::PickerCouleurOuvrir);
    let _ = update(
        &mut state,
        Message::PickerCouleurRapide("#e74c3c".to_string()),
    );
    assert!(
        state.picker_couleur.is_some(),
        "la roue reste ouverte pour la suite"
    );
    assert!(
        state.editor.content().contains("#e74c3c"),
        "la rapide s'applique : {:?}",
        state.editor.content()
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn la_roue_du_theme_vise_son_champ() {
    let mut state = etat_isole();

    // Le bouton de couleur d'un champ du thème ouvre la même roue, visée
    // sur ce champ.
    let _ = update(&mut state, Message::ThemeEditeurColorPicker("accent"));
    let picker = state.picker_couleur.as_ref().expect("la roue est ouverte");
    assert_eq!(picker.champ, Some("accent"), "la roue vise le champ");

    // La couleur choisie va au champ du thème, et la roue se referme.
    let _ = update(&mut state, Message::PickerCouleurHex("#e67e22".to_string()));
    let _ = update(&mut state, Message::PickerCouleurAppliquer);
    assert!(state.picker_couleur.is_none(), "la roue se referme");
    assert_eq!(
        state.editeur_theme_textes.get("accent").map(String::as_str),
        Some("#e67e22"),
        "le champ du thème reçoit la couleur"
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn les_normes_sont_une_fenetre_au_contrat_de_clavier() {
    let mut state = etat_isole();

    // La fenêtre des normes est comptée comme fenêtre : `Tabulation`
    // circule, `Maj+Tabulation` revient — le contrat de toutes les
    // fenêtres (éprouvé par ailleurs), dont la navigation clavier des
    // normes (bug-report-7, point 19).
    let _ = update(&mut state, Message::MiseEnPageOuvrir);
    assert!(
        state.mise_en_page.is_some(),
        "la fenêtre des normes est ouverte"
    );
    assert!(
        super::super::fenetre_ouverte(&state),
        "elle entre dans le contrat des fenêtres"
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn la_note_se_ecrit_dans_son_panneau_et_pose_sa_marque() {
    let mut state = etat_isole();

    // Le geste n'écrit rien dans la ligne : il ouvre le panneau.
    let _ = update(&mut state, Message::NoteInserer);
    assert!(state.note_dialogue.is_some(), "le panneau s'ouvre");
    assert_eq!(state.editor.content(), "", "rien dans la ligne");

    // La note s'écrit au panneau, puis la seule marque vient au curseur.
    let _ = update(&mut state, Message::NoteTexte("voici ma note".to_string()));
    let _ = update(&mut state, Message::NoteValider);
    assert!(state.note_dialogue.is_none(), "le panneau se referme");
    assert_eq!(
        state.editor.content(),
        "^[voici ma note]",
        "la marque porte la note"
    );

    // Annuler n'écrit rien du tout.
    let _ = update(&mut state, Message::NoteInserer);
    let _ = update(&mut state, Message::NoteTexte("abandonnée".to_string()));
    let _ = update(&mut state, Message::NoteAnnuler);
    assert_eq!(state.editor.content(), "^[voici ma note]", "rien de plus");
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn les_metadonnées_se_reglent_et_se_sauvent() {
    let mut state = etat_isole();
    let _ = update(&mut state, Message::MiseEnPageOuvrir);

    // Une paire change : le dialogue la porte, et le fichier la garde —
    // des données, elles ne s'annulent pas.
    let _ = update(
        &mut state,
        Message::MetaChamp(
            "directeur".to_string(),
            "directeur".to_string(),
            "M. X".to_string(),
        ),
    );
    let _ = update(&mut state, Message::MetaAjouter);
    let dialogue = state.mise_en_page.as_ref().expect("le dialogue");
    assert_eq!(
        dialogue.metadonnees.valeur("directeur"),
        Some("M. X"),
        "la quick part a sa valeur"
    );
    assert_eq!(
        dialogue.metadonnees.paires.len(),
        2,
        "la ligne de l'auteur + la ligne neuve"
    );

    // Le fichier porte la même chose — la relève est durable.
    let relue = crate::project::metadonnees::Metadonnees::charger(&state.project.dir);
    assert_eq!(relue.valeur("directeur"), Some("M. X"));

    // Supprimer retire la ligne, du dialogue comme du fichier.
    let derniere = relue.paires.last().expect("une ligne").0.clone();
    let _ = update(&mut state, Message::MetaSupprimer(derniere.clone()));
    let relue = crate::project::metadonnees::Metadonnees::charger(&state.project.dir);
    assert!(relue.valeur(&derniere).is_none(), "la ligne est partie");
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn la_creation_porte_les_metadonnées() {
    let mut state = etat_isole();

    // Le dialogue de création propose les métadonnées, et le projet naît
    // avec (bug-report-7, point 9).
    let _ = update(&mut state, Message::ProjetCreer);
    let _ = update(&mut state, Message::ProjetNom("un-devoir".to_string()));
    let _ = update(
        &mut state,
        Message::ProjetMeta("titre".to_string(), "Un devoir".to_string()),
    );
    let _ = update(
        &mut state,
        Message::ProjetMeta("auteur".to_string(), "Alice".to_string()),
    );
    let _ = update(&mut state, Message::ProjetValider);

    let meta = crate::project::metadonnees::Metadonnees::charger(
        &crate::project::projects_dir().join("un-devoir"),
    );
    assert_eq!(meta.valeur("titre"), Some("Un devoir"));
    assert_eq!(meta.valeur("auteur"), Some("Alice"));
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn le_composeur_ecrit_le_code_a_sa_place() {
    let mut state = etat_isole();

    // Le composeur s'ouvre : les champs naissent des paramètres.
    let _ = update(&mut state, Message::ExtensionOuvrir);
    let dialogue = state.extension.as_ref().expect("le composeur");
    assert!(
        dialogue.valeurs.contains_key("largeur"),
        "les champs naissent de parametres()"
    );

    // Le choix devient le code posé au curseur — l'écrivain ne le tape
    // jamais (bug-report-7, point 4).
    let _ = update(
        &mut state,
        Message::ExtensionParam("largeur".to_string(), "12cm".to_string()),
    );
    let _ = update(&mut state, Message::ExtensionInserer);
    assert!(state.extension.is_none(), "le composeur se referme");
    let contenu = state.editor.content();
    assert!(
        contenu.contains("width: 12cm"),
        "le réglage devient du code : {contenu}"
    );
    assert!(
        contenu.contains("@local/cetz"),
        "le paquet est nommé : {contenu}"
    );

    // Annuler n'écrit rien.
    let _ = update(&mut state, Message::ExtensionOuvrir);
    let _ = update(&mut state, Message::ExtensionAnnuler);
    assert_eq!(state.editor.content(), contenu, "rien de plus");
}

/// Le glossaire se gère du panneau gauche : une entrée s'écrit, se
/// corrige, se supprime — et sa référence s'insère au curseur dans le
/// dialecte du tampon (bug-report-8 : l'écrivain ne tape jamais le code).
#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn le_glossaire_se_gere_du_panneau() {
    let mut state = etat_isole();

    // Une entrée neuve : la fiche s'ouvre, la clé se choisit.
    let _ = update(&mut state, Message::GlossaireCreer);
    assert!(state.edition_glossaire.is_some(), "la fiche s'ouvre");

    let _ = update(
        &mut state,
        Message::EditionGlossaireChamp("cle".into(), "sdo".into()),
    );
    let _ = update(
        &mut state,
        Message::EditionGlossaireChamp("court".into(), "sdo".into()),
    );
    let _ = update(
        &mut state,
        Message::EditionGlossaireChamp("long".into(), "docteur scolastique".into()),
    );
    let _ = update(&mut state, Message::EditionGlossaireEnregistrer);

    assert!(state.edition_glossaire.is_none(), "la fiche se referme");
    assert_eq!(state.glossaire.len(), 1, "le terme est là");
    assert_eq!(state.glossaire[0].key, "sdo");

    // Le fichier porte tout — la persistance est au projet.
    let relu = crate::project::glossaire::Glossaire::charger(&state.project.dir);
    assert_eq!(relu.entrees.len(), 1, "le fichier est écrit");
    assert_eq!(relu.entrees[0].long.as_deref(), Some("docteur scolastique"));

    // Un clic insère la référence, dans le dialecte du tampon (`.tw`).
    let _ = update(&mut state, Message::GlossaireInserer("sdo".into()));
    assert!(
        state.editor.content().contains("{glo:sdo}"),
        "la balise est insérée : {}",
        state.editor.content()
    );

    // Une clé fausse se refuse — la fiche reste ouverte, rien ne s'écrit.
    let _ = update(&mut state, Message::GlossaireCreer);
    let _ = update(
        &mut state,
        Message::EditionGlossaireChamp("cle".into(), "deux mots".into()),
    );
    let _ = update(&mut state, Message::EditionGlossaireEnregistrer);
    assert!(state.edition_glossaire.is_some(), "la fiche reste ouverte");
    assert!(
        state.note.as_ref().is_some_and(|n| n.erreur),
        "le refus se dit"
    );
    assert_eq!(state.glossaire.len(), 1, "rien n'a été écrit");

    // La correction se fait en place.
    let _ = update(&mut state, Message::EditionGlossaireFermer);
    let _ = update(&mut state, Message::GlossaireModifier("sdo".into()));
    let _ = update(
        &mut state,
        Message::EditionGlossaireChamp("long".into(), "docteur de l'École".into()),
    );
    let _ = update(&mut state, Message::EditionGlossaireEnregistrer);
    assert_eq!(state.glossaire.len(), 1, "pas de doublon");
    assert_eq!(
        state.glossaire[0].long.as_deref(),
        Some("docteur de l'École"),
        "le terme est corrigé en place"
    );

    // Et la suppression part pour de bon.
    let _ = update(&mut state, Message::GlossaireSupprimer("sdo".into()));
    assert!(state.glossaire.is_empty(), "le terme est parti");
    let relu = crate::project::glossaire::Glossaire::charger(&state.project.dir);
    assert!(relu.entrees.is_empty(), "le fichier suit");
}

/// Les normes ouvertes prennent les mains : le clavier n'arrive plus à
/// l'éditeur, et une couleur rapide de la roue va au réglage visé —
/// jamais au texte (bug-report-9, T1.1).
#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn les_normes_prennent_les_mains() {
    use crate::mise_en_page::modele::Champ;

    let mut state = etat_isole();

    // L'écriture en cours : un vrai clic donne le clavier à l'éditeur
    // (`has_canvas_focus`) — sans cette main, le test ci-dessous passerait
    // sans le correctif.
    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::MouseClick(iced::Point::new(
            4.0, 4.0,
        ))),
    );
    let avant = state.editor.content().to_string();

    // Les normes ouvertes : le clavier ne va plus au document.
    let _ = update(&mut state, Message::MiseEnPageOuvrir);
    assert!(state.mise_en_page.is_some(), "le panneau s'ouvre");
    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::CharacterInput('x')),
    );
    assert_eq!(
        state.editor.content(),
        avant,
        "le clavier n'arrive plus au document"
    );

    // Une couleur rapide de la roue va au réglage visé — pas au texte.
    let _ = update(
        &mut state,
        Message::PickerCouleurPourReglage(Champ::CouleurTexte),
    );
    let _ = update(&mut state, Message::PickerCouleurRapide("#112233".into()));
    let dialogue = state.mise_en_page.as_ref().expect("le panneau");
    assert_eq!(
        dialogue.reglages.valeur(Champ::CouleurTexte),
        "#112233",
        "la couleur rapide va au réglage visé"
    );
    assert_eq!(state.editor.content(), avant, "le texte ne bouge pas");

    // Et les mains reviennent à l'écriture à la fermeture.
    let _ = update(&mut state, Message::MiseEnPageFermer);
    assert!(state.editor.editor.is_focused(), "l'écriture reprend");
}

/// Bug-report-9 : la bascule éditeur → rendu régénère systématiquement le
/// PDF, et l'aperçu se refait depuis ce fichier livré.
#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn la_bascule_vers_le_rendu_genere_le_pdf() {
    let mut state = etat_isole();

    // Le geste : passer à la vue rendu.
    let _ = update(&mut state, Message::ModeBasculer);
    assert!(
        state.mode == crate::app::state::ModeVue::Rendu,
        "la bascule ouvre la vue rendu"
    );
    assert!(
        state.render.export_en_cours,
        "la bascule demande systématiquement la génération du PDF"
    );

    // La réponse de la tâche, telle que la boucle de messages la livre.
    let dir = state.document_dir();
    let (entry, overrides) = state.compilation_unit();
    let octets = crate::typst::render::render_pdf(&dir, &entry, &overrides).expect("PDF généré");
    let _ = update(&mut state, Message::PdfApercu(Ok(octets)));

    assert!(
        state.render.lecteur.is_some(),
        "le lecteur affiche le PDF généré"
    );
    assert!(!state.render.export_en_cours, "la génération est close");
}

/// Bug-report-9 : un geste de zoom (dix crans de molette) ne déclenche
/// qu'**un** rendu net, lancé quand le geste s'arrête — jamais un rendu
/// par cran.
#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn le_geste_de_zoom_ne_declenche_qu_un_rendu() {
    let mut state = etat_isole();

    // Le rendu est en place (la bascule l'a demandé) — la réponse, telle
    // que la boucle la livre.
    let _ = update(&mut state, Message::ModeBasculer);
    let dir = state.document_dir();
    let (entry, overrides) = state.compilation_unit();
    let octets = crate::typst::render::render_pdf(&dir, &entry, &overrides).expect("PDF");
    let _ = update(&mut state, Message::PdfApercu(Ok(octets)));

    // La compilation, telle que la tâche la livrerait : l'aperçu a
    // besoin du document pour savoir quoi rasteriser.
    let outcome = crate::typst::render::compile(&dir, &entry, &overrides);
    let _ = update(&mut state, Message::RenderDone(outcome));

    // Dix crans de molette, sans pause.
    let avant = state.render.requested;
    for _ in 0..10 {
        let _ = update(&mut state, Message::PreviewZoomStep(1));
    }
    assert!(state.render.zoom_depuis.is_some(), "le geste est suivi");
    assert_eq!(
        state.render.requested, avant,
        "aucun rendu ne part pendant le geste"
    );

    // Le geste s'arrête : le tic suivant lance le rendu net — une fois.
    std::thread::sleep(std::time::Duration::from_millis(250));
    let _ = update(&mut state, Message::RenderTick);
    assert!(
        state.render.requested > avant,
        "le rendu net part après le geste"
    );
    assert!(state.render.zoom_depuis.is_none(), "le geste est clos");

    let apres = state.render.requested;
    let _ = update(&mut state, Message::RenderTick);
    assert_eq!(state.render.requested, apres, "aucun second rendu ne part");
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn le_panneau_des_normes_connait_la_norme_du_document() {
    use crate::mise_en_page::modele::Champ;

    let mut state = etat_isole();

    let _ = update(&mut state, Message::MiseEnPageOuvrir);
    let dialogue = state.mise_en_page.as_ref().unwrap();
    assert_eq!(
        dialogue.norme_originale,
        crate::project::norme_qui_correspond(&dialogue.reglages).unwrap_or_default(),
        "la norme chargée est exactement celle du document — sinon « Normes personnalisées »"
    );
    assert!(!dialogue.modifie, "rien n'est modifié au premier regard");

    // Une édition : la norme en cours de modification se signale.
    let _ = update(
        &mut state,
        Message::MiseEnPageChamp(Champ::MargeHaut, "13cm".to_string()),
    );
    let dialogue = state.mise_en_page.as_ref().unwrap();
    assert!(dialogue.modifie, "l'édition marque la norme modifiée");
    assert!(
        crate::project::norme_qui_correspond(&dialogue.reglages).is_none(),
        "des réglages modifiés ne sont plus aucune norme connue"
    );

    // L'enregistrement refuse le nom d'une norme qui existe déjà.
    let existant = crate::project::noms_de_normes()
        .into_iter()
        .next()
        .expect("la bibliothèque est semée");
    let _ = update(&mut state, Message::NormesEnregistrer(existant.clone()));
    let dialogue = state.mise_en_page.as_ref().unwrap();
    assert!(
        dialogue.norme_originale != existant,
        "la copie ne vole pas un nom existant"
    );
    assert!(dialogue.modifie, "la copie refusée laisse la modification");

    // La copie sous un nom neuf devient la norme chargée.
    let _ = update(
        &mut state,
        Message::NormesEnregistrer("une copie".to_string()),
    );
    let dialogue = state.mise_en_page.as_ref().unwrap();
    assert_eq!(
        dialogue.norme_originale, "une copie",
        "la copie est chargée"
    );
    assert!(!dialogue.modifie, "enregistrer retire la modification");

    // Écraser la norme chargée : la bibliothèque la porte à jour.
    let _ = update(
        &mut state,
        Message::MiseEnPageChamp(Champ::MargeBas, "5cm".to_string()),
    );
    assert!(state.mise_en_page.as_ref().unwrap().modifie, "retouchée");
    let _ = update(&mut state, Message::NormesEcraser);
    let attendu = state.mise_en_page.as_ref().unwrap().reglages.clone();
    assert_eq!(
        crate::project::norme("une copie"),
        Some(attendu),
        "écraser met la bibliothèque à jour"
    );
    assert!(
        !state.mise_en_page.as_ref().unwrap().modifie,
        "écraser retire la modification"
    );

    // Le document porte maintenant la norme : l'appliquer, puis rouvrir.
    let _ = update(&mut state, Message::MiseEnPageAppliquer);
    let _ = update(&mut state, Message::MiseEnPageFermer);
    let _ = update(&mut state, Message::MiseEnPageOuvrir);
    let dialogue = state.mise_en_page.as_ref().unwrap();
    assert_eq!(
        dialogue.norme_originale, "une copie",
        "la bibliothèque reconnaît la norme à l'ouverture"
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn au_demarrage_les_fenetres_de_rendu_se_remplissent() {
    let mut state = etat_isole();
    state.mode = crate::app::state::ModeVue::Rendu;

    // La fenêtre de rendu du démarrage reçoit l'ordre de montrer son
    // document — elle n'attend pas une bascule de mode (bug-report-9).
    assert!(
        crate::app::rendre_les_fenetres_de_rendu(&mut state).is_some(),
        "la fenêtre de rendu du démarrage est servie"
    );

    // Une compilation lancée n'est pas relancée.
    state.render.pending = true;
    assert!(
        crate::app::rendre_les_fenetres_de_rendu(&mut state).is_none(),
        "rien ne se lance deux fois"
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn le_choix_de_couleur_n_insere_rien() {
    let mut state = etat_isole();

    // Le sélecteur s'ouvre, et une couleur personnalisée s'y choisit.
    let _ = update(&mut state, Message::PickerCouleurOuvrir);
    let _ = update(&mut state, Message::PickerCouleurHex("#123456".to_string()));
    let avant = state.editor.content();

    // Le choix : la couleur s'enregistre — le texte ne bouge pas, et le
    // sélecteur reste ouvert pour appliquer si on le souhaite.
    let _ = update(&mut state, Message::PickerCouleurEnregistrer);
    assert_eq!(
        state.derniere_couleur.as_deref(),
        Some("#123456"),
        "le choix enregistre la couleur"
    );
    assert_eq!(
        state.editor.content(),
        avant,
        "le choix n'insère rien dans le texte"
    );
    assert!(
        state.picker_couleur.is_some(),
        "le sélecteur reste ouvert après le choix"
    );

    // L'application : là, la balise va au texte.
    let _ = update(&mut state, Message::PickerCouleurAppliquer);
    assert!(
        state.editor.content() != avant,
        "l'appliquée touche le texte"
    );
    assert!(
        state.picker_couleur.is_none(),
        "l'application ferme le sélecteur"
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn la_fenetre_du_glossaire_s_ouvre_et_se_ferme() {
    let mut state = etat_isole();

    let _ = update(&mut state, Message::GlossaireFenetreOuvrir);
    assert!(state.glossaire_fenetre, "la fenêtre s'ouvre");

    // La recherche vit dans la fenêtre.
    let _ = update(&mut state, Message::GlossaireFiltrer("un".to_string()));
    assert_eq!(state.glossaire_filtre, "un", "la recherche y vit");

    let _ = update(&mut state, Message::GlossaireFenetreFermer);
    assert!(!state.glossaire_fenetre, "la fenêtre se ferme");
    assert!(
        state.glossaire_filtre.is_empty(),
        "la recherche ne sert plus à rien"
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_arrobase_au_curseur_ouvre_le_composeur() {
    let mut state = etat_isole();
    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::Paste("@".into())),
    );
    assert!(
        state.composeur.is_some(),
        "le @ tapé ouvre le composeur de notes"
    );
}
