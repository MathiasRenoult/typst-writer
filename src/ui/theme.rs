//! Thème et palette.
//!
//! L'application suit le thème du système : `iced` expose le mode courant
//! (`iced::system::theme`) et ses changements (`iced::system::theme_changes`),
//! détectés par `mundy` sous Linux. Tout le reste de l'interface dérive ses
//! couleurs de la palette iced plutôt que de coder des valeurs en dur, afin
//! que basculer entre clair et sombre ne demande aucun changement de code.

use iced::{Color, Theme};

/// Comment le thème est choisi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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

/// Le mode du système, tel que le rapportent `iced::system::theme` et
/// `theme_changes`. `None` signifie « pas de préférence déclarée ».
pub type SystemMode = Option<iced::theme::Mode>;

/// Résout le thème à appliquer.
pub fn resolve(mode: ThemeMode, system: SystemMode) -> Theme {
    match mode {
        ThemeMode::System => match system {
            Some(iced::theme::Mode::Light) => Theme::Light,
            _ => Theme::Dark,
        },
        ThemeMode::Light => Theme::Light,
        ThemeMode::Dark => Theme::Dark,
    }
}

/// Couleurs de l'interface, déduites du thème courant.
///
/// Centraliser ces valeurs évite que chaque panneau invente sa propre nuance
/// de gris — et garantit qu'un changement de thème se propage partout.
#[derive(Debug, Clone, Copy)]
pub struct Palette {
    /// Fond d'un panneau (barre latérale, bandeaux).
    pub surface: Color,

    /// Fond survolé, légèrement détaché du fond de panneau.
    pub surface_hover: Color,

    /// Fond d'un élément actif ou sélectionné.
    pub surface_active: Color,

    /// Texte principal.
    pub text: Color,

    /// Texte secondaire : légendes, tailles de fichier, statuts discrets.
    pub text_muted: Color,

    /// Trait de séparation, bord de panneau, poignée de redimensionnement.
    pub border: Color,

    /// Couleur d'accent (liens, élément actif, surbrillance).
    pub accent: Color,

    /// Erreur.
    pub danger: Color,

    /// Avertissement.
    pub warning: Color,

    /// Succès.
    pub success: Color,
}

impl Palette {
    /// Déduit la palette du thème iced courant.
    pub fn from_theme(theme: &Theme) -> Self {
        let palette = theme.palette();
        let extended = theme.extended_palette();

        let surface = extended.background.weak.color;
        let text = palette.text;
        let text_muted = extended.background.weak.text;

        Self {
            surface,
            surface_hover: extended.background.strong.color,
            surface_active: extended.primary.weak.color,
            text,
            text_muted,
            border: extended.background.strong.color,
            accent: palette.primary,
            danger: palette.danger,
            warning: palette.warning,
            success: palette.success,
        }
    }
}

/// Icône Lucide, à la taille et dans la couleur voulues.
///
/// Les fonctions de `lucide_icons::iced` produisent un `Text` ; on le
/// dimensionne et le colore ici pour que les icônes de toute l'interface
/// partagent la même gravure.
pub fn icon<'a>(
    glyph: iced::widget::Text<'a, Theme>,
    size: f32,
    color: Color,
) -> iced::widget::Text<'a, Theme> {
    glyph.size(size).color(color)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_mode_cycles_through_all_three() {
        let mode = ThemeMode::System;

        let visited = [
            mode,
            mode.next(),
            mode.next().next(),
            mode.next().next().next(),
        ];

        assert_eq!(
            visited,
            [
                ThemeMode::System,
                ThemeMode::Light,
                ThemeMode::Dark,
                ThemeMode::System,
            ],
            "le cycle doit revenir à son point de départ"
        );
    }

    /// Une préférence système absente retombe sur le sombre : c'est le thème
    /// historique de l'application, et le plus lisible pour de la rédaction.
    #[test]
    fn system_theme_falls_back_to_dark() {
        assert_eq!(resolve(ThemeMode::System, None), Theme::Dark);
    }

    #[test]
    fn explicit_choice_overrides_the_system() {
        assert_eq!(
            resolve(ThemeMode::Light, Some(iced::theme::Mode::Dark)),
            Theme::Light
        );
        assert_eq!(
            resolve(ThemeMode::Dark, Some(iced::theme::Mode::Light)),
            Theme::Dark
        );
    }

    /// Les deux thèmes doivent produire une palette exploitable : c'est ce qui
    /// garantit qu'aucun panneau ne dépende de couleurs codées en dur.
    #[test]
    fn palette_is_derivable_from_both_themes() {
        for theme in [Theme::Light, Theme::Dark] {
            let palette = Palette::from_theme(&theme);

            assert_eq!(palette.text, theme.palette().text);
            assert_eq!(palette.accent, theme.palette().primary);
            assert_eq!(palette.danger, theme.palette().danger);
        }
    }

    /// Les deux thèmes doivent se distinguer : si clair et sombre donnaient la
    /// même couleur de fond de panneau, suivre le système ne servirait à rien.
    #[test]
    fn light_and_dark_palettes_differ() {
        let light = Palette::from_theme(&Theme::Light);
        let dark = Palette::from_theme(&Theme::Dark);

        assert_ne!(light.surface, dark.surface);
        assert_ne!(light.text, dark.text);
    }
}
