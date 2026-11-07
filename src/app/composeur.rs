//! Le composeur de notes de références.
//!
//! La fenêtre de suggestions n'est plus une liste qu'on accepte : c'est un
//! composeur, à étapes, où Tab avance. On choisit une référence ; Tab propose
//! son numéro de page ; Tab propose une autre référence **dans la même note** ;
//! et ainsi de suite. Le document ne reçoit la note qu'à la fin, d'un seul
//! tenant. Échap ne laisse rien.
//!
//! Une note peut réunir plusieurs références, et chacune se reprend : la
//! fenêtre montre la note, et choisir une de ses lignes remet sa clé et sa
//! page sous les doigts — **à son rang**, sans jamais déranger l'ordre.
//!
//! Tout ce qui décide vit ici, en pur : les touches arrivent en transitions,
//! et rien n'y touche à l'interface. C'est ce qui rend la boucle
//! « référence → page → autre référence » éprouvable sans écran.

use crate::app::Message;
use crate::app::State;
use crate::app::update;
use crate::langage::schema::Reference;
use iced::Task;
use std::path::Path;

/// L'étape où est le composeur.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Etape {
    /// Choisir une référence : la première, ou une de plus dans la note.
    Reference,
    /// Donner sa page à la référence qui vient d'être choisie.
    Page,
}

/// Le composeur, tenu ouvert tant que la note n'est pas écrite.
#[derive(Debug, Clone)]
pub struct Composeur {
    /// L'étape courante.
    pub etape: Etape,

    /// Les clés citables du projet, avec leur genre et leur texte de
    /// recherche (auteurs, titre, année), telles qu'elles étaient à
    /// l'ouverture : le composeur n'écoute personne, il filtre sa liste.
    pub cles: Vec<(String, String, String)>,

    /// La note, entière, dans l'ordre où elle s'écrira.
    pub references: Vec<Reference>,

    /// La référence qu'on tient : sa clé au champ de l'étape « référence »,
    /// sa page à celle de l'étape « page ».
    pub brouillon: Reference,

    /// Où le brouillon se rangera : l'index qu'il **remplace** dans la note,
    /// ou `None` pour une référence de plus, en bout.
    pub rang: Option<usize>,

    /// Ce qui est tapé dans le champ de l'étape : le filtre des clés à
    /// l'étape « référence », la mention de page à l'étape « page ».
    pub saisie: String,

    /// La ligne surlignée dans la liste des clés.
    pub selection: usize,

    /// La rangée tenue par les flèches, dans l'ordre de la fenêtre : les
    /// lignes de la note d'abord, puis les clés filtrées.
    ///
    /// Poser les flèches sur une ligne de note la **remet sous les doigts** ;
    /// poser sur une clé la surligne. Une seule liste, un seul geste.
    pub position: usize,

    /// L'offset, dans le document, où la note s'insérera : celui du `@`
    /// déclencheur, que le composeur a mangé.
    pub ancre: usize,

    /// La fin de la plage que la note remplacera, en édition d'une citation
    /// déjà écrite.
    ///
    /// `None` en création : la note s'écrit à l'ancre. `Some(fin)` en
    /// édition : la citation d'origine **reste en place** dans le document
    /// tant qu'on ne l'a pas réécrite — Échap ne la touche donc pas.
    pub edition: Option<usize>,
}

impl Composeur {
    /// L'identifiant du champ de saisie du composeur.
    ///
    /// Un seul champ pour les deux étapes — le filtre des clés, puis la
    /// mention de page. La forme de l'arbre de widgets ne change donc jamais
    /// quand l'étape change, et le focus y reste : c'est la règle de la
    /// fenêtre flottante, appliquée à son champ.
    pub const fn champ_id() -> iced::widget::Id {
        iced::widget::Id::new("composeur-champ")
    }

    /// Ouvre le composeur sur la liste entière des clés du projet.
    pub fn ouvert(cles: Vec<(String, String, String)>, ancre: usize) -> Self {
        Self {
            etape: Etape::Reference,
            cles,
            references: Vec::new(),
            brouillon: Reference::new(""),
            rang: None,
            saisie: String::new(),
            selection: 0,
            position: 0,
            ancre,
            edition: None,
        }
    }

    /// Charge une note déjà écrite, pour la rééditer.
    ///
    /// La note entière se tient dans la fenêtre, et sa **dernière** référence
    /// est celle qu'on regarde d'abord — la plus fréquemment reprise. Choisir
    /// une autre ligne la remettra sous les doigts.
    pub fn charger(&mut self, references: Vec<Reference>) {
        self.references = references;

        if let Some(dernier) = self.references.len().checked_sub(1) {
            self.choisir(dernier);
        }
    }

    /// Remet sous les doigts la référence d'une ligne de la note.
    ///
    /// Elle reste **à son rang** : c'est sa clé et sa page qui reviennent au
    /// champ, et la réécriture la remettra à la même place.
    pub fn choisir(&mut self, index: usize) {
        let Some(reference) = self.references.get(index).cloned() else {
            return;
        };

        self.brouillon = reference;
        self.rang = Some(index);
        self.etape = Etape::Reference;
        self.saisir(self.brouillon.cle.clone());
        self.position = index;

        // La sélection retombe sur la clé elle-même, même si le filtre en
        // montre d'autres devant elle.
        if let Some(position) = self
            .visibles()
            .iter()
            .position(|cle| self.cles[*cle].0 == self.saisie)
        {
            self.selection = position;
        }
    }

    /// Les indices des clés qui correspondent à ce qui est tapé.
    ///
    /// La recherche est celle de la bibliothèque : auteur, titre, année,
    /// clé — pas seulement la clé, sinon la moitié des notices serait
    /// injouable au composeur (R3). Deux passes, comme la liste de
    /// complétion : d'abord ce qui *commence* par le filtre, puis ce qui le
    /// *contient*.
    pub fn visibles(&self) -> Vec<usize> {
        if self.saisie.is_empty() {
            return (0..self.cles.len()).collect();
        }

        let voulu = self.saisie.to_lowercase();
        let mut debuts = Vec::new();
        let mut contient = Vec::new();

        for (index, (cle, _, recherche)) in self.cles.iter().enumerate() {
            let candidate = format!("{cle} {recherche}").to_lowercase();

            if candidate.starts_with(&voulu) {
                debuts.push(index);
            } else if candidate.contains(&voulu) {
                contient.push(index);
            }
        }

        debuts.extend(contient);
        debuts
    }

    /// La clé surlignée, s'il en reste une après filtrage.
    pub fn choisie(&self) -> Option<&str> {
        let visibles = self.visibles();
        visibles
            .get(self.selection)
            .map(|index| self.cles[*index].0.as_str())
    }

    /// La clé que prendrait la prise : la ligne surlignée affine, le texte du
    /// champ sert de recours — une clé absente de la bibliographie ne se perd
    /// jamais en silence.
    fn cle_tenue(&self) -> Option<String> {
        self.choisie().map(str::to_string).or_else(|| {
            let tape = self.saisie.trim();
            (!tape.is_empty()).then(|| tape.to_string())
        })
    }

    /// Ce qui est tapé dans le champ de l'étape courante.
    pub fn saisir(&mut self, texte: String) {
        self.saisie = texte;
        self.selection = 0;
        self.position = self.references.len();
    }

    /// La sélection des flèches est-elle sur les clés, plutôt que sur une
    /// ligne de la note ?
    pub const fn sur_palette(&self) -> bool {
        self.position >= self.references.len()
    }

    /// Déplace la sélection, sans jamais sortir de la fenêtre.
    ///
    /// Les flèches parcourent **les rangées de la fenêtre** dans leur ordre :
    /// les lignes de la note d'abord — et poser sur l'une d'elles la remet
    /// sous les doigts, prête à s'éditer — puis les clés filtrées.
    pub fn deplacer(&mut self, delta: isize) {
        let total = (self.references.len() + self.visibles().len()) as isize;

        if total == 0 {
            return;
        }

        let suivant = (self.position as isize + delta).clamp(0, total - 1) as usize;

        if suivant < self.references.len() {
            self.choisir(suivant);
        } else {
            self.selection = suivant - self.references.len();
            self.position = suivant;
        }
    }

    /// Tab : prendre la clé tenue, ou valider la page saisie.
    ///
    /// La boucle est celle du composeur : référence → page → référence → …
    pub fn avancer(&mut self) {
        match self.etape {
            Etape::Reference => self.prendre(),
            Etape::Page => self.valider_page(),
        }
    }

    /// Prend la clé tenue dans le champ, et propose son numéro de page.
    fn prendre(&mut self) {
        let Some(cle) = self.cle_tenue() else {
            return;
        };

        self.brouillon.cle = cle;
        self.saisie = self.brouillon.page.clone().unwrap_or_default();
        self.etape = Etape::Page;
    }

    /// Valide la mention de page — même vide, une page n'est pas obligatoire
    /// — range la référence à son rang, et propose une autre référence dans la
    /// même note.
    fn valider_page(&mut self) {
        let page = self.saisie.trim().to_string();

        self.brouillon.page = (!page.is_empty()).then_some(page);
        self.ranger();

        self.brouillon = Reference::new("");
        self.rang = None;
        self.etape = Etape::Reference;
        self.saisie.clear();
        self.selection = 0;
        self.position = self.references.len();
    }

    /// Range le brouillon dans la note, à la place qu'il s'est choisie.
    fn ranger(&mut self) {
        match self.rang {
            Some(index) if index < self.references.len() => {
                self.references[index] = self.brouillon.clone();
            }
            _ => self.references.push(self.brouillon.clone()),
        }
    }

    /// Entrée : terminer, et dire ce qui s'insère.
    ///
    /// À l'étape « page », la page saisie valide la référence en cours. À
    /// l'étape « référence », la clé **tenue dans le champ** rejoint la note à
    /// son rang ; un champ vide, lui, ne prend rien : la dernière liste
    /// n'était qu'une proposition. C'est le geste d'hier : `@dup` + Entrée
    /// insère `@dupont2023`.
    pub fn terminer(&mut self, intermediaire: bool) -> Option<String> {
        match self.etape {
            Etape::Page => self.valider_page(),
            Etape::Reference if !self.saisie.is_empty() => {
                self.prendre();
                self.valider_page();
            }
            Etape::Reference => {}
        }

        (!self.references.is_empty()).then(|| self.texte_insere(intermediaire))
    }

    /// Le texte de la note, dans la syntaxe du fichier ouvert.
    ///
    /// Un fichier intermédiaire reçoit l'appel de notre langage —
    /// `@dupont2023[p. 45]; @martin2020` —, un fichier Typst brut les appels
    /// qui rendent la même note — `#cite(<…>) #cite(<…>)`. Une seule source
    /// pour chaque forme : le sérialiseur et le générateur, jamais une
    /// recopie ici.
    pub fn texte_insere(&self, intermediaire: bool) -> String {
        texte_des(&self.references, intermediaire)
    }

    /// La note telle qu'elle se compose, brouillon tenu compris.
    ///
    /// C'est ce que la fenêtre montre : exactement ce que le document
    /// recevra — la référence tenue y figure comme Entrée la rangerait, à son
    /// rang.
    pub fn apercu(&self, intermediaire: bool) -> String {
        let mut etat = self.clone();

        match self.etape {
            Etape::Reference if self.cle_tenue().is_some() => {
                etat.prendre();
                etat.valider_page();
            }
            Etape::Page => {
                let page = self.saisie.trim().to_string();

                etat.brouillon.page = (!page.is_empty()).then_some(page);
                etat.ranger();
            }
            _ => {}
        }

        texte_des(&etat.references, intermediaire)
    }
}

/// Le texte d'une note, dans la syntaxe du fichier ouvert.
fn texte_des(references: &[Reference], intermediaire: bool) -> String {
    if intermediaire {
        crate::langage::serialiser::texte_appel(references)
    } else {
        crate::langage::serialiser::texte_citation(references)
    }
}

#[allow(dead_code)] // Rebranché par la complétion locale (feuille de route).
pub fn reserved_keys() -> Vec<iced::keyboard::Key> {
    use iced::keyboard::{Key, key::Named};

    vec![
        Key::Named(Named::ArrowUp),
        Key::Named(Named::ArrowDown),
        Key::Named(Named::Enter),
        Key::Named(Named::Tab),
        Key::Named(Named::Escape),
    ]
}

#[allow(dead_code)] // Rebranché par la complétion locale (feuille de route).
/// Repose les glyphes de notes : le document change, les numéros suivent.
///
/// C'est ici que « dynamique » a lieu — à chaque frappe, à chaque déplacement
/// du curseur. La règle de pliage est **une**, dans l'éditeur
/// (`EditorState::marques_a_jour`) : elle suit aussi les remplacements faits
/// par le composeur, qui ne passent pas par la boucle des messages.
/// Ouvre le composeur de notes, sur le `@` qui vient d'être tapé.
///
/// Le déclencheur **part** : le `@`, et ce qui a pu être tapé derrière, sont
/// mangés — leur place est dans le champ du panneau. Le document ne revoit
/// donc la note qu'à la fin, et Échap ne laisse rien. Le clavier part aux
/// champs ; l'éditeur le rend, et ne le reprend qu'à la fermeture.
pub fn ouvrir_composeur(
    state: &mut State,
    context: &crate::editor::completion::Context,
) -> Task<Message> {
    let cles = state
        .bibliography
        .iter()
        .map(|entry| {
            let auteurs = entry
                .auteurs
                .iter()
                .map(|(nom, prenom)| format!("{nom} {prenom}"))
                .collect::<Vec<_>>()
                .join(" ");
            let recherche = format!(
                "{auteurs} {} {}",
                entry.titre,
                entry.annee.clone().unwrap_or_default()
            );
            (entry.key.clone(), entry.kind.clone(), recherche)
        })
        .collect();

    // L'ancre est celle du `@` déclencheur : le mot tapé commence juste après.
    let ancre = context.start.saturating_sub(1);
    let manges = state.editor.cursor_offset() - ancre;

    // Toute réponse tinymist encore en vol n'a plus lieu d'être.

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
pub fn ouvrir_edition(
    state: &mut State,
    references: Vec<crate::langage::schema::Reference>,
    debut: usize,
    fin: usize,
) -> Task<Message> {
    let cles = state
        .bibliography
        .iter()
        .map(|entry| {
            let auteurs = entry
                .auteurs
                .iter()
                .map(|(nom, prenom)| format!("{nom} {prenom}"))
                .collect::<Vec<_>>()
                .join(" ");
            let recherche = format!(
                "{auteurs} {} {}",
                entry.titre,
                entry.annee.clone().unwrap_or_default()
            );
            (entry.key.clone(), entry.kind.clone(), recherche)
        })
        .collect();

    // Toute réponse tinymist encore en vol n'a plus lieu d'être.
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
pub fn composeur_touche(
    state: &mut State,
    key: &iced::keyboard::Key,
    modifiers: iced::keyboard::Modifiers,
) -> Option<Task<Message>> {
    use iced::keyboard::{Key, key::Named};

    // Le composeur fermé ne prend rien : sans cette garde, il volait
    // Tab, Entrée et Échap à toutes les fenêtres — la fiche de
    // référence ne pouvait ni naviguer, ni valider, ni fermer.
    state.composeur.as_ref()?;

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
pub fn composeur_update(state: &mut State, message: Message) -> Task<Message> {
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
pub fn fermer_composeur(state: &mut State, texte: Option<String>) -> Task<Message> {
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

            state
                .editor
                .replace_span(composeur.ancre, fin, &texte)
                .map(Message::Editor)
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

/// Va d'un endroit à un autre : le même document, ou un autre chapitre.
///
/// Un aller à la définition peut traverser le projet. Le trajet se fait donc en
/// deux temps lorsque le fichier visé n'est pas celui qui est ouvert : on retient
/// l'endroit, on ouvre le chapitre, et c'est son ouverture qui y pose le curseur.
pub fn jump_to(state: &mut State, path: &Path, line: usize, column: usize) -> Task<Message> {
    let current = state.editor.path.clone();

    if current.is_some_and(|open| crate::project::same_file(&open, path)) {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Un composeur sur trois clés, ouvert au déclencheur.
    ///
    /// Chaque clé porte son genre et son texte de recherche (auteurs, titre,
    /// année) — comme le construit l'application.
    fn composeur() -> Composeur {
        Composeur::ouvert(
            vec![
                (
                    "dupont2023".to_string(),
                    "article".to_string(),
                    "dupont jean théologie 2023".to_string(),
                ),
                (
                    "martin2020".to_string(),
                    "book".to_string(),
                    "martin marie histoire 2020".to_string(),
                ),
                (
                    "bernard2021".to_string(),
                    "article".to_string(),
                    "bernard paul droit 2021".to_string(),
                ),
            ],
            4,
        )
    }

    /// La recherche trouve un auteur, un titre ou une année — pas seulement
    /// la clé (R3 : « recherche auteur/titre/année/clé »).
    #[test]
    fn le_filtre_cherche_auteur_titre_et_annee() {
        let mut note = composeur();

        note.saisir("théologie".to_string());
        assert_eq!(note.visibles(), vec![0], "le titre se trouve");
        note.saisir("marie".to_string());
        assert_eq!(note.visibles(), vec![1], "l'auteur se trouve");
        note.saisir("2021".to_string());
        assert_eq!(note.visibles(), vec![2], "l'année se trouve");
        note.saisir("martin2020".to_string());
        assert_eq!(note.visibles(), vec![1], "la clé se trouve");
        note.saisir("nulle part".to_string());
        assert!(note.visibles().is_empty(), "rien d'autre ne ment");
    }

    /// La boucle complète : Tab donne la page, Tab propose une autre
    /// référence, et la note entière s'écrit à la fin.
    #[test]
    fn la_boucle_compose_une_note_a_plusieurs_references() {
        let mut note = composeur();

        note.saisir("dup".to_string());
        note.avancer(); // prendre dupont2023 → étape page

        assert_eq!(note.etape, Etape::Page);

        note.saisir("p. 45".to_string());
        note.avancer(); // valider la page → étape référence

        assert_eq!(note.etape, Etape::Reference);
        assert_eq!(note.saisie, "");

        note.saisir("mar".to_string());
        note.avancer(); // prendre martin2020 → étape page

        assert_eq!(note.etape, Etape::Page);

        let texte = note.terminer(true); // la page reste vide, on insère

        assert_eq!(
            texte.as_deref(),
            Some("@dupont2023[p. 45]; @martin2020"),
            "une seule note, ses deux références, la page de la première"
        );
    }

    /// Les flèches parcourent les lignes de la note, puis les clés : poser
    /// sur une ligne la remet sous les doigts.
    #[test]
    fn les_fleches_parcourent_la_note_puis_les_cles() {
        let mut note = composeur();
        note.saisir("dup".to_string());
        note.avancer();
        note.saisir("p. 45".to_string());
        note.avancer();
        note.saisir("mar".to_string());
        note.avancer();
        note.saisir("p. 7".to_string());
        note.avancer(); // la note tient deux références

        assert_eq!(note.references.len(), 2);
        assert!(
            note.sur_palette(),
            "après la page, la sélection attend sur les clés"
        );

        note.deplacer(-1); // remonte sur la dernière ligne de la note

        assert_eq!(
            note.rang,
            Some(1),
            "la deuxième référence est sous les doigts"
        );
        assert_eq!(note.brouillon.cle, "martin2020");

        note.deplacer(-1); // puis la première

        assert_eq!(note.rang, Some(0));
        assert_eq!(
            note.saisie, "dupont2023",
            "sa clé revient au champ, prête à s'éditer"
        );

        note.deplacer(1);
        note.deplacer(1); // au-delà des lignes, les clés

        assert!(note.sur_palette(), "la sélection est sur les clés");
    }

    /// Les flèches ne sortent jamais de la fenêtre.
    #[test]
    fn les_fleches_ne_sortent_pas_de_la_fenetre() {
        let mut note = composeur();

        note.deplacer(-1);
        assert_eq!(note.position, 0, "pas au-dessus de la première rangée");

        note.deplacer(1);
        note.deplacer(1);
        note.deplacer(1);
        note.deplacer(1);
        note.deplacer(1);
        assert_eq!(
            note.position,
            note.visibles().len().saturating_sub(1),
            "pas sous la dernière rangée"
        );
    }

    /// Le geste d'hier survit : `@dup` + Entrée insère la clé choisie.
    #[test]
    fn entree_avec_une_note_vide_prend_la_reference() {
        let mut note = composeur();
        note.saisir("martin".to_string());

        assert_eq!(
            note.terminer(true).as_deref(),
            Some("@martin2020"),
            "la première référence se prend d'Entrée, sans passer par la page"
        );
    }

    /// Proposer une autre référence est une proposition : Entrée termine la
    /// note **sans** prendre la ligne surlignée.
    #[test]
    fn entree_apres_une_note_commencee_ne_prend_plus_rien() {
        let mut note = composeur();

        note.avancer(); // prendre dupont2023
        note.avancer(); // valider sa page (vide)

        assert_eq!(note.etape, Etape::Reference);
        assert_eq!(
            note.terminer(true).as_deref(),
            Some("@dupont2023"),
            "la dernière liste n'était qu'une proposition"
        );
    }

    /// Échap ne laisse rien : rien n'est jamais écrit sans `terminer`.
    #[test]
    fn rien_ne_s_ecrit_sans_terminer() {
        let note = composeur();

        // Le composeur ne porte aucune tâche d'écriture : c'est son absence
        // de méthode d'insertion qui le garantit, et l'appelant referme.
        assert!(note.references.is_empty());
    }

    /// Le filtre garde ce qui commence par la saisie devant ce qui la contient,
    /// et la sélection ne sort jamais de la liste.
    #[test]
    fn le_filtre_et_la_selection_restent_bornes() {
        let mut note = composeur();

        note.saisir("2020".to_string());
        assert_eq!(note.visibles(), vec![1]);

        note.deplacer(1);
        assert_eq!(note.selection, 0, "pas au-delà de la liste");

        note.saisir(String::new());
        note.deplacer(1);
        note.deplacer(1);
        note.deplacer(1);
        assert_eq!(note.selection, 2, "dernière ligne, pas plus");
    }

    /// Un fichier Typst brut reçoit les appels qui rendent la même note.
    #[test]
    fn un_fichier_brut_recoit_les_appels_de_cite() {
        let mut note = composeur();

        note.avancer(); // prendre dupont2023 → étape page
        note.saisir("p. 45".to_string());
        note.avancer(); // valider la page → étape référence
        note.saisir("mar".to_string());
        note.avancer(); // prendre martin2020 → étape page

        // C'est Entrée qui range : le texte lu est celui qu'elle écrirait.
        assert_eq!(
            note.terminer(false).as_deref(),
            Some("#cite(<dupont2023>, supplement: [p. 45]) #cite(<martin2020>)")
        );
    }

    /// Revenir sur une citation déjà écrite : sa dernière référence se tient
    /// sous les doigts, et sa page survit même à une Entrée immédiate.
    #[test]
    fn une_citation_ecrite_se_recharge_pour_etre_reecrite() {
        let mut note = composeur();

        note.charger(vec![
            Reference::new("dupont2023").avec_page("p. 45"),
            Reference::new("martin2020").avec_page("pp. 12-15"),
        ]);

        assert_eq!(note.references.len(), 2, "la note entière est là");
        assert_eq!(note.rang, Some(1), "et l'on tient la dernière");
        assert_eq!(note.saisie, "martin2020", "sa clé tient dans le champ");

        // Entrée tout de suite : la page existe déjà, elle ne se perd pas.
        assert_eq!(
            note.terminer(true).as_deref(),
            Some("@dupont2023[p. 45]; @martin2020[pp. 12-15]"),
            "la note ressort entière, inchangée"
        );
    }

    /// Choisir une référence de la note la remet sous les doigts **à son
    /// rang** : la réécriture ne dérange pas l'ordre de la note.
    #[test]
    fn choisir_une_reference_garde_sa_place() {
        let mut note = composeur();

        note.charger(vec![
            Reference::new("dupont2023").avec_page("p. 1"),
            Reference::new("martin2020"),
        ]);

        note.choisir(0); // la première, pas la dernière

        assert_eq!(note.saisie, "dupont2023");
        assert_eq!(note.etape, Etape::Reference);

        note.avancer(); // → étape page, sa page sous les doigts
        assert_eq!(note.saisie, "p. 1");

        note.saisir("p. 9".to_string());
        note.avancer(); // valider : rang 0, l'ordre tient

        assert_eq!(
            note.terminer(true).as_deref(),
            Some("@dupont2023[p. 9]; @martin2020"),
            "la première corrigée, la seconde à sa suite — rien n'a bougé"
        );
    }
}
