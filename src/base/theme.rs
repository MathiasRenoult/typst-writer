//! Types de valeur partagés : le mode de thème, persisté en session.

use serde::{Deserialize, Serialize};

/// Le mode est enregistré dans la session : il doit donc être lisible et
/// stable. Les variantes s'écrivent en minuscules dans le fichier, pour qu'il
/// reste modifiable à la main sans surprendre.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    /// Suit le réglage du système (défaut).
    #[default]
    System,

    /// Clair, quoi que décide le système.
    Light,

    /// Sombre, quoi que décide le système.
    Dark,
}

impl ThemeMode {
    /// Le mode suivant, pour un bouton qui cycle.
    pub fn next(self) -> Self {
        match self {
            ThemeMode::System => ThemeMode::Light,
            ThemeMode::Light => ThemeMode::Dark,
            ThemeMode::Dark => ThemeMode::System,
        }
    }

    /// Libellé lisible, pour l'infobulle du bouton.
    pub fn label(self) -> &'static str {
        match self {
            ThemeMode::System => "Thème : système",
            ThemeMode::Light => "Thème : clair",
            ThemeMode::Dark => "Thème : sombre",
        }
    }
}
