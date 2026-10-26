//! Thèmes locaux.
//!
//! Un thème est un **fichier texte** que l'on écrit à la main, pour l'instant
//! hors de l'application :
//!
//! ```text
//! ~/.typst-writer/themes/<nom>.toml
//! ```
//!
//! Un thème ne décrit que des **couleurs** et des **polices**. Rien de la
//! disposition — largeur des panneaux, marges de la page, comportement de
//! l'éditeur — n'en dépend : un thème mal écrit doit pouvoir défigurer les
//! couleurs sans jamais casser l'application.
//!
//! Deux thèmes sont **natifs** : ils ne viennent d'aucun fichier et ne peuvent
//! être ni supprimés ni modifiés. Ce sont les deux apparences que l'application
//! a toujours connues, et ils garantissent qu'il reste toujours un thème
//! utilisable si les fichiers locaux sont tous cassés.
//!
//! Toutes les clés d'un thème sont optionnelles, sauf le nom : une couleur
//! omise est **déduite** des autres plutôt que laissée vide. Un fichier de cinq
//! lignes est donc un thème valide, et l'on n'écrit que ce que l'on veut
//! changer.

use std::path::{Path, PathBuf};

use iced::Color;
use serde::{Deserialize, Serialize};

/// Corps du texte de l'éditeur, par défaut.
///
/// Nommé assez précisément pour ne pas se confondre avec les tailles de texte
/// de l'interface, qui n'ont rien à voir.
pub const DEFAULT_EDITOR_SIZE: f32 = 17.0;

/// Interligne de l'éditeur, par défaut.
pub const DEFAULT_LINE_HEIGHT: f32 = 1.6;

/// Éclaircit une couleur — ou l'assombrit si elle est déjà claire.
///
/// « Plus clair » n'a pas de sens absolu : sur un thème clair, un fond de survol
/// plus clair que le fond serait invisible. On s'éloigne donc du fond dans la
/// direction qui reste perceptible, ce qui donne un survol lisible aussi bien en
/// thème clair qu'en thème sombre.
pub fn lighten(color: Color, amount: f32) -> Color {
    let sum = color.r + color.g + color.b;

    let target = if sum > 1.5 { 0.0 } else { 1.0 };

    Color {
        r: color.r + (target - color.r) * amount,
        g: color.g + (target - color.g) * amount,
        b: color.b + (target - color.b) * amount,
        a: color.a,
    }
}

/// Mélange deux couleurs : `amount` va de `first` (0) à `second` (1).
pub fn blend(first: Color, second: Color, amount: f32) -> Color {
    Color {
        r: first.r + (second.r - first.r) * amount,
        g: first.g + (second.g - first.g) * amount,
        b: first.b + (second.b - first.b) * amount,
        a: first.a + (second.a - first.a) * amount,
    }
}

/// Donne à un nom de famille une durée de vie statique.
///
/// `iced` exige des noms de famille à durée de vie statique — `Family::Name`
/// prend un `&'static str` — alors qu'un thème tire le sien d'un fichier lu à
/// l'exécution. Chaque nom **distinct** n'est donc copié qu'une fois dans toute
/// la vie du processus : basculer de thème cent fois ne laisse pas cent copies
/// derrière soi.
pub fn intern_family(name: &str) -> &'static str {
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};

    static INTERNED: OnceLock<Mutex<HashMap<String, &'static str>>> = OnceLock::new();

    let table = INTERNED.get_or_init(|| Mutex::new(HashMap::new()));

    // Un verrou empoisonné par un autre fil de discussion ne doit pas faire
    // échouer le rendu : la table n'est qu'un cache, elle reste utilisable.
    let mut table = table
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    if let Some(interned) = table.get(name) {
        return interned;
    }

    let interned: &'static str = Box::leak(name.to_string().into_boxed_str());

    table.insert(name.to_string(), interned);

    interned
}

/// Clarté d'un thème : à quel mode système il répond.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Appearance {
    /// Thème clair.
    Light,

    /// Thème sombre. C'est le défaut historique de l'application.
    #[default]
    Dark,
}

/// Une couleur, écrite en hexadécimal dans le fichier.
///
/// `#rrggbb` ou `#rrggbbaa`, le `#` étant facultatif. L'erreur produite nomme
/// la valeur fautive : dans un fichier écrit à la main, « couleur invalide :
/// « bleu » » vaut mieux qu'un message de conversion générique.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HexColor(pub Color);

impl HexColor {
    /// Lit une couleur hexadécimale.
    pub fn parse(text: &str) -> Result<Self, String> {
        let digits = text.trim().trim_start_matches('#');

        if !matches!(digits.len(), 6 | 8) {
            return Err(format!(
                "couleur invalide : « {text} » (attendu #rrggbb ou #rrggbbaa)"
            ));
        }

        let byte = |index: usize| {
            u8::from_str_radix(&digits[index..index + 2], 16).map_err(|_| {
                format!("couleur invalide : « {text} » (chiffres hexadécimaux attendus)")
            })
        };

        let (r, g, b) = (byte(0)?, byte(2)?, byte(4)?);

        let a = if digits.len() == 8 { byte(6)? } else { 255 };

        Ok(HexColor(Color::from_rgba8(r, g, b, a as f32 / 255.0)))
    }
}

impl<'de> Deserialize<'de> for HexColor {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;

        HexColor::parse(&text).map_err(serde::de::Error::custom)
    }
}

/// Les couleurs de l'interface.
///
/// C'est la base du thème : les couleurs de l'éditeur et de la coloration en
/// sont déduites si elles ne sont pas écrites.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Colors {
    /// Fond de l'application.
    pub background: HexColor,

    /// Fond des panneaux (barre latérale, bandeaux).
    ///
    /// Doit différer du fond de l'application : c'est ce qui fait apparaître la
    /// zone de rédaction comme une page, et donc ses marges.
    pub surface: HexColor,

    /// Texte principal.
    pub text: HexColor,

    /// Couleur d'accent : élément actif, lien, surbrillance.
    pub accent: HexColor,

    /// Fond survolé. Déduit de `surface` si omis.
    #[serde(default)]
    pub surface_hover: Option<HexColor>,

    /// Fond d'un élément actif ou sélectionné. Déduit de `surface` si omis.
    #[serde(default)]
    pub surface_active: Option<HexColor>,

    /// Texte secondaire : légendes, comptes, statuts discrets.
    #[serde(default)]
    pub text_muted: Option<HexColor>,

    /// Trait de séparation, bord de panneau, poignée de redimensionnement.
    #[serde(default)]
    pub border: Option<HexColor>,

    /// Erreur.
    #[serde(default)]
    pub danger: Option<HexColor>,

    /// Avertissement.
    #[serde(default)]
    pub warning: Option<HexColor>,

    /// Succès.
    #[serde(default)]
    pub success: Option<HexColor>,
}

impl Colors {
    /// Fond survolé, effectif.
    pub fn surface_hover(&self) -> Color {
        self.surface_hover
            .map(|color| color.0)
            .unwrap_or_else(|| lighten(self.surface.0, 0.08))
    }

    /// Fond actif, effectif.
    pub fn surface_active(&self) -> Color {
        self.surface_active
            .map(|color| color.0)
            .unwrap_or_else(|| lighten(self.surface.0, 0.16))
    }

    /// Texte secondaire, effectif.
    pub fn text_muted(&self) -> Color {
        self.text_muted
            .map(|color| color.0)
            .unwrap_or_else(|| blend(self.text.0, self.background.0, 0.45))
    }

    /// Trait de séparation, effectif.
    pub fn border(&self) -> Color {
        self.border
            .map(|color| color.0)
            .unwrap_or_else(|| self.surface_hover())
    }

    /// Erreur, effective.
    pub fn danger(&self) -> Color {
        self.danger
            .map(|color| color.0)
            .unwrap_or(Color::from_rgb8(0xf3, 0x8b, 0xa8))
    }

    /// Avertissement, effectif.
    pub fn warning(&self) -> Color {
        self.warning
            .map(|color| color.0)
            .unwrap_or(Color::from_rgb8(0xf9, 0xe2, 0xaf))
    }

    /// Succès, effectif.
    pub fn success(&self) -> Color {
        self.success
            .map(|color| color.0)
            .unwrap_or(Color::from_rgb8(0xa6, 0xe3, 0xa1))
    }
}

/// Les polices.
///
/// Seules des **familles** et des **tailles** : aucune mesure de disposition.
/// Une famille absente du système retombe sur la police par défaut du moteur,
/// ce qui dégrade l'aspect sans jamais empêcher d'écrire.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Fonts {
    /// Famille de l'interface. Vide : la police par défaut.
    #[serde(default)]
    pub interface: Option<String>,

    /// Famille de l'éditeur. Vide : la police à chasse fixe par défaut.
    #[serde(default)]
    pub editor: Option<String>,

    /// Corps du texte dans l'éditeur, en pixels.
    #[serde(default)]
    pub editor_size: Option<f32>,

    /// Interligne, en multiple de la taille du texte.
    #[serde(default)]
    pub line_height: Option<f32>,
}

impl Fonts {
    /// Famille de l'interface.
    pub fn interface_font(&self) -> iced::Font {
        match family(&self.interface) {
            Some(name) => iced::Font {
                family: iced::font::Family::Name(intern_family(name)),
                ..iced::Font::DEFAULT
            },
            None => iced::Font::DEFAULT,
        }
    }

    /// Famille de l'éditeur.
    pub fn editor_font(&self) -> iced::Font {
        match family(&self.editor) {
            Some(name) => iced::Font {
                family: iced::font::Family::Name(intern_family(name)),
                ..iced::Font::MONOSPACE
            },
            None => iced::Font::MONOSPACE,
        }
    }

    /// Corps du texte de l'éditeur.
    pub fn editor_size(&self) -> f32 {
        self.editor_size.unwrap_or(DEFAULT_EDITOR_SIZE)
    }

    /// Interligne de l'éditeur.
    pub fn line_height(&self) -> f32 {
        self.line_height.unwrap_or(DEFAULT_LINE_HEIGHT)
    }
}

/// Les couleurs de l'éditeur.
///
/// Toutes déduites des couleurs de l'interface si elles sont omises : on peut
/// écrire un thème complet sans jamais toucher à cette section.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct EditorColors {
    /// Fond de la page.
    #[serde(default)]
    pub background: Option<HexColor>,

    /// Texte.
    #[serde(default)]
    pub text: Option<HexColor>,

    /// Sélection.
    #[serde(default)]
    pub selection: Option<HexColor>,

    /// Ligne du curseur.
    #[serde(default)]
    pub current_line: Option<HexColor>,

    /// Fond de la gouttière.
    #[serde(default)]
    pub gutter_background: Option<HexColor>,

    /// Numéros de ligne.
    #[serde(default)]
    pub line_number: Option<HexColor>,

    /// Barre de défilement, et son curseur.
    #[serde(default)]
    pub scrollbar: Option<HexColor>,

    #[serde(default)]
    pub scroller: Option<HexColor>,

    /// Occurrences de recherche, et l'occurrence courante.
    #[serde(default)]
    pub search_match: Option<HexColor>,

    #[serde(default)]
    pub search_match_current: Option<HexColor>,

    /// Paire de délimiteurs mise en évidence.
    #[serde(default)]
    pub bracket_match: Option<HexColor>,
}

/// Une couleur de coloration syntaxique.
///
/// Les champs sont des **rôles**, pas des portées TextMate : c'est
/// l'application qui sait à quelles portées chaque rôle correspond, et dans
/// quel ordre les appliquer. Un thème n'a donc pas à connaître les détails du
/// moteur de coloration, et l'ordre d'application reste celui du code.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Syntax {
    /// Texte ordinaire.
    #[serde(default)]
    pub text: Option<HexColor>,

    /// Commentaires.
    #[serde(default)]
    pub comment: Option<HexColor>,

    /// Mots-clés du langage.
    #[serde(default)]
    pub keyword: Option<HexColor>,

    /// Chaînes de caractères.
    #[serde(default)]
    pub string: Option<HexColor>,

    /// Nombres.
    #[serde(default)]
    pub number: Option<HexColor>,

    /// Noms de fonctions appelées.
    #[serde(default)]
    pub function: Option<HexColor>,

    /// Noms de types.
    #[serde(default)]
    pub type_name: Option<HexColor>,

    /// Constantes et littéraux nommés (`true`, `none`).
    #[serde(default)]
    pub constant: Option<HexColor>,

    /// Opérateurs.
    #[serde(default)]
    pub operator: Option<HexColor>,

    /// Ponctuation et délimiteurs.
    #[serde(default)]
    pub punctuation: Option<HexColor>,

    /// Titres de section (`= Titre`).
    #[serde(default)]
    pub heading: Option<HexColor>,

    /// Gras et italique.
    #[serde(default)]
    pub strong: Option<HexColor>,

    #[serde(default)]
    pub emphasis: Option<HexColor>,

    /// Verbatim et blocs de code.
    #[serde(default)]
    pub raw: Option<HexColor>,

    /// Liens et renvois.
    #[serde(default)]
    pub link: Option<HexColor>,
}

/// Une famille de police écrite, s'il y en a une.
///
/// Une chaîne vide ou faite d'espaces n'est pas une famille : elle retombe sur
/// celle du moteur, plutôt que de demander une police qui n'existe pas.
fn family(declared: &Option<String>) -> Option<&str> {
    declared
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
}

/// Un thème tel qu'il est écrit dans un fichier.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThemeFile {
    /// Métadonnées : nom affiché et clarté.
    pub meta: Meta,

    /// Les couleurs de l'interface.
    pub colors: Colors,

    /// Les polices.
    #[serde(default)]
    pub fonts: Fonts,

    /// Les couleurs de l'éditeur.
    #[serde(default)]
    pub editor: EditorColors,

    /// Les couleurs de la coloration syntaxique.
    #[serde(default)]
    pub syntax: Syntax,

    /// D'où ce thème vient. `None` : il n'a pas encore été enregistré.
    #[serde(skip)]
    pub source: Option<PathBuf>,
}

/// Le nom et la clarté d'un thème.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Meta {
    /// Nom affiché dans la liste des thèmes.
    pub name: String,

    /// Clarté. Sombre par défaut.
    #[serde(default)]
    pub appearance: Appearance,

    /// Qui a écrit ce thème. Pure courtoisie.
    #[serde(default)]
    pub author: Option<String>,
}

impl ThemeFile {
    /// Lit un thème depuis un texte TOML.
    ///
    /// Le chemin sert à situer les erreurs et à retenir la provenance du
    /// thème ; il n'est pas ouvert.
    pub fn parse(text: &str, path: Option<&Path>) -> Result<Self, String> {
        let mut theme: ThemeFile = toml::from_str(text).map_err(|error| error.to_string())?;

        let name = theme.meta.name.trim().to_string();

        if name.is_empty() {
            return Err("le thème doit porter un nom".to_string());
        }

        theme.meta.name = name;
        theme.source = path.map(Path::to_path_buf);

        theme.check()?;

        Ok(theme)
    }

    /// Lit un thème depuis un fichier.
    pub fn load(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|error| format!("lecture de {} : {error}", path.display()))?;

        Self::parse(&text, Some(path))
    }

    /// Vérifie ce qu'aucun type ne peut exprimer.
    ///
    /// Une couleur est toujours valide une fois convertie : ce qui reste à
    /// contrôler, ce sont les **relations** entre elles. Un fond de page égal
    /// au fond de l'application rend les marges invisibles ; une police de
    /// corps nulle ou une interligne absurde casserait l'éditeur. Autant le
    /// dire au chargement, où l'on peut encore nommer le fichier fautif.
    fn check(&self) -> Result<(), String> {
        if self.colors.background.0 == self.colors.surface.0 {
            return Err(
                "« colors.background » et « colors.surface » sont identiques : la marge de la \
                 page ne se verrait plus"
                    .to_string(),
            );
        }

        if let Some(size) = self.fonts.editor_size
            && !(6.0..=72.0).contains(&size)
        {
            return Err(format!(
                "« fonts.editor_size » vaut {size} : attendu entre 6 et 72 pixels"
            ));
        }

        if let Some(height) = self.fonts.line_height
            && !(0.8..=4.0).contains(&height)
        {
            return Err(format!(
                "« fonts.line_height » vaut {height} : attendu entre 0,8 et 4"
            ));
        }

        Ok(())
    }
}

/// Les thèmes choisis par l'utilisateur, un par apparence.
///
/// C'est ce qui permet de **suivre le système** tout en gardant ses propres
/// thèmes : on se souvient du thème clair et du thème sombre retenus, et le
/// mode système ne fait que décider lequel des deux s'applique.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Chosen {
    /// Nom du thème clair retenu.
    pub light: String,

    /// Nom du thème sombre retenu.
    pub dark: String,
}

impl Default for Chosen {
    /// Les deux thèmes natifs : au démarrage, l'application ressemble à ce
    /// qu'elle a toujours été.
    fn default() -> Self {
        Self {
            light: "Clair".to_string(),
            dark: "Sombre".to_string(),
        }
    }
}

impl Chosen {
    /// Le nom retenu pour une apparence.
    pub fn get(&self, appearance: Appearance) -> &str {
        match appearance {
            Appearance::Light => &self.light,
            Appearance::Dark => &self.dark,
        }
    }

    /// Retient un thème pour son apparence.
    pub fn set(&mut self, appearance: Appearance, name: &str) {
        match appearance {
            Appearance::Light => self.light = name.to_string(),
            Appearance::Dark => self.dark = name.to_string(),
        }
    }
}

/// Le thème à appliquer.
///
/// La sélection est faite **par nom**, jamais par position : supprimer ou
/// charger un thème décale la liste, et retenir un index finirait par appliquer
/// un autre thème que celui qu'on avait choisi.
///
/// Un nom introuvable — thème supprimé, fichier devenu illisible — retombe sur
/// un autre thème de la même apparence, puis sur le premier de la liste. Une
/// erreur de fichier ne doit jamais laisser la fenêtre sans couleurs.
pub fn select(themes: &[Theme], appearance: Appearance, chosen: &Chosen) -> usize {
    let wanted = chosen.get(appearance);

    themes
        .iter()
        .position(|theme| theme.name == wanted && theme.appearance == appearance)
        .or_else(|| {
            themes
                .iter()
                .position(|theme| theme.appearance == appearance)
        })
        .unwrap_or(0)
}

/// Un thème disponible, natif ou chargé depuis un fichier.
#[derive(Debug, Clone)]
pub struct Theme {
    /// Le nom affiché.
    pub name: String,

    /// Sa clarté.
    pub appearance: Appearance,

    /// Le thème écrit, `None` pour un thème natif.
    ///
    /// Un thème natif n'a pas de fichier : il est décrit par le code, et c'est
    /// ce qui le rend indestructible.
    pub file: Option<ThemeFile>,
}

impl Theme {
    /// Le thème est-il natif ?
    ///
    /// Seuls les thèmes non natifs peuvent être supprimés : les natifs sont le
    /// filet de sécurité, celui qui garantit qu'il reste toujours un thème
    /// utilisable si les fichiers locaux sont cassés.
    pub fn is_native(&self) -> bool {
        self.file.is_none()
    }
}

/// Les deux thèmes natifs : l'apparence claire et l'apparence sombre que
/// l'application a toujours connues.
///
/// Ils ne portent aucune couleur : ils délèguent à `iced::Theme::Light` et
/// `iced::Theme::Dark`, et à la palette par défaut de l'éditeur. C'est ce qui
/// garantit que les choisir ne change rien à ce qui existait avant les thèmes.
pub const NATIVE: &[(Appearance, &str)] =
    &[(Appearance::Dark, "Sombre"), (Appearance::Light, "Clair")];

/// Les thèmes natifs, dans l'ordre d'affichage.
pub fn natives() -> Vec<Theme> {
    NATIVE
        .iter()
        .map(|(appearance, name)| Theme {
            name: name.to_string(),
            appearance: *appearance,
            file: None,
        })
        .collect()
}

/// Un nom est-il réservé par un thème natif ?
pub fn is_reserved_name(name: &str) -> bool {
    NATIVE
        .iter()
        .any(|(_, native)| native.eq_ignore_ascii_case(name.trim()))
}

/// Les thèmes **livrés** avec l'application : écrits une fois dans le
/// dossier des thèmes, puis traités comme n'importe quel thème local —
/// l'utilisateur peut les ouvrir, les lire, les modifier.
const EXPEDIES: &[(&str, &str)] =
    &[("everforest.toml", include_str!("everforest.toml"))];

/// Sème les thèmes livrés dans le dossier, s'ils n'y sont pas encore.
///
/// Un thème livré qui a été modifié **n'est pas** récrasé : le dossier
/// appartient à l'utilisateur, et une mise à jour de l'application ne doit
/// pas effacer ses réglages.
fn semer() {
    for (nom, contenu) in EXPEDIES {
        let chemin = themes_dir().join(nom);

        if !chemin.exists() {
            let _ = std::fs::write(&chemin, contenu);
        }
    }
}

/// Le dossier des thèmes locaux.
pub fn themes_dir() -> PathBuf {
    let dir = crate::project::work_dir().join("themes");

    let _ = std::fs::create_dir_all(&dir);

    dir
}

/// Liste les thèmes disponibles : les natifs d'abord, puis les fichiers locaux.
///
/// Un fichier illisible est **ignoré**, et non fatal : un thème cassé dans le
/// dossier ne doit pas empêcher l'application de démarrer. Il est revanche
/// rapporté, pour que l'utilisateur sache pourquoi son thème n'apparaît pas.
pub fn list() -> (Vec<Theme>, Vec<String>) {
    semer();

    let mut themes = natives();
    let mut problems = Vec::new();

    let Ok(entries) = std::fs::read_dir(themes_dir()) else {
        return (themes, problems);
    };

    let mut files: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().map(|ext| ext == "toml").unwrap_or(false))
        .collect();

    // Ordre stable : deux thèmes du même dossier apparaissent toujours dans le
    // même ordre, quelle que soit la façon dont le système énumère les
    // fichiers.
    files.sort();

    for path in files {
        match ThemeFile::load(&path) {
            Ok(file) => themes.push(Theme {
                name: file.meta.name.clone(),
                appearance: file.meta.appearance,
                file: Some(file),
            }),
            Err(error) => {
                let name = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("?")
                    .to_string();

                problems.push(format!("{name} : {error}"));
            }
        }
    }

    (themes, problems)
}

/// Charge un thème depuis un fichier extérieur dans le dossier des thèmes.
///
/// Le fichier est lu **avant** d'être copié : un thème invalide ne doit pas
/// entrer dans le dossier, où il resterait à traîner et à faire échouer le
/// chargement à chaque démarrage.
///
/// Le nom du fichier de destination suit le nom du thème, pour qu'on retrouve
/// `Moka sombre` sous `moka-sombre.toml`.
pub fn import(source: &Path) -> Result<ThemeFile, String> {
    let file = ThemeFile::load(source)?;

    if is_reserved_name(&file.meta.name) {
        return Err(format!(
            "« {} » est le nom d'un thème natif : choisissez un autre nom",
            file.meta.name
        ));
    }

    let destination = themes_dir().join(format!("{}.toml", crate::project::slug(&file.meta.name)));

    std::fs::copy(source, &destination)
        .map_err(|error| format!("copie vers {} : {error}", destination.display()))?;

    // Le thème est relu depuis sa destination : l'appelant dispose ainsi de la
    // provenance réelle du fichier, et non de celle qu'on lui a donnée.
    ThemeFile::load(&destination)
}

/// Retire un thème local.
///
/// Les thèmes natifs n'ont pas de fichier : la tentative est refusée plutôt
/// qu'ignorée, pour que l'interface ne propose jamais une suppression qui ne
/// ferait rien.
pub fn remove(name: &str) -> Result<(), String> {
    if is_reserved_name(name) {
        return Err(format!(
            "« {name} » est un thème natif et ne peut être supprimé"
        ));
    }

    let path = themes_dir().join(format!("{}.toml", crate::project::slug(name)));

    std::fs::remove_file(&path)
        .map_err(|error| format!("suppression de {} : {error}", path.display()))
}

/// Écrit un thème modèle dans le dossier des thèmes, s'il n'y est pas déjà.
///
/// Sert de point de départ : recopier un thème natif n'est pas possible
/// puisqu'ils n'ont pas de fichier, et partir d'un fichier vide obligerait à
/// deviner le format.
pub fn write_example() -> Result<PathBuf, String> {
    let path = themes_dir().join("exemple.toml");

    if path.exists() {
        return Ok(path);
    }

    std::fs::write(&path, EXAMPLE)
        .map_err(|error| format!("écriture de {} : {error}", path.display()))?;

    Ok(path)
}

/// Le thème d'exemple, écrit dans le dossier au premier besoin.
///
/// Il est **commenté** : c'est la documentation du format, au plus près de
/// l'endroit où l'on écrit.
pub const EXAMPLE: &str = r##"# Thème de typst-writer.
#
# Un thème ne change que des couleurs et des polices. La disposition
# (largeur des panneaux, marges de la page, comportement de l'éditeur) n'en
# dépend pas.
#
# Seules les quatre couleurs de [colors] sont obligatoires. Tout le reste est
# déduit des autres si vous ne l'écrivez pas : un fichier valide peut tenir en
# cinq lignes.

[meta]
name = "Mon thème"
# "light" ou "dark" : à quel mode du système ce thème répond.
appearance = "dark"

[colors]
# Fond de l'application, et fond des panneaux. Ils doivent DIFFÉRER : c'est
# cette différence qui fait apparaître la zone de rédaction comme une page,
# et donc ses marges.
background = "#1e1e2e"
surface    = "#181825"
text       = "#cdd6f4"
accent     = "#89b4fa"

# Facultatives : déduites de surface / text / background si omises.
# surface_hover  = "#313244"
# surface_active = "#45475a"
# text_muted     = "#a6adc8"
# border         = "#313244"
# danger         = "#f38ba8"
# warning        = "#f9e2af"
# success        = "#a6e3a1"

# Les polices. Seules des familles et des tailles ; une famille absente du
# système retombe sur la police par défaut du moteur.
[fonts]
# interface   = "Inter"
# editor      = "JetBrains Mono"
# editor_size = 17.0
# line_height = 1.6

# Les couleurs de l'éditeur. Déduites de [colors] si omises.
[editor]
# background        = "#181825"
# text              = "#cdd6f4"
# selection         = "#45475a"
# current_line      = "#313244"
# gutter_background = "#181825"
# line_number       = "#6c7086"
# scrollbar         = "#313244"
# scroller          = "#585b70"
# search_match      = "#f9e2af"
# bracket_match     = "#89b4fa"

# Les couleurs de la coloration syntaxique. Déduites de [colors] si omises.
[syntax]
# comment      = "#6c7086"
# keyword      = "#cba6f7"
# string       = "#a6e3a1"
# number       = "#fab387"
# function     = "#89b4fa"
# type_name    = "#f9e2af"
# constant     = "#fab387"
# operator     = "#89dceb"
# punctuation  = "#9399b2"
# heading      = "#89b4fa"
# strong       = "#f9e2af"
# emphasis     = "#f5c2e7"
# raw          = "#a6e3a1"
# link         = "#89dceb"
"##;

#[cfg(test)]
mod tests {
    use super::*;

    /// Le thème livré se charge, et tient son style : le papier beige, une
    /// encre gris-brun, un vert calme. Un fichier livré cassé ne doit pas se
    /// semer en silence — ce test le relirait à chaque suite.
    #[test]
    fn everforest_se_charge_et_tient_son_style() {
        let fichier = ThemeFile::parse(include_str!("everforest.toml"), None)
            .expect("everforest.toml se charge");

        assert_eq!(fichier.meta.name, "Everforest");
        assert_eq!(fichier.meta.appearance, Appearance::Light);

        // Le beige du papier : plus de rouge et de vert que de bleu.
        let fond = fichier.colors.background.0;
        assert!(fond.r > fond.b, "le papier est beige, pas bleu : {fond:?}");

        // L'encre : un gris-brun profond, sombre sur le papier clair.
        let encre = fichier.colors.text.0;
        assert!(fond.r > encre.r, "l'encre est plus sombre que le papier");
    }

    /// La famille de l'éditeur vient du thème, et n'a pas d'autre source.
    ///
    /// Un thème qui nomme une famille doit l'obtenir ; un thème qui n'en nomme
    /// aucune garde la chasse fixe. C'est le seul endroit où ces deux issues se
    /// décident.
    #[test]
    fn the_editor_font_comes_from_the_theme() {
        let mut fonts = Fonts::default();

        assert_eq!(
            fonts.editor_font().family,
            iced::font::Family::Monospace,
            "sans famille nommée, la chasse fixe reste la règle"
        );

        fonts.editor = Some("JetBrains Mono".to_string());

        assert_eq!(
            fonts.editor_font().family,
            iced::font::Family::Name("JetBrains Mono"),
            "une famille nommée par le thème est celle de l'éditeur"
        );
    }

    /// Ce que l'application trouve réellement dans le dossier de thèmes de
    /// l'utilisateur — et ce qu'elle reproche aux fichiers qu'elle écarte.
    ///
    /// Un thème qui ne se charge pas est un thème qui ne se voit pas : le
    /// réglage retombe **en silence** sur un thème natif, et le fichier fautif
    /// reste en place. Le premier symptôme n'est pas « mon thème manque » mais
    /// « les couleurs ne suivent pas ce que je choisis » — d'où cette sonde,
    /// qui nomme le fichier et la raison.
    #[test]
    #[ignore = "lit le dossier de thèmes de l'utilisateur"]
    fn diagnose_user_themes() {
        let dir = themes_dir();
        println!("dossier des thèmes : {}", dir.display());

        let (themes, problems) = list();

        for theme in themes.iter().filter(|theme| theme.file.is_some()) {
            println!("  fichier : « {} » ({:?})", theme.name, theme.appearance);
        }

        for problem in &problems {
            println!("  ÉCARTÉ  : {problem}");
        }

        println!(
            "{} thèmes ({} natifs, {} fichiers), {} écarté(s)",
            themes.len(),
            themes.iter().filter(|theme| theme.file.is_none()).count(),
            themes.iter().filter(|theme| theme.file.is_some()).count(),
            problems.len()
        );
    }

    /// Le fichier d'exemple doit rester un thème valide.
    ///
    /// C'est lui que recopiera l'utilisateur : s'il était cassé, la première
    /// chose qu'on lui donnerait à lire serait une erreur.
    #[test]
    fn the_example_file_is_a_valid_theme() {
        let theme = ThemeFile::parse(EXAMPLE, None).expect("l'exemple doit être valide");

        assert_eq!(theme.meta.name, "Mon thème");
        assert_eq!(theme.meta.appearance, Appearance::Dark);

        // L'exemple n'écrit que l'essentiel : tout le reste doit être déduit.
        assert_eq!(theme.fonts.editor_size(), DEFAULT_EDITOR_SIZE);

        let _ = theme.colors.text_muted();
        let _ = theme.colors.success();
    }

    /// L'auteur, quand il est écrit, est bien lu.
    #[test]
    fn the_author_is_read_when_present() {
        let theme = ThemeFile::parse(
            r##"
            [meta]
            name = "Signé"
            author = "Quelqu'un"

            [colors]
            background = "#101010"
            surface = "#181818"
            text = "#eeeeee"
            accent = "#5588ff"
            "##,
            None,
        )
        .unwrap();

        assert_eq!(theme.meta.author.as_deref(), Some("Quelqu'un"));

        // Omis, il n'y a simplement pas d'auteur.
        let anonymous = ThemeFile::parse(EXAMPLE, None).unwrap();

        assert_eq!(anonymous.meta.author, None);
    }

    /// Le nom est la seule clé obligatoire avec les quatre couleurs de base.
    #[test]
    fn a_minimal_theme_is_enough() {
        let theme = ThemeFile::parse(
            r##"
            [meta]
            name = "Court"

            [colors]
            background = "#101010"
            surface = "#181818"
            text = "#eeeeee"
            accent = "#5588ff"
            "##,
            None,
        )
        .expect("un thème de huit lignes doit être valide");

        assert_eq!(theme.meta.appearance, Appearance::Dark, "sombre par défaut");
        assert_eq!(theme.fonts.editor_font(), iced::Font::MONOSPACE);
        assert!(theme.editor.background.is_none());
    }

    /// Un nom vide ne produit pas un thème : rien à afficher dans la liste.
    #[test]
    fn a_theme_without_a_name_is_refused() {
        let error = ThemeFile::parse(
            r##"
            [meta]
            name = "   "

            [colors]
            background = "#101010"
            surface = "#181818"
            text = "#eeeeee"
            accent = "#5588ff"
            "##,
            None,
        )
        .unwrap_err();

        assert!(error.contains("nom"), "message obtenu : {error}");
    }

    /// Une couleur illisible est refusée en nommant la valeur, pas en laissant
    /// un message de conversion générique.
    #[test]
    fn a_bad_color_names_the_offending_value() {
        let error = ThemeFile::parse(
            r##"
            [meta]
            name = "Cassé"

            [colors]
            background = "bleu nuit"
            surface = "#181818"
            text = "#eeeeee"
            accent = "#5588ff"
            "##,
            None,
        )
        .unwrap_err();

        assert!(error.contains("bleu nuit"), "message obtenu : {error}");
    }

    /// Deux clés inconnues ne passent pas inaperçues : une faute de frappe dans
    /// un fichier écrit à la main laisserait sinon une couleur silencieusement
    /// ignorée.
    #[test]
    fn an_unknown_key_is_refused() {
        let error = ThemeFile::parse(
            r##"
            [meta]
            name = "Faute de frappe"

            [colors]
            background = "#101010"
            surface = "#181818"
            text = "#eeeeee"
            accent = "#5588ff"
            accnet = "#ffffff"
            "##,
            None,
        )
        .unwrap_err();

        assert!(error.contains("accnet"), "message obtenu : {error}");
    }

    /// Un fond de page identique au fond d'application rend la marge invisible.
    #[test]
    fn identical_background_and_surface_are_refused() {
        let error = ThemeFile::parse(
            r##"
            [meta]
            name = "Plat"

            [colors]
            background = "#181818"
            surface = "#181818"
            text = "#eeeeee"
            accent = "#5588ff"
            "##,
            None,
        )
        .unwrap_err();

        assert!(error.contains("surface"), "message obtenu : {error}");
    }

    /// Une taille de texte absurde est refusée au chargement, où l'on peut
    /// encore nommer le fichier fautif.
    #[test]
    fn an_absurd_font_size_is_refused() {
        for size in ["0.5", "300.0"] {
            let text = format!(
                r##"
                [meta]
                name = "Immense"

                [colors]
                background = "#101010"
                surface = "#181818"
                text = "#eeeeee"
                accent = "#5588ff"

                [fonts]
                editor_size = {size}
                "##
            );

            let error = ThemeFile::parse(&text, None).unwrap_err();

            assert!(error.contains("editor_size"), "message obtenu : {error}");
        }
    }

    /// La lecture hexadécimale accepte les deux écritures, avec ou sans `#`.
    #[test]
    fn colors_are_read_with_or_without_a_hash() {
        assert_eq!(
            HexColor::parse("#a6e3a1").unwrap().0,
            HexColor::parse("a6e3a1").unwrap().0
        );

        // Sans transparence, la couleur est opaque.
        assert_eq!(HexColor::parse("#ff0000").unwrap().0.a, 1.0);

        // Avec, elle est respectée.
        let half = HexColor::parse("#ff000080").unwrap().0;

        assert!((half.a - 0.50).abs() < 0.01, "alpha obtenu : {}", half.a);
    }

    /// Les couleurs omises sont déduites, et non laissées vides : c'est ce qui
    /// permet d'écrire un thème de cinq lignes.
    #[test]
    fn omitted_colors_are_derived_from_the_others() {
        let theme = ThemeFile::parse(EXAMPLE, None).unwrap();

        let surface = theme.colors.surface.0;
        let hover = theme.colors.surface_hover();
        let active = theme.colors.surface_active();

        assert_ne!(hover, surface, "le survol doit se détacher du fond");
        assert_ne!(active, hover, "l'état actif doit se détacher du survol");

        // Le texte secondaire se place entre le texte et le fond.
        let muted = theme.colors.text_muted();

        assert_ne!(muted, theme.colors.text.0);
        assert_ne!(muted, theme.colors.background.0);

        // Et les couleurs de statut ont bien une valeur par défaut.
        for color in [
            theme.colors.danger(),
            theme.colors.warning(),
            theme.colors.success(),
        ] {
            assert!(color.a > 0.0);
        }
    }

    /// Les thèmes natifs sont ceux que l'application a toujours connus, et ils
    /// ne viennent d'aucun fichier.
    #[test]
    fn native_themes_have_no_file() {
        let natives = natives();

        assert_eq!(natives.len(), 2);

        for theme in &natives {
            assert!(theme.is_native(), "« {} » doit être natif", theme.name);
            assert!(theme.file.is_none());
        }

        assert_eq!(natives[0].name, "Sombre");
        assert_eq!(natives[1].name, "Clair");
    }

    /// Un nom de thème natif ne peut pas être repris par un fichier : il
    /// masquerait le filet de sécurité, ou pire, ferait croire qu'on l'a
    /// remplacé.
    #[test]
    fn native_names_are_reserved() {
        assert!(is_reserved_name("Sombre"));
        assert!(is_reserved_name("Sombre "));
        assert!(is_reserved_name("clair"));

        assert!(!is_reserved_name("Mon thème"));
        assert!(!is_reserved_name("Clair de lune"));
    }

    /// Le thème retenu est retrouvé **par son nom**.
    ///
    /// C'est ce qui permet de le restaurer au redémarrage : la liste peut avoir
    /// changé entre-temps — un thème chargé, un autre supprimé — sans que le
    /// choix soit perdu ou décalé.
    #[test]
    fn the_chosen_theme_is_found_by_name() {
        let mut themes = natives();

        let file = ThemeFile::parse(EXAMPLE, None).unwrap();
        let name = file.meta.name.clone();

        themes.push(Theme {
            name: name.clone(),
            appearance: file.meta.appearance,
            file: Some(file),
        });

        let chosen = Chosen {
            light: "Clair".to_string(),
            dark: name.clone(),
        };

        let index = select(&themes, Appearance::Dark, &chosen);

        assert_eq!(themes[index].name, name);
        assert_eq!(themes[index].appearance, Appearance::Dark);
    }

    /// Chaque apparence garde son propre thème : suivre le système ne fait que
    /// choisir laquelle des deux s'applique.
    #[test]
    fn each_appearance_keeps_its_own_theme() {
        let mut themes = natives();

        let file = ThemeFile::parse(EXAMPLE, None).unwrap();
        let name = file.meta.name.clone();

        themes.push(Theme {
            name: name.clone(),
            appearance: file.meta.appearance,
            file: Some(file),
        });

        let chosen = Chosen {
            light: "Clair".to_string(),
            dark: name.clone(),
        };

        assert_eq!(
            themes[select(&themes, Appearance::Dark, &chosen)].name,
            name,
            "en sombre, le thème sombre retenu"
        );
        assert_eq!(
            themes[select(&themes, Appearance::Light, &chosen)].name,
            "Clair",
            "en clair, le thème clair retenu"
        );
    }

    /// Un thème retenu puis supprimé ne laisse pas la fenêtre sans couleurs.
    ///
    /// C'est le cas qui guette au redémarrage : la session nomme un thème dont
    /// le fichier a disparu entre-temps. La sélection doit se rabattre sur un
    /// thème de la **même apparence** — retomber sur le premier de la liste
    /// donnerait un thème clair en pleine nuit.
    #[test]
    fn a_deleted_theme_falls_back_within_its_appearance() {
        let themes = natives();

        let chosen = Chosen {
            light: "Clair".to_string(),
            dark: "Thème disparu".to_string(),
        };

        let index = select(&themes, Appearance::Dark, &chosen);

        assert_eq!(themes[index].name, "Sombre");
        assert_eq!(themes[index].appearance, Appearance::Dark);
    }

    /// Le nom retenu est bien celui qu'on relit.
    #[test]
    fn a_chosen_name_survives_a_round_trip() {
        let mut chosen = Chosen::default();

        chosen.set(Appearance::Light, "Moka clair");
        chosen.set(Appearance::Dark, "Moka sombre");

        assert_eq!(chosen.get(Appearance::Light), "Moka clair");
        assert_eq!(chosen.get(Appearance::Dark), "Moka sombre");

        // Les deux apparences sont indépendantes : écrire l'une ne touche pas
        // l'autre.
        chosen.set(Appearance::Light, "Autre clair");

        assert_eq!(chosen.get(Appearance::Dark), "Moka sombre");
    }

    /// Une police vide n'est pas une police : elle retombe sur celle du moteur.
    #[test]
    fn an_empty_font_family_falls_back_to_the_default() {
        let mut fonts = Fonts {
            interface: Some("   ".to_string()),
            editor: Some(String::new()),
            ..Fonts::default()
        };

        assert_eq!(fonts.interface_font(), iced::Font::DEFAULT);

        fonts.editor = None;

        assert_eq!(fonts.editor_font(), iced::Font::MONOSPACE);
    }

    /// Une famille nommée est bien celle qui est demandée.
    #[test]
    fn a_named_font_family_is_used() {
        let fonts = Fonts {
            editor: Some("JetBrains Mono".to_string()),
            editor_size: Some(19.0),
            line_height: Some(1.4),
            ..Fonts::default()
        };

        assert_eq!(
            fonts.editor_font().family,
            iced::font::Family::Name("JetBrains Mono")
        );
        assert_eq!(fonts.editor_size(), 19.0);
        assert_eq!(fonts.line_height(), 1.4);
    }

    /// La suppression d'un thème natif est refusée, et non ignorée : une
    /// suppression qui ne ferait rien serait un mensonge.
    #[test]
    fn removing_a_native_theme_is_refused() {
        let error = remove("Sombre").unwrap_err();

        assert!(error.contains("natif"), "message obtenu : {error}");
    }

    /// Le thème d'exemple s'écrit, et n'écrase pas un fichier existant.
    #[test]
    fn the_example_is_written_once() {
        let path = write_example().expect("écriture de l'exemple");

        assert!(path.exists());

        std::fs::write(&path, "# modifié par l'utilisateur\n").unwrap();

        let again = write_example().expect("seconde écriture");

        assert_eq!(again, path);
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "# modifié par l'utilisateur\n",
            "l'exemple ne doit pas écraser le travail de l'utilisateur"
        );

        let _ = std::fs::remove_file(&path);
    }
}
