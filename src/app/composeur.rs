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

use crate::langage::schema::Reference;

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

    /// Les clés citables du projet, avec leur genre, telles qu'elles étaient
    /// à l'ouverture : le composeur n'écoute personne, il filtre sa liste.
    pub cles: Vec<(String, String)>,

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
    pub fn champ_id() -> iced::widget::Id {
        iced::widget::Id::new("composeur-champ")
    }

    /// Ouvre le composeur sur la liste entière des clés du projet.
    pub fn ouvert(cles: Vec<(String, String)>, ancre: usize) -> Self {
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
    /// Deux passes, comme la liste de complétion : d'abord ce qui *commence*
    /// par le filtre, puis ce qui le *contient*.
    pub fn visibles(&self) -> Vec<usize> {
        if self.saisie.is_empty() {
            return (0..self.cles.len()).collect();
        }

        let voulu = self.saisie.to_lowercase();
        let mut debuts = Vec::new();
        let mut contient = Vec::new();

        for (index, (cle, _)) in self.cles.iter().enumerate() {
            let candidate = cle.to_lowercase();

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
    pub fn sur_palette(&self) -> bool {
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
        let mut out = String::new();
        crate::langage::generer::generer_citation(references, &mut out);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Un composeur sur trois clés, ouvert au déclencheur.
    fn composeur() -> Composeur {
        Composeur::ouvert(
            vec![
                ("dupont2023".to_string(), "article".to_string()),
                ("martin2020".to_string(), "book".to_string()),
                ("bernard2021".to_string(), "article".to_string()),
            ],
            4,
        )
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

        assert_eq!(note.rang, Some(1), "la deuxième référence est sous les doigts");
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
