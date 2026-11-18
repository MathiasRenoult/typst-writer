//! La structure de la mise en page, et sa forme sur le disque.
//!
//! Un document a des normes, et elles se **disent** plutôt qu'elles ne
//! s'écrivent : une police, une taille, des marges, une numérotation, une
//! casse. Les voici en une structure — c'est elle qui voyage (export, import),
//! qui se compare, et qui se réécrit sans perte.
//!
//! Les longueurs sont des chaînes du type « 2.5cm » : c'est exactement ce
//! qu'écrit Typst, ce que le lecteur voit dans le fichier, et ce que le champ
//! de texte rend sans conversion cachée. Une longueur invalide n'est pas une
//! panne : elle se dit, et rien n'est écrit.

use serde::{Deserialize, Serialize};

/// Les fonctionnalités du projet (type partagé dans `base`).
pub use crate::base::valeurs::AddOns as Fonctionnalites;

/// La mise en page d'un document.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MiseEnPage {
    /// Le papier : son format et son orientation.
    pub papier: Papier,

    /// Les marges, chacune de son côté.
    pub marges: Marges,

    /// Les numéros de page.
    pub numerotation: NumerotationPage,

    /// L'en-tête et le pied de page : leur contenu exact, et la forme des
    /// numéros. `{page}` y est le numéro courant, `{pages}` le nombre de
    /// pages — « p. {page} », « page {page} sur {pages} », « {page}/{pages} ».
    pub entete_pied: EntetePiedDePage,

    /// Les références bibliques : forme des livres, chiffres, séparateurs et
    /// traductions explicites.
    pub references: References,

    /// L'index des citations bibliques : pages, regroupement, forme, colonnes.
    pub index: IndexCitations,

    /// Le texte courant : police, corps, interligne, justification.
    pub texte: Texte,

    /// Les titres de chapitres.
    pub titres: Titres,

    /// Les couleurs du document.
    pub couleurs: Couleurs,

    /// La typographie du texte composé.
    pub typographie: Typographie,

    /// Les défauts du décor : bordures, remplissages, formes.
    ///
    /// C'est ici que vivent les mesures — jamais dans la source. Le
    /// document dit l'intention (`{encadré}`), les normes disent la
    /// couleur, l'épaisseur et le rayon.
    pub decor: Decor,

    /// La forme de la numérotation des figures : "1", "A", "a", "I", "i", "1.1".
    /// Vide = les défauts de Typst.
    pub numerotation_figures: String,

    /// Le libellé de la légende : "fig.", "Figure", ou autre chose.
    /// Vide = le libellé de Typst.
    pub legende_prefixe: String,

    /// Le style de la légende : "gras", "italique", ou rien.
    pub legende_style: String,

    /// La page de titre : née des métadonnées du document (titre,
    /// auteur, date) — une norme, pas du Typst caché dans `main.typ`
    /// (bug-report-7, point 8).
    pub page_de_titre: bool,

    /// Le sommaire : composé par le moteur, une norme comme le reste.
    pub sommaire: bool,

    /// Les mots de la balise `{date:...}`.
    pub dates: Dates,

    /// La bibliographie : le style des citations et de la liste finale.
    pub bibliographie: Bibliographie,

    /// Les fonctionnalités branchées ou non (la bibliographie, le
    /// glossaire, les graphiques).
    pub fonctionnalites: Fonctionnalites,

    /// Les réglages propres aux plugins, embarqués dans le fichier de
    /// normes avec la mise en page : ils suivent le document quand on
    /// l'ouvre, sans dépendre d'un réglage de l'application.
    pub parametres_plugins: ParametresPlugins,
}

/// Les réglages des plugins qui accompagnent une norme.
///
/// La mise en page dit **ce que le document compose** ; cette section dit
/// **comment ses plugins se comportent à l'ouverture**. Elle voyage dans le
/// même `mise-en-page.toml`, et un ancien fichier qui ne la connaît pas
/// reçoit les défauts sans rien perdre.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ParametresPlugins {
    /// Le plugin biblique.
    pub biblique: ParametresBiblique,

    /// Les préréglages des callouts, embarqués dans la norme.
    pub callouts: ParametresCallouts,
}

/// Les callouts prédéfinis d'un document.
///
/// La source nomme un préréglage (`{callout attention "…"}`) ; c'est ici que
/// vivent ses couleurs, sa bordure et ses mesures. Le panneau d'insertion peut
/// surcharger ces valeurs pour une occurrence, et la source garde alors ces
/// écarts explicitement — le fichier de normes n'est pas modifié par surprise.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ParametresCallouts {
    /// Les préréglages, dans l'ordre où le panneau les montre.
    pub presets: Vec<crate::langage::callout::CalloutPreset>,
}

impl Default for ParametresCallouts {
    fn default() -> Self {
        Self {
            presets: crate::langage::callout::presets_defaut(),
        }
    }
}

impl ParametresCallouts {
    /// Le préréglage nommé, s'il existe.
    pub fn preset(&self, cle: &str) -> Option<&crate::langage::callout::CalloutPreset> {
        self.presets.iter().find(|preset| preset.cle == cle)
    }

    /// Un préréglage libre, pour le bouton « Nouveau » : une clé stable et
    /// lisible, jamais en collision avec une autre.
    pub fn cle_libre(&self) -> String {
        for rang in 1.. {
            let cle = format!("perso-{rang}");
            if self.preset(&cle).is_none() {
                return cle;
            }
        }

        "perso".to_string()
    }
}

/// Les réglages du plugin biblique embarqués dans une norme.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ParametresBiblique {
    /// La traduction consultée par défaut : `lsg`, `darby` ou le code d'une
    /// version biblique ajoutée localement.
    pub traduction: String,

    /// L'ordre de la liste des références : `apparition` ou `canonique`.
    pub tri: String,
}

impl Default for ParametresBiblique {
    fn default() -> Self {
        Self {
            traduction: "lsg".to_string(),
            tri: "apparition".to_string(),
        }
    }
}

/// La bibliographie d'un projet.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Bibliographie {
    /// Le style des citations et de la liste finale, tel que le moteur
    /// Typst l'attend (« chicago-notes », « apa »…).
    ///
    /// `None` : le document décide par sa propre directive, et à défaut le
    /// style de base du moteur. Un nom refusé par le moteur n'est jamais
    /// appliqué — il se dit (R4).
    pub style: Option<String>,

    /// Un saut de page avant la bibliographie (bug-report-10, 1.1).
    #[serde(default)]
    pub saut_de_page: bool,

    /// La bibliographie en titre de niveau 1 (bug-report-10, 1.2).
    #[serde(default)]
    pub titre_un: bool,
}

/// Les défauts du décor d'un document.
///
/// Un bloc habillé dit un **nom**, jamais une mesure : `{encadré}` prend
/// ici sa couleur de bordure, son épaisseur, son rayon et sa marge. Les
/// valeurs sont des expressions Typst, ou `aucun` pour rien du tout.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Decor {
    /// La couleur de bordure — `gray`, `#555`, `luma(160)`…
    pub couleur: String,

    /// L'épaisseur de la bordure — `1pt`, `0.5pt`…
    pub epaisseur: String,

    /// Le rayon des coins — `4pt`, `0pt`, `(top-left: 4pt)`…
    pub rayon: String,

    /// La marge intérieure — `8pt`, `(x: 4pt, y: 8pt)`…
    pub marge: String,

    /// Le remplissage — `aucun`, `luma(248)`, `#fdf6e3`…
    pub remplissage: String,

    /// La couleur des séparateurs horizontaux (`---`) — `gray`, `#555`…
    pub separateur_couleur: String,

    /// L'épaisseur des séparateurs horizontaux — `0.5pt`, `1pt`…
    pub separateur_epaisseur: String,
}

impl Default for Decor {
    fn default() -> Self {
        Self {
            couleur: "gray".to_string(),
            epaisseur: "1pt".to_string(),
            rayon: "4pt".to_string(),
            marge: "8pt".to_string(),
            remplissage: "aucun".to_string(),
            separateur_couleur: "gray".to_string(),
            separateur_epaisseur: "0.5pt".to_string(),
        }
    }
}

/// Le papier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Papier {
    /// Le format : `a4`, `a5`, `a3`, `letter`, `legal`, ou `auto` (les marges
    /// décident seules de la taille).
    pub format: String,

    /// `portrait` ou `paysage`.
    pub orientation: String,
}

impl Default for Papier {
    fn default() -> Self {
        Self {
            format: "a4".to_string(),
            orientation: "portrait".to_string(),
        }
    }
}

/// Les marges.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Marges {
    pub haut: String,
    pub bas: String,
    pub gauche: String,
    pub droite: String,
}

impl Default for Marges {
    fn default() -> Self {
        Self {
            haut: "2cm".to_string(),
            bas: "2cm".to_string(),
            gauche: "2cm".to_string(),
            droite: "2cm".to_string(),
        }
    }
}

/// Les numéros de page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct NumerotationPage {
    /// Les numéros sont-ils imprimés ?
    pub actif: bool,

    /// Leur forme : `1`, `I`, `i`, `A`, `a`.
    pub forme: String,

    /// `haut` ou `bas`.
    pub position: String,

    /// `gauche`, `centre` ou `droite`.
    pub alignement: String,
}

impl Default for NumerotationPage {
    fn default() -> Self {
        Self {
            actif: true,
            forme: "1".to_string(),
            position: "bas".to_string(),
            alignement: "centre".to_string(),
        }
    }
}

/// L'en-tête et le pied de page : ce qu'ils portent, et comment le numéro
/// s'écrit.
///
/// La source dit l'intention, les normes disent la mesure : le contenu est
/// du texte libre où `{page}` et `{pages}` prennent leur place, et la
/// numérotation est une **forme** — `1`, `a`, `A`, `i`, `I`, `α`, `א` —
/// celles que Typst sait composer. Les formes parlées (« p. 1 », « page 1
/// sur 12 », « 1/12 ») sont des motifs de contenu, pas des numérotations :
/// la mesure reste au gabarit, le choix à la personne.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EntetePiedDePage {
    /// L'en-tête, en ses trois places — vides : pas d'en-tête.
    pub entete: Emplacement,

    /// Le pied de page, en ses trois places — vides : la numérotation
    /// seule, comme avant que ce champ existe.
    pub pied: Emplacement,

    /// La forme du numéro : `1`, `a`, `A`, `i`, `I`, `α`, `א`.
    pub numerotation: String,

    /// La hauteur des images, en pourcentage de la hauteur disponible de
    /// la zone qui les porte (bug-report-8, T4.4). Les proportions se
    /// gardent : seule la hauteur se règle.
    pub images_hauteur: String,
}

/// Les trois places d'un en-tête ou d'un pied de page : ce qui va à
/// gauche, au centre, à droite.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Emplacement {
    pub gauche: String,
    pub centre: String,
    pub droite: String,
}

impl Emplacement {
    /// Les trois places sont-elles vides ?
    pub fn est_vide(&self) -> bool {
        self.gauche.is_empty() && self.centre.is_empty() && self.droite.is_empty()
    }
}

impl Default for EntetePiedDePage {
    fn default() -> Self {
        Self {
            entete: Emplacement::default(),
            pied: Emplacement::default(),
            numerotation: "1".to_string(),
            images_hauteur: "80".to_string(),
        }
    }
}

/// Le canon biblique qui décide de la séparation de l'index.
///
/// Le canon ne change jamais le texte cité : il dit seulement quels livres
/// l'index place sous « Livres canoniques » et lesquels sous « Livres non
/// canoniques ». `Tout` conserve une seule liste, comme avant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Canon {
    /// Les 66 livres du canon protestant.
    Protestant,
    /// Le canon catholique, avec les deutérocanoniques.
    Catholique,
    /// Le canon orthodoxe, avec ses livres supplémentaires.
    Orthodoxe,
    /// Le canon de l'Église orthodoxe tewahedo.
    Tewahedo,
    /// Tout le corpus connu, sans séparation.
    #[default]
    Tout,
}

impl Canon {
    /// Le code écrit dans les normes.
    pub fn code(self) -> &'static str {
        match self {
            Canon::Protestant => "protestant",
            Canon::Catholique => "catholique",
            Canon::Orthodoxe => "orthodoxe",
            Canon::Tewahedo => "tewahedo",
            Canon::Tout => "tout",
        }
    }

    /// Le libellé de la liste de choix.
    pub fn libelle(self) -> &'static str {
        match self {
            Canon::Protestant => "Protestant",
            Canon::Catholique => "Catholique",
            Canon::Orthodoxe => "Orthodoxe",
            Canon::Tewahedo => "Téwahedo",
            Canon::Tout => "Tout le corpus",
        }
    }

    /// Ce livre appartient-il au canon choisi ?
    pub fn contient(self, osis: &str) -> bool {
        match self {
            Canon::Tout => true,
            Canon::Protestant => CANON_PROTESTANT.contains(&osis),
            Canon::Catholique => {
                CANON_PROTESTANT.contains(&osis) || CANON_CATHOLIQUE.contains(&osis)
            }
            Canon::Orthodoxe => {
                CANON_PROTESTANT.contains(&osis)
                    || CANON_CATHOLIQUE.contains(&osis)
                    || CANON_ORTHODOXE.contains(&osis)
            }
            Canon::Tewahedo => {
                CANON_PROTESTANT.contains(&osis)
                    || CANON_CATHOLIQUE.contains(&osis)
                    || CANON_ORTHODOXE.contains(&osis)
                    || CANON_TEWAHEDO.contains(&osis)
            }
        }
    }
}

/// Les 66 livres du canon protestant, en codes OSIS.
const CANON_PROTESTANT: &[&str] = &[
    "Gen", "Exod", "Lev", "Num", "Deut", "Josh", "Judg", "Ruth", "1Sam", "2Sam", "1Kgs", "2Kgs",
    "1Chr", "2Chr", "Ezra", "Neh", "Esth", "Job", "Ps", "Prov", "Eccl", "Song", "Isa", "Jer",
    "Lam", "Ezek", "Dan", "Hos", "Joel", "Amos", "Obad", "Jonah", "Mic", "Nah", "Hab", "Zeph",
    "Hag", "Zech", "Mal", "Matt", "Mark", "Luke", "John", "Acts", "Rom", "1Cor", "2Cor", "Gal",
    "Eph", "Phil", "Col", "1Thess", "2Thess", "1Tim", "2Tim", "Titus", "Phlm", "Heb", "Jas",
    "1Pet", "2Pet", "1John", "2John", "3John", "Jude", "Rev",
];

/// Les deutérocanoniques du canon catholique.
const CANON_CATHOLIQUE: &[&str] = &[
    "Tob", "Jdt", "AddEst", "1Macc", "2Macc", "Wis", "Sir", "Bar", "EpJer",
];

/// Les livres supplémentaires du canon orthodoxe.
const CANON_ORTHODOXE: &[&str] = &["1Esd", "2Esd", "3Esd", "PrMan", "Ps151", "3Macc", "4Macc"];

/// Les livres propres au canon tewahedo.
const CANON_TEWAHEDO: &[&str] = &[
    "1En", "2En", "Jub", "TReu", "TSim", "TLevi", "TJud", "TIss", "TZeb", "TDan", "TNaph", "TGad",
    "TAsh", "TJos", "TBen", "2Bar", "3Bar", "ApAb", "ApZeph", "ApPet", "ApPaul", "ApMos",
];

/// La configuration de l'index des citations bibliques.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct IndexCitations {
    /// Montrer, après chaque référence, les pages où elle apparaît.
    pub afficher_pages: bool,
    /// Regrouper les références sous le nom de leur livre.
    pub regrouper_par_livre: bool,
    /// La forme des livres dans l'index ; `None` hérite des références.
    pub forme: Option<SystemeAbreviation>,
    /// Le nombre de colonnes de l'index (1 à 4).
    pub colonnes: usize,
    /// Le canon qui sépare les livres canoniques des autres.
    pub canon: Canon,
}

impl Default for IndexCitations {
    fn default() -> Self {
        Self {
            afficher_pages: false,
            regrouper_par_livre: false,
            forme: None,
            colonnes: 1,
            canon: Canon::Tout,
        }
    }
}

/// La syntaxe des références bibliques : comment elles s'écrivent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct References {
    /// Le système d'abréviation du livre.
    pub livre: SystemeAbreviation,
    /// Les chiffres des chapitres et des versets.
    pub chiffres: Chiffres,
    /// Le séparateur entre le chapitre et le verset : `.`, `:` ou `,`.
    pub separateur_chapitre_verset: String,
    /// Le séparateur d'une plage de versets : `-`, `–` ou ` à `.
    pub separateur_plage: String,
    /// Le séparateur entre les livres d'un groupe : `;` ou `; `.
    pub separateur_livre: String,
    /// Le séparateur entre deux occurrences du même livre : `, `.
    pub separateur_occurrence: String,
    /// Le mot qui relie deux chapitres entiers : ` à `.
    pub separateur_chapitres: String,
    /// Le gabarit d'une référence. Jetons : `{livre}`, `{chapitres}`,
    /// `{versets}`, `{nombres}`, `{chapitre}`, `{chapitre_fin}`,
    /// `{verset}`, `{verset_fin}`, `{reference}`.
    pub gabarit: String,
    /// Ranger toutes les références d'une balise dans l'ordre canonique.
    pub ordre_canonique: bool,
}

impl Default for References {
    fn default() -> Self {
        Self {
            livre: SystemeAbreviation::FrancaiseCourte,
            chiffres: Chiffres::Arabes,
            separateur_chapitre_verset: ".".to_string(),
            separateur_plage: "-".to_string(),
            separateur_livre: " ; ".to_string(),
            separateur_occurrence: ", ".to_string(),
            separateur_chapitres: " à ".to_string(),
            gabarit: "{livre} {nombres}".to_string(),
            ordre_canonique: false,
        }
    }
}

/// Le système d'abréviation du livre — les quatre usages les plus courants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemeAbreviation {
    /// La forme française courte : « Gn », « Jn ».
    FrancaiseCourte,
    /// La forme française longue : « Genèse », « Jean ».
    FrancaiseLongue,
    /// La forme OSIS : « Gen », « John ».
    Osis,
    /// La forme courte sans point : « Gn », « Jn ».
    SansPoint,
    /// Les abréviations de la TOB : « Gn », « 1 S », « 1 Co »…
    Tob,
}

/// Les chiffres des chapitres et des versets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Chiffres {
    /// « 3:16 ».
    Arabes,
    /// « III, 16 ».
    Romains,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Texte {
    /// La police, ou plusieurs séparées par des virgules — la seconde sert à
    /// ce que la première ne sait pas écrire.
    pub police: String,

    /// Le corps.
    pub taille: String,

    /// L'interligne, comme Typst le nomme : la distance entre les lignes d'un
    /// même paragraphe.
    pub interligne: String,

    /// L'espace entre les paragraphes.
    pub espacement: String,

    /// Le retrait de la première ligne d'un paragraphe.
    pub retrait: String,

    /// Les paragraphes sont-ils justifiés ?
    pub justification: bool,

    /// Les mots sont-ils coupés en fin de ligne ?
    pub coupure: bool,

    /// La langue du texte, pour Typst (césure, guillemets…).
    pub langue: String,

    /// La typographie d'avant la section dédiée, héritée des normes écrites
    /// par une version précédente. La migration de [`MiseEnPage::lire`] la
    /// porte dans la section `typographie` — ce champ ne s'écrit plus.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub typographie: Option<String>,
}

impl Default for Texte {
    fn default() -> Self {
        Self {
            police: "New Computer Modern".to_string(),
            taille: "11pt".to_string(),
            interligne: "0.65em".to_string(),
            espacement: "1.2em".to_string(),
            retrait: "0cm".to_string(),
            justification: true,
            coupure: true,
            langue: "fr".to_string(),
            typographie: None,
        }
    }
}

/// Les titres de chapitres.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Titres {
    /// Le corps d'un titre de chaque niveau, du premier au sixième.
    pub taille_1: String,
    pub taille_2: String,
    pub taille_3: String,
    pub taille_4: String,
    pub taille_5: String,
    pub taille_6: String,

    /// Les titres sont-ils en gras ?
    pub gras: bool,

    /// La casse : `normale`, `majuscules`, `minuscules`,
    /// `petites-majuscules`.
    pub casse: String,

    /// La casse par niveau — si présente, elle gagne sur `casse`.
    pub casse_1: Option<String>,
    pub casse_2: Option<String>,
    pub casse_3: Option<String>,
    pub casse_4: Option<String>,
    pub casse_5: Option<String>,
    pub casse_6: Option<String>,

    /// La couleur par niveau — si présente, elle gagne sur `couleurs.titres`.
    pub couleur_1: Option<String>,
    pub couleur_2: Option<String>,
    pub couleur_3: Option<String>,
    pub couleur_4: Option<String>,
    pub couleur_5: Option<String>,
    pub couleur_6: Option<String>,

    /// La graisse par niveau — si présente, elle gagne sur `gras`.
    pub gras_1: Option<bool>,
    pub gras_2: Option<bool>,
    pub gras_3: Option<bool>,
    pub gras_4: Option<bool>,
    pub gras_5: Option<bool>,
    pub gras_6: Option<bool>,
    /// L'italique par niveau — absent : le texte courant.
    pub italique_1: Option<bool>,
    pub italique_2: Option<bool>,
    pub italique_3: Option<bool>,
    pub italique_4: Option<bool>,
    pub italique_5: Option<bool>,
    pub italique_6: Option<bool>,
    /// Le souligné par niveau — absent : le texte courant.
    pub souligne_1: Option<bool>,
    pub souligne_2: Option<bool>,
    pub souligne_3: Option<bool>,
    pub souligne_4: Option<bool>,
    pub souligne_5: Option<bool>,
    pub souligne_6: Option<bool>,

    /// L'espacement avant/après par niveau — si présent, il gagne sur `espacement`.
    pub espacement_1: Option<String>,
    pub espacement_2: Option<String>,
    pub espacement_3: Option<String>,
    pub espacement_4: Option<String>,
    pub espacement_5: Option<String>,
    pub espacement_6: Option<String>,

    /// La numérotation des chapitres : `1.1`, `A.1`, ou vide pour aucune.
    pub numerotation: String,

    /// L'espace au-dessus et en dessous d'un titre.
    pub espacement: String,
}

impl Default for Titres {
    fn default() -> Self {
        Self {
            taille_1: "16pt".to_string(),
            taille_2: "14pt".to_string(),
            taille_3: "12pt".to_string(),
            taille_4: "11pt".to_string(),
            taille_5: "10pt".to_string(),
            taille_6: "10pt".to_string(),
            gras: true,
            casse: "normale".to_string(),
            casse_1: None,
            casse_2: None,
            casse_3: None,
            casse_4: None,
            casse_5: None,
            casse_6: None,
            couleur_1: None,
            couleur_2: None,
            couleur_3: None,
            couleur_4: None,
            couleur_5: None,
            couleur_6: None,
            gras_1: None,
            gras_2: None,
            gras_3: None,
            gras_4: None,
            gras_5: None,
            gras_6: None,
            italique_1: None,
            italique_2: None,
            italique_3: None,
            italique_4: None,
            italique_5: None,
            italique_6: None,
            souligne_1: None,
            souligne_2: None,
            souligne_3: None,
            souligne_4: None,
            souligne_5: None,
            souligne_6: None,
            espacement_1: None,
            espacement_2: None,
            espacement_3: None,
            espacement_4: None,
            espacement_5: None,
            espacement_6: None,
            numerotation: "1.1".to_string(),
            espacement: "1.2em".to_string(),
        }
    }
}

/// Les couleurs du document, en hexadécimal.
/// La typographie appliquée au texte composé : un défaut de langue, et ses
/// règles, reprises une par une.
///
/// Le défaut charge les règles de la langue choisie — `française`, `anglaise`,
/// `aucune` — et chaque règle se reprend ensuite dans le panneau de
/// typographie. Les guillemets se composent **par niveau d'imbrication** :
/// `«`, `"` puis `'` en français.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Typographie {
    /// Le défaut chargé : `aucune`, `française`, `anglaise`.
    pub base: String,

    /// L'espace devant les signes doubles `; : ! ?` : `aucune`, `normale`,
    /// `insécable`.
    pub avant_signes: String,

    /// L'espace après les signes doubles.
    pub apres_signes: String,

    /// L'espace après la virgule et le point, hors nombres.
    pub apres_virgule_point: String,

    /// L'espace à l'intérieur des guillemets.
    pub espace_guillemets: String,

    /// Les paires de guillemets par niveau d'imbrication : deux caractères
    /// par paire — ouvrant puis fermant — `«»`, `""`, `''`.
    pub guillemets: Vec<String>,

    /// Les caractères que la frappe reconnaît comme guillemets ouvrants.
    /// La paire produite reste celle de `guillemets` ; ces caractères ne
    /// font que **déclencher** la composition (ÉVOLUTION-002).
    pub guillemets_ouvrants: String,

    /// Les caractères reconnus comme guillemets fermants.
    pub guillemets_fermants: String,
}

impl Default for Typographie {
    fn default() -> Self {
        let base = crate::langage::typographie::Regles::aucune();

        Self {
            base: "aucune".to_string(),
            avant_signes: base.avant_signes.nom().to_string(),
            apres_signes: base.apres_signes.nom().to_string(),
            apres_virgule_point: base.apres_virgule_point.nom().to_string(),
            espace_guillemets: base.espace_guillemets.nom().to_string(),
            guillemets: paires_en_texte(&base.guillemets),
            guillemets_ouvrants: "«".to_string(),
            guillemets_fermants: "»".to_string(),
        }
    }
}

/// Les mots de la balise `{date:...}` — siècles, millénaires, époques,
/// format des dates pleines. Personnalisation de la demande du
/// 2026-11-03 : « tout cela soit customisable dans le panneau des
/// normes ».
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Dates {
    /// Le mot du siècle (« siècle », « s. »…).
    pub siecle: String,
    /// Le mot du millénaire.
    pub millenaire: String,
    /// L'époque après (« ap. J.-C. », « EC »…).
    pub epoque_apres: String,
    /// L'époque avant (« av. J.-C. », « AEC »…).
    pub epoque_avant: String,
    /// Le format par défaut des dates pleines (« jj/mm/aaaa »).
    pub format: String,
    /// La langue des dates — noms de mois et de jours (bug-report-8,
    /// T3.2). Indépendante de la langue de l'interface.
    pub langue: String,
}

impl Default for Dates {
    fn default() -> Self {
        Self {
            siecle: "siècle".to_string(),
            millenaire: "millénaire".to_string(),
            epoque_apres: "ap. J.-C.".to_string(),
            epoque_avant: "av. J.-C.".to_string(),
            format: "dd/MM/yyyy".to_string(),
            langue: "fr".to_string(),
        }
    }
}

impl Typographie {
    /// Les règles de composition de ces normes.
    pub fn regles(&self, references: &References) -> crate::langage::typographie::Regles {
        crate::langage::typographie::Regles {
            avant_signes: crate::langage::typographie::Espace::depuis(&self.avant_signes),
            apres_signes: crate::langage::typographie::Espace::depuis(&self.apres_signes),
            apres_virgule_point: crate::langage::typographie::Espace::depuis(
                &self.apres_virgule_point,
            ),
            espace_guillemets: crate::langage::typographie::Espace::depuis(&self.espace_guillemets),
            guillemets: paires_depuis(&self.guillemets),
            guillemets_ouvrants: self.guillemets_ouvrants.chars().collect(),
            guillemets_fermants: self.guillemets_fermants.chars().collect(),
            numerotation_figures: String::new(),
            legende_prefixe: "Figure".to_string(),
            callouts: ParametresCallouts::default(),
            dates: crate::langage::dates::Regles::default(),
            references: references.clone(),
            index: crate::mise_en_page::modele::IndexCitations::default(),
        }
    }

    /// Reprend les règles d'un défaut de langue, en gardant la base nommée.
    pub fn charger_le_defaut(&mut self, nom: &str) {
        let regles = crate::langage::typographie::Regles::depuis(nom);

        self.base = nom.to_string();
        self.avant_signes = regles.avant_signes.nom().to_string();
        self.apres_signes = regles.apres_signes.nom().to_string();
        self.apres_virgule_point = regles.apres_virgule_point.nom().to_string();
        self.espace_guillemets = regles.espace_guillemets.nom().to_string();
        self.guillemets = paires_en_texte(&regles.guillemets);
        if let Some((ouvrant, fermant)) = regles.guillemets.first() {
            self.guillemets_ouvrants = ouvrant.to_string();
            self.guillemets_fermants = fermant.to_string();
        }
    }
}

/// Les paires de guillemets en deux caractères par champ.
fn paires_en_texte(paires: &[(char, char)]) -> Vec<String> {
    paires
        .iter()
        .map(|(ouvrant, fermant)| format!("{ouvrant}{fermant}"))
        .collect()
}

/// Les paires de guillemets écrites dans les normes.
fn paires_depuis(textes: &[String]) -> Vec<(char, char)> {
    let mut paires: Vec<(char, char)> = textes
        .iter()
        .filter_map(|texte| {
            let mut caracteres = texte.chars().filter(|c| !c.is_whitespace());
            Some((caracteres.next()?, caracteres.next()?))
        })
        .collect();

    if paires.is_empty() {
        paires.push(('«', '»'));
    }

    paires
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Couleurs {
    /// Le texte courant.
    pub texte: String,

    /// Les titres.
    pub titres: String,

    /// Les liens.
    pub liens: String,

    /// Le papier lui-même.
    pub fond: String,

    /// Le texte en gras, et en italique — vides : la couleur du texte.
    pub gras: String,
    pub italique: String,
}

impl Default for Couleurs {
    fn default() -> Self {
        Self {
            texte: "#000000".to_string(),
            titres: "#000000".to_string(),
            liens: "#0000ff".to_string(),
            gras: String::new(),
            italique: String::new(),
            fond: "#ffffff".to_string(),
        }
    }
}

/// La paire de guillemets d'un niveau, ou vide.
fn paire_lue(paires: &[String], niveau: usize) -> String {
    paires.get(niveau).cloned().unwrap_or_default()
}

/// Écrit la paire de guillemets d'un niveau.
fn ecrire_paire(paires: &mut Vec<String>, niveau: usize, valeur: String) {
    while paires.len() <= niveau {
        paires.push(String::new());
    }

    paires[niveau] = valeur;
}

impl MiseEnPage {
    /// Lit une mise en page écrite au format TOML.
    pub fn lire(texte: &str) -> Result<Self, String> {
        let mut mise_en_page: Self = toml::from_str(texte).map_err(|erreur| erreur.to_string())?;

        // Une norme écrite par la version précédente : la typographie vivait
        // dans `texte`. Elle porte dans sa section, sans quoi le réglage se
        // perdrait à la lecture — et un réglage ne se perd pas.
        if let Some(ancienne) = mise_en_page.texte.typographie.take()
            && mise_en_page.typographie.base == "aucune"
        {
            mise_en_page.typographie.charger_le_defaut(&ancienne);
        }

        Ok(mise_en_page)
    }

    /// L'écrit au format TOML — celui qui s'exporte et s'importe.
    ///
    /// Une sérialisation qui échoue le **dit** : un `String` vide écrit à sa
    /// place détruirait la valeur précédente du fichier.
    pub fn ecrire(&self) -> Result<String, String> {
        toml::to_string_pretty(self).map_err(|erreur| erreur.to_string())
    }

    /// Ce qui ne va pas, dit en une phrase par valeur : `[]` quand tout va.
    ///
    /// Une mise en page fausse n'écrit rien : elle dit ce qui l'est, champ par
    /// champ, et c'est le dialogue qui montre.
    pub fn problemes(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();

        let longueurs = [
            ("marge haute", &self.marges.haut),
            ("marge basse", &self.marges.bas),
            ("marge gauche", &self.marges.gauche),
            ("marge droite", &self.marges.droite),
            ("corps du texte", &self.texte.taille),
            ("interligne", &self.texte.interligne),
            ("espacement des paragraphes", &self.texte.espacement),
            ("retrait", &self.texte.retrait),
            ("titre 1", &self.titres.taille_1),
            ("titre 2", &self.titres.taille_2),
            ("titre 3", &self.titres.taille_3),
            ("titre 4", &self.titres.taille_4),
            ("titre 5", &self.titres.taille_5),
            ("titre 6", &self.titres.taille_6),
            ("espacement des titres", &self.titres.espacement),
        ];

        for (nom, valeur) in longueurs {
            if !longueur_valide(valeur) {
                out.push((
                    nom.to_string(),
                    format!("« {valeur} » n'est pas une longueur (ex. 2.5cm, 11pt, 0.65em)"),
                ));
            }
        }

        for (nom, valeur) in [
            ("couleur du texte", &self.couleurs.texte),
            ("couleur des titres", &self.couleurs.titres),
            ("couleur des liens", &self.couleurs.liens),
            ("couleur du fond", &self.couleurs.fond),
            ("couleur du gras", &self.couleurs.gras),
            ("couleur de l'italique", &self.couleurs.italique),
        ] {
            if !valeur.is_empty() && !couleur_valide(valeur) {
                out.push((
                    nom.to_string(),
                    format!("« {valeur} » n'est pas une couleur (ex. #1e1e2e)"),
                ));
            }
        }

        match self.titres.casse.as_str() {
            "normale" | "majuscules" | "minuscules" | "petites-majuscules" => {}
            autre => out.push((
                "casse des titres".to_string(),
                format!("« {autre} » n'est pas une casse connue"),
            )),
        }

        // Les réglages **par niveau** suivent exactement les mêmes règles
        // que les globaux : une édition cohérente par niveau, une
        // validation cohérente par niveau (R5). Rien ne s'ignore en silence.
        for niveau in 1..=6usize {
            let espacement = match niveau {
                1 => self.titres.espacement_1.as_deref(),
                2 => self.titres.espacement_2.as_deref(),
                3 => self.titres.espacement_3.as_deref(),
                4 => self.titres.espacement_4.as_deref(),
                5 => self.titres.espacement_5.as_deref(),
                _ => self.titres.espacement_6.as_deref(),
            };
            if let Some(valeur) = espacement
                && !longueur_valide(valeur)
            {
                out.push((
                    format!("espacement du titre {niveau}"),
                    format!("« {valeur} » n'est pas une longueur (ex. 2.5cm, 11pt, 0.65em)"),
                ));
            }

            let casse = match niveau {
                1 => self.titres.casse_1.as_deref(),
                2 => self.titres.casse_2.as_deref(),
                3 => self.titres.casse_3.as_deref(),
                4 => self.titres.casse_4.as_deref(),
                5 => self.titres.casse_5.as_deref(),
                _ => self.titres.casse_6.as_deref(),
            };
            if let Some(valeur) = casse
                && !matches!(
                    valeur,
                    "normale" | "majuscules" | "minuscules" | "petites-majuscules"
                )
            {
                out.push((
                    format!("casse du titre {niveau}"),
                    format!("« {valeur} » n'est pas une casse connue"),
                ));
            }

            let couleur = match niveau {
                1 => self.titres.couleur_1.as_deref(),
                2 => self.titres.couleur_2.as_deref(),
                3 => self.titres.couleur_3.as_deref(),
                4 => self.titres.couleur_4.as_deref(),
                5 => self.titres.couleur_5.as_deref(),
                _ => self.titres.couleur_6.as_deref(),
            };
            if let Some(valeur) = couleur.filter(|valeur| !valeur.is_empty())
                && !couleur_valide(valeur)
            {
                out.push((
                    format!("couleur du titre {niveau}"),
                    format!("« {valeur} » n'est pas une couleur (ex. #1e1e2e)"),
                ));
            }
        }

        // Les réglages embarqués des plugins : une valeur inconnue se dit
        // plutôt que de retomber en silence sur le défaut.
        if crate::langage::references::Traduction::depuis_code(
            &self.parametres_plugins.biblique.traduction,
        )
        .is_none()
        {
            out.push((
                "traduction biblique du fichier de normes".to_string(),
                format!(
                    "« {} » n'est pas une traduction installée",
                    self.parametres_plugins.biblique.traduction
                ),
            ));
        }
        if !matches!(
            self.parametres_plugins.biblique.tri.as_str(),
            "apparition" | "canonique"
        ) {
            out.push((
                "ordre biblique du fichier de normes".to_string(),
                format!(
                    "« {} » n'est pas un ordre connu (apparition, canonique)",
                    self.parametres_plugins.biblique.tri
                ),
            ));
        }

        // Les callouts prédéfinis : leurs couleurs et leurs mesures entrent
        // dans Typst, donc une valeur douteuse se dit avant l'application.
        if self.parametres_plugins.callouts.presets.is_empty() {
            out.push((
                "callouts prédéfinis".to_string(),
                "il faut au moins un préréglage".to_string(),
            ));
        }
        let mut cles = std::collections::HashSet::new();
        for (rang, preset) in self.parametres_plugins.callouts.presets.iter().enumerate() {
            if !cles.insert(preset.cle.as_str()) {
                out.push((
                    format!("callout prédéfini {}", rang + 1),
                    format!("la clé « {} » est utilisée deux fois", preset.cle),
                ));
            }
            for probleme in preset.problemes() {
                out.push((
                    format!(
                        "callout prédéfini « {} »",
                        if preset.nom.trim().is_empty() {
                            &preset.cle
                        } else {
                            &preset.nom
                        }
                    ),
                    probleme,
                ));
            }
        }

        // La typographie : une valeur inconnue se perdait en silence, et
        // la règle tombait en « aucune » sans le dire à personne.
        let espaces = [
            ("espace devant les signes", &self.typographie.avant_signes),
            ("espace après les signes", &self.typographie.apres_signes),
            (
                "espace après virgule et point",
                &self.typographie.apres_virgule_point,
            ),
            (
                "espace dans les guillemets",
                &self.typographie.espace_guillemets,
            ),
        ];

        for (nom, valeur) in espaces {
            if !["aucune", "normale", "insécable", "fine-insécable"].contains(&valeur.as_str()) {
                out.push((
                    nom.to_string(),
                    format!("« {valeur} » n'est pas une espace connue"),
                ));
            }
        }

        for (rang, paire) in self.typographie.guillemets.iter().enumerate() {
            if paire.chars().count() != 2 {
                out.push((
                    format!("guillemets, niveau {}", rang + 1),
                    format!("« {paire} » : deux caractères, ouvrant puis fermant"),
                ));
            }
        }

        out
    }
}

/// Une longueur Typst : un nombre, puis une unité.
///
/// Les unités sont celles de Typst — et `%`, qui s'entend du bloc parent.
pub fn longueur_valide(valeur: &str) -> bool {
    let valeur = valeur.trim();

    let Some(position) = valeur
        .find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-')
        .filter(|position| *position > 0)
    else {
        return false;
    };

    let (nombre, unite) = valeur.split_at(position);

    nombre.parse::<f64>().is_ok()
        && matches!(unite.trim(), "pt" | "mm" | "cm" | "in" | "em" | "fr" | "%")
}

/// Une couleur : `#rrggbb` ou `#rgb`.
pub fn couleur_valide(valeur: &str) -> bool {
    let Some(hexa) = valeur.trim().strip_prefix('#') else {
        return false;
    };

    (hexa.len() == 6 || hexa.len() == 3) && hexa.chars().all(|c| c.is_ascii_hexdigit())
}

/// Un champ de la mise en page, celui que la fenêtre gouverne.
///
/// Il y a une trentaine de réglages : les nommer par leur nom de champ est
/// plus court et plus sûr que trente messages, et un champ inconnu n'est pas
/// un accident possible — c'est le compilateur qui refuse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Champ {
    PapierFormat,
    PapierOrientation,
    MargeHaut,
    MargeBas,
    MargeGauche,
    MargeDroite,
    NumerotationActive,
    NumerotationForme,
    NumerotationPosition,
    NumerotationAlignement,
    EnteteGauche,
    EnteteCentre,
    EnteteDroite,
    PiedGauche,
    PiedCentre,
    PiedDroite,
    EntetePiedNumerotation,
    ImagesHauteur,
    NumerotationFigures,
    BiblioSautDePage,
    BiblioTitreUn,
    LegendePrefixe,
    LegendeStyle,
    PageDeTitre,
    Sommaire,
    DatesSiecle,
    DatesMillenaire,
    DatesEpoqueApres,
    DatesEpoqueAvant,
    DatesFormat,
    DatesLangue,
    TextePolice,
    TexteTaille,
    TexteInterligne,
    TexteEspacement,
    TexteRetrait,
    TexteJustification,
    TexteCoupure,
    TexteLangue,
    TypoBase,
    TypoAvantSignes,
    TypoApresSignes,
    TypoApresVirgulePoint,
    TypoEspaceGuillemets,
    TypoGuillemets1,
    TypoGuillemets2,
    TypoGuillemets3,
    TypoGuillemetsOuvrants,
    TypoGuillemetsFermants,
    DecorCouleur,
    DecorEpaisseur,
    DecorRayon,
    DecorMarge,
    DecorRemplissage,
    DecorSeparateurCouleur,
    DecorSeparateurEpaisseur,
    TitresTaille1,
    TitresTaille2,
    TitresTaille3,
    TitresTaille4,
    TitresTaille5,
    TitresTaille6,
    TitresGras,
    TitresCasse,
    TitresCasse1,
    TitresCasse2,
    TitresCasse3,
    TitresCasse4,
    TitresCasse5,
    TitresCasse6,
    TitresCouleur1,
    TitresCouleur2,
    TitresCouleur3,
    TitresCouleur4,
    TitresCouleur5,
    TitresCouleur6,
    TitresGras1,
    TitresGras2,
    TitresGras3,
    TitresGras4,
    TitresGras5,
    TitresGras6,
    TitresItalique1,
    TitresItalique2,
    TitresItalique3,
    TitresItalique4,
    TitresItalique5,
    TitresItalique6,
    TitresSouligne1,
    TitresSouligne2,
    TitresSouligne3,
    TitresSouligne4,
    TitresSouligne5,
    TitresSouligne6,
    TitresEspacement1,
    TitresEspacement2,
    TitresEspacement3,
    TitresEspacement4,
    TitresEspacement5,
    TitresEspacement6,
    TitresNumerotation,
    TitresEspacement,
    CouleurTexte,
    CouleurTitres,
    CouleurLiens,
    CouleurGras,
    CouleurItalique,
    CouleurFond,
    BibliographieStyle,
    ReferencesLivre,
    ReferencesChiffres,
    ReferencesOrdreCanonique,
    ReferencesSeparateurChapitreVerset,
    ReferencesSeparateurPlage,
    ReferencesSeparateurLivre,
    ReferencesSeparateurOccurrence,
    ReferencesSeparateurChapitres,
    ReferencesGabarit,
    IndexAfficherPages,
    IndexRegrouperParLivre,
    IndexForme,
    IndexColonnes,
    IndexCanon,
    /// La traduction biblique consultée par défaut (`lsg`, `darby` ou une
    /// version ajoutée).
    BibliqueTraduction,
    /// L'ordre de la liste biblique (`apparition`, `canonique`).
    BibliqueTri,
}

impl MiseEnPage {
    /// Ce qu'un champ porte, en mots de fenêtre.
    pub fn valeur(&self, champ: Champ) -> String {
        match champ {
            Champ::PapierFormat => self.papier.format.clone(),
            Champ::PapierOrientation => self.papier.orientation.clone(),
            Champ::MargeHaut => self.marges.haut.clone(),
            Champ::MargeBas => self.marges.bas.clone(),
            Champ::MargeGauche => self.marges.gauche.clone(),
            Champ::MargeDroite => self.marges.droite.clone(),
            Champ::NumerotationActive => self.numerotation.actif.to_string(),
            Champ::NumerotationForme => self.numerotation.forme.clone(),
            Champ::NumerotationPosition => self.numerotation.position.clone(),
            Champ::NumerotationAlignement => self.numerotation.alignement.clone(),
            Champ::EnteteGauche => self.entete_pied.entete.gauche.clone(),
            Champ::EnteteCentre => self.entete_pied.entete.centre.clone(),
            Champ::EnteteDroite => self.entete_pied.entete.droite.clone(),
            Champ::PiedGauche => self.entete_pied.pied.gauche.clone(),
            Champ::PiedCentre => self.entete_pied.pied.centre.clone(),
            Champ::PiedDroite => self.entete_pied.pied.droite.clone(),
            Champ::EntetePiedNumerotation => self.entete_pied.numerotation.clone(),
            Champ::ImagesHauteur => self.entete_pied.images_hauteur.clone(),
            Champ::NumerotationFigures => self.numerotation_figures.clone(),
            Champ::BiblioSautDePage => self.bibliographie.saut_de_page.to_string(),
            Champ::BiblioTitreUn => self.bibliographie.titre_un.to_string(),
            Champ::LegendePrefixe => self.legende_prefixe.clone(),
            Champ::LegendeStyle => self.legende_style.clone(),
            Champ::PageDeTitre => self.page_de_titre.to_string(),
            Champ::Sommaire => self.sommaire.to_string(),
            Champ::DatesSiecle => self.dates.siecle.clone(),
            Champ::DatesMillenaire => self.dates.millenaire.clone(),
            Champ::DatesEpoqueApres => self.dates.epoque_apres.clone(),
            Champ::DatesEpoqueAvant => self.dates.epoque_avant.clone(),
            Champ::DatesFormat => self.dates.format.clone(),
            Champ::DatesLangue => self.dates.langue.clone(),
            Champ::TextePolice => self.texte.police.clone(),
            Champ::TexteTaille => self.texte.taille.clone(),
            Champ::TexteInterligne => self.texte.interligne.clone(),
            Champ::TexteEspacement => self.texte.espacement.clone(),
            Champ::TexteRetrait => self.texte.retrait.clone(),
            Champ::TexteJustification => self.texte.justification.to_string(),
            Champ::TexteCoupure => self.texte.coupure.to_string(),
            Champ::TexteLangue => self.texte.langue.clone(),
            Champ::DecorCouleur => self.decor.couleur.clone(),
            Champ::DecorEpaisseur => self.decor.epaisseur.clone(),
            Champ::DecorRayon => self.decor.rayon.clone(),
            Champ::DecorMarge => self.decor.marge.clone(),
            Champ::DecorRemplissage => self.decor.remplissage.clone(),
            Champ::DecorSeparateurCouleur => self.decor.separateur_couleur.clone(),
            Champ::DecorSeparateurEpaisseur => self.decor.separateur_epaisseur.clone(),
            Champ::TypoBase => self.typographie.base.clone(),
            Champ::TypoAvantSignes => self.typographie.avant_signes.clone(),
            Champ::TypoApresSignes => self.typographie.apres_signes.clone(),
            Champ::TypoApresVirgulePoint => self.typographie.apres_virgule_point.clone(),
            Champ::TypoEspaceGuillemets => self.typographie.espace_guillemets.clone(),
            Champ::TypoGuillemets1 => paire_lue(&self.typographie.guillemets, 0),
            Champ::TypoGuillemets2 => paire_lue(&self.typographie.guillemets, 1),
            Champ::TypoGuillemets3 => paire_lue(&self.typographie.guillemets, 2),
            Champ::TypoGuillemetsOuvrants => self.typographie.guillemets_ouvrants.clone(),
            Champ::TypoGuillemetsFermants => self.typographie.guillemets_fermants.clone(),
            Champ::TitresTaille1 => self.titres.taille_1.clone(),
            Champ::TitresTaille2 => self.titres.taille_2.clone(),
            Champ::TitresTaille3 => self.titres.taille_3.clone(),
            Champ::TitresTaille4 => self.titres.taille_4.clone(),
            Champ::TitresTaille5 => self.titres.taille_5.clone(),
            Champ::TitresTaille6 => self.titres.taille_6.clone(),
            Champ::TitresGras => self.titres.gras.to_string(),
            Champ::TitresCasse => self.titres.casse.clone(),
            Champ::TitresCasse1 => self.titres.casse_1.clone().unwrap_or_default(),
            Champ::TitresCasse2 => self.titres.casse_2.clone().unwrap_or_default(),
            Champ::TitresCasse3 => self.titres.casse_3.clone().unwrap_or_default(),
            Champ::TitresCasse4 => self.titres.casse_4.clone().unwrap_or_default(),
            Champ::TitresCasse5 => self.titres.casse_5.clone().unwrap_or_default(),
            Champ::TitresCasse6 => self.titres.casse_6.clone().unwrap_or_default(),
            Champ::TitresCouleur1 => self.titres.couleur_1.clone().unwrap_or_default(),
            Champ::TitresCouleur2 => self.titres.couleur_2.clone().unwrap_or_default(),
            Champ::TitresCouleur3 => self.titres.couleur_3.clone().unwrap_or_default(),
            Champ::TitresCouleur4 => self.titres.couleur_4.clone().unwrap_or_default(),
            Champ::TitresCouleur5 => self.titres.couleur_5.clone().unwrap_or_default(),
            Champ::TitresCouleur6 => self.titres.couleur_6.clone().unwrap_or_default(),
            Champ::TitresGras1 => self
                .titres
                .gras_1
                .map(|v| v.to_string())
                .unwrap_or_default(),
            Champ::TitresGras2 => self
                .titres
                .gras_2
                .map(|v| v.to_string())
                .unwrap_or_default(),
            Champ::TitresGras3 => self
                .titres
                .gras_3
                .map(|v| v.to_string())
                .unwrap_or_default(),
            Champ::TitresGras4 => self
                .titres
                .gras_4
                .map(|v| v.to_string())
                .unwrap_or_default(),
            Champ::TitresGras5 => self
                .titres
                .gras_5
                .map(|v| v.to_string())
                .unwrap_or_default(),
            Champ::TitresGras6 => self
                .titres
                .gras_6
                .map(|v| v.to_string())
                .unwrap_or_default(),
            Champ::TitresItalique1 => self
                .titres
                .italique_1
                .map(|v| v.to_string())
                .unwrap_or_default(),
            Champ::TitresItalique2 => self
                .titres
                .italique_2
                .map(|v| v.to_string())
                .unwrap_or_default(),
            Champ::TitresItalique3 => self
                .titres
                .italique_3
                .map(|v| v.to_string())
                .unwrap_or_default(),
            Champ::TitresItalique4 => self
                .titres
                .italique_4
                .map(|v| v.to_string())
                .unwrap_or_default(),
            Champ::TitresItalique5 => self
                .titres
                .italique_5
                .map(|v| v.to_string())
                .unwrap_or_default(),
            Champ::TitresItalique6 => self
                .titres
                .italique_6
                .map(|v| v.to_string())
                .unwrap_or_default(),
            Champ::TitresSouligne1 => self
                .titres
                .souligne_1
                .map(|v| v.to_string())
                .unwrap_or_default(),
            Champ::TitresSouligne2 => self
                .titres
                .souligne_2
                .map(|v| v.to_string())
                .unwrap_or_default(),
            Champ::TitresSouligne3 => self
                .titres
                .souligne_3
                .map(|v| v.to_string())
                .unwrap_or_default(),
            Champ::TitresSouligne4 => self
                .titres
                .souligne_4
                .map(|v| v.to_string())
                .unwrap_or_default(),
            Champ::TitresSouligne5 => self
                .titres
                .souligne_5
                .map(|v| v.to_string())
                .unwrap_or_default(),
            Champ::TitresSouligne6 => self
                .titres
                .souligne_6
                .map(|v| v.to_string())
                .unwrap_or_default(),
            Champ::TitresEspacement1 => self.titres.espacement_1.clone().unwrap_or_default(),
            Champ::TitresEspacement2 => self.titres.espacement_2.clone().unwrap_or_default(),
            Champ::TitresEspacement3 => self.titres.espacement_3.clone().unwrap_or_default(),
            Champ::TitresEspacement4 => self.titres.espacement_4.clone().unwrap_or_default(),
            Champ::TitresEspacement5 => self.titres.espacement_5.clone().unwrap_or_default(),
            Champ::TitresEspacement6 => self.titres.espacement_6.clone().unwrap_or_default(),
            Champ::TitresNumerotation => self.titres.numerotation.clone(),
            Champ::TitresEspacement => self.titres.espacement.clone(),
            Champ::CouleurTexte => self.couleurs.texte.clone(),
            Champ::CouleurTitres => self.couleurs.titres.clone(),
            Champ::CouleurLiens => self.couleurs.liens.clone(),
            Champ::CouleurGras => self.couleurs.gras.clone(),
            Champ::CouleurItalique => self.couleurs.italique.clone(),
            Champ::CouleurFond => self.couleurs.fond.clone(),
            Champ::BibliographieStyle => self.bibliographie.style.clone().unwrap_or_default(),
            Champ::ReferencesLivre => match self.references.livre {
                SystemeAbreviation::FrancaiseCourte => "courte".to_string(),
                SystemeAbreviation::FrancaiseLongue => "longue".to_string(),
                SystemeAbreviation::Osis => "osis".to_string(),
                SystemeAbreviation::SansPoint => "sans_point".to_string(),
                SystemeAbreviation::Tob => "tob".to_string(),
            },
            Champ::ReferencesChiffres => match self.references.chiffres {
                Chiffres::Arabes => "arabes".to_string(),
                Chiffres::Romains => "romains".to_string(),
            },
            Champ::ReferencesOrdreCanonique => self.references.ordre_canonique.to_string(),
            Champ::ReferencesSeparateurChapitreVerset => {
                self.references.separateur_chapitre_verset.clone()
            }
            Champ::ReferencesSeparateurPlage => self.references.separateur_plage.clone(),
            Champ::ReferencesSeparateurLivre => self.references.separateur_livre.clone(),
            Champ::ReferencesSeparateurOccurrence => self.references.separateur_occurrence.clone(),
            Champ::ReferencesSeparateurChapitres => self.references.separateur_chapitres.clone(),
            Champ::ReferencesGabarit => self.references.gabarit.clone(),
            Champ::IndexAfficherPages => self.index.afficher_pages.to_string(),
            Champ::IndexRegrouperParLivre => self.index.regrouper_par_livre.to_string(),
            Champ::IndexForme => match self.index.forme {
                None => "herite".to_string(),
                Some(SystemeAbreviation::FrancaiseCourte) => "courte".to_string(),
                Some(SystemeAbreviation::FrancaiseLongue) => "longue".to_string(),
                Some(SystemeAbreviation::Osis) => "osis".to_string(),
                Some(SystemeAbreviation::SansPoint) => "sans_point".to_string(),
                Some(SystemeAbreviation::Tob) => "tob".to_string(),
            },
            Champ::IndexColonnes => self.index.colonnes.to_string(),
            Champ::IndexCanon => self.index.canon.code().to_string(),
            Champ::BibliqueTraduction => self.parametres_plugins.biblique.traduction.clone(),
            Champ::BibliqueTri => self.parametres_plugins.biblique.tri.clone(),
        }
    }

    /// Change ce qu'un champ porte.
    pub fn modifier(&mut self, champ: Champ, valeur: String) {
        match champ {
            Champ::PapierFormat => self.papier.format = valeur,
            Champ::PapierOrientation => self.papier.orientation = valeur,
            Champ::MargeHaut => self.marges.haut = valeur,
            Champ::MargeBas => self.marges.bas = valeur,
            Champ::MargeGauche => self.marges.gauche = valeur,
            Champ::MargeDroite => self.marges.droite = valeur,
            Champ::NumerotationActive => self.numerotation.actif = valeur == "true",
            Champ::NumerotationForme => self.numerotation.forme = valeur,
            Champ::NumerotationPosition => self.numerotation.position = valeur,
            Champ::NumerotationAlignement => self.numerotation.alignement = valeur,
            Champ::EnteteGauche => self.entete_pied.entete.gauche = valeur,
            Champ::EnteteCentre => self.entete_pied.entete.centre = valeur,
            Champ::EnteteDroite => self.entete_pied.entete.droite = valeur,
            Champ::PiedGauche => self.entete_pied.pied.gauche = valeur,
            Champ::PiedCentre => self.entete_pied.pied.centre = valeur,
            Champ::PiedDroite => self.entete_pied.pied.droite = valeur,
            Champ::EntetePiedNumerotation => self.entete_pied.numerotation = valeur,
            Champ::ImagesHauteur => self.entete_pied.images_hauteur = valeur,
            Champ::NumerotationFigures => self.numerotation_figures = valeur,
            Champ::BiblioSautDePage => self.bibliographie.saut_de_page = valeur == "true",
            Champ::BiblioTitreUn => self.bibliographie.titre_un = valeur == "true",
            Champ::LegendePrefixe => self.legende_prefixe = valeur,
            Champ::LegendeStyle => self.legende_style = valeur,
            Champ::PageDeTitre => {
                self.page_de_titre = matches!(valeur.trim(), "oui" | "true" | "1" | "vrai")
            }
            Champ::Sommaire => {
                self.sommaire = matches!(valeur.trim(), "oui" | "true" | "1" | "vrai")
            }
            Champ::DatesSiecle => self.dates.siecle = valeur,
            Champ::DatesMillenaire => self.dates.millenaire = valeur,
            Champ::DatesEpoqueApres => self.dates.epoque_apres = valeur,
            Champ::DatesEpoqueAvant => self.dates.epoque_avant = valeur,
            Champ::DatesFormat => self.dates.format = valeur,
            Champ::DatesLangue => self.dates.langue = valeur,
            Champ::TextePolice => self.texte.police = valeur,
            Champ::TexteTaille => self.texte.taille = valeur,
            Champ::TexteInterligne => self.texte.interligne = valeur,
            Champ::TexteEspacement => self.texte.espacement = valeur,
            Champ::TexteRetrait => self.texte.retrait = valeur,
            Champ::TexteJustification => self.texte.justification = valeur == "true",
            Champ::TexteCoupure => self.texte.coupure = valeur == "true",
            Champ::TexteLangue => self.texte.langue = valeur,
            Champ::DecorCouleur => self.decor.couleur = valeur,
            Champ::DecorEpaisseur => self.decor.epaisseur = valeur,
            Champ::DecorRayon => self.decor.rayon = valeur,
            Champ::DecorMarge => self.decor.marge = valeur,
            Champ::DecorRemplissage => self.decor.remplissage = valeur,
            Champ::DecorSeparateurCouleur => self.decor.separateur_couleur = valeur,
            Champ::DecorSeparateurEpaisseur => self.decor.separateur_epaisseur = valeur,
            Champ::TypoBase => self.typographie.base = valeur,
            Champ::TypoAvantSignes => self.typographie.avant_signes = valeur,
            Champ::TypoApresSignes => self.typographie.apres_signes = valeur,
            Champ::TypoApresVirgulePoint => self.typographie.apres_virgule_point = valeur,
            Champ::TypoEspaceGuillemets => self.typographie.espace_guillemets = valeur,
            Champ::TypoGuillemets1 => ecrire_paire(&mut self.typographie.guillemets, 0, valeur),
            Champ::TypoGuillemets2 => ecrire_paire(&mut self.typographie.guillemets, 1, valeur),
            Champ::TypoGuillemets3 => ecrire_paire(&mut self.typographie.guillemets, 2, valeur),
            Champ::TypoGuillemetsOuvrants => self.typographie.guillemets_ouvrants = valeur,
            Champ::TypoGuillemetsFermants => self.typographie.guillemets_fermants = valeur,
            Champ::TitresTaille1 => self.titres.taille_1 = valeur,
            Champ::TitresTaille2 => self.titres.taille_2 = valeur,
            Champ::TitresTaille3 => self.titres.taille_3 = valeur,
            Champ::TitresTaille4 => self.titres.taille_4 = valeur,
            Champ::TitresTaille5 => self.titres.taille_5 = valeur,
            Champ::TitresTaille6 => self.titres.taille_6 = valeur,
            Champ::TitresGras => self.titres.gras = valeur == "true",
            Champ::TitresCasse => self.titres.casse = valeur,
            Champ::TitresCasse1 => self.titres.casse_1 = (!valeur.is_empty()).then_some(valeur),
            Champ::TitresCasse2 => self.titres.casse_2 = (!valeur.is_empty()).then_some(valeur),
            Champ::TitresCasse3 => self.titres.casse_3 = (!valeur.is_empty()).then_some(valeur),
            Champ::TitresCasse4 => self.titres.casse_4 = (!valeur.is_empty()).then_some(valeur),
            Champ::TitresCasse5 => self.titres.casse_5 = (!valeur.is_empty()).then_some(valeur),
            Champ::TitresCasse6 => self.titres.casse_6 = (!valeur.is_empty()).then_some(valeur),
            Champ::TitresCouleur1 => self.titres.couleur_1 = (!valeur.is_empty()).then_some(valeur),
            Champ::TitresCouleur2 => self.titres.couleur_2 = (!valeur.is_empty()).then_some(valeur),
            Champ::TitresCouleur3 => self.titres.couleur_3 = (!valeur.is_empty()).then_some(valeur),
            Champ::TitresCouleur4 => self.titres.couleur_4 = (!valeur.is_empty()).then_some(valeur),
            Champ::TitresCouleur5 => self.titres.couleur_5 = (!valeur.is_empty()).then_some(valeur),
            Champ::TitresCouleur6 => self.titres.couleur_6 = (!valeur.is_empty()).then_some(valeur),
            Champ::TitresGras1 => {
                self.titres.gras_1 = (!valeur.is_empty()).then_some(valeur == "true")
            }
            Champ::TitresGras2 => {
                self.titres.gras_2 = (!valeur.is_empty()).then_some(valeur == "true")
            }
            Champ::TitresGras3 => {
                self.titres.gras_3 = (!valeur.is_empty()).then_some(valeur == "true")
            }
            Champ::TitresGras4 => {
                self.titres.gras_4 = (!valeur.is_empty()).then_some(valeur == "true")
            }
            Champ::TitresGras5 => {
                self.titres.gras_5 = (!valeur.is_empty()).then_some(valeur == "true")
            }
            Champ::TitresGras6 => {
                self.titres.gras_6 = (!valeur.is_empty()).then_some(valeur == "true")
            }
            Champ::TitresItalique1 => {
                self.titres.italique_1 = (!valeur.is_empty()).then_some(valeur == "true")
            }
            Champ::TitresItalique2 => {
                self.titres.italique_2 = (!valeur.is_empty()).then_some(valeur == "true")
            }
            Champ::TitresItalique3 => {
                self.titres.italique_3 = (!valeur.is_empty()).then_some(valeur == "true")
            }
            Champ::TitresItalique4 => {
                self.titres.italique_4 = (!valeur.is_empty()).then_some(valeur == "true")
            }
            Champ::TitresItalique5 => {
                self.titres.italique_5 = (!valeur.is_empty()).then_some(valeur == "true")
            }
            Champ::TitresItalique6 => {
                self.titres.italique_6 = (!valeur.is_empty()).then_some(valeur == "true")
            }
            Champ::TitresSouligne1 => {
                self.titres.souligne_1 = (!valeur.is_empty()).then_some(valeur == "true")
            }
            Champ::TitresSouligne2 => {
                self.titres.souligne_2 = (!valeur.is_empty()).then_some(valeur == "true")
            }
            Champ::TitresSouligne3 => {
                self.titres.souligne_3 = (!valeur.is_empty()).then_some(valeur == "true")
            }
            Champ::TitresSouligne4 => {
                self.titres.souligne_4 = (!valeur.is_empty()).then_some(valeur == "true")
            }
            Champ::TitresSouligne5 => {
                self.titres.souligne_5 = (!valeur.is_empty()).then_some(valeur == "true")
            }
            Champ::TitresSouligne6 => {
                self.titres.souligne_6 = (!valeur.is_empty()).then_some(valeur == "true")
            }
            Champ::TitresEspacement1 => {
                self.titres.espacement_1 = (!valeur.is_empty()).then_some(valeur)
            }
            Champ::TitresEspacement2 => {
                self.titres.espacement_2 = (!valeur.is_empty()).then_some(valeur)
            }
            Champ::TitresEspacement3 => {
                self.titres.espacement_3 = (!valeur.is_empty()).then_some(valeur)
            }
            Champ::TitresEspacement4 => {
                self.titres.espacement_4 = (!valeur.is_empty()).then_some(valeur)
            }
            Champ::TitresEspacement5 => {
                self.titres.espacement_5 = (!valeur.is_empty()).then_some(valeur)
            }
            Champ::TitresEspacement6 => {
                self.titres.espacement_6 = (!valeur.is_empty()).then_some(valeur)
            }
            Champ::TitresNumerotation => self.titres.numerotation = valeur,
            Champ::TitresEspacement => self.titres.espacement = valeur,
            Champ::CouleurTexte => self.couleurs.texte = valeur,
            Champ::CouleurTitres => self.couleurs.titres = valeur,
            Champ::CouleurLiens => self.couleurs.liens = valeur,
            Champ::CouleurGras => self.couleurs.gras = valeur,
            Champ::CouleurItalique => self.couleurs.italique = valeur,
            Champ::CouleurFond => self.couleurs.fond = valeur,
            Champ::BibliographieStyle => {
                self.bibliographie.style = (!valeur.is_empty()).then_some(valeur)
            }
            Champ::ReferencesLivre => {
                self.references.livre = match valeur.as_str() {
                    "longue" => SystemeAbreviation::FrancaiseLongue,
                    "osis" => SystemeAbreviation::Osis,
                    "sans_point" => SystemeAbreviation::SansPoint,
                    "tob" => SystemeAbreviation::Tob,
                    _ => SystemeAbreviation::FrancaiseCourte,
                }
            }
            Champ::ReferencesChiffres => {
                self.references.chiffres = match valeur.as_str() {
                    "romains" => Chiffres::Romains,
                    _ => Chiffres::Arabes,
                }
            }
            Champ::ReferencesOrdreCanonique => self.references.ordre_canonique = valeur == "true",
            Champ::ReferencesSeparateurChapitreVerset => {
                self.references.separateur_chapitre_verset = valeur
            }
            Champ::ReferencesSeparateurPlage => self.references.separateur_plage = valeur,
            Champ::ReferencesSeparateurLivre => self.references.separateur_livre = valeur,
            Champ::ReferencesSeparateurOccurrence => self.references.separateur_occurrence = valeur,
            Champ::ReferencesSeparateurChapitres => self.references.separateur_chapitres = valeur,
            Champ::ReferencesGabarit => self.references.gabarit = valeur,
            Champ::IndexAfficherPages => self.index.afficher_pages = valeur == "true",
            Champ::IndexRegrouperParLivre => self.index.regrouper_par_livre = valeur == "true",
            Champ::IndexForme => {
                self.index.forme = match valeur.as_str() {
                    "longue" => Some(SystemeAbreviation::FrancaiseLongue),
                    "courte" => Some(SystemeAbreviation::FrancaiseCourte),
                    "osis" => Some(SystemeAbreviation::Osis),
                    "sans_point" => Some(SystemeAbreviation::SansPoint),
                    "tob" => Some(SystemeAbreviation::Tob),
                    _ => None,
                }
            }
            Champ::IndexColonnes => {
                self.index.colonnes = valeur.trim().parse::<usize>().unwrap_or(1).clamp(1, 4);
            }
            Champ::IndexCanon => {
                self.index.canon = match valeur.as_str() {
                    "protestant" => Canon::Protestant,
                    "catholique" => Canon::Catholique,
                    "orthodoxe" => Canon::Orthodoxe,
                    "tewahedo" => Canon::Tewahedo,
                    _ => Canon::Tout,
                }
            }
            Champ::BibliqueTraduction => self.parametres_plugins.biblique.traduction = valeur,
            Champ::BibliqueTri => self.parametres_plugins.biblique.tri = valeur,
        }
    }

    /// Inverse une case à cocher.
    pub fn basculer(&mut self, champ: Champ) {
        let actuel = self.valeur(champ) == "true";
        self.modifier(champ, (!actuel).to_string());
    }

    /// Les champs de mesure : un nombre nu se lit en `em`, et rien de
    /// négatif ne s'y pose (demande du 2026-11-04).
    pub fn est_mesure(champ: Champ) -> bool {
        matches!(
            champ,
            Champ::MargeHaut
                | Champ::MargeBas
                | Champ::MargeGauche
                | Champ::MargeDroite
                | Champ::TexteTaille
                | Champ::TexteInterligne
                | Champ::TexteEspacement
                | Champ::TexteRetrait
                | Champ::DecorEpaisseur
                | Champ::DecorRayon
                | Champ::DecorMarge
                | Champ::TitresEspacement
                | Champ::TitresEspacement1
                | Champ::TitresEspacement2
                | Champ::TitresEspacement3
                | Champ::TitresEspacement4
                | Champ::TitresEspacement5
                | Champ::TitresEspacement6
                | Champ::TitresTaille1
                | Champ::TitresTaille2
                | Champ::TitresTaille3
                | Champ::TitresTaille4
                | Champ::TitresTaille5
                | Champ::TitresTaille6
        )
    }
}

/// L'aperçu des guillemets : l'imbrication réelle des trois niveaux
/// configurés (bug-report-8, T5.2) — « un « deux « trois » suite » fin ».
pub fn apercu_guillemets(guillemets: &[String]) -> String {
    /// Les deux caractères d'une paire : l'ouvrant, puis le fermant — les
    /// défauts français servent de secours.
    fn paire(paires: &[String], niveau: usize, defaut: (char, char)) -> (char, char) {
        let paire = paires.get(niveau).cloned().unwrap_or_default();
        let mut caracteres = paire.chars();
        (
            caracteres.next().unwrap_or(defaut.0),
            caracteres.next().unwrap_or(defaut.1),
        )
    }

    let (o1, f1) = paire(guillemets, 0, ('«', '»'));
    let (o2, f2) = paire(guillemets, 1, ('"', '"'));
    let (o3, f3) = paire(guillemets, 2, ('\'', '\''));

    format!("{o1} un {o2}deux {o3}trois{f3} suite{f2} fin {f1}")
}

/// L'aperçu d'un format de numérotation personnalisé (bug-report-8,
/// T2.3) : « 1.1 » montre « 1. Introduction — 1.2 Une partie — 1.2.3 Un
/// détail », un échantillon par niveau.
///
/// La syntaxe est celle de Typst : `1` chiffre arabe, `a`/`A` lettre,
/// `i`/`I` romain — tout le reste s'écrit tel quel (points, parenthèses,
/// tirets).
pub fn apercu_numerotation(format: &str) -> String {
    /// Un nombre, dans la forme du jeton.
    fn chiffre(jeton: char, n: usize) -> String {
        match jeton {
            'a' | 'A' => {
                let mut reste = n;
                let mut out = String::new();
                while reste > 0 {
                    reste -= 1;
                    out.insert(0, (b'a' + (reste % 26) as u8) as char);
                    reste /= 26;
                }
                if jeton == 'A' {
                    out.to_uppercase()
                } else {
                    out
                }
            }
            'i' | 'I' => {
                let romains = [
                    (1000, "m"),
                    (900, "cm"),
                    (500, "d"),
                    (400, "cd"),
                    (100, "c"),
                    (90, "xc"),
                    (50, "l"),
                    (40, "xl"),
                    (10, "x"),
                    (9, "ix"),
                    (5, "v"),
                    (4, "iv"),
                    (1, "i"),
                ];
                let mut reste = n;
                let mut out = String::new();
                for (valeur, symbole) in romains {
                    while reste >= valeur {
                        out.push_str(symbole);
                        reste -= valeur;
                    }
                }
                if jeton == 'I' {
                    out.to_uppercase()
                } else {
                    out
                }
            }
            _ => n.to_string(),
        }
    }

    // Le motif : des jetons de numérotation (1 a A i I) et du texte. Le
    // texte qui précède un jeton est son littéral, le texte final reste
    // le suffixe — la règle est sondée au vrai Typst, pas devinée.
    let mut paires: Vec<(String, char)> = Vec::new();
    let mut litteral = String::new();
    for caractere in format.chars() {
        if "1aAiI".contains(caractere) {
            paires.push((std::mem::take(&mut litteral), caractere));
        } else {
            litteral.push(caractere);
        }
    }
    if paires.is_empty() {
        return String::new();
    }
    let suffixe = litteral;

    // Le séparateur entre deux nombres : le texte du deuxième jeton, ou
    // le suffixe quand le motif n'a qu'un jeton (« 1) » donne « 1)2) »).
    let separateur = if paires.len() >= 2 {
        paires[1].0.clone()
    } else {
        suffixe.clone()
    };

    let echantillons: Vec<String> = (1..=3)
        .map(|niveau| {
            let mut out = paires[0].0.clone();
            for rang in 0..niveau {
                if rang > 0 {
                    out.push_str(&separateur);
                }
                // Le dernier jeton se répète au-delà du motif.
                let jeton = paires[rang.min(paires.len() - 1)].1;
                out.push_str(&chiffre(jeton, rang + 1));
            }
            out.push_str(&suffixe);
            out
        })
        .collect();

    echantillons.join(" — ")
}

/// Une mesure saisie : un nombre nu se lit en `em`, et rien de négatif ne
/// passe — `None` refuse la valeur.
pub fn normaliser_mesure(valeur: &str) -> Option<String> {
    let valeur = valeur.trim();
    if valeur.starts_with('-') {
        return None;
    }
    if valeur.is_empty() {
        return Some(String::new());
    }
    let nombre = valeur
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .count();
    if nombre == valeur.len() {
        return Some(format!("{valeur}em"));
    }
    Some(valeur.to_string())
}

#[cfg(test)]
mod tests {

    use super::normaliser_mesure;

    /// Une mesure : un nombre nu devient des `em`, l'unité se garde, et
    /// le négatif est refusé.
    #[test]
    fn une_mesure_se_lit_en_em_et_refuse_le_negatif() {
        assert_eq!(normaliser_mesure("12").as_deref(), Some("12em"));
        assert_eq!(normaliser_mesure("0.65").as_deref(), Some("0.65em"));
        assert_eq!(normaliser_mesure("12pt").as_deref(), Some("12pt"));
        assert_eq!(normaliser_mesure(" 2cm ").as_deref(), Some("2cm"));
        assert_eq!(normaliser_mesure("").as_deref(), Some(""));
        assert_eq!(normaliser_mesure("-1"), None, "pas de négatif");
        assert_eq!(normaliser_mesure("-0.5em"), None, "pas de négatif");
    }

    /// L'aperçu des guillemets montre les **trois** niveaux imbriqués,
    /// chacun dans sa propre paire (bug-report-8, T5.2).
    #[test]
    fn l_apercu_des_guillemets_imbrique_trois_niveaux() {
        use super::apercu_guillemets;

        // Le français, tel que la table du rapport le décrit.
        let paires = vec!["«»".to_string(), "“”".to_string(), "‘’".to_string()];
        assert_eq!(
            apercu_guillemets(&paires),
            "« un “deux ‘trois’ suite” fin »"
        );

        // Un niveau non réglé garde ses défauts — jamais un aperçu vide.
        assert_eq!(apercu_guillemets(&[]), "« un \"deux 'trois' suite\" fin »");
    }

    /// L'aperçu du format de numérotation : les jetons donnent des
    /// nombres, le reste s'écrit tel quel (bug-report-8, T2.3).
    #[test]
    fn un_format_de_numerotation_se_prevoit() {
        use super::apercu_numerotation;

        assert_eq!(apercu_numerotation("1.1"), "1 — 1.2 — 1.2.3");
        assert_eq!(apercu_numerotation("A.a"), "A — A.b — A.b.c");
        assert_eq!(apercu_numerotation("I.i"), "I — I.ii — I.ii.iii");
        assert_eq!(apercu_numerotation("1)"), "1) — 1)2) — 1)2)3)");
        assert_eq!(apercu_numerotation("(1.1)"), "(1) — (1.2) — (1.2.3)");
    }

    /// **La persistance (bug-report-8, T8.3)** : chaque paramètre neuf
    /// s'écrit, se relit, et un ancien document — qui les ignore — reçoit
    /// leurs défauts sans rien perdre.
    #[test]
    fn les_reglages_neufs_se_persistent_et_les_anciens_chargent() {
        let mut mise = MiseEnPage::default();
        mise.entete_pied.images_hauteur = "60".to_string();
        mise.dates.langue = "de".to_string();
        mise.titres.italique_2 = Some(true);
        mise.titres.souligne_3 = Some(true);
        mise.titres.couleur_1 = Some("#123456".to_string());

        // L'aller-retour par le fichier.
        let texte = mise.ecrire().expect("écriture");
        let relu = MiseEnPage::lire(&texte).expect("lecture");
        assert_eq!(relu.entete_pied.images_hauteur, "60");
        assert_eq!(relu.dates.langue, "de");
        assert_eq!(relu.titres.italique_2, Some(true));
        assert_eq!(relu.titres.souligne_3, Some(true));
        assert_eq!(relu.titres.couleur_1.as_deref(), Some("#123456"));

        // Un ancien fichier, sans aucun de ces champs : les défauts
        // viennent, et rien ne casse.
        let ancien = "page_de_titre = true\n";
        let relu = MiseEnPage::lire(ancien).expect("lecture de l'ancien");
        assert!(relu.page_de_titre, "l'ancien réglage se lit");
        assert_eq!(relu.entete_pied.images_hauteur, "80", "le défaut vient");
        assert_eq!(relu.dates.langue, "fr", "le défaut vient");
        assert_eq!(relu.titres.italique_2, None, "rien de forcé");
        assert_eq!(relu.titres.souligne_3, None, "rien de forcé");
    }

    /// La typographie d'avant la section dédiée se porte dans sa section :
    /// un réglage ne se perd pas à la lecture.
    #[test]
    fn la_typographie_heritee_se_porte_dans_sa_section() {
        let ancien = "[texte]\nlangue = \"fr\"\ntypographie = \"française\"\n";
        let mise_en_page = MiseEnPage::lire(ancien).expect("lisible");

        assert_eq!(mise_en_page.typographie.base, "française");
        assert_eq!(mise_en_page.typographie.avant_signes, "insécable");
        assert!(
            !mise_en_page
                .ecrire()
                .expect("sérialisé")
                .contains("typographie ="),
            "le champ hérité ne se réécrit pas : {}",
            mise_en_page.ecrire().expect("sérialisé")
        );
    }

    /// Le défaut de langue charge ses règles, qu'on reprend ensuite.
    #[test]
    fn un_defaut_charge_ses_regles_et_elles_se_prennent_une_par_une() {
        let mut typo = Typographie::default();
        typo.charger_le_defaut("française");

        assert_eq!(typo.avant_signes, "insécable");
        assert_eq!(typo.guillemets[0], "«»");

        // Une reprise, sans toucher au reste.
        typo.guillemets[0] = "“”".to_string();
        let regles = typo.regles(&References::default());
        assert_eq!(regles.guillemets[0], ('“', '”'));
        assert_eq!(
            regles.avant_signes,
            crate::langage::typographie::Espace::Insécable
        );

        // L'anglais recharge ses règles.
        typo.charger_le_defaut("anglaise");
        assert_eq!(typo.avant_signes, "aucune");
        assert_eq!(typo.guillemets[0], "\"\"");
    }

    /// Les champs de typographie se lisent et s'écrivent.
    #[test]
    fn les_champs_de_typographie_se_prennent() {
        let mut mise_en_page = MiseEnPage::default();

        assert_eq!(
            mise_en_page.valeur(Champ::TypoGuillemets1),
            mise_en_page.typographie.guillemets[0]
        );

        mise_en_page.modifier(Champ::TypoApresVirgulePoint, "insécable".to_string());
        assert_eq!(mise_en_page.typographie.apres_virgule_point, "insécable");

        mise_en_page.modifier(Champ::TypoGuillemets3, "‹›".to_string());
        assert_eq!(mise_en_page.valeur(Champ::TypoGuillemets3), "‹›");
    }

    use super::*;

    /// Une mise en page se lit, se perd, se retrouve.
    #[test]
    fn a_layout_survives_the_round_trip() {
        let mut mise_en_page = MiseEnPage::default();
        mise_en_page.papier.format = "a5".to_string();
        mise_en_page.marges.gauche = "3cm".to_string();
        mise_en_page.titres.casse = "majuscules".to_string();
        mise_en_page.couleurs.texte = "#1e1e2e".to_string();

        let texte = mise_en_page.ecrire().expect("sérialisé");
        let relu = MiseEnPage::lire(&texte).expect("lisible");

        assert_eq!(relu, mise_en_page, "rien ne se perd en chemin");
    }

    /// Un fichier partiel ne casse rien : ce qui manque garde sa valeur.
    #[test]
    fn a_partial_file_keeps_the_defaults() {
        let mise_en_page = MiseEnPage::lire("[papier]\nformat = \"a5\"\n").expect("lisible");

        assert_eq!(mise_en_page.papier.format, "a5");
        assert_eq!(
            mise_en_page.marges.haut, "2cm",
            "ce qui manque parle tout seul"
        );
    }

    /// Ce qui ne va pas se dit, champ par champ.
    #[test]
    fn problems_name_the_field_they_are_about() {
        let mut mise_en_page = MiseEnPage::default();
        mise_en_page.marges.gauche = "trois centimètres".to_string();
        mise_en_page.couleurs.texte = "rouge sombre".to_string();

        let problemes = mise_en_page.problemes();
        let noms: Vec<&str> = problemes.iter().map(|(nom, _)| nom.as_str()).collect();

        assert_eq!(noms, ["marge gauche", "couleur du texte"]);
    }

    /// Les réglages **par niveau** sont validés comme les globaux : une
    /// édition cohérente par niveau, une validation cohérente par niveau.
    #[test]
    fn les_reglages_par_niveau_sont_valides_comme_les_globaux() {
        let mut mise_en_page = MiseEnPage::default();
        mise_en_page.titres.espacement_1 = Some("beaucoup".to_string());
        mise_en_page.titres.casse_2 = Some("Majuscules".to_string());
        mise_en_page.titres.couleur_3 = Some("rouge".to_string());

        let problemes = mise_en_page.problemes();
        let noms: Vec<&str> = problemes.iter().map(|(nom, _)| nom.as_str()).collect();

        assert_eq!(
            noms,
            [
                "espacement du titre 1",
                "casse du titre 2",
                "couleur du titre 3"
            ]
        );
    }
}

#[cfg(test)]
mod tests_des_fonctionnalites {
    use super::*;

    /// Un projet naît avec toutes ses fonctionnalités (demande de
    /// l'auteur : le défaut est le confort).
    #[test]
    fn les_fonctionnalites_naissent_branches() {
        let normes = MiseEnPage::default();

        assert!(normes.fonctionnalites.bibliographie);
        assert!(normes.fonctionnalites.glossaire);
        assert!(normes.fonctionnalites.graphiques);
        assert!(normes.fonctionnalites.callouts);
    }

    /// Une mise en page d'avant, sans le bloc, les porte toutes.
    #[test]
    fn un_ancien_fichier_garde_ses_fonctionnalites() {
        let normes: MiseEnPage = toml::from_str("sommaire = true").unwrap();

        assert!(normes.fonctionnalites.bibliographie);
        assert!(normes.fonctionnalites.glossaire);
        assert!(normes.fonctionnalites.graphiques);
        assert!(normes.fonctionnalites.callouts);
    }

    /// Un fichier qui connaît la section mais pas le nouveau champ le reçoit
    /// actif, sans erreur de lecture.
    #[test]
    fn un_ancien_bloc_de_fonctionnalites_recoit_les_callouts() {
        let normes: MiseEnPage = toml::from_str(
            "[fonctionnalites]\nbibliographie = false\nglossaire = true\ngraphiques = true\nreferences_bibliques = true",
        )
        .unwrap();

        assert!(!normes.fonctionnalites.bibliographie);
        assert!(normes.fonctionnalites.callouts);
    }

    /// Le fichier de normes porte aussi le paramétrage des plugins : la
    /// traduction et l'ordre bibliques se relisent tels quels, et un ancien
    /// fichier reçoit les défauts sans que rien ne manque.
    #[test]
    fn les_parametres_des_plugins_voyagent_avec_la_norme() {
        let mut normes = MiseEnPage::default();
        normes.parametres_plugins.biblique.traduction = "darby".to_string();
        normes.parametres_plugins.biblique.tri = "canonique".to_string();
        normes.fonctionnalites.glossaire = false;

        let texte = normes.ecrire().expect("la norme s'écrit");
        assert!(
            texte.contains("parametres_plugins"),
            "la section des plugins est dans le fichier :\n{texte}"
        );
        let relu = MiseEnPage::lire(&texte).expect("la norme se relit");
        assert_eq!(relu, normes, "rien ne se perd en chemin");

        let ancien = MiseEnPage::lire("sommaire = true").expect("l'ancien se lit");
        assert_eq!(ancien.parametres_plugins, ParametresPlugins::default());
        assert!(normes.problemes().is_empty());
    }

    /// La forme des livres, les chiffres et l'ordre canonique se règlent par
    /// les champs du plugin biblique et survivent à l'écriture.
    #[test]
    fn la_forme_des_livres_se_regle_et_se_relit() {
        let mut normes = MiseEnPage::default();
        normes.modifier(Champ::ReferencesLivre, "longue".to_string());
        normes.modifier(Champ::ReferencesChiffres, "romains".to_string());
        normes.modifier(Champ::ReferencesOrdreCanonique, "true".to_string());
        normes.modifier(Champ::ReferencesSeparateurChapitreVerset, ", ".to_string());
        normes.modifier(Champ::ReferencesSeparateurOccurrence, " · ".to_string());
        normes.modifier(
            Champ::ReferencesGabarit,
            "{livre} {chapitre}, {verset}".to_string(),
        );
        normes.modifier(Champ::IndexAfficherPages, "true".to_string());
        normes.modifier(Champ::IndexRegrouperParLivre, "true".to_string());
        normes.modifier(Champ::IndexForme, "longue".to_string());
        normes.modifier(Champ::IndexColonnes, "3".to_string());

        assert_eq!(normes.valeur(Champ::ReferencesLivre), "longue");
        assert_eq!(normes.valeur(Champ::ReferencesChiffres), "romains");
        assert_eq!(normes.valeur(Champ::ReferencesOrdreCanonique), "true");
        assert_eq!(
            normes.valeur(Champ::ReferencesSeparateurChapitreVerset),
            ", "
        );
        assert!(normes.index.afficher_pages);
        assert!(normes.index.regrouper_par_livre);
        assert_eq!(
            normes.index.forme,
            Some(SystemeAbreviation::FrancaiseLongue)
        );
        assert_eq!(normes.index.colonnes, 3);

        let relu = MiseEnPage::lire(&normes.ecrire().expect("sérialisé")).expect("relu");
        assert_eq!(
            relu.references.livre,
            SystemeAbreviation::FrancaiseLongue,
            "la forme choisie ne se perd pas"
        );
        assert_eq!(relu.references.chiffres, Chiffres::Romains);
        assert!(
            relu.references.ordre_canonique,
            "l'ordre canonique se retient"
        );
        assert_eq!(relu.references.separateur_chapitre_verset, ", ");
        assert_eq!(relu.references.separateur_occurrence, " · ");
        assert_eq!(relu.references.gabarit, "{livre} {chapitre}, {verset}");
        assert!(relu.index.afficher_pages);
        assert!(relu.index.regrouper_par_livre);
        assert_eq!(relu.index.forme, Some(SystemeAbreviation::FrancaiseLongue));
        assert_eq!(relu.index.colonnes, 3, "le nombre de colonnes se retient");
    }
}
