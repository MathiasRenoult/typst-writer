//! Les plugins natifs : métadonnées dans `base::valeurs`, état par vue,
//! un module par add-on.

pub mod bible;
pub mod bibliographie;
pub mod glossaire;

pub use bible::BibleEtat;
pub use bibliographie::BibliographieEtat;
pub use glossaire::GlossaireEtat;

/// L'état des plugins d'une vue.
#[derive(Debug, Clone, Default)]
pub struct Plugins {
    /// Le plugin biblique : références, survol, statistiques.
    pub bible: BibleEtat,
    /// Le plugin bibliographie : la fiche ouverte.
    pub bibliographie: BibliographieEtat,
    /// Le plugin glossaire : la fiche ouverte.
    pub glossaire: GlossaireEtat,
}

impl Plugins {
    /// Les plugins d'une vue neuve, réglés par le fichier de normes du projet.
    ///
    /// Une vue doit naître avec les choix du document, comme l'interface
    /// active : sans cela, un second onglet du même projet afficherait les
    /// défauts de l'application plutôt que la traduction et l'ordre voulus.
    pub fn depuis_les_normes(normes: &crate::mise_en_page::modele::MiseEnPage) -> Self {
        let mut plugins = Self::default();
        plugins.bible.traduction = crate::plugins::bible::Traduction::depuis_code(
            &normes.parametres_plugins.biblique.traduction,
        )
        .unwrap_or_default();
        plugins.bible.tri = if normes.parametres_plugins.biblique.tri == "canonique" {
            crate::plugins::bible::TriReferences::Canonique
        } else {
            crate::plugins::bible::TriReferences::Apparition
        };
        plugins
    }

    /// Échange l'état des trois plugins entre une vue et l'état actif.
    ///
    /// C'est un `std::mem::swap` par plugin, pas une copie : après l'appel,
    /// `autre` porte l'ancien état de `self`. Les trois états voyagent
    /// ensemble, comme le reste du contexte de `Vue::echanger`.
    pub fn echanger(&mut self, autre: &mut Plugins) {
        std::mem::swap(&mut self.bible, &mut autre.bible);
        std::mem::swap(&mut self.bibliographie, &mut autre.bibliographie);
        std::mem::swap(&mut self.glossaire, &mut autre.glossaire);
    }
}

#[cfg(test)]
mod epreuves {
    use super::*;

    #[test]
    fn echanger_deplace_l_etat_du_plugin() {
        let mut a = Plugins::default();
        let mut b = Plugins::default();
        a.bible.repliees = true;
        b.bible.repliees = false;
        a.echanger(&mut b);
        assert!(!a.bible.repliees);
        assert!(b.bible.repliees);
    }

    /// Une vue neuve prend la traduction et l'ordre inscrits dans les normes.
    #[test]
    fn une_vue_neuve_suit_les_normes_du_projet() {
        let mut normes = crate::mise_en_page::modele::MiseEnPage::default();
        normes.parametres_plugins.biblique.traduction = "darby".to_string();
        normes.parametres_plugins.biblique.tri = "canonique".to_string();

        let plugins = Plugins::depuis_les_normes(&normes);

        assert_eq!(
            plugins.bible.traduction,
            crate::plugins::bible::Traduction::Darby
        );
        assert_eq!(
            plugins.bible.tri,
            crate::plugins::bible::TriReferences::Canonique
        );
    }
}
