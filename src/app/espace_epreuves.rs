//! Épreuves d'intégration exécutées uniquement dans un HOME jetable.
use super::*;

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
fn un_renommage_dit_le_detail_de_ce_qu_il_a_fait() {
    use crate::lsp::messages::{FileEdit, Position, Range, TextEdit};
    let mut state = etat_isole();
    let premier = state.project.dir.join("a.typ");
    let second = state.project.dir.join("b.typ");
    std::fs::write(&premier, "X").expect("fichier synthétique");
    std::fs::write(&second, "X").expect("fichier synthétique");
    let edition = |chemin: std::path::PathBuf| FileEdit {
        path: chemin,
        edits: vec![TextEdit {
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 1,
                },
            },
            new_text: "Y".into(),
        }],
    };
    state.renommage = Some(crate::app::state::Renommage {
        nom: "Y".into(),
        changements: Some(vec![edition(premier), edition(second)]),
    });
    let _ = update(&mut state, Message::SymbolRenameApply);
    let rapport = state.export_status.expect("un résultat est dit");
    assert!(
        rapport.contains("a.typ") && rapport.contains("b.typ"),
        "le résultat nomme les fichiers touchés : {rapport}"
    );
    assert!(
        rapport.contains("2 fichier(s)"),
        "le résultat compte les fichiers : {rapport}"
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_fichier_illisible_arrete_le_renommage_et_le_dit() {
    use crate::lsp::messages::{FileEdit, Position, Range, TextEdit};
    let mut state = etat_isole();
    let illisible = state.project.dir.join("a.typ");
    let intact = state.project.dir.join("b.typ");
    std::fs::write(&illisible, "X").expect("fichier synthétique");
    std::fs::write(&intact, "X").expect("fichier synthétique");
    let mut permissions = std::fs::metadata(&illisible)
        .expect("métadonnées")
        .permissions();
    use std::os::unix::fs::PermissionsExt;
    permissions.set_mode(0o000);
    std::fs::set_permissions(&illisible, permissions).expect("verrou de lecture");
    let edition = |chemin: std::path::PathBuf| FileEdit {
        path: chemin,
        edits: vec![TextEdit {
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 1,
                },
            },
            new_text: "Y".into(),
        }],
    };
    state.renommage = Some(crate::app::state::Renommage {
        nom: "Y".into(),
        changements: Some(vec![edition(illisible.clone()), edition(intact.clone())]),
    });
    let _ = update(&mut state, Message::SymbolRenameApply);
    let note = state.note.expect("le problème est dit");
    assert!(
        note.texte.contains("a.typ"),
        "la note nomme le fichier illisible : {}",
        note.texte
    );
    assert_eq!(
        std::fs::read_to_string(&intact).expect("fichier"),
        "X",
        "rien ne s'écrit après un échec de lecture"
    );
    // Le ménage : les permissions reprennent pour laisser le HOME jetable
    // se supprimer sans drame.
    let mut permissions = std::fs::metadata(&illisible)
        .expect("métadonnées")
        .permissions();
    permissions.set_mode(0o644);
    std::fs::set_permissions(&illisible, permissions).expect("déverrou");
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_renommage_lsp_preserve_un_fichier_temporaire_existant() {
    use crate::lsp::messages::{FileEdit, Position, Range, TextEdit};
    let mut state = etat_isole();
    let cible = state.project.dir.join("auxiliaire.typ");
    let ancien_temporaire = state.project.dir.join(".typst-writer-tmp");
    std::fs::write(&cible, "ABC").expect("fichier synthétique");
    std::fs::write(&ancien_temporaire, "À CONSERVER").expect("ancien temporaire");
    state.renommage = Some(crate::app::state::Renommage {
        nom: "X".into(),
        changements: Some(vec![FileEdit {
            path: cible.clone(),
            edits: vec![TextEdit {
                range: Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: 0,
                        character: 1,
                    },
                },
                new_text: "X".into(),
            }],
        }]),
    });
    let _ = update(&mut state, Message::SymbolRenameApply);
    assert_eq!(std::fs::read_to_string(cible).unwrap(), "XBC");
    assert_eq!(
        std::fs::read_to_string(ancien_temporaire).unwrap(),
        "À CONSERVER"
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_renommage_lsp_ne_perd_pas_un_tampon_non_sauvegarde() {
    use crate::lsp::messages::{FileEdit, Position, Range, TextEdit};
    let mut state = etat_isole();
    let chemin = state.editor.path.clone().expect("document");
    let disque = std::fs::read(&chemin).expect("document initial");
    let _ = update(
        &mut state,
        Message::Editor(iced_code_editor::Message::Paste("TEXTE NON SAUVÉ".into())),
    );
    let avant = state.editor.content();
    state.renommage = Some(crate::app::state::Renommage {
        nom: "X".into(),
        changements: Some(vec![FileEdit {
            path: chemin.clone(),
            edits: vec![TextEdit {
                range: Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: 0,
                        character: 1,
                    },
                },
                new_text: "X".into(),
            }],
        }]),
    });
    let _ = update(&mut state, Message::SymbolRenameApply);
    assert_eq!(
        state.editor.content(),
        avant,
        "le texte non sauvegardé est conservé"
    );
    assert!(state.editor.is_modified());
    assert_eq!(
        std::fs::read(chemin).unwrap(),
        disque,
        "aucune écriture ne commence"
    );
}

#[test]
#[ignore = "HOME jetable et TYPST_WRITER_TEST_ESPACE=1 exigés"]
fn un_renommage_lsp_ne_modifie_aucun_projet_si_une_cible_est_etrangere() {
    use crate::lsp::messages::{FileEdit, Position, Range, TextEdit};
    let premiere = etat_isole();
    let a = premiere.project.dir.join("document.tw");
    std::fs::write(&a, "AAA").expect("document A synthétique");
    let mut seconde = etat_isole();
    let _ = update(&mut seconde, Message::NewProject);
    let b = seconde.project.dir.join("document.tw");
    assert_ne!(a, b);
    std::fs::write(&b, "BBB").expect("document B synthétique");
    let remplacement = TextEdit {
        range: Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 0,
                character: 1,
            },
        },
        new_text: "X".into(),
    };
    seconde.renommage = Some(crate::app::state::Renommage {
        nom: "X".into(),
        changements: Some(vec![
            FileEdit {
                path: b.clone(),
                edits: vec![remplacement.clone()],
            },
            FileEdit {
                path: a.clone(),
                edits: vec![remplacement],
            },
        ]),
    });
    let _ = update(&mut seconde, Message::SymbolRenameApply);
    assert_eq!(
        std::fs::read_to_string(&a).unwrap(),
        "AAA",
        "projet A protégé"
    );
    assert_eq!(
        std::fs::read_to_string(&b).unwrap(),
        "BBB",
        "aucune application partielle"
    );
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

    // Le geste : le devoir naît du modèle.
    let _ = update(
        &mut state,
        Message::ProjetCreerAvecModele(crate::project::model::Modele::DevoirCourt),
    );
    let dir = state.project.dir.clone();

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

    for promesse in [
        "titre du devoir",
        "sommaire",
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

    let _ = update(
        &mut state,
        Message::ProjetCreerAvecModele(crate::project::model::Modele::TravailLong),
    );
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

    let main = std::fs::read_to_string(dir.join("main.typ")).expect("assemblage");
    let (entry, corrections, rapports) =
        crate::langage::pipeline::compilation_unit_avec_diagnostics(&dir, &main, "");

    assert!(
        !rapports
            .iter()
            .any(|diagnostic| diagnostic.message.contains("neutralisée")),
        "rien n'est neutralisé : {rapports:?}"
    );

    let outcome = crate::typst::render::compile(&dir, &entry, &corrections);
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

    for promesse in [
        "titre du travail",
        "sommaire",
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

    // L'en-tête courant : le titre apparaît page de titre **et** en-tête.
    assert!(
        texte.matches("titre du travail").count() >= 2,
        "l'en-tête courant est résolu : {texte}"
    );

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
    // Sur un document `.tw` : la note est `^[…]`, et le curseur entre les
    // crochets.
    let mut state = etat_isole();
    let _ = update(&mut state, Message::NoteInserer);
    let contenu = state.editor.content();
    assert_eq!(contenu, "^[]");
    assert_eq!(
        state.editor.cursor_offset(),
        contenu.len() - "]".len(),
        "le curseur est entre les crochets de la note"
    );

    // Le renvoi : le `@` ouvre la complétion.
    let _ = state.editor.set_cursor_offset(state.editor.content().len());
    let _ = update(&mut state, Message::RenvoiInserer);
    let contenu = state.editor.content();
    assert!(contenu.ends_with('@'), "{contenu}");
    assert_eq!(state.editor.cursor_offset(), contenu.len());

    // L'étiquette n'a pas de syntaxe en `.tw` : le geste le dit, et n'insère
    // rien de mort.
    let avant = state.editor.content();
    let _ = update(&mut state, Message::EtiquetteInserer);
    assert_eq!(state.editor.content(), avant, "rien d'inerte n'est inséré");
    assert!(
        state.note.as_ref().is_some_and(|note| !note.erreur),
        "le geste s'explique"
    );

    // Sur un chapitre `.typ` : la note parle Typst, l'étiquette entre, et le
    // curseur se pose à l'intérieur.
    state.editor.path = Some(state.project.dir.join("chapitres").join("01.typ"));
    let _ = state.editor.set_cursor_offset(state.editor.content().len());
    let _ = update(&mut state, Message::NoteInserer);
    let contenu = state.editor.content();
    assert!(contenu.ends_with("#footnote[]"), "{contenu}");
    assert_eq!(
        state.editor.cursor_offset(),
        contenu.len() - "]".len(),
        "le curseur est entre les crochets de la note"
    );

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
fn un_geste_cree_le_projet_du_modele_choisi() {
    let mut state = etat_isole();

    // Le geste : un clic, et le devoir naît complet.
    let _ = update(
        &mut state,
        Message::ProjetCreerAvecModele(crate::project::model::Modele::DevoirCourt),
    );
    let devoir = state.project.dir.clone();
    let contenu = std::fs::read_to_string(devoir.join("main.typ")).expect("assemblage");
    for promesse in ["Titre du devoir", "Sommaire", "Bibliographie"] {
        assert!(
            contenu.contains(promesse),
            "« {promesse} » manque : {contenu}"
        );
    }

    // Le second geste : le travail long, avec ses préliminaires.
    let _ = update(
        &mut state,
        Message::ProjetCreerAvecModele(crate::project::model::Modele::TravailLong),
    );
    let long = state.project.dir.clone();
    assert_ne!(long, devoir, "deux projets, deux dossiers");
    let contenu = std::fs::read_to_string(long.join("main.typ")).expect("assemblage");
    for promesse in [
        "Titre du travail",
        "Sommaire",
        "header: context",
        "Bibliographie",
    ] {
        assert!(
            contenu.contains(promesse),
            "« {promesse} » manque : {contenu}"
        );
    }
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
