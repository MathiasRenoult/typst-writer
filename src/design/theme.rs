//! Thème et palette.
//!
//! L'application suit le thème du système : `iced` expose le mode courant
//! (`iced::system::theme`) et ses changements (`iced::system::theme_changes`),
//! détectés par `mundy` sous Linux. Tout le reste de l'interface dérive ses
//! couleurs de la palette iced plutôt que de coder des valeurs en dur, afin
//! que basculer entre clair et sombre ne demande aucun changement de code.

pub use crate::base::theme::ThemeMode;
use iced::{Color, Theme};

/// Le mode du système, tel que le rapportent `iced::system::theme` et
/// `theme_changes`. `None` signifie « pas de préférence déclarée ».
pub type SystemMode = Option<iced::theme::Mode>;

impl ThemeMode {
    /// L'apparence à appliquer, une fois le système consulté.
    ///
    /// C'est le seul point où le mode du système entre en jeu : partout
    /// ailleurs, l'application ne raisonne que par apparence — claire ou
    /// sombre — et choisit le thème retenu pour celle-ci.
    pub fn appearance(self, system: SystemMode) -> crate::theming::Appearance {
        let light = match self {
            ThemeMode::System => match system {
                Some(iced::theme::Mode::Light) => true,
                // Aucune préférence déclarée — ou pas de préférence du tout :
                // le sombre, thème historique de l'application et le plus
                // confortable pour écrire longtemps.
                _ => false,
            },
            ThemeMode::Light => true,
            ThemeMode::Dark => false,
        };

        if light {
            crate::theming::Appearance::Light
        } else {
            crate::theming::Appearance::Dark
        }
    }
}

/// Couleurs de l'interface, déduites du thème courant.
///
/// Centraliser ces valeurs évite que chaque panneau invente sa propre nuance
/// de gris — et garantit qu'un changement de thème se propage partout.
#[derive(Debug, Clone, Copy)]
pub struct Palette {
    /// Fond de l'application.
    ///
    /// Distinct du fond de panneau : c'est ce qui rend visibles les marges de
    /// la zone de rédaction, qui flotte sur le fond de l'application.
    pub background: Color,

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

    /// Famille de la police d'interface.
    ///
    /// `iced` ne sait changer sa police par défaut qu'**au démarrage** : la
    /// famille choisie dans un thème ne peut donc s'appliquer qu'en la posant
    /// sur chaque texte. La porter ici, avec les couleurs, évite de la faire
    /// circuler en parallèle dans chaque vue.
    pub interface: iced::Font,
}

impl Palette {
    /// Déduit la palette du thème iced courant.
    pub fn from_theme(theme: &Theme) -> Self {
        let palette = theme.palette();
        let extended = theme.extended_palette();

        let surface = extended.background.weak.color;
        let text = palette.text;
        // Le texte secondaire doit rester **lisible** : la valeur dérivée
        // d'iced est parfois trop proche du fond. On la rapproche du texte
        // principal d'un cran, sans jamais l'égaler.
        let text_muted = melange_fin(extended.background.weak.text, palette.text, 0.25);

        Self {
            background: extended.background.base.color,
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
            // Le thème natif ne touche pas aux polices : c'est celui du moteur.
            interface: iced::Font::DEFAULT,
        }
    }

    /// Construit la palette à partir des couleurs écrites dans un thème.
    ///
    /// Contrairement à [`Palette::from_theme`], rien n'est déduit du thème iced :
    /// on lit directement ce que l'auteur du thème a écrit, et l'on ne déduit
    /// que ce qu'il a omis. C'est ce qui donne à un fichier de thème un effet
    /// plus fin que les six couleurs cardinales qu'iced accepte.
    pub fn from_colors(colors: &crate::theming::Colors) -> Self {
        Self {
            background: colors.background.0,
            surface: colors.surface.0,
            surface_hover: colors.surface_hover(),
            surface_active: colors.surface_active(),
            text: colors.text.0,
            text_muted: colors.text_muted(),
            border: colors.border(),
            accent: colors.accent.0,
            danger: colors.danger(),
            warning: colors.warning(),
            success: colors.success(),
            // Rempli par l'appelant, qui seul connaît les polices du thème.
            interface: iced::Font::DEFAULT,
        }
    }

    /// Un texte de l'interface, dans la police du thème.
    ///
    /// Raccourci pour ne pas répéter un `.font()` sur chaque texte : `iced` ne
    /// permet pas de changer sa police par défaut en cours d'exécution, la
    /// famille choisie doit donc être posée sur chacun.
    pub fn label<'a>(
        &self,
        text: impl iced::widget::text::IntoFragment<'a>,
    ) -> iced::widget::Text<'a, iced::Theme> {
        iced::widget::text(text).font(self.interface)
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

/// Les jetons de dessin : le même rayon, les mêmes espacements, partout.
///
/// Une interface moderne est d'abord une interface **répétée** : un panneau,
/// un bouton, une rangée partagent un rayon, un pas, une échelle de corps.
/// Ce que les yeux appellent « épuré » est souvent cette seule régularité.
pub mod jetons {
    /// Le rayon des surfaces : panneaux, boutons, champs, infobulles.
    /// Un coin nettement arrondi — la douceur iOS — sans aller au pilule.
    pub const RAYON: f32 = 12.0;

    /// Le pas de l'entre-deux : entre des rangées, des blocs, des champs.
    pub const PAS: f32 = 8.0;

    /// Le pas large : entre les grandes sections d'une fenêtre.
    pub const GRAND_PAS: f32 = 12.0;

    /// L'échelle des corps de texte, du plus discret au titre.
    pub const CHAPEAU: f32 = 10.0;
    /// Texte secondaire, mentions et aides.
    pub const DETAIL: f32 = 11.0;
    /// Corps courant des libellés.
    pub const CORPS: f32 = 12.0;
    /// Libellé qui doit ressortir dans une liste.
    pub const SAILLANT: f32 = 13.0;
    /// Titre de section ou de fenêtre.
    pub const TITRE: f32 = 15.0;
}

/// Le fond d'un panneau flottant : surface, lisière, rayon commun.
pub fn panneau(palette: Palette) -> impl Fn(&iced::Theme) -> iced::widget::container::Style {
    move |_| iced::widget::container::Style {
        background: Some(palette.surface.into()),
        border: iced::Border {
            color: palette.border,
            width: 1.0,
            radius: jetons::RAYON.into(),
        },
        ..Default::default()
    }
}

/// Les polices résolues d'un thème.
#[derive(Debug, Clone, Copy)]
pub struct Typography {
    /// Famille de l'interface.
    pub interface: iced::Font,

    /// Famille de l'éditeur.
    pub editor: iced::Font,

    /// Corps du texte de l'éditeur, en pixels.
    pub editor_size: f32,

    /// Interligne de l'éditeur.
    pub line_height: f32,
}

impl Default for Typography {
    fn default() -> Self {
        Self {
            interface: iced::Font::DEFAULT,
            editor: iced::Font::MONOSPACE,
            editor_size: crate::theming::DEFAULT_EDITOR_SIZE,
            line_height: crate::theming::DEFAULT_LINE_HEIGHT,
        }
    }
}

/// L'aspect de l'éditeur, quand un thème le décrit.
#[derive(Debug, Clone)]
pub struct EditorLook {
    /// Les couleurs de l'éditeur.
    pub style: iced_code_editor::Style,

    /// La palette de coloration syntaxique.
    pub syntax: iced_code_editor::SyntaxPalette,
}

/// Un thème prêt à l'emploi : tout ce que l'affichage consomme.
///
/// C'est le seul endroit où un thème de fichier devient des objets d'iced et du
/// widget d'édition. Le reste de l'application ne connaît que ce type, si bien
/// qu'aucun panneau n'a à savoir d'où vient le thème.
pub struct Resolved {
    /// Le thème iced : habille les widgets de la bibliothèque (boutons,
    /// listes, champs de saisie).
    pub iced: iced::Theme,

    /// La palette de nos propres panneaux.
    pub palette: Palette,

    /// Les polices.
    pub typography: Typography,

    /// L'aspect de l'éditeur. `None` : les défauts du widget, ce qui est le cas
    /// des thèmes natifs.
    pub editor: Option<EditorLook>,
}

impl Resolved {
    /// Résout un thème du magasin.
    pub fn of(theme: &crate::theming::Theme) -> Self {
        let Some(file) = &theme.file else {
            // Un thème natif délègue tout à iced et au widget : le choisir ne
            // change donc rien à l'apparence qui existait avant les thèmes.
            // C'est ce qui garantit un point de comparaison stable.
            let iced = match theme.appearance {
                crate::theming::Appearance::Light => iced::Theme::Light,
                crate::theming::Appearance::Dark => iced::Theme::Dark,
            };

            return Self {
                palette: Palette::from_theme(&iced),
                iced,
                typography: Typography::default(),
                editor: None,
            };
        };

        let colors = &file.colors;

        // iced n'a besoin que des six couleurs cardinales : il en dérive
        // lui-même les nuances de survol, de bordure et de fond de ses propres
        // widgets. Nos panneaux, eux, ont leur propre jeu, plus fin.
        let iced = iced::Theme::custom(
            theme.name.clone(),
            iced::theme::Palette {
                background: colors.background.0,
                text: colors.text.0,
                primary: colors.accent.0,
                success: colors.success(),
                warning: colors.warning(),
                danger: colors.danger(),
            },
        );

        let typography = Typography {
            interface: file.fonts.interface_font(),
            editor: file.fonts.editor_font(),
            editor_size: file.fonts.editor_size(),
            line_height: file.fonts.line_height(),
        };

        // La police d'interface accompagne les couleurs : tous les textes la
        // lisent au même endroit.
        let palette = Palette {
            interface: typography.interface,
            ..Palette::from_colors(colors)
        };

        Self {
            palette,
            iced,
            typography,
            editor: Some(EditorLook {
                style: editor_style(file),
                syntax: syntax_palette(file),
            }),
        }
    }
}

/// Les couleurs de l'éditeur, déduites des couleurs d'interface.
///
/// Chaque champ de la section `[editor]` est facultatif ; ce qui n'est pas écrit
/// se déduit, pour qu'un thème de cinq lignes reste cohérent au lieu de laisser
/// des zones transparentes ou noires.
fn editor_style(file: &crate::theming::ThemeFile) -> iced_code_editor::Style {
    let colors = &file.colors;
    let editor = &file.editor;
    let pick = |declared: Option<crate::theming::HexColor>, fallback: Color| {
        declared.map(|color| color.0).unwrap_or(fallback)
    };

    // **Le fond de la rédaction est celui de l'application.** `surface` est le
    // ton des *panneaux* (barre latérale, bandeaux) : le donner à la zone de
    // rédaction inversait les deux rôles, et l'éditeur paraissait appartenir à
    // une autre fenêtre que celle qui l'entoure.
    let background = pick(editor.background, colors.background.0);

    iced_code_editor::Style {
        background,
        text_color: pick(editor.text, colors.text.0),
        gutter_background: pick(editor.gutter_background, background),
        gutter_border: pick(editor.line_number, colors.border()),
        line_number_color: pick(editor.line_number, colors.text_muted()),
        scrollbar_background: pick(editor.scrollbar, background),
        scroller_color: alpha(pick(editor.scroller, colors.surface_hover()), 0.4),
        // La ligne courante et la sélection doivent rester **discrètes** : un
        // aplat opaque au milieu du texte gênerait la lecture, ce qui est
        // exactement ce qu'on cherche à éviter dans un éditeur de rédaction.
        current_line_highlight: alpha(pick(editor.current_line, colors.surface_hover()), 0.45),
        whitespace_color: alpha(colors.text_muted(), 0.25),
        indent_guide_color: alpha(colors.border(), 0.6),
        // Les occurrences de recherche sont des repères, pas du texte : elles
        // doivent se voir sans masquer ce qu'elles désignent.
        search_match_color: alpha(pick(editor.search_match, colors.warning()), 0.35),
        search_match_current_color: alpha(pick(editor.search_match_current, colors.accent.0), 0.55),
        selection_color: alpha(pick(editor.selection, colors.surface_active()), 0.55),
        bracket_match_color: alpha(pick(editor.bracket_match, colors.accent.0), 0.4),
        ime_preedit_background_color: alpha(colors.surface_hover(), 0.5),
        // Les repères posés par l'éditeur sur le texte se teintent à
        // l'accent : un signal, pas du texte courant.
        mark_color: colors.accent.0,
    }
}

/// Une couleur rendue translucide.
fn alpha(color: Color, alpha: f32) -> Color {
    Color { a: alpha, ..color }
}

/// Mélange finement deux couleurs — `poids` de 0 donne `a`, 1 donne `b`.
fn melange_fin(a: Color, b: Color, poids: f32) -> Color {
    Color {
        r: a.r + (b.r - a.r) * poids,
        g: a.g + (b.g - a.g) * poids,
        b: a.b + (b.b - a.b) * poids,
        a: a.a,
    }
}

/// Les règles de coloration, **dans l'ordre d'application**.
///
/// L'ordre est le point délicat : syntect retient la **dernière** règle qui
/// correspond à la portée, et il compare par préfixe — `constant` correspond
/// aussi bien à `constant.numeric` qu'à `constant.language`. Une règle générale
/// doit donc venir **avant** une règle plus fine, sans quoi la fine ne
/// s'appliquerait jamais : placée avant `constant`, la couleur des nombres
/// serait écrasée par celle des constantes, et un thème qui distingue les deux
/// n'aurait aucun effet visible.
///
/// Chaque rôle a une déduction à partir des couleurs d'interface : un thème qui
/// ne dit rien de la coloration obtient malgré tout une palette cohérente, et
/// non du texte brut.
fn syntax_palette(file: &crate::theming::ThemeFile) -> iced_code_editor::SyntaxPalette {
    let syntax = &file.syntax;
    let colors = &file.colors;

    let pick = |declared: Option<crate::theming::HexColor>, fallback: Color| {
        declared.map(|color| color.0).unwrap_or(fallback)
    };

    let muted = colors.text_muted();
    let accent = colors.accent.0;

    // Le fond est celui de l'éditeur : c'est sur lui que syntect départage ses
    // règles internes.
    let background = editor_style(file).background;

    // Général d'abord, spécifique ensuite.
    let rules: Vec<(&str, Color)> = vec![
        ("comment", pick(syntax.comment, muted)),
        ("string", pick(syntax.string, colors.success())),
        // `keyword` couvre `keyword.control`, `keyword.operator`, etc.
        ("keyword", pick(syntax.keyword, accent)),
        ("storage", pick(syntax.keyword, accent)),
        // `constant` couvre `constant.numeric`, `constant.language`…
        ("constant", pick(syntax.constant, colors.warning())),
        ("entity.name.type", pick(syntax.type_name, colors.warning())),
        ("support.type", pick(syntax.type_name, colors.warning())),
        ("storage.type", pick(syntax.type_name, colors.warning())),
        ("variable.function", pick(syntax.function, accent)),
        ("support.function", pick(syntax.function, accent)),
        ("entity.name.function", pick(syntax.function, accent)),
        ("punctuation", pick(syntax.punctuation, muted)),
        // Spécifiques : après leurs règles générales, donc.
        ("constant.numeric", pick(syntax.number, colors.warning())),
        ("keyword.operator", pick(syntax.operator, muted)),
        // Le balisage Markdown/Typst : titres, emphases, listes et citations.
        ("markup.heading", pick(syntax.heading, accent)),
        ("markup.bold", pick(syntax.strong, colors.warning())),
        ("markup.italic", pick(syntax.emphasis, accent)),
        ("markup.list", pick(syntax.keyword, accent)),
        ("markup.quote", pick(syntax.comment, muted)),
        ("markup.raw", pick(syntax.raw, colors.success())),
        ("markup.underline.link", pick(syntax.link, accent)),
    ];

    iced_code_editor::SyntaxPalette {
        text: pick(syntax.text, colors.text.0),
        background,
        rules: rules
            .into_iter()
            .map(|(selector, color)| iced_code_editor::SyntaxRule {
                selector: selector.to_string(),
                color,
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Un thème minimal, avec les quatre couleurs obligatoires.
    const MINIMAL: &str = r##"
[meta]
name = "Essai"
appearance = "dark"

[colors]
background = "#1e1e2e"
surface = "#181825"
text = "#cdd6f4"
accent = "#89b4fa"
"##;

    /// **La zone de rédaction prend le ton de l'application**, et le tour qui
    /// l'entoure celui des panneaux.
    ///
    /// Les deux rôles étaient inversés : `surface` — le ton des panneaux, dans
    /// un thème comme dans la documentation — était donné à la rédaction, si
    /// bien que l'éditeur paraissait appartenir à une autre fenêtre que la
    /// sienne. Et comme c'est leur **différence** qui rend les marges de la page
    /// visibles, les deux tons doivent rester distincts : l'invariant est donc
    /// vérifié ici.
    #[test]
    fn the_writing_area_takes_the_application_tone() {
        let file = crate::theming::ThemeFile::parse(MINIMAL, None).expect("thème valide");

        assert_eq!(
            editor_style(&file).background,
            file.colors.background.0,
            "la rédaction doit prendre le fond de l'application, pas celui des panneaux"
        );

        assert_ne!(
            file.colors.background.0, file.colors.surface.0,
            "les deux tons doivent différer, sinon les marges disparaissent"
        );
    }

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
    fn a_silent_system_falls_back_to_dark() {
        assert_eq!(
            ThemeMode::System.appearance(None),
            crate::theming::Appearance::Dark
        );
    }

    /// Un choix explicite l'emporte sur ce que dit le système.
    #[test]
    fn an_explicit_mode_overrides_the_system() {
        assert_eq!(
            ThemeMode::Light.appearance(Some(iced::theme::Mode::Dark)),
            crate::theming::Appearance::Light
        );
        assert_eq!(
            ThemeMode::Dark.appearance(Some(iced::theme::Mode::Light)),
            crate::theming::Appearance::Dark
        );
    }

    /// En mode système, c'est bien le système qui décide.
    #[test]
    fn the_system_mode_follows_the_system() {
        assert_eq!(
            ThemeMode::System.appearance(Some(iced::theme::Mode::Light)),
            crate::theming::Appearance::Light
        );
        assert_eq!(
            ThemeMode::System.appearance(Some(iced::theme::Mode::Dark)),
            crate::theming::Appearance::Dark
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

    /// Le fond de l'application et le fond de panneau doivent différer.
    ///
    /// C'est cette différence qui fait apparaître la zone de rédaction comme une
    /// page — et donc ses marges. Sur deux teintes identiques, la marge existe
    /// en géométrie mais reste invisible ; c'est précisément ce qui la rendait
    /// imperceptible.
    #[test]
    fn background_and_surface_differ() {
        for theme in [Theme::Light, Theme::Dark] {
            let palette = Palette::from_theme(&theme);

            assert_ne!(
                palette.background, palette.surface,
                "la page ne se détacherait pas du fond en thème {theme:?}"
            );
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
