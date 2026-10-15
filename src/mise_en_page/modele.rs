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

    /// Le texte courant.
    pub texte: Texte,

    /// Les titres de chapitres.
    pub titres: Titres,

    /// Les couleurs du document.
    pub couleurs: Couleurs,
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

    /// La langue du texte, celle qui décide des règles typographiques.
    pub langue: String,
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
        }
    }
}

/// Les titres de chapitres.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Titres {
    /// La police des titres — vide : celle du texte.
    pub police: String,

    /// Le corps d'un titre de niveau 1, puis 2, puis 3.
    pub taille_1: String,
    pub taille_2: String,
    pub taille_3: String,

    /// Les titres sont-ils en gras ?
    pub gras: bool,

    /// La casse : `normale`, `majuscules`, `minuscules`.
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
            gras: true,
            casse: "normale".to_string(),
            numerotation: "1.1".to_string(),
            espacement: "1.2em".to_string(),
        }
    }
}

/// Les couleurs du document, en hexadécimal.
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
}

impl Default for Couleurs {
    fn default() -> Self {
        Self {
            texte: "#000000".to_string(),
            titres: "#000000".to_string(),
            liens: "#0000ff".to_string(),
            fond: "#ffffff".to_string(),
        }
    }
}

impl MiseEnPage {
    /// Lit une mise en page écrite au format TOML.
    pub fn lire(texte: &str) -> Result<Self, String> {
        toml::from_str(texte).map_err(|erreur| erreur.to_string())
    }

    /// L'écrit au format TOML — celui qui s'exporte et s'importe.
    pub fn ecrire(&self) -> String {
        toml::to_string_pretty(self).unwrap_or_default()
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
        ] {
            if !couleur_valide(valeur) {
                out.push((
                    nom.to_string(),
                    format!("« {valeur} » n'est pas une couleur (ex. #1e1e2e)"),
                ));
            }
        }

        match self.titres.casse.as_str() {
            "normale" | "majuscules" | "minuscules" => {}
            autre => out.push((
                "casse des titres".to_string(),
                format!("« {autre} » n'est pas une casse connue"),
            )),
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
    TextePolice,
    TexteTaille,
    TexteInterligne,
    TexteEspacement,
    TexteRetrait,
    TexteJustification,
    TexteCoupure,
    TexteLangue,
    TitresPolice,
    TitresTaille1,
    TitresTaille2,
    TitresTaille3,
    TitresGras,
    TitresCasse,
    TitresNumerotation,
    TitresEspacement,
    CouleurTexte,
    CouleurTitres,
    CouleurLiens,
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
            Champ::TextePolice => self.texte.police.clone(),
            Champ::TexteTaille => self.texte.taille.clone(),
            Champ::TexteInterligne => self.texte.interligne.clone(),
            Champ::TexteEspacement => self.texte.espacement.clone(),
            Champ::TexteRetrait => self.texte.retrait.clone(),
            Champ::TexteJustification => self.texte.justification.to_string(),
            Champ::TexteCoupure => self.texte.coupure.to_string(),
            Champ::TexteLangue => self.texte.langue.clone(),
            Champ::TitresPolice => self.titres.police.clone(),
            Champ::TitresTaille1 => self.titres.taille_1.clone(),
            Champ::TitresTaille2 => self.titres.taille_2.clone(),
            Champ::TitresTaille3 => self.titres.taille_3.clone(),
            Champ::TitresGras => self.titres.gras.to_string(),
            Champ::TitresCasse => self.titres.casse.clone(),
            Champ::TitresNumerotation => self.titres.numerotation.clone(),
            Champ::TitresEspacement => self.titres.espacement.clone(),
            Champ::CouleurTexte => self.couleurs.texte.clone(),
            Champ::CouleurTitres => self.couleurs.titres.clone(),
            Champ::CouleurLiens => self.couleurs.liens.clone(),
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
            Champ::TextePolice => self.texte.police = valeur,
            Champ::TexteTaille => self.texte.taille = valeur,
            Champ::TexteInterligne => self.texte.interligne = valeur,
            Champ::TexteEspacement => self.texte.espacement = valeur,
            Champ::TexteRetrait => self.texte.retrait = valeur,
            Champ::TexteJustification => self.texte.justification = valeur == "true",
            Champ::TexteCoupure => self.texte.coupure = valeur == "true",
            Champ::TexteLangue => self.texte.langue = valeur,
            Champ::TitresPolice => self.titres.police = valeur,
            Champ::TitresTaille1 => self.titres.taille_1 = valeur,
            Champ::TitresTaille2 => self.titres.taille_2 = valeur,
            Champ::TitresTaille3 => self.titres.taille_3 = valeur,
            Champ::TitresGras => self.titres.gras = valeur == "true",
            Champ::TitresCasse => self.titres.casse = valeur,
            Champ::TitresNumerotation => self.titres.numerotation = valeur,
            Champ::TitresEspacement => self.titres.espacement = valeur,
            Champ::CouleurTexte => self.couleurs.texte = valeur,
            Champ::CouleurTitres => self.couleurs.titres = valeur,
            Champ::CouleurLiens => self.couleurs.liens = valeur,
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
    use super::*;

    /// Une mise en page se lit, se perd, se retrouve.
    #[test]
    fn a_layout_survives_the_round_trip() {
        let mut mise_en_page = MiseEnPage::default();
        mise_en_page.papier.format = "a5".to_string();
        mise_en_page.marges.gauche = "3cm".to_string();
        mise_en_page.titres.casse = "majuscules".to_string();
        mise_en_page.couleurs.texte = "#1e1e2e".to_string();

        let texte = mise_en_page.ecrire();
        let relu = MiseEnPage::lire(&texte).expect("lisible");

        assert_eq!(relu, mise_en_page, "rien ne se perd en chemin");
    }

    /// Un fichier partiel ne casse rien : ce qui manque garde sa valeur.
    #[test]
    fn a_partial_file_keeps_the_defaults() {
        let mise_en_page = MiseEnPage::lire("[papier]\nformat = \"a5\"\n").expect("lisible");

        assert_eq!(mise_en_page.papier.format, "a5");
        assert_eq!(mise_en_page.marges.haut, "2cm", "ce qui manque parle tout seul");
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
