//! Le composeur de notes de références.
//!
//! La fenêtre de suggestions n'est plus une liste qu'on accepte : c'est un
//! composeur, à étapes, où Tab avance. On choisit une référence ; Tab propose
//! son numéro de page ; Tab propose une autre référence **dans la même note** ;
//! et ainsi de suite. Le document ne reçoit la note qu'à la fin, d'un seul
//! tenant. Échap ne laisse rien.
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

    /// Les références de la note, dans l'ordre où elles s'inscriront. La
    /// dernière est celle dont on attend la page, à l'étape « page ».
    pub references: Vec<Reference>,

    /// Ce qui a été tapé dans le champ de l'étape : le filtre des clés à
    /// l'étape « référence », la mention de page à l'étape « page ».
    pub saisie: String,

    /// La ligne surlignée dans la liste.
    pub selection: usize,

    /// L'offset, dans le document, où la note s'insérera : celui du `@`
    /// déclencheur, que le composeur a mangé.
    pub ancre: usize,

    /// La page de la référence en cours d'édition, à réafficher à l'étape
    /// « page ». Chargée depuis une note déjà écrite, et consommée à la
    /// première prise.
    pub page_en_attente: String,

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
            saisie: String::new(),
            selection: 0,
            ancre,
            page_en_attente: String::new(),
            edition: None,
        }
    }

    /// Charge une note déjà écrite, pour la rééditer.
    ///
    /// La dernière référence devient celle qu'on tient : sa clé dans le champ,
    /// sa page en attente. Les autres restent dans la note, et l'aperçu montre
    /// le tout — c'est la même machine qu'en création, seulement pré-remplie.
    pub fn charger(&mut self, mut references: Vec<Reference>) {
        if let Some(derniere) = references.pop() {
            self.saisir(derniere.cle);
            self.page_en_attente = derniere.page.unwrap_or_default();

            // La sélection doit retomber sur la clé existante, même si le
            // filtre en montre d'autres devant elle.
            if let Some(position) = self.visibles().iter().position(|index| {
                self.cles[*index].0 == self.saisie
            }) {
                self.selection = position;
            }
        }

        self.references = references;
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

    /// Ce qui est tapé dans le champ de l'étape courante.
    pub fn saisir(&mut self, texte: String) {
        self.saisie = texte;
        self.selection = 0;
    }

    /// Déplace la sélection, sans jamais sortir de la liste filtrée.
    pub fn deplacer(&mut self, delta: isize) {
        let dernier = self.visibles().len().saturating_sub(1) as isize;
        let suivant = (self.selection as isize + delta).clamp(0, dernier);

        self.selection = suivant as usize;
    }

    /// Tab : prendre la référence choisie, ou valider la page saisie.
    ///
    /// La boucle est celle du composeur : référence → page → référence → …
    pub fn avancer(&mut self) {
        match self.etape {
            Etape::Reference => self.prendre(),
            Etape::Page => self.valider_page(),
        }
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

    /// Prend la clé tenue dans le champ, et propose son numéro de page.
    fn prendre(&mut self) {
        let Some(cle) = self.cle_tenue() else {
            return;
        };

        // La page déjà portée par une citation en cours de réécriture rejoint
        // la référence d'emblée : Entrée sans passer par l'étape page ne la
        // perd pas.
        let page = std::mem::take(&mut self.page_en_attente);
        let mut reference = Reference::new(cle);

        if !page.is_empty() {
            reference = reference.avec_page(page);
        }

        self.saisie = reference.page.clone().unwrap_or_default();
        self.references.push(reference);
        self.etape = Etape::Page;
    }

    /// Valide la mention de page — même vide, une page n'est pas obligatoire
    /// — et propose une autre référence dans la même note.
    fn valider_page(&mut self) {
        let page = self.saisie.trim().to_string();

        if let Some(reference) = self.references.last_mut() {
            reference.page = (!page.is_empty()).then_some(page);
        }

        self.etape = Etape::Reference;
        self.saisie.clear();
        self.selection = 0;
    }

    /// Entrée : terminer, et dire ce qui s'insère.
    ///
    /// À l'étape « page », la page saisie valide la référence en cours. À
    /// l'étape « référence », la clé **tenue dans le champ** — tapée, ou celle
    /// d'une citation en cours de réécriture — rejoint la note ; un champ vide,
    /// lui, ne prend rien : la dernière liste n'était qu'une proposition. C'est
    /// le geste d'hier : `@dup` + Entrée insère `@dupont2023`.
    pub fn terminer(&mut self, intermediaire: bool) -> Option<String> {
        match self.etape {
            Etape::Page => self.valider_page(),
            Etape::Reference if !self.saisie.is_empty() => self.prendre(),
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

    /// La note telle qu'elle se compose, page en cours comprise.
    ///
    /// C'est ce que la fenêtre montre : exactement ce que le document
    /// recevra — la clé tenue dans le champ y figure comme Entrée la prendrait.
    pub fn apercu(&self, intermediaire: bool) -> String {
        let mut references = self.references.clone();

        match self.etape {
            Etape::Reference => {
                if let Some(cle) = self.cle_tenue() {
                    let mut reference = Reference::new(cle);

                    if !self.page_en_attente.is_empty() {
                        reference = reference.avec_page(self.page_en_attente.clone());
                    }

                    references.push(reference);
                }
            }
            Etape::Page => {
                let page = self.saisie.trim().to_string();

                if let Some(derniere) = references.last_mut() {
                    derniere.page = (!page.is_empty()).then_some(page);
                }
            }
        }

        texte_des(&references, intermediaire)
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
        let mut composeur = composeur();

        composeur.saisir("dup".to_string());
        composeur.avancer(); // prendre dupont2023 → étape page

        assert_eq!(composeur.etape, Etape::Page);

        composeur.saisir("p. 45".to_string());
        composeur.avancer(); // valider la page → étape référence

        assert_eq!(composeur.etape, Etape::Reference);
        assert_eq!(composeur.saisie, "");

        composeur.saisir("mar".to_string());
        composeur.avancer(); // prendre martin2020 → étape page

        assert_eq!(composeur.etape, Etape::Page);

        let texte = composeur.terminer(true); // la page reste vide, on insère

        assert_eq!(
            texte.as_deref(),
            Some("@dupont2023[p. 45]; @martin2020"),
            "une seule note, ses deux références, la page de la première"
        );
    }

    /// Le geste d'hier survit : `@dup` + Entrée insère la clé choisie.
    #[test]
    fn entree_avec_une_note_vide_prend_la_reference() {
        let mut composeur = composeur();
        composeur.saisir("martin".to_string());

        assert_eq!(
            composeur.terminer(true).as_deref(),
            Some("@martin2020"),
            "la première référence se prend d'Entrée, sans passer par la page"
        );
    }

    /// Proposer une autre référence est une proposition : Entrée termine la
    /// note **sans** prendre la ligne surlignée.
    #[test]
    fn entree_apres_une_note_commencee_ne_prend_plus_rien() {
        let mut composeur = composeur();

        composeur.avancer(); // prendre dupont2023
        composeur.avancer(); // valider sa page (vide)

        assert_eq!(composeur.etape, Etape::Reference);
        assert_eq!(
            composeur.terminer(true).as_deref(),
            Some("@dupont2023"),
            "la dernière liste n'était qu'une proposition"
        );
    }

    /// Échap ne laisse rien : rien n'est jamais écrit sans `terminer`.
    #[test]
    fn rien_ne_s_ecrit_sans_terminer() {
        let composeur = composeur();

        // Le composeur ne porte aucune tâche d'écriture : c'est son absence
        // de méthode d'insertion qui le garantit, et l'appelant referme.
        assert!(composeur.references.is_empty());
    }

    /// Le filtre garde ce qui commence par la saisie devant ce qui la contient,
    /// et la sélection ne sort jamais de la liste.
    #[test]
    fn le_filtre_et_la_selection_restent_bornes() {
        let mut composeur = composeur();

        composeur.saisir("2020".to_string());
        assert_eq!(composeur.visibles(), vec![1]);

        composeur.deplacer(1);
        assert_eq!(composeur.selection, 0, "pas au-delà de la liste");

        composeur.saisir(String::new());
        composeur.deplacer(1);
        composeur.deplacer(1);
        composeur.deplacer(1);
        assert_eq!(composeur.selection, 2, "dernière ligne, pas plus");
    }

    /// Revenir sur une citation déjà écrite : elle se charge entière — clé
    /// sélectionnée, page réaffichée — et sa page survit même à une Entrée
    /// immédiate.
    #[test]
    fn une_citation_ecrite_se_recharge_pour_etre_reecrite() {
        let mut note = composeur();

        note.charger(vec![
            Reference::new("dupont2023").avec_page("p. 45"),
            Reference::new("martin2020").avec_page("pp. 12-15"),
        ]);

        assert_eq!(note.references.len(), 1, "les autres restent la note");
        assert_eq!(note.saisie, "martin2020", "la clé tient dans le champ");
        assert_eq!(note.choisie(), Some("martin2020"), "et elle est choisie");

        // Entrée tout de suite : la page existe déjà, elle ne se perd pas.
        assert_eq!(
            note.terminer(true).as_deref(),
            Some("@dupont2023[p. 45]; @martin2020[pp. 12-15]"),
            "la note ressort entière, inchangée"
        );

        // Et l'étape page l'aurait montrée toute prête.
        let mut recharge = composeur();
        recharge.charger(vec![Reference::new("dupont2023").avec_page("p. 45")]);
        recharge.avancer();

        assert_eq!(recharge.etape, Etape::Page);
        assert_eq!(recharge.saisie, "p. 45", "la page est sous les doigts");

        recharge.saisir("p. 47".to_string());

        assert_eq!(
            recharge.terminer(true).as_deref(),
            Some("@dupont2023[p. 47]"),
            "et sa correction réécrit la citation"
        );
    }

    /// Un fichier Typst brut reçoit les appels qui rendent la même note.
    #[test]
    fn un_fichier_brut_recoit_les_appels_de_cite() {
        let mut composeur = composeur();

        composeur.avancer(); // prendre dupont2023 → étape page
        composeur.saisir("p. 45".to_string());
        composeur.avancer(); // valider la page → étape référence
        composeur.saisir("mar".to_string());
        composeur.avancer(); // prendre martin2020 → étape page

        assert_eq!(
            composeur.texte_insere(false),
            "#cite(<dupont2023>, supplement: [p. 45]) #cite(<martin2020>)"
        );
    }
}
