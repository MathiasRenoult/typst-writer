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
}
