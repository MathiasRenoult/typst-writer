//! Valeurs partagées : clés de plugins et activation des add-ons.
//!
//! L'activation suit les normes du document ; elle est stockée en champs
//! nommés pour rester compatible avec les documents existants. Toutes les
//! clés publiques sont définies ici et réutilisées par le registre, les
//! messages et l'interface : une seule orthographe, un seul endroit à changer.

use serde::{Deserialize, Serialize};

/// Clé de l'add-on bibliographie.
pub const CLE_BIBLIOGRAPHIE: &str = "bibliographie";
/// Clé de l'add-on glossaire.
pub const CLE_GLOSSAIRE: &str = "glossaire";
/// Clé de l'add-on graphiques.
pub const CLE_GRAPHIQUES: &str = "graphiques";
/// Clé de l'add-on des références bibliques.
pub const CLE_REFERENCES_BIBLIQUES: &str = "references_bibliques";
/// Clé de l'add-on des callouts.
pub const CLE_CALLOUTS: &str = "callouts";
/// Clé de l'add-on des personnes citées.
pub const CLE_PERSONNES: &str = "personnes";

/// Les fonctionnalités du projet : chacune se débranche, et son interface
/// disparaît avec elle — ses boutons, ses sections de panneau.
///
/// Même forme et même sérialisation que l'ancien `Fonctionnalites` : les
/// documents existants restent lisibles. Le défaut est « tout actif ».
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AddOns {
    /// La bibliographie : ses boutons, sa section, sa fenêtre.
    pub bibliographie: bool,

    /// Le glossaire : ses boutons, sa section, sa fenêtre.
    pub glossaire: bool,

    /// Les graphiques (primaviz) : le bouton d'insertion.
    pub graphiques: bool,

    /// L'add-on des références bibliques : son insertion, son survol, sa trace.
    pub references_bibliques: bool,

    /// Les callouts : les citations encadrées et leurs tons.
    #[serde(default = "actif_par_defaut")]
    pub callouts: bool,

    /// Les personnes citées : la balise `{{p …}}` et son index.
    #[serde(default = "actif_par_defaut")]
    pub personnes: bool,
}

/// Un add-on nouveau est actif dans les documents qui ne le connaissent pas.
fn actif_par_defaut() -> bool {
    true
}

impl Default for AddOns {
    fn default() -> Self {
        Self {
            bibliographie: true,
            glossaire: true,
            graphiques: true,
            references_bibliques: true,
            callouts: true,
            personnes: true,
        }
    }
}

impl AddOns {
    /// Bascule un add-on par sa clé. Une clé inconnue ne fait rien : un
    /// message d'une version future ne corrompt pas les normes.
    pub fn basculer(&mut self, cle: &str, actif: bool) {
        match cle {
            CLE_BIBLIOGRAPHIE => self.bibliographie = actif,
            CLE_GLOSSAIRE => self.glossaire = actif,
            CLE_GRAPHIQUES => self.graphiques = actif,
            CLE_REFERENCES_BIBLIQUES => self.references_bibliques = actif,
            CLE_CALLOUTS => self.callouts = actif,
            CLE_PERSONNES => self.personnes = actif,
            _ => {}
        }
    }

    /// Vrai si l'add-on de cette clé est actif. Une clé inconnue est active :
    /// un plugin futur naît branché dans les anciens documents.
    pub fn actif(&self, cle: &str) -> bool {
        match cle {
            CLE_BIBLIOGRAPHIE => self.bibliographie,
            CLE_GLOSSAIRE => self.glossaire,
            CLE_GRAPHIQUES => self.graphiques,
            CLE_REFERENCES_BIBLIQUES => self.references_bibliques,
            CLE_CALLOUTS => self.callouts,
            CLE_PERSONNES => self.personnes,
            _ => true,
        }
    }
}

/// La description d'un plugin compilé avec l'application.
pub struct PluginDesc {
    /// Clé stable, identique à celle des réglages.
    pub cle: &'static str,
    /// Nom affiché.
    pub nom: &'static str,
    /// Aide courte, affichée sous le nom.
    pub aide: &'static str,
}

impl PluginDesc {
    /// Construit une description de plugin à partir de ses trois libellés.
    pub const fn new(cle: &'static str, nom: &'static str, aide: &'static str) -> Self {
        Self { cle, nom, aide }
    }
}

/// Le registre des plugins natifs : une seule liste, lue par l'interface.
pub const PLUGINS: &[PluginDesc] = &[
    PluginDesc::new(
        CLE_BIBLIOGRAPHIE,
        "Bibliographie",
        "Les références et leur bibliographie.",
    ),
    PluginDesc::new(
        CLE_GLOSSAIRE,
        "Glossaire",
        "Les termes définis et leurs occurrences.",
    ),
    PluginDesc::new(CLE_GRAPHIQUES, "Graphiques", "Les figures et les courbes."),
    PluginDesc::new(
        CLE_CALLOUTS,
        "Callouts",
        "Les citations encadrées et leurs tons.",
    ),
    PluginDesc::new(
        CLE_REFERENCES_BIBLIQUES,
        "Références bibliques",
        "L'insertion, le survol et la trace des citations.",
    ),
    PluginDesc::new(
        CLE_PERSONNES,
        "Personnes citées",
        "Le registre des personnes et leur index nominum.",
    ),
];

#[cfg(test)]
mod epreuves {
    use super::*;

    #[test]
    fn une_cle_inconnue_est_active() {
        assert!(AddOns::default().actif("plugin-futur"));
    }

    #[test]
    fn la_bascule_ne_connait_que_ses_cles() {
        let mut add_ons = AddOns::default();
        add_ons.basculer(CLE_GLOSSAIRE, false);
        assert!(!add_ons.actif(CLE_GLOSSAIRE));
        add_ons.basculer(CLE_REFERENCES_BIBLIQUES, false);
        assert!(!add_ons.actif(CLE_REFERENCES_BIBLIQUES));
        add_ons.basculer("plugin-futur", false);
        assert!(add_ons.actif(CLE_BIBLIOGRAPHIE));
    }

    #[test]
    fn chaque_add_on_se_debranche() {
        let add_ons = AddOns {
            glossaire: false,
            references_bibliques: false,
            ..Default::default()
        };
        assert!(!add_ons.actif(CLE_GLOSSAIRE));
        assert!(!add_ons.actif(CLE_REFERENCES_BIBLIQUES));
        assert!(add_ons.actif(CLE_BIBLIOGRAPHIE));
    }

    #[test]
    fn les_cles_du_registre_sont_uniques() {
        let mut cles: Vec<_> = PLUGINS.iter().map(|desc| desc.cle).collect();
        cles.sort_unstable();
        cles.dedup();
        assert_eq!(cles.len(), PLUGINS.len());
    }

    #[test]
    fn chaque_cle_du_registre_se_bascule() {
        for desc in PLUGINS {
            let mut add_ons = AddOns::default();
            add_ons.basculer(desc.cle, false);

            assert!(
                !add_ons.actif(desc.cle),
                "la clé {} doit être reconnue par basculer",
                desc.cle
            );

            for autre in PLUGINS {
                if autre.cle != desc.cle {
                    assert!(
                        add_ons.actif(autre.cle),
                        "basculer {} ne touche pas {}",
                        desc.cle,
                        autre.cle
                    );
                }
            }
        }
    }
}
