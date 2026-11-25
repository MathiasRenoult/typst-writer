//! L'état de vue du plugin des personnes : sa liste, son filtre, sa fiche.
//!
//! Les entrées appartiennent au projet (`personnes.toml`) ; ici vivent ce
//! qu'une vue retient : le filtre, la fenêtre et la fiche ouverte.

use crate::project::personnes::EntreePersonne;

/// La fiche d'édition d'une personne.
#[derive(Debug, Clone)]
pub struct EditionPersonne {
    /// La clé — identifiant appelé par `{{p clé}}`. Immuable en édition.
    pub cle: String,
    /// Le nom complet.
    pub nom: String,
    /// Le nom simple.
    pub nom_simple: String,
    /// L'alias ou le nom de plume.
    pub alias: String,
    /// La clé de classement dans l'index.
    pub tri: String,
    /// Les dates libres historiques.
    pub dates: String,
    /// L'année de naissance.
    pub naissance: String,
    /// L'année de mort.
    pub mort: String,
    /// La qualité ou la fonction.
    pub qualite: String,
    /// Les variantes cherchables, séparées par des virgules.
    pub variantes: String,
    /// Une entrée neuve, pas encore écrite.
    pub creation: bool,
}

impl EditionPersonne {
    /// Une fiche neuve : la clé se choisit.
    pub fn nouvelle() -> Self {
        Self {
            cle: String::new(),
            nom: String::new(),
            nom_simple: String::new(),
            alias: String::new(),
            tri: String::new(),
            dates: String::new(),
            naissance: String::new(),
            mort: String::new(),
            qualite: String::new(),
            variantes: String::new(),
            creation: true,
        }
    }

    /// La fiche d'une entrée existante ; sa clé est déjà citée, donc figée.
    pub fn depuis_entree(entree: &EntreePersonne) -> Self {
        Self {
            cle: entree.cle.clone(),
            nom: entree.nom.clone(),
            nom_simple: entree.nom_simple.clone().unwrap_or_default(),
            alias: entree.alias.clone().unwrap_or_default(),
            tri: entree.tri.clone().unwrap_or_default(),
            dates: entree.dates.clone().unwrap_or_default(),
            naissance: entree.naissance.clone().unwrap_or_default(),
            mort: entree.mort.clone().unwrap_or_default(),
            qualite: entree.qualite.clone().unwrap_or_default(),
            variantes: entree.variantes.join(", "),
            creation: false,
        }
    }

    /// L'entrée de registre correspondant à la fiche.
    pub fn entree(&self) -> EntreePersonne {
        let variantes = self
            .variantes
            .split(',')
            .map(str::trim)
            .filter(|variante| !variante.is_empty())
            .map(str::to_string)
            .collect();

        EntreePersonne {
            cle: self.cle.trim().to_string(),
            nom: self.nom.trim().to_string(),
            nom_simple: champ(self.nom_simple.as_str()),
            alias: champ(self.alias.as_str()),
            tri: champ(self.tri.as_str()),
            dates: champ(self.dates.as_str()),
            naissance: champ(self.naissance.as_str()),
            mort: champ(self.mort.as_str()),
            qualite: champ(self.qualite.as_str()),
            variantes,
        }
    }
}

/// Une chaîne vide devient l'absence.
fn champ(valeur: &str) -> Option<String> {
    let valeur = valeur.trim();
    if valeur.is_empty() {
        None
    } else {
        Some(valeur.to_string())
    }
}

/// Une option de la liste déroulante du panneau d'insertion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonneChoix {
    /// La clé stable, seule écrite dans la balise.
    pub cle: String,
    /// Le libellé cherché et affiché : nom, formes et clé.
    pub libelle: String,
}

impl PersonneChoix {
    /// Construit le libellé d'une entrée : toutes ses formes sont
    /// cherchables, mais la clé reste visible pour la source.
    pub fn depuis(entree: &EntreePersonne) -> Self {
        let mut formes = vec![entree.nom_affiche().to_string()];
        if entree.nom_simple_affiche() != entree.nom_affiche() {
            formes.push(entree.nom_simple_affiche().to_string());
        }
        if entree.alias_affiche() != entree.nom_affiche()
            && entree.alias_affiche() != entree.nom_simple_affiche()
        {
            formes.push(entree.alias_affiche().to_string());
        }
        for variante in &entree.variantes {
            if !formes.iter().any(|forme| forme == variante) {
                formes.push(variante.clone());
            }
        }
        formes.push(entree.cle.clone());

        Self {
            cle: entree.cle.clone(),
            libelle: formes.join(" — "),
        }
    }
}

impl std::fmt::Display for PersonneChoix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.libelle)
    }
}

/// Le panneau d'insertion d'une personne : une liste cherchable, tous les
/// champs du registre, et l'expression d'affichage facultative.
#[derive(Debug, Clone)]
pub struct PersonnePanneau {
    /// Le texte saisi dans le champ de recherche de la liste.
    pub recherche: String,
    /// La clé sélectionnée, si une entrée est choisie.
    pub selection: Option<String>,
    /// La fiche en cours d'édition : création ou mise à jour d'un choix.
    pub edition: EditionPersonne,
    /// Le suffixe après `|` ; vide = nom simple par défaut.
    pub expression: String,
    /// La balise existante à remplacer, pour un futur clic droit.
    pub remplacement: Option<std::ops::Range<usize>>,
}

impl PersonnePanneau {
    /// Un panneau neuf, avec sa liste déjà remplie.
    pub fn nouveau(_entrees: &[EntreePersonne]) -> Self {
        Self {
            recherche: String::new(),
            selection: None,
            edition: EditionPersonne::nouvelle(),
            expression: String::new(),
            remplacement: None,
        }
    }

    /// La fiche d'une entrée déjà choisie.
    pub fn choisir(&mut self, entree: &EntreePersonne) {
        self.selection = Some(entree.cle.clone());
        self.edition = EditionPersonne::depuis_entree(entree);
    }

    /// Repart d'une fiche vierge et retire la sélection de la liste.
    pub fn nouvelle(&mut self, _entrees: &[EntreePersonne]) {
        self.selection = None;
        self.edition = EditionPersonne::nouvelle();
        self.expression.clear();
        self.recherche.clear();
    }

    /// La personne actuellement sélectionnée dans la liste.
    pub fn choix_courant(&self, entrees: &[EntreePersonne]) -> Option<PersonneChoix> {
        let cle = self.selection.as_deref()?;
        entrees
            .iter()
            .find(|e| e.cle == cle)
            .map(PersonneChoix::depuis)
    }

    /// Les options visibles, filtrées par le champ de recherche. Le libellé
    /// contient le nom complet, le nom simple, l'alias et la clé : toutes les
    /// formes restent donc cherchables.
    pub fn options(&self, entrees: &[EntreePersonne]) -> Vec<PersonneChoix> {
        let requete = self.recherche.trim().to_lowercase();
        entrees
            .iter()
            .map(PersonneChoix::depuis)
            .filter(|choix| requete.is_empty() || choix.libelle.to_lowercase().contains(&requete))
            .collect()
    }

    /// Le texte exact qui sera inséré, ou le motif qui l'empêche.
    pub fn texte(&self) -> Result<String, String> {
        let cle = self.edition.cle.trim();
        if !EntreePersonne::cle_valide(cle) {
            return Err("Choisissez une personne ou donnez-lui une clé valide.".to_string());
        }

        let expression = self.expression.trim();
        if expression.is_empty() {
            Ok(format!("{{{{p {cle}}}}}"))
        } else {
            Ok(format!("{{{{p {cle}|{expression}}}}}"))
        }
    }
}

/// Ce qu'une vue retient des personnes citées.
#[derive(Debug, Clone, Default)]
pub struct PersonnesEtat {
    /// Les entrées du projet, dans l'ordre de l'auteur.
    pub entrees: Vec<EntreePersonne>,
    /// La fenêtre du registre est-elle ouverte ?
    pub fenetre: bool,
    /// Le filtre de la liste.
    pub filtre: String,
    /// La fiche d'édition de la fenêtre de registre.
    pub edition: Option<EditionPersonne>,
    /// Le panneau d'insertion d'une balise de personne.
    pub panneau: Option<PersonnePanneau>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn augustin() -> EntreePersonne {
        EntreePersonne {
            cle: "augustin".into(),
            nom: "Augustin d'Hippone".into(),
            nom_simple: Some("Augustin".into()),
            alias: Some("Saint Augustin".into()),
            tri: Some("Augustin".into()),
            dates: Some("354-430".into()),
            naissance: Some("354".into()),
            mort: Some("430".into()),
            qualite: Some("évêque d'Hippone".into()),
            variantes: Vec::new(),
        }
    }

    #[test]
    fn la_fiche_du_panneau_remplit_tous_les_champs_du_registre() {
        let entree = augustin();
        let edition = EditionPersonne::depuis_entree(&entree);
        let relue = edition.entree();

        assert_eq!(relue.cle, "augustin");
        assert_eq!(relue.nom, "Augustin d'Hippone");
        assert_eq!(relue.nom_simple.as_deref(), Some("Augustin"));
        assert_eq!(relue.alias.as_deref(), Some("Saint Augustin"));
        assert_eq!(relue.naissance.as_deref(), Some("354"));
        assert_eq!(relue.mort.as_deref(), Some("430"));
        assert_eq!(relue.qualite.as_deref(), Some("évêque d'Hippone"));
    }

    #[test]
    fn un_panneau_ecrit_la_cle_seule_par_defaut() {
        let entree = augustin();
        let mut panneau = PersonnePanneau::nouveau(std::slice::from_ref(&entree));

        assert!(panneau.texte().is_err(), "aucune clé au départ");
        panneau.choisir(&entree);
        assert_eq!(panneau.texte().unwrap(), "{{p augustin}}");
        assert_eq!(panneau.selection.as_deref(), Some("augustin"));

        panneau.expression = "nom+dates".into();
        assert_eq!(panneau.texte().unwrap(), "{{p augustin|nom+dates}}");
    }

    #[test]
    fn la_liste_cherche_toutes_les_formes_et_la_cle() {
        let entree = augustin();
        let mut panneau = PersonnePanneau::nouveau(std::slice::from_ref(&entree));
        let choix = PersonneChoix::depuis(&entree);

        assert!(choix.libelle.contains("Augustin d'Hippone"));
        assert!(choix.libelle.contains("Saint Augustin"));
        assert!(choix.libelle.contains("augustin"));

        panneau.choisir(&entree);
        assert_eq!(
            panneau.choix_courant(std::slice::from_ref(&entree)),
            Some(choix)
        );
    }

    #[test]
    fn le_champ_de_recherche_filtre_la_liste() {
        let entree = augustin();
        let mut panneau = PersonnePanneau::nouveau(std::slice::from_ref(&entree));
        let entrees = std::slice::from_ref(&entree);

        panneau.recherche = "saint".into();
        assert_eq!(panneau.options(entrees).len(), 1);

        panneau.recherche = "personne inconnue".into();
        assert!(panneau.options(entrees).is_empty());
    }

    #[test]
    fn nouvelle_fiche_retire_la_selection_et_l_expression() {
        let entree = augustin();
        let mut panneau = PersonnePanneau::nouveau(std::slice::from_ref(&entree));
        panneau.choisir(&entree);
        panneau.expression = "auto".into();

        panneau.nouvelle(std::slice::from_ref(&entree));

        assert_eq!(panneau.selection, None);
        assert!(panneau.expression.is_empty());
        assert!(panneau.texte().is_err());
    }
}
