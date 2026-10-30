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

    /// Le texte courant.
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

/// Le texte courant.
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
        }
    }
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
    /// La police des titres — vide : celle du texte.
    pub police: String,

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

    /// La numérotation des chapitres : `1.1`, `A.1`, ou vide pour aucune.
    pub numerotation: String,

    /// L'espace au-dessus et en dessous d'un titre.
    pub espacement: String,
}

impl Default for Titres {
    fn default() -> Self {
        Self {
            police: String::new(),
            taille_1: "16pt".to_string(),
            taille_2: "14pt".to_string(),
            taille_3: "12pt".to_string(),
            taille_4: "11pt".to_string(),
            taille_5: "10pt".to_string(),
            taille_6: "10pt".to_string(),
            gras: true,
            casse: "normale".to_string(),
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
        }
    }
}

impl Typographie {
    /// Les règles de composition de ces normes.
    pub fn regles(&self) -> crate::langage::typographie::Regles {
        crate::langage::typographie::Regles {
            avant_signes: crate::langage::typographie::Espace::depuis(&self.avant_signes),
            apres_signes: crate::langage::typographie::Espace::depuis(&self.apres_signes),
            apres_virgule_point: crate::langage::typographie::Espace::depuis(
                &self.apres_virgule_point,
            ),
            espace_guillemets: crate::langage::typographie::Espace::depuis(&self.espace_guillemets),
            guillemets: paires_depuis(&self.guillemets),
            numerotation_figures: String::new(),
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
    NumerotationFigures,
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
    DecorCouleur,
    DecorEpaisseur,
    DecorRayon,
    DecorMarge,
    DecorRemplissage,
    DecorSeparateurCouleur,
    DecorSeparateurEpaisseur,
    TitresPolice,
    TitresTaille1,
    TitresTaille2,
    TitresTaille3,
    TitresTaille4,
    TitresTaille5,
    TitresTaille6,
    TitresGras,
    TitresCasse,
    TitresNumerotation,
    TitresEspacement,
    CouleurTexte,
    CouleurTitres,
    CouleurLiens,
    CouleurGras,
    CouleurItalique,
    CouleurFond,
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
            Champ::NumerotationFigures => self.numerotation_figures.clone(),
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
            Champ::TitresPolice => self.titres.police.clone(),
            Champ::TitresTaille1 => self.titres.taille_1.clone(),
            Champ::TitresTaille2 => self.titres.taille_2.clone(),
            Champ::TitresTaille3 => self.titres.taille_3.clone(),
            Champ::TitresTaille4 => self.titres.taille_4.clone(),
            Champ::TitresTaille5 => self.titres.taille_5.clone(),
            Champ::TitresTaille6 => self.titres.taille_6.clone(),
            Champ::TitresGras => self.titres.gras.to_string(),
            Champ::TitresCasse => self.titres.casse.clone(),
            Champ::TitresNumerotation => self.titres.numerotation.clone(),
            Champ::TitresEspacement => self.titres.espacement.clone(),
            Champ::CouleurTexte => self.couleurs.texte.clone(),
            Champ::CouleurTitres => self.couleurs.titres.clone(),
            Champ::CouleurLiens => self.couleurs.liens.clone(),
            Champ::CouleurGras => self.couleurs.gras.clone(),
            Champ::CouleurItalique => self.couleurs.italique.clone(),
            Champ::CouleurFond => self.couleurs.fond.clone(),
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
            Champ::NumerotationFigures => self.numerotation_figures = valeur,
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
            Champ::TitresPolice => self.titres.police = valeur,
            Champ::TitresTaille1 => self.titres.taille_1 = valeur,
            Champ::TitresTaille2 => self.titres.taille_2 = valeur,
            Champ::TitresTaille3 => self.titres.taille_3 = valeur,
            Champ::TitresTaille4 => self.titres.taille_4 = valeur,
            Champ::TitresTaille5 => self.titres.taille_5 = valeur,
            Champ::TitresTaille6 => self.titres.taille_6 = valeur,
            Champ::TitresGras => self.titres.gras = valeur == "true",
            Champ::TitresCasse => self.titres.casse = valeur,
            Champ::TitresNumerotation => self.titres.numerotation = valeur,
            Champ::TitresEspacement => self.titres.espacement = valeur,
            Champ::CouleurTexte => self.couleurs.texte = valeur,
            Champ::CouleurTitres => self.couleurs.titres = valeur,
            Champ::CouleurLiens => self.couleurs.liens = valeur,
            Champ::CouleurGras => self.couleurs.gras = valeur,
            Champ::CouleurItalique => self.couleurs.italique = valeur,
            Champ::CouleurFond => self.couleurs.fond = valeur,
        }
    }

    /// Inverse une case à cocher.
    pub fn basculer(&mut self, champ: Champ) {
        let actuel = self.valeur(champ) == "true";
        self.modifier(champ, (!actuel).to_string());
    }
}

#[cfg(test)]
mod tests {

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
        let regles = typo.regles();
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
}
