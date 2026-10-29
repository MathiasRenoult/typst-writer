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
        self.espace.grille.get_mut(self.espace.focus).unwrap().onglets = vec![1];
        self.enregistrer_source();
        self.actualiser_typst();
        let Some(disposition) = disposition else { return; };
        let sauvees = disposition.vues.clone();
        if sauvees.is_empty() { return; }
        // Charger d'abord tous les documents dans un espace temporaire. Un
        // chemin absent ou étranger ne remplace jamais un document utilisable.
        let mut vues = std::collections::BTreeMap::new();
        for vue in sauvees {
            if vue.id == 0 || vues.contains_key(&vue.id) || !vue.projet.is_dir() { continue; }
            let projet = crate::project::load(&vue.projet);
            if !projet.owns(&vue.document) { continue; }
            let Ok(contenu) = std::fs::read_to_string(&vue.document) else { continue; };
            vues.insert(vue.id, Vue::nouvelle(projet, vue.document, contenu, vue.mode, &self.resolved));
        }
        if vues.is_empty() { return; }
        let verrous = std::mem::take(&mut self.espace.verrous);
        self.espace = super::espace::Espace::depuis_disposition(disposition);
        self.espace.verrous = verrous;
        self.espace.initialise = true;
        for (_, panneau) in self.espace.grille.iter_mut() {
            panneau.onglets.retain(|id| vues.contains_key(id));
            panneau.actif = panneau.actif.min(panneau.onglets.len().saturating_sub(1));
        }
        let wanted = self.espace.grille.get(self.espace.focus).and_then(|p| p.onglets.get(p.actif)).copied()
            .or_else(|| self.espace.grille.iter().find_map(|(_, p)| p.onglets.first().copied()));
        let Some(wanted) = wanted else { return; };
        self.espace.prochain = vues.keys().max().copied().unwrap_or(1).saturating_add(1);
        for vue in vues.values() {
            if !self.espace.verrous.contains_key(&vue.project.dir) {
                match crate::project::Verrou::prendre(&vue.project.dir) {
                    Ok(v) => { self.espace.verrous.insert(vue.project.dir.clone(), v); }
                    Err(e) => { self.note = Some(Note::erreur(format!("{e} Projet en lecture seule."))); }
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
            self.espace.documents.entry(cle).or_insert_with(|| Document {
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
                chemin.file_name().is_some_and(|n| n == crate::langage::pipeline::GENERATED_FILE)
            })
            .map(|(_, contenu)| contenu.clone())
            .unwrap_or(entry);
        self.typst_genere = genere;
    }

    pub(crate) fn disposition_complete(&self) -> Option<Disposition> {
        if !self.espace.initialise { return None; }
        let mut disposition = self.espace.disposition();
        if let Some(document) = &self.editor.path {
            disposition.vues.push(VueSauvee { id: self.espace.active, projet: self.project.dir.clone(), document: document.clone(), mode: self.mode });
        }
        for (&id, vue) in &self.espace.vues {
            // Les vues orphelines (onglet remplacé) ne sont pas sauvées.
            if !self.espace.grille.iter().any(|(_, p)| p.onglets.contains(&id)) { continue; }
            if let Some(document) = &vue.editor.path {
                disposition.vues.push(VueSauvee { id, projet: vue.project.dir.clone(), document: document.clone(), mode: vue.mode });
            }
        }
        Some(disposition)
    }

    /// Déplace les contextes, pas leur texte. Ne change pas le focus logique :
    /// les tâches de fond empruntent une vue puis rendent immédiatement la main.
    fn changer_contexte(&mut self, id: VueId) -> bool {
        if id == self.espace.active { return true; }
        let Some(mut vue) = self.espace.vues.remove(&id) else { return false; };
        let ancien = self.espace.active;
        vue.echanger(self);
        self.espace.vues.insert(ancien, vue);
        self.espace.active = id;
        true
    }

    pub(crate) fn choisir_vue(&mut self, id: VueId) {
        if id == self.espace.active { return; }
        if !self.changer_contexte(id) { return; }
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
    /// `nouvel_onglet = true` (shift+clic) : un nouvel onglet, toujours.
    pub(crate) fn ouvrir_vue(&mut self, projet: crate::project::Project, chemin: PathBuf, contenu: String, ouverture: Ouverture) {
        if !projet.owns(&chemin) {
            self.note = Some(Note::erreur("Ce fichier n'appartient pas au projet."));
            return;
        }
        if ouverture != Ouverture::Nouvel {
            if self.project.dir == projet.dir && self.editor.path.as_ref() == Some(&chemin) { return; }
            if let Some(id) = self.espace.vues.iter().find_map(|(&id, v)| (v.project.dir == projet.dir && v.editor.path.as_ref() == Some(&chemin)).then_some(id)) {
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
                let position = self.espace.grille.get(panneau).and_then(|p| p.onglets.iter().position(|v| *v == ancien));
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

    fn ouvrir_projet_espace(&mut self, dir: PathBuf) {
        let projet = crate::project::load(&dir);
        if !self.espace.verrous.contains_key(&dir) {
            match crate::project::Verrou::prendre(&dir) {
                Ok(v) => { self.espace.verrous.insert(dir.clone(), v); }
                Err(e) => { self.note = Some(Note::erreur(format!("{e} Projet en lecture seule."))); }
            }
        }
        if self.espace.verrous.contains_key(&dir)
            && let Err(e) = crate::project::migrate_to_document(&projet)
        {
            self.note = Some(Note::erreur(format!("Ouverture du projet : {e}")));
            return;
        }
        let projet = crate::project::load(&dir);
        let chemin = projet.document.clone().unwrap_or_else(|| projet.main.clone());
        match std::fs::read_to_string(&chemin) {
            Ok(contenu) => { self.ouvrir_vue(projet, chemin, contenu, Ouverture::Remplacer); if self.project.dir == dir { self.projets_vue = false; } },
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
    if state.espace.transaction { return super::update_interne(state, message); }
    let id = state.espace.active;
    let cle = state.cle_document();
    if matches!(message, Message::RenderTick) && cle.as_ref().is_some_and(|c| state.espace.compilations.contains(c)) {
        state.render.pending = true;
    }
    let etait_en_cours = state.render.pending;
    let mut document = cle.as_ref().and_then(|c| state.espace.documents.remove(c));
    if let Some(d) = &mut document { d.source.ouvrir_transaction(&mut state.editor.editor); }
    state.espace.transaction = true;
    let task = super::update_interne(state, message);
    state.espace.transaction = false;
    if !etait_en_cours && state.render.pending
        && let Some(cle) = &cle
    {
        state.espace.compilations.insert(cle.clone());
    }
    if let (Some(cle), Some(mut document)) = (cle, document) {
        let editor = if state.espace.active == id { Some(&mut state.editor) } else { state.espace.vues.get_mut(&id).map(|v| &mut v.editor) };
        let change = editor.is_some_and(|editor| document.source.fermer_transaction(&mut editor.editor));
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
                    if state.espace.active == id { vue.typst.clone_from(&state.typst_genere); }
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
        Message::RetourDocument(cle, message) => {
            state.espace.compilations.remove(&cle);
            let ancien = state.espace.active;
            let mut ids: Vec<_> = state.espace.vues.iter().filter_map(|(&id, v)| (v.project.dir == cle.0 && v.editor.path.as_ref() == Some(&cle.1)).then_some(id)).collect();
            if state.cle_document().as_ref() == Some(&cle) { ids.push(ancien); }
            let mut tasks = Vec::new();
            for id in ids { state.changer_contexte(id); tasks.push(transaction(state, (*message).clone())); }
            state.changer_contexte(ancien);
            Task::batch(tasks)
        }
        Message::RetourVue(id, message) => {
            let ancien = state.espace.active;
            if !state.changer_contexte(id) { return Task::none(); }
            let task = update(state, *message);
            state.changer_contexte(ancien);
            task
        }
        Message::VersVue(id, message) => {
            state.choisir_vue(id);
            if state.espace.active != id { return Task::none(); }
            update(state, *message)
        }
        Message::VueChoisir(id) => { state.choisir_vue(id); Task::none() }
        Message::PanneauChoisir(p) => {
            if let Some(id) = state.espace.grille.get(p).and_then(|p| p.onglets.get(p.actif)).copied() { state.choisir_vue(id); }
            Task::none()
        }
        Message::PanneauRedimensionner(e) => {
            state.espace.grille.resize(e.split, e.ratio.clamp(0.1, 0.9));
            state.save_session();
            Task::none()
        }
        Message::OngletPrendre(id) => { state.choisir_vue(id); state.espace.glisse = Some(id); state.espace.origine_glisse = state.layout.cursor; state.espace.cible = None; Task::none() }
        Message::OngletCible(p, bord) => { if state.espace.glisse.is_some() && state.layout.cursor.distance(state.espace.origine_glisse) >= 6.0 { state.espace.cible = Some((p, bord)); } Task::none() }
        Message::SplitterReleased if state.espace.glisse.is_some() => update(state, Message::OngletPoser),
        Message::OngletPoser => {
            if let (Some(id), Some((p, bord))) = (state.espace.glisse.take(), state.espace.cible.take()) {
                state.espace.deplacer(id, p, bord);
                state.choisir_vue(id);
            }
            state.save_session();
            Task::none()
        }
        Message::VueDupliquer => {
            if let Some(path) = state.editor.path.clone() {
                let mode = state.mode;
                state.ouvrir_vue(state.project.clone(), path, state.editor.content(), Ouverture::Nouvel);
                state.mode = mode;
                state.save_session();
            }
            Task::none()
        }
        Message::Scinder(bord) => {
            if let Some(path) = state.editor.path.clone() {
                let mode = state.mode;
                let focus = state.espace.focus;
                state.ouvrir_vue(state.project.clone(), path, state.editor.content(), Ouverture::Nouvel);
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
            if state.espace.vues.is_empty() { return Task::none(); }
            let ancien = state.espace.active;
            if !state.changer_contexte(id) { return Task::none(); }
            // Sauvegarde réussie avant de retirer la dernière projection.
            let task = transaction(state, Message::SaveFile);
            if state.editor.is_modified() { state.changer_contexte(ancien); return task; }
            let suivant = state.espace.vues.keys().copied().find(|v| *v != id).unwrap();
            state.changer_contexte(suivant);
            state.espace.vues.remove(&id);
            let origine = state.espace.grille.iter().find_map(|(p, c)| c.onglets.contains(&id).then_some(*p));
            if let Some(p) = origine {
                let panneau = state.espace.grille.get_mut(p).unwrap();
                panneau.onglets.retain(|v| *v != id);
                panneau.actif = panneau.actif.min(panneau.onglets.len().saturating_sub(1));
                if panneau.onglets.is_empty() && state.espace.grille.len() > 1 { state.espace.grille.close(p); }
            }
            state.choisir_vue(if ancien == id { suivant } else { ancien });
            state.save_session();
            task
        }
        Message::OpenProject(dir) | Message::OngletChoisir(dir) => { state.ouvrir_projet_espace(dir); Task::none() }
        Message::NewProject => {
            let nom = super::next_project_name(&state.projects);
            match crate::project::create(&nom) {
                Ok(dir) => { state.projects = crate::project::list_projects(); state.ouvrir_projet_espace(dir); }
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
                tasks.push(transaction(state, if rendu { Message::RenderTick } else { Message::SaveTick }));
            }
            state.changer_contexte(ancien);
            Task::batch(tasks)
        }
        message => {
            let sauver = matches!(message, Message::ModeBasculer | Message::ModeTypst);
            let task = transaction(state, message);
            if sauver { state.actualiser_typst(); state.save_session(); }
            task
        }
    }
}
