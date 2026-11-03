//! Routage transactionnel : une réponse garde l'identité de sa vue, même
//! quand le clavier travaille déjà dans un autre projet.
use super::espace::{Disposition, Document, Ouverture, Vue, VueId, VueSauvee};
use super::state::{ModeVue, Note};
use super::{Message, State};
use iced::Task;
use std::path::PathBuf;

#[cfg(test)]
#[path = "espace_epreuves.rs"]
mod epreuves;

impl State {
    pub(crate) fn initialiser_espace(&mut self, disposition: Option<Disposition>) {
        if let Some(verrou) = self.verrou.take() {
            self.espace.verrous.insert(self.project.dir.clone(), verrou);
        }
        self.espace.initialise = true;
        self.espace
            .grille
            .get_mut(self.espace.focus)
            .unwrap()
            .onglets = vec![1];
        self.enregistrer_source();
        self.actualiser_typst();
        let Some(disposition) = disposition else {
            return;
        };
        let sauvees = disposition.vues.clone();
        if sauvees.is_empty() {
            return;
        }
        // Charger d'abord tous les documents dans un espace temporaire. Un
        // chemin absent ou étranger ne remplace jamais un document utilisable.
        let mut vues = std::collections::BTreeMap::new();
        for vue in sauvees {
            if vue.id == 0 || vues.contains_key(&vue.id) || !vue.projet.is_dir() {
                continue;
            }
            let projet = crate::project::load(&vue.projet);
            if !projet.owns(&vue.document) {
                continue;
            }
            let Ok(contenu) = std::fs::read_to_string(&vue.document) else {
                continue;
            };
            vues.insert(
                vue.id,
                Vue::nouvelle(projet, vue.document, contenu, vue.mode, &self.resolved),
            );
        }
        if vues.is_empty() {
            return;
        }
        let verrous = std::mem::take(&mut self.espace.verrous);
        self.espace = super::espace::Espace::depuis_disposition(disposition);
        self.espace.verrous = verrous;
        self.espace.initialise = true;
        for (_, panneau) in self.espace.grille.iter_mut() {
            panneau.onglets.retain(|id| vues.contains_key(id));
            panneau.actif = panneau.actif.min(panneau.onglets.len().saturating_sub(1));
        }
        let wanted = self
            .espace
            .grille
            .get(self.espace.focus)
            .and_then(|p| p.onglets.get(p.actif))
            .copied()
            .or_else(|| {
                self.espace
                    .grille
                    .iter()
                    .find_map(|(_, p)| p.onglets.first().copied())
            });
        let Some(wanted) = wanted else {
            return;
        };
        self.espace.prochain = vues.keys().max().copied().unwrap_or(1).saturating_add(1);
        for vue in vues.values() {
            if !self.espace.verrous.contains_key(&vue.project.dir) {
                match crate::project::Verrou::prendre(&vue.project.dir) {
                    Ok(v) => {
                        self.espace.verrous.insert(vue.project.dir.clone(), v);
                    }
                    Err(e) => {
                        self.note = Some(Note::erreur(format!("{e} Projet en lecture seule.")));
                    }
                }
            }
        }
        let mut active = vues.remove(&wanted).unwrap();
        active.echanger(self);
        self.espace.active = wanted;
        self.espace.vues = vues;
        self.enregistrer_source();
        let ids: Vec<_> = self.espace.vues.keys().copied().collect();
        for id in ids {
            self.changer_contexte(id);
            self.enregistrer_source();
            self.actualiser_typst();
        }
        self.changer_contexte(wanted);
        self.actualiser_typst();
    }

    fn cle_document(&self) -> Option<(PathBuf, PathBuf)> {
        Some((self.project.dir.clone(), self.editor.path.clone()?))
    }

    fn enregistrer_source(&mut self) {
        if let Some(cle) = self.cle_document() {
            // Première ouverture : ce que le tampon contient vient du disque,
            // et c'est la référence dont tout changement extérieur s'écartera.
            self.espace.disques.entry(cle.clone()).or_insert_with(|| {
                crate::project::empreinte_contenu(self.editor.content().as_bytes())
            });
            self.espace
                .documents
                .entry(cle)
                .or_insert_with(|| Document {
                    source: iced_code_editor::SourcePartagee::nouvelle(&self.editor.editor),
                });
        }
    }

    fn actualiser_typst(&mut self) {
        // Le vrai pipeline du projet : décor, normes et bibliographie ne sont
        // pas réinventés par la vue de lecture seule.
        let (entry, overrides) = self.compilation_unit();
        // Le texte complet, pas le `#include` du point d'entrée : la vue
        // Typst montre ce que le compilateur reçoit vraiment.
        let genere = overrides
            .iter()
            .find(|(chemin, _)| {
                chemin
                    .file_name()
                    .is_some_and(|n| n == crate::langage::pipeline::GENERATED_FILE)
            })
            .map(|(_, contenu)| contenu.clone())
            .unwrap_or(entry);
        self.typst_genere = genere;
    }

    pub(crate) fn disposition_complete(&self) -> Option<Disposition> {
        if !self.espace.initialise {
            return None;
        }
        let mut disposition = self.espace.disposition();
        if let Some(document) = &self.editor.path {
            disposition.vues.push(VueSauvee {
                id: self.espace.active,
                projet: self.project.dir.clone(),
                document: document.clone(),
                mode: self.mode,
            });
        }
        for (&id, vue) in &self.espace.vues {
            // Les vues orphelines (onglet remplacé) ne sont pas sauvées.
            if !self
                .espace
                .grille
                .iter()
                .any(|(_, p)| p.onglets.contains(&id))
            {
                continue;
            }
            if let Some(document) = &vue.editor.path {
                disposition.vues.push(VueSauvee {
                    id,
                    projet: vue.project.dir.clone(),
                    document: document.clone(),
                    mode: vue.mode,
                });
            }
        }
        Some(disposition)
    }

    /// Déplace les contextes, pas leur texte. Ne change pas le focus logique :
    /// les tâches de fond empruntent une vue puis rendent immédiatement la main.
    fn changer_contexte(&mut self, id: VueId) -> bool {
        if id == self.espace.active {
            return true;
        }
        let Some(mut vue) = self.espace.vues.remove(&id) else {
            return false;
        };
        let ancien = self.espace.active;
        vue.echanger(self);
        self.espace.vues.insert(ancien, vue);
        self.espace.active = id;
        true
    }

    /// Sauvegarde les vues d'un projet avant son déplacement vers la corbeille.
    /// Si une écriture échoue, aucun déplacement ne doit avoir lieu.
    pub(crate) fn sauvegarder_avant_deplacer(
        &mut self,
        projet: &std::path::Path,
    ) -> Result<(), String> {
        let ancien = self.espace.active;
        let mut ids: Vec<_> = self.espace.vues.keys().copied().collect();
        ids.push(ancien);
        let mut erreur = None;
        for id in ids {
            if !self.changer_contexte(id)
                || self.project.dir != projet
                || !self.editor.is_modified()
            {
                continue;
            }
            let _ = transaction(self, Message::SaveFile);
            if self.editor.is_modified() {
                let chemin = self.editor.path.as_ref().map_or_else(
                    || "document sans chemin".to_string(),
                    |chemin| chemin.display().to_string(),
                );
                let detail = self
                    .note
                    .as_ref()
                    .map_or("sauvegarde impossible", |note| &note.texte);
                erreur = Some(format!("{chemin} : {detail}"));
                break;
            }
        }
        self.changer_contexte(ancien);
        if let Some(erreur) = erreur {
            Err(erreur)
        } else {
            Ok(())
        }
    }

    /// Avant de supprimer la vue active, rejoindre un autre projet utilisable.
    /// En cas d'échec, le projet ciblé n'a pas encore été déplacé.
    pub(crate) fn preparer_suppression_projet(
        &mut self,
        cible: &std::path::Path,
    ) -> Result<(), String> {
        if self.project.dir != cible {
            return Ok(());
        }
        let autre_vue = self.espace.vues.iter().find_map(|(&id, vue)| {
            (vue.project.dir != cible
                && self
                    .espace
                    .grille
                    .iter()
                    .any(|(_, p)| p.onglets.contains(&id)))
            .then_some(id)
        });
        if let Some(id) = autre_vue {
            self.choisir_vue(id);
        }
        if self.project.dir != cible {
            return Ok(());
        }

        self.projects = crate::project::list_projects();
        let autre = self
            .projects
            .iter()
            .find(|projet| projet.dir != cible)
            .map(|projet| projet.dir.clone());
        let autre = match autre {
            Some(autre) => autre,
            None => {
                let nom = super::next_project_name(&self.projects);
                let nouveau = crate::project::create(&nom)
                    .map_err(|e| format!("création du projet de remplacement : {e}"))?;
                self.projects = crate::project::list_projects();
                nouveau
            }
        };
        self.ouvrir_projet_espace(autre, Ouverture::Nouvel);
        if self.project.dir == cible {
            return Err("impossible d'ouvrir un autre projet avant la suppression".to_string());
        }
        Ok(())
    }

    /// Retire les vues d'un projet déjà sauvegardé et déplacé à la corbeille.
    pub(crate) fn retirer_projet_supprime(&mut self, projet: &std::path::Path) {
        let ids: std::collections::BTreeSet<_> = self
            .espace
            .vues
            .iter()
            .filter_map(|(&id, vue)| (vue.project.dir == projet).then_some(id))
            .collect();
        self.espace.vues.retain(|id, _| !ids.contains(id));
        for (_, panneau) in self.espace.grille.iter_mut() {
            let actif = panneau.onglets.get(panneau.actif).copied();
            panneau.onglets.retain(|id| !ids.contains(id));
            panneau.actif = actif
                .and_then(|id| panneau.onglets.iter().position(|v| *v == id))
                .unwrap_or_else(|| panneau.actif.min(panneau.onglets.len().saturating_sub(1)));
        }
        let vides: Vec<_> = self
            .espace
            .grille
            .iter()
            .filter_map(|(&id, p)| p.onglets.is_empty().then_some(id))
            .collect();
        for panneau in vides {
            if self.espace.grille.len() > 1 {
                self.espace.grille.close(panneau);
            }
        }
        self.espace.documents.retain(|(dir, _), _| dir != projet);
        self.espace.disques.retain(|(dir, _), _| dir != projet);
        self.espace.compilations.retain(|(dir, _)| dir != projet);
        self.onglets.retain(|dir| dir != projet);
        if self.espace.glisse.is_some_and(|id| ids.contains(&id)) {
            self.espace.glisse = None;
            self.espace.cible = None;
        }
        if self
            .renaming
            .as_ref()
            .is_some_and(|renommage| renommage.path.starts_with(projet))
        {
            self.renaming = None;
        }
        self.save_session();
    }

    /// Suit le nouveau dossier dans toutes les vues sans remplacer leur texte
    /// ni leur historique partagé. Les réponses LSP et rendus de l'ancien
    /// chemin deviennent périmés.
    pub(crate) fn suivre_projet_renomme(
        &mut self,
        ancien: &std::path::Path,
        nouveau: &std::path::Path,
    ) {
        let projet = crate::project::load(nouveau);
        let chemin_deplace = |chemin: &std::path::Path| {
            chemin
                .strip_prefix(ancien)
                .ok()
                .map(|relatif| nouveau.join(relatif))
        };
        if self.project.dir == ancien {
            if let Some(chemin) = self.editor.path.as_deref().and_then(chemin_deplace) {
                self.editor.set_path(chemin);
            }
            self.project = projet.clone();
            self.bibliography = projet
                .bibliography
                .as_deref()
                .map(crate::project::parse_bibliography)
                .unwrap_or_default();
            self.lsp = Default::default();
            self.render.reset();
        }
        for vue in self.espace.vues.values_mut() {
            if vue.project.dir != ancien {
                continue;
            }
            if let Some(chemin) = vue.editor.path.as_deref().and_then(chemin_deplace) {
                vue.editor.set_path(chemin);
            }
            vue.project = projet.clone();
            vue.bibliography = projet
                .bibliography
                .as_deref()
                .map(crate::project::parse_bibliography)
                .unwrap_or_default();
            vue.lsp = Default::default();
            vue.render.reset();
        }
        let documents = std::mem::take(&mut self.espace.documents);
        for ((dossier, chemin), document) in documents {
            let cle = if dossier == ancien {
                (
                    nouveau.to_path_buf(),
                    chemin_deplace(&chemin).unwrap_or(chemin),
                )
            } else {
                (dossier, chemin)
            };
            self.espace.documents.insert(cle, document);
        }
        let disques = std::mem::take(&mut self.espace.disques);
        for ((dossier, chemin), empreinte) in disques {
            let cle = if dossier == ancien {
                (
                    nouveau.to_path_buf(),
                    chemin_deplace(&chemin).unwrap_or(chemin),
                )
            } else {
                (dossier, chemin)
            };
            self.espace.disques.insert(cle, empreinte);
        }
        self.espace
            .compilations
            .retain(|(dossier, _)| dossier != ancien);
        for onglet in &mut self.onglets {
            if onglet == ancien {
                *onglet = nouveau.to_path_buf();
            }
        }
        if self.project.dir == nouveau {
            self.actualiser_typst();
        }
    }

    pub(crate) fn choisir_vue(&mut self, id: VueId) {
        if id == self.espace.active {
            return;
        }
        if !self.changer_contexte(id) {
            return;
        }
        for (p, panneau) in self.espace.grille.iter_mut() {
            if let Some(index) = panneau.onglets.iter().position(|v| *v == id) {
                panneau.actif = index;
                self.espace.focus = *p;
                break;
            }
        }
        self.editor.retheme(&self.resolved);
        self.editor.editor.request_focus();
        self.save_session();
    }

    /// Ouvre un document dans l'espace de travail.
    ///
    /// `nouvel_onglet = false` (clic simple) : le document remplace le contenu
    /// de l'onglet actif, ou rejoint son onglet existant s'il est déjà ouvert.
    /// Ouvre un document dans l'espace de travail.
    ///
    /// Les règles de clic sont strictes — WINDOWS-02 :
    /// - **Clic simple** (`Ouverture::Remplacer`) : ouvre dans l'onglet actuel ;
    ///   si le document est déjà ouvert, bascule vers son onglet existant.
    /// - **Maj+clic** (`Ouverture::Nouvel`) : toujours un nouvel onglet, même si
    ///   le document est déjà ouvert. C'est le geste explicite de « montrer
    ///   côte à côte », pas un simple basculement.
    pub(crate) fn ouvrir_vue(
        &mut self,
        projet: crate::project::Project,
        chemin: PathBuf,
        contenu: String,
        ouverture: Ouverture,
    ) {
        if !projet.owns(&chemin) {
            self.note = Some(Note::erreur("Ce fichier n'appartient pas au projet."));
            return;
        }
        // Une écriture interrompue a pu laisser un temporaire qui porte la
        // dernière version du document : il devient récupérable, à condition
        // de détenir le verrou — un projet en lecture seule ne s'écrit pas.
        if self.projet_verrouille(&projet.dir) {
            crate::project::recuperer_temporaires_perdus(&projet.dir, &chemin);
        }
        if ouverture != Ouverture::Nouvel {
            if self.project.dir == projet.dir && self.editor.path.as_ref() == Some(&chemin) {
                return;
            }
            if let Some(id) = self.espace.vues.iter().find_map(|(&id, v)| {
                (self
                    .espace
                    .grille
                    .iter()
                    .any(|(_, p)| p.onglets.contains(&id))
                    && v.project.dir == projet.dir
                    && v.editor.path.as_ref() == Some(&chemin))
                .then_some(id)
            }) {
                self.choisir_vue(id);
                return;
            }
        }
        let id = self.espace.prochain;
        self.espace.prochain = self.espace.prochain.saturating_add(1);
        let mut vue = Vue::nouvelle(projet, chemin, contenu, ModeVue::Edition, &self.resolved);
        let cle = (vue.project.dir.clone(), vue.editor.path.clone().unwrap());
        if let Some(document) = self.espace.documents.get(&cle) {
            document.source.projeter_force(&mut vue.editor.editor);
            vue.editor.synchroniser_source();
        }
        self.espace.vues.insert(id, vue);
        match ouverture {
            Ouverture::Nouvel => {
                let panneau = self.espace.grille.get_mut(self.espace.focus).unwrap();
                panneau.onglets.push(id);
                panneau.actif = panneau.onglets.len() - 1;
            }
            Ouverture::Remplacer => {
                let ancien = self.espace.active;
                let panneau = self.espace.focus;
                let position = self
                    .espace
                    .grille
                    .get(panneau)
                    .and_then(|p| p.onglets.iter().position(|v| *v == ancien));
                if let (Some(position), Some(p)) = (position, self.espace.grille.get_mut(panneau)) {
                    p.onglets[position] = id;
                    p.actif = position;
                } else {
                    let p = self.espace.grille.get_mut(self.espace.focus).unwrap();
                    p.onglets.push(id);
                    p.actif = p.onglets.len() - 1;
                }
            }
        }
        self.choisir_vue(id);
        self.enregistrer_source();
        self.actualiser_typst();
        self.save_session();
    }

    fn ouvrir_projet_espace(&mut self, dir: PathBuf, ouverture: Ouverture) {
        let projet = crate::project::load(&dir);
        if !self.espace.verrous.contains_key(&dir) {
            match crate::project::Verrou::prendre(&dir) {
                Ok(v) => {
                    self.espace.verrous.insert(dir.clone(), v);
                }
                Err(e) => {
                    self.note = Some(Note::erreur(format!("{e} Projet en lecture seule.")));
                }
            }
        }
        if self.espace.verrous.contains_key(&dir)
            && let Err(e) = crate::project::migrate_to_document(&projet)
        {
            self.note = Some(Note::erreur(format!("Ouverture du projet : {e}")));
            return;
        }
        let projet = crate::project::load(&dir);
        let chemin = projet
            .document
            .clone()
            .unwrap_or_else(|| projet.main.clone());
        match std::fs::read_to_string(&chemin) {
            Ok(contenu) => {
                self.ouvrir_vue(projet, chemin, contenu, ouverture);
                if self.project.dir == dir {
                    self.projets_vue = false;
                }
            }
            Err(e) => self.note = Some(Note::erreur(format!("Ouverture : {e}"))),
        }
    }
}

fn etiqueter(id: VueId, message: Message) -> Message {
    match message {
        Message::RetourDocument(..) | Message::RetourVue(..) | Message::VersVue(..) => message,
        _ => Message::RetourVue(id, Box::new(message)),
    }
}

fn transaction(state: &mut State, message: Message) -> Task<Message> {
    if state.espace.transaction {
        return super::update_interne(state, message);
    }
    let id = state.espace.active;
    let cle = state.cle_document();
    if matches!(message, Message::RenderTick)
        && cle
            .as_ref()
            .is_some_and(|c| state.espace.compilations.contains(c))
    {
        state.render.pending = true;
    }
    let etait_en_cours = state.render.pending;
    let mut document = cle.as_ref().and_then(|c| state.espace.documents.remove(c));
    if let Some(d) = &mut document {
        d.source.ouvrir_transaction(&mut state.editor.editor);
    }
    state.espace.transaction = true;
    let task = super::update_interne(state, message);
    state.espace.transaction = false;
    if !etait_en_cours
        && state.render.pending
        && let Some(cle) = &cle
    {
        state.espace.compilations.insert(cle.clone());
    }
    if let (Some(cle), Some(mut document)) = (cle, document) {
        let editor = if state.espace.active == id {
            Some(&mut state.editor)
        } else {
            state.espace.vues.get_mut(&id).map(|v| &mut v.editor)
        };
        let change =
            editor.is_some_and(|editor| document.source.fermer_transaction(&mut editor.editor));
        if change {
            if state.espace.active == id {
                state.render.words_pending = true;
                state.actualiser_typst();
            }
            for vue in state.espace.vues.values_mut() {
                if vue.project.dir == cle.0 && vue.editor.path.as_ref() == Some(&cle.1) {
                    document.source.projeter(&mut vue.editor.editor);
                    vue.editor.synchroniser_source();
                    vue.render.words_pending = true;
                    if state.espace.active == id {
                        vue.typst.clone_from(&state.typst_genere);
                    }
                }
            }
        }
        state.espace.documents.insert(cle, document);
    }
    let destinataire = state.espace.active;
    task.map(move |m| etiqueter(destinataire, m))
}

pub(super) fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        // Ces gestes déplacent ou retirent les clés des documents : aucune
        // source ne doit être empruntée par une transaction pendant l'opération.
        Message::ProjetValider => super::update_interne(state, Message::ProjetValider),
        Message::ProjetSupprimer(dir) => {
            super::update_interne(state, Message::ProjetSupprimer(dir))
        }
        Message::ConfirmationConfirmer
            if state
                .confirmation
                .as_ref()
                .is_some_and(|c| matches!(c.action, Message::ProjetSupprimer(_))) =>
        {
            super::update_interne(state, Message::ConfirmationConfirmer)
        }
        Message::FermetureDemandee => {
            let ancien = state.espace.active;
            let mut ids: Vec<_> = state.espace.vues.keys().copied().collect();
            ids.push(ancien);
            let note_precedente = state.note.take();
            let mut probleme = None;
            for id in ids {
                if !state.changer_contexte(id) || !state.editor.is_modified() {
                    continue;
                }
                let _ = transaction(state, Message::SaveFile);
                if state.editor.is_modified() {
                    let chemin = state.editor.path.as_ref().map_or_else(
                        || "document sans chemin".to_string(),
                        |chemin| chemin.display().to_string(),
                    );
                    let detail = state.note.take().map_or_else(
                        || "Le projet est peut-être ouvert en lecture seule.".to_string(),
                        |note| note.texte,
                    );
                    probleme = Some(format!(
                        "Fermeture annulée : {chemin}. {detail} Le texte reste ouvert ; corrigez le problème puis réessayez."
                    ));
                    break;
                }
            }
            state.changer_contexte(ancien);
            if let Some(probleme) = probleme {
                state.note = Some(Note::erreur(probleme));
                return Task::none();
            }
            state.note = note_precedente;
            state.save_session();
            if let Some(fenetre) = state.window {
                iced::window::close(fenetre)
            } else {
                state.note = Some(Note::erreur("Fenêtre introuvable : réessayez de fermer."));
                Task::none()
            }
        }
        Message::RetourDocument(cle, message) => {
            state.espace.compilations.remove(&cle);
            let ancien = state.espace.active;
            let mut ids: Vec<_> = state
                .espace
                .vues
                .iter()
                .filter_map(|(&id, v)| {
                    (v.project.dir == cle.0 && v.editor.path.as_ref() == Some(&cle.1)).then_some(id)
                })
                .collect();
            if state.cle_document().as_ref() == Some(&cle) {
                ids.push(ancien);
            }
            let mut tasks = Vec::new();
            for id in ids {
                state.changer_contexte(id);
                tasks.push(transaction(state, (*message).clone()));
            }
            state.changer_contexte(ancien);
            Task::batch(tasks)
        }
        Message::RetourVue(id, message) => {
            // Une vue remplacée peut rester en mémoire pour sauver son texte,
            // mais ses tâches ne peuvent plus modifier le projet en arrière-plan.
            if !state
                .espace
                .grille
                .iter()
                .any(|(_, p)| p.onglets.contains(&id))
            {
                return Task::none();
            }
            let ancien = state.espace.active;
            if !state.changer_contexte(id) {
                return Task::none();
            }
            let task = update(state, *message);
            state.changer_contexte(ancien);
            task
        }
        Message::VersVue(id, message) => {
            state.choisir_vue(id);
            if state.espace.active != id {
                return Task::none();
            }
            update(state, *message)
        }
        Message::VueChoisir(id) => {
            state.choisir_vue(id);
            Task::none()
        }
        Message::PanneauChoisir(p) => {
            if let Some(id) = state
                .espace
                .grille
                .get(p)
                .and_then(|p| p.onglets.get(p.actif))
                .copied()
            {
                state.choisir_vue(id);
            }
            Task::none()
        }
        Message::PanneauSuivant => {
            // WINDOWS-01 : le focus passe au panneau suivant, cycliquement.
            let panneaux: Vec<_> = state.espace.grille.iter().map(|(&id, _)| id).collect();
            if let Some(position) = panneaux.iter().position(|&id| id == state.espace.focus) {
                let suivant = panneaux[(position + 1) % panneaux.len()];
                if let Some(id) = state
                    .espace
                    .grille
                    .get(suivant)
                    .and_then(|p| p.onglets.get(p.actif))
                    .copied()
                {
                    state.choisir_vue(id);
                }
            }
            Task::none()
        }
        Message::PanneauRedimensionner(e) => {
            state.espace.grille.resize(e.split, e.ratio.clamp(0.1, 0.9));
            state.save_session();
            Task::none()
        }
        Message::OngletPrendre(id) => {
            state.choisir_vue(id);
            state.espace.glisse = Some(id);
            state.espace.origine_glisse = state.layout.cursor;
            state.espace.cible = None;
            Task::none()
        }
        Message::OngletCible(p, bord) => {
            if state.espace.glisse.is_some()
                && state.layout.cursor.distance(state.espace.origine_glisse) >= 6.0
            {
                state.espace.cible = Some((p, bord));
            }
            Task::none()
        }
        Message::SplitterReleased if state.espace.glisse.is_some() => {
            update(state, Message::OngletPoser)
        }
        Message::OngletPoser => {
            let glisse = state.espace.glisse.take();
            let cible = state.espace.cible.take();
            let distance = state.layout.cursor.distance(state.espace.origine_glisse);
            if let (Some(id), Some((p, bord))) = (glisse, cible) {
                state.espace.deplacer(id, p, bord);
                state.choisir_vue(id);
            } else if let Some(id) = glisse
                && distance >= 6.0
            {
                // WINDOWS-04 : dépôt hors panneau = nouveau panneau à droite.
                // Un clic sans déplacement ne split pas la fenêtre.
                let focus = state.espace.focus;
                state
                    .espace
                    .deplacer(id, focus, Some(iced::widget::pane_grid::Edge::Right));
                state.choisir_vue(id);
            }
            state.save_session();
            Task::none()
        }
        Message::VueDupliquer => {
            if let Some(path) = state.editor.path.clone() {
                let mode = state.mode;
                state.ouvrir_vue(
                    state.project.clone(),
                    path,
                    state.editor.content(),
                    Ouverture::Nouvel,
                );
                state.mode = mode;
                state.save_session();
            }
            Task::none()
        }
        Message::Scinder(bord) => {
            if let Some(path) = state.editor.path.clone() {
                let mode = state.mode;
                let focus = state.espace.focus;
                state.ouvrir_vue(
                    state.project.clone(),
                    path,
                    state.editor.content(),
                    Ouverture::Nouvel,
                );
                state.mode = mode;
                let id = state.espace.active;
                state.espace.deplacer(id, focus, Some(bord));
                state.choisir_vue(id);
                state.save_session();
            }
            Task::none()
        }
        Message::OngletSuivant => {
            if let Some(p) = state.espace.grille.get(state.espace.focus)
                && !p.onglets.is_empty()
            {
                let id = p.onglets[(p.actif + 1) % p.onglets.len()];
                state.choisir_vue(id);
            }
            Task::none()
        }
        Message::VueFermer(id) => {
            if state.espace.vues.is_empty() {
                return Task::none();
            }
            let ancien = state.espace.active;
            if !state.changer_contexte(id) {
                return Task::none();
            }
            // Sauvegarde réussie avant de retirer la dernière projection.
            let task = transaction(state, Message::SaveFile);
            if state.editor.is_modified() {
                state.changer_contexte(ancien);
                return task;
            }
            let suivant = state
                .espace
                .vues
                .keys()
                .copied()
                .find(|v| *v != id)
                .unwrap();
            state.changer_contexte(suivant);
            state.espace.vues.remove(&id);
            let origine = state
                .espace
                .grille
                .iter()
                .find_map(|(p, c)| c.onglets.contains(&id).then_some(*p));
            if let Some(p) = origine {
                let panneau = state.espace.grille.get_mut(p).unwrap();
                panneau.onglets.retain(|v| *v != id);
                panneau.actif = panneau.actif.min(panneau.onglets.len().saturating_sub(1));
                if panneau.onglets.is_empty() && state.espace.grille.len() > 1 {
                    state.espace.grille.close(p);
                }
            }
            state.choisir_vue(if ancien == id { suivant } else { ancien });
            state.save_session();
            task
        }
        Message::OpenProject(dir) => {
            // NAV-06-01 : Shift ou Ctrl = nouvel onglet, sans remplacer l'actif.
            let ouverture = if state.maj_tenu || state.controle_tenu {
                Ouverture::Nouvel
            } else {
                Ouverture::Remplacer
            };
            state.ouvrir_projet_espace(dir, ouverture);
            Task::none()
        }
        Message::OngletChoisir(dir) => {
            state.ouvrir_projet_espace(dir, Ouverture::Remplacer);
            Task::none()
        }
        Message::NewProject => {
            let nom = super::next_project_name(&state.projects);
            match crate::project::create(&nom) {
                Ok(dir) => {
                    state.projects = crate::project::list_projects();
                    state.ouvrir_projet_espace(dir, Ouverture::Remplacer);
                }
                Err(e) => state.note = Some(Note::erreur(format!("Création du projet : {e}"))),
            }
            Task::none()
        }
        Message::ProjetCreerAvecModele(modele) => {
            // Le modèle se choisit en un geste ; le projet naît complet —
            // page de titre, sommaire, bibliographie — et reste à écrire.
            let nom = super::next_project_name(&state.projects);
            match crate::project::model::create_avec_modele(&nom, modele) {
                Ok(dir) => {
                    state.projects = crate::project::list_projects();
                    state.ouvrir_projet_espace(dir, Ouverture::Remplacer);
                    state.note = Some(Note::info(format!(
                        "Projet créé d'après le modèle « {} ».",
                        modele.nom()
                    )));
                }
                Err(e) => state.note = Some(Note::erreur(format!("Création du projet : {e}"))),
            }
            Task::none()
        }
        Message::SaveTick | Message::RenderTick if !state.espace.transaction => {
            let rendu = matches!(message, Message::RenderTick);
            let ancien = state.espace.active;
            let mut ids: Vec<_> = state.espace.vues.keys().copied().collect();
            ids.push(ancien);
            let mut tasks = Vec::new();
            for id in ids {
                state.changer_contexte(id);
                tasks.push(transaction(
                    state,
                    if rendu {
                        Message::RenderTick
                    } else {
                        Message::SaveTick
                    },
                ));
            }
            state.changer_contexte(ancien);
            Task::batch(tasks)
        }
        message => {
            let sauver = matches!(message, Message::ModeBasculer | Message::ModeTypst);
            let task = transaction(state, message);
            if sauver {
                state.actualiser_typst();
                state.save_session();
            }
            task
        }
    }
}
