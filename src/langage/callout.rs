//! Les callouts : des blocs de citation habillés par showybox.
//!
//! Un callout n'est pas un bloc de plus dans le document : c'est un **décor**
//! — `{callout note "À retenir"}` devant le bloc qu'il habille, exactement
//! comme `{encadré}`. La source reste donc lisible, et le générateur remplace
//! le `#block(...)` habituel par un `#showybox(...)`.
//!
//! Un callout choisit un **préréglage** nommé (`note`, `attention`, …). Le
//! préréglage vit dans le fichier de normes du document ; le panneau
//! d'insertion peut en modifier les couleurs, la bordure et les mesures pour
//! une occurrence précise. Ces écarts sont alors écrits dans la source sous
//! forme de propriétés explicites — jamais cachés dans un rendu parallèle.
//!
//! Le paquet showybox est vendu avec l'application, sous l'identité
//! [`crate::typst::paquets::SHOWYBOX`].

use serde::{Deserialize, Serialize};

use crate::langage::schema::Proprietes;

/// Un préréglage de callout, embarqué dans le fichier de normes.
///
/// Les couleurs sont des hexadécimaux (`#3b6ea5`) : le générateur les entoure
/// de `rgb(...)`, ce qui interdit qu'une valeur saisie devienne du code Typst.
/// Les mesures sont des longueurs Typst (`4pt`, `1.2em`, `100%`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct CalloutPreset {
    /// La clé écrite dans la source : `note`, `attention`, `veille`…
    pub cle: String,

    /// Le nom affiché dans les menus.
    pub nom: String,

    /// Le titre proposé par défaut ; vide pour un cadre sans titre.
    pub titre: String,

    /// La couleur de la bordure.
    pub bordure: String,
    /// La couleur du fond du corps.
    pub fond: String,
    /// La couleur du fond de la bande de titre.
    pub titre_fond: String,
    /// La couleur du texte du titre.
    pub titre_texte: String,
    /// La couleur du texte du corps.
    pub corps_texte: String,

    /// L'épaisseur de la bordure.
    pub epaisseur: String,
    /// Le rayon des coins.
    pub rayon: String,
    /// La marge intérieure horizontale.
    pub marge_x: String,
    /// La marge intérieure verticale.
    pub marge_y: String,
    /// La largeur du cadre (`100%`, `8cm`, `auto`…).
    pub largeur: String,

    /// `solid`, `dashed` ou `dotted`.
    pub bordure_style: String,
    /// `left`, `center` ou `right`.
    pub alignement: String,

    /// Le titre est-il en gras ?
    pub gras: bool,
    /// Le cadre peut-il se couper entre deux pages ?
    pub rupture: bool,
}

impl Default for CalloutPreset {
    fn default() -> Self {
        Self {
            cle: "note".to_string(),
            nom: "Note".to_string(),
            titre: String::new(),
            bordure: "#3b6ea5".to_string(),
            fond: "#eef4fb".to_string(),
            titre_fond: "#3b6ea5".to_string(),
            titre_texte: "#ffffff".to_string(),
            corps_texte: "#1a1a1a".to_string(),
            epaisseur: "0.8pt".to_string(),
            rayon: "4pt".to_string(),
            marge_x: "10pt".to_string(),
            marge_y: "8pt".to_string(),
            largeur: "100%".to_string(),
            bordure_style: "solid".to_string(),
            alignement: "left".to_string(),
            gras: true,
            rupture: true,
        }
    }
}

/// Un champ modifiable d'un préréglage. Les booléens ont leurs propres
/// messages, pour que la case à cocher n'ait pas à écrire `true` en texte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalloutChamp {
    Nom,
    Titre,
    Bordure,
    Fond,
    TitreFond,
    TitreTexte,
    CorpsTexte,
    Epaisseur,
    Rayon,
    MargeX,
    MargeY,
    Largeur,
    BordureStyle,
    Alignement,
    Gras,
    Rupture,
}

impl CalloutPreset {
    /// La valeur d'un champ texte — `""` pour un champ booléen.
    pub fn valeur(&self, champ: CalloutChamp) -> &str {
        match champ {
            CalloutChamp::Nom => &self.nom,
            CalloutChamp::Titre => &self.titre,
            CalloutChamp::Bordure => &self.bordure,
            CalloutChamp::Fond => &self.fond,
            CalloutChamp::TitreFond => &self.titre_fond,
            CalloutChamp::TitreTexte => &self.titre_texte,
            CalloutChamp::CorpsTexte => &self.corps_texte,
            CalloutChamp::Epaisseur => &self.epaisseur,
            CalloutChamp::Rayon => &self.rayon,
            CalloutChamp::MargeX => &self.marge_x,
            CalloutChamp::MargeY => &self.marge_y,
            CalloutChamp::Largeur => &self.largeur,
            CalloutChamp::BordureStyle => &self.bordure_style,
            CalloutChamp::Alignement => &self.alignement,
            CalloutChamp::Gras | CalloutChamp::Rupture => "",
        }
    }

    /// Modifie un champ texte. Rend `false` pour un booléen : `gras` et
    /// `rupture` passent par leurs méthodes dédiées.
    pub fn modifier(&mut self, champ: CalloutChamp, valeur: String) -> bool {
        if matches!(champ, CalloutChamp::Gras | CalloutChamp::Rupture) {
            return false;
        }
        let cible = match champ {
            CalloutChamp::Nom => &mut self.nom,
            CalloutChamp::Titre => &mut self.titre,
            CalloutChamp::Bordure => &mut self.bordure,
            CalloutChamp::Fond => &mut self.fond,
            CalloutChamp::TitreFond => &mut self.titre_fond,
            CalloutChamp::TitreTexte => &mut self.titre_texte,
            CalloutChamp::CorpsTexte => &mut self.corps_texte,
            CalloutChamp::Epaisseur => &mut self.epaisseur,
            CalloutChamp::Rayon => &mut self.rayon,
            CalloutChamp::MargeX => &mut self.marge_x,
            CalloutChamp::MargeY => &mut self.marge_y,
            CalloutChamp::Largeur => &mut self.largeur,
            CalloutChamp::BordureStyle => &mut self.bordure_style,
            CalloutChamp::Alignement => &mut self.alignement,
            CalloutChamp::Gras | CalloutChamp::Rupture => return false,
        };
        *cible = valeur;
        true
    }

    /// Les couleurs et mesures valides, pour le panneau des réglages et pour
    /// le rendu. Une valeur invalide se dit ; elle n'entre jamais dans Typst.
    pub fn problemes(&self) -> Vec<String> {
        let mut out = Vec::new();

        if self.cle.trim().is_empty()
            || !self
                .cle
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            out.push("la clé du callout doit être un nom simple".to_string());
        }
        if self.nom.trim().is_empty() {
            out.push("le nom affiché du callout est vide".to_string());
        }
        for (nom, valeur) in [
            ("bordure", &self.bordure),
            ("fond", &self.fond),
            ("fond du titre", &self.titre_fond),
            ("texte du titre", &self.titre_texte),
            ("texte du corps", &self.corps_texte),
        ] {
            if !couleur_valide(valeur) {
                out.push(format!("« {valeur} » n'est pas une couleur pour {nom}"));
            }
        }
        for (nom, valeur) in [
            ("épaisseur", &self.epaisseur),
            ("rayon", &self.rayon),
            ("marge horizontale", &self.marge_x),
            ("marge verticale", &self.marge_y),
        ] {
            if !longueur_valide(valeur) {
                out.push(format!("« {valeur} » n'est pas une mesure pour {nom}"));
            }
        }
        if !largeur_valide(&self.largeur) {
            out.push(format!(
                "« {} » n'est pas une largeur (mesure ou auto)",
                self.largeur
            ));
        }
        if !matches!(self.bordure_style.trim(), "solid" | "dashed" | "dotted") {
            out.push(format!(
                "« {} » n'est pas un style de bordure (solid, dashed, dotted)",
                self.bordure_style
            ));
        }
        if !matches!(self.alignement.trim(), "left" | "center" | "right") {
            out.push(format!(
                "« {} » n'est pas un alignement (left, center, right)",
                self.alignement
            ));
        }

        out
    }
}

/// Les cinq préréglages livrés avec l'application. Ils sont volontairement
/// sobres : un callout signale une note de lecture, il ne crie pas.
pub fn presets_defaut() -> Vec<CalloutPreset> {
    vec![
        preset(
            "citation", "Citation", "#9aa0a6", "#f7f7f8", "#e9ebee", "#2f343a",
        ),
        preset("note", "Note", "#3b6ea5", "#eef4fb", "#3b6ea5", "#ffffff"),
        preset(
            "important",
            "Important",
            "#a66b12",
            "#fdf6e6",
            "#a66b12",
            "#ffffff",
        ),
        preset(
            "attention",
            "Attention",
            "#b23b36",
            "#fdf0ef",
            "#b23b36",
            "#ffffff",
        ),
        preset(
            "astuce", "Astuce", "#3f7d57", "#eef7f1", "#3f7d57", "#ffffff",
        ),
    ]
}

fn preset(
    cle: &str,
    nom: &str,
    bordure: &str,
    fond: &str,
    titre_fond: &str,
    titre_texte: &str,
) -> CalloutPreset {
    CalloutPreset {
        cle: cle.to_string(),
        nom: nom.to_string(),
        bordure: bordure.to_string(),
        fond: fond.to_string(),
        titre_fond: titre_fond.to_string(),
        titre_texte: titre_texte.to_string(),
        ..CalloutPreset::default()
    }
}

/// Le préréglage par défaut d'un type historique. Un type inconnu retombe
/// sur `note`, jamais sur une erreur de compilation.
pub fn preset_defaut(type_: &str) -> CalloutPreset {
    let cle = normaliser_type(type_);
    presets_defaut()
        .into_iter()
        .find(|preset| preset.cle == cle)
        .unwrap_or_default()
}

/// Le décor porte-t-il un callout ?
pub fn est_callout(proprietes: &Proprietes) -> bool {
    proprietes.noms.first().map(String::as_str) == Some("callout")
}

/// Le type écrit dans le décor, ou vide s'il n'est pas nommé.
pub fn type_de(proprietes: &Proprietes) -> &str {
    proprietes.noms.get(1).map(String::as_str).unwrap_or("")
}

/// Le titre écrit dans le décor, ou vide.
pub fn titre_de(proprietes: &Proprietes) -> &str {
    proprietes.obtenir("titre").unwrap_or("")
}

/// Le nom affiché d'un type historique.
#[allow(dead_code)] // contrat historique, éprouvé par les tests
pub fn nom(type_: &str) -> String {
    preset_defaut(type_).nom
}

/// Ramène un type — ou un alias — à l'une des cinq formes canoniques.
pub fn normaliser_type(type_: &str) -> &'static str {
    match type_.trim().to_lowercase().as_str() {
        "citation" | "citation-bloc" | "citation_bloc" | "quote" => "citation",
        "important" => "important",
        "attention" | "warning" | "danger" | "alerte" => "attention",
        "astuce" | "tip" | "conseil" => "astuce",
        "note" | "info" | "information" | "remarque" => "note",
        _ => "note",
    }
}

/// Le type est-il connu, alias compris ?
#[allow(dead_code)] // contrat historique, gardé pour les appelants du langage
pub fn type_connu(type_: &str) -> bool {
    matches!(
        type_.trim().to_lowercase().as_str(),
        "citation"
            | "citation-bloc"
            | "citation_bloc"
            | "quote"
            | "important"
            | "attention"
            | "warning"
            | "danger"
            | "alerte"
            | "astuce"
            | "tip"
            | "conseil"
            | "note"
            | "info"
            | "information"
            | "remarque"
    )
}

/// Le préréglage nommé, d'abord dans le document, puis dans les tons
/// historiques. Rend `None` pour un type vide.
pub fn preset_dans<'a>(type_: &str, presets: &'a [CalloutPreset]) -> Option<&'a CalloutPreset> {
    let type_ = type_.trim();
    if type_.is_empty() {
        return None;
    }
    presets
        .iter()
        .find(|preset| preset.cle == type_ || preset.cle.eq_ignore_ascii_case(type_))
        .or_else(|| {
            presets
                .iter()
                .find(|preset| preset.cle == normaliser_type(type_))
        })
}

/// La forme intérieure d'un callout, sans les accolades : `callout note "T"`.
///
/// Les propriétés de style explicites suivent le titre. Elles sont écrites
/// dans l'ordre où le panneau les a posées, pour que la source reste diffable.
pub fn forme_proprietes(proprietes: &Proprietes) -> String {
    let type_ = type_de(proprietes);
    let mut out = String::from("callout");
    if !type_.trim().is_empty() {
        out.push(' ');
        out.push_str(type_.trim());
    }

    let titre = titre_de(proprietes);
    if !type_.trim().is_empty() && !titre.trim().is_empty() {
        out.push_str(" \"");
        echapper_chaine(titre.trim(), &mut out);
        out.push('"');
    }

    for (cle, valeur) in &proprietes.paires {
        if cle == "titre" {
            continue;
        }
        out.push(' ');
        out.push_str(cle);
        out.push('=');
        if valeur.contains(' ') {
            out.push('{');
            out.push_str(valeur);
            out.push('}');
        } else {
            out.push_str(valeur);
        }
    }

    out
}

/// La forme d'écriture d'un callout : `{callout note "À retenir"}`.
#[allow(dead_code)] // contrat historique, éprouvé par les tests
pub fn source(type_: &str, titre: &str) -> String {
    let mut proprietes = Proprietes::vide();
    proprietes.noms.push("callout".to_string());
    if !type_.trim().is_empty() {
        proprietes.noms.push(type_.trim().to_string());
    }
    if !titre.trim().is_empty() {
        proprietes.poser("titre", titre.trim().to_string());
    }
    format!("{{{}}}", forme_proprietes(&proprietes))
}

/// La source d'un callout avec ses écarts de style par rapport au préréglage.
///
/// Seuls les champs réellement différents sont écrits : un callout qui suit
/// son préréglage garde la forme courte, et une personnalisation ne noie pas
/// le texte sous des propriétés inutiles.
pub fn source_avec_style(
    cle: &str,
    titre: &str,
    style: &CalloutPreset,
    base: &CalloutPreset,
) -> String {
    let mut proprietes = Proprietes::vide();
    proprietes.noms.push("callout".to_string());
    if !cle.trim().is_empty() {
        proprietes.noms.push(cle.trim().to_string());
    }
    if !titre.trim().is_empty() {
        proprietes.poser("titre", titre.trim().to_string());
    }

    let champs = [
        (
            CalloutChamp::Bordure,
            "bordure",
            &style.bordure,
            &base.bordure,
        ),
        (CalloutChamp::Fond, "fond", &style.fond, &base.fond),
        (
            CalloutChamp::TitreFond,
            "titre-fond",
            &style.titre_fond,
            &base.titre_fond,
        ),
        (
            CalloutChamp::TitreTexte,
            "titre-texte",
            &style.titre_texte,
            &base.titre_texte,
        ),
        (
            CalloutChamp::CorpsTexte,
            "corps-texte",
            &style.corps_texte,
            &base.corps_texte,
        ),
        (
            CalloutChamp::Epaisseur,
            "epaisseur",
            &style.epaisseur,
            &base.epaisseur,
        ),
        (CalloutChamp::Rayon, "rayon", &style.rayon, &base.rayon),
        (
            CalloutChamp::MargeX,
            "marge-x",
            &style.marge_x,
            &base.marge_x,
        ),
        (
            CalloutChamp::MargeY,
            "marge-y",
            &style.marge_y,
            &base.marge_y,
        ),
        (
            CalloutChamp::Largeur,
            "largeur",
            &style.largeur,
            &base.largeur,
        ),
        (
            CalloutChamp::BordureStyle,
            "bordure-style",
            &style.bordure_style,
            &base.bordure_style,
        ),
        (
            CalloutChamp::Alignement,
            "alignement",
            &style.alignement,
            &base.alignement,
        ),
    ];
    for (_, cle_champ, valeur, origine) in champs {
        if valeur.trim() != origine.trim() {
            proprietes.poser(cle_champ, valeur.trim());
        }
    }
    if style.gras != base.gras {
        proprietes.poser("gras", style.gras.to_string());
    }
    if style.rupture != base.rupture {
        proprietes.poser("rupture", style.rupture.to_string());
    }

    format!("{{{}}}", forme_proprietes(&proprietes))
}

/// Le rendu Typst d'un callout avec les préréglages du document.
pub fn rendre_avec(
    type_: &str,
    titre: &str,
    corps: &str,
    presets: &[CalloutPreset],
    proprietes: &Proprietes,
) -> String {
    let base = preset_dans(type_, presets)
        .cloned()
        .unwrap_or_else(|| preset_defaut(type_));
    let style = appliquer_surcharges(base, proprietes);

    let bordure = couleur_rgb(&style.bordure, "#3b6ea5");
    let fond = couleur_rgb(&style.fond, "#eef4fb");
    let titre_fond = couleur_rgb(&style.titre_fond, "#3b6ea5");
    let titre_texte = couleur_rgb(&style.titre_texte, "#ffffff");
    let corps_texte = couleur_rgb(&style.corps_texte, "#1a1a1a");
    let epaisseur = longueur_ou(&style.epaisseur, "0.8pt");
    let rayon = longueur_ou(&style.rayon, "4pt");
    let marge_x = longueur_ou(&style.marge_x, "10pt");
    let marge_y = longueur_ou(&style.marge_y, "8pt");
    let largeur = largeur_ou(&style.largeur, "100%");
    let bordure_style = match style.bordure_style.trim() {
        "dashed" | "dotted" => style.bordure_style.trim(),
        _ => "solid",
    };
    let alignement = match style.alignement.trim() {
        "center" | "right" => style.alignement.trim(),
        _ => "left",
    };
    let graisse = if style.gras { "bold" } else { "regular" };

    let mut titre_typst = String::new();
    echapper_chaine(titre.trim(), &mut titre_typst);

    format!(
        "#showybox(\n  frame: (border-color: {bordure}, title-color: {titre_fond}, body-color: {fond}, radius: {rayon}, thickness: {epaisseur}, inset: (x: {marge_x}, y: {marge_y}), dash: \"{bordure_style}\"),\n  title-style: (color: {titre_texte}, weight: \"{graisse}\", align: {alignement}),\n  body-style: (color: {corps_texte}),\n  breakable: {rupture},\n  width: {largeur},\n  title: \"{titre_typst}\",\n)[\n{corps}\n]\n\n",
        rupture = style.rupture,
    )
}

/// Le rendu historique : les cinq tons par défaut, sans surcharge.
#[allow(dead_code)] // contrat historique, éprouvé par le test de compilation
pub fn rendre(type_: &str, titre: &str, corps: &str) -> String {
    rendre_avec(type_, titre, corps, &presets_defaut(), &Proprietes::vide())
}

/// Applique les propriétés explicites de la source au préréglage.
fn appliquer_surcharges(mut base: CalloutPreset, proprietes: &Proprietes) -> CalloutPreset {
    for (cle, valeur) in &proprietes.paires {
        let champ = match cle.as_str() {
            "bordure" => Some(CalloutChamp::Bordure),
            "fond" => Some(CalloutChamp::Fond),
            "titre-fond" => Some(CalloutChamp::TitreFond),
            "titre-texte" => Some(CalloutChamp::TitreTexte),
            "corps-texte" => Some(CalloutChamp::CorpsTexte),
            "epaisseur" => Some(CalloutChamp::Epaisseur),
            "rayon" => Some(CalloutChamp::Rayon),
            "marge-x" => Some(CalloutChamp::MargeX),
            "marge-y" => Some(CalloutChamp::MargeY),
            "largeur" => Some(CalloutChamp::Largeur),
            "bordure-style" => Some(CalloutChamp::BordureStyle),
            "alignement" => Some(CalloutChamp::Alignement),
            _ => None,
        };
        if let Some(champ) = champ {
            base.modifier(champ, valeur.clone());
        }
    }
    if let Some(valeur) = proprietes.obtenir("gras") {
        base.gras = oui(valeur);
    }
    if let Some(valeur) = proprietes.obtenir("rupture") {
        base.rupture = oui(valeur);
    }
    base
}

fn oui(valeur: &str) -> bool {
    matches!(
        valeur.trim().to_lowercase().as_str(),
        "true" | "oui" | "1" | "yes"
    )
}

fn couleur_valide(valeur: &str) -> bool {
    let Some(hexa) = valeur.trim().strip_prefix('#') else {
        return false;
    };
    (hexa.len() == 6 || hexa.len() == 3) && hexa.chars().all(|c| c.is_ascii_hexdigit())
}

fn couleur_rgb(valeur: &str, defaut: &str) -> String {
    let valeur = if couleur_valide(valeur) {
        valeur.trim()
    } else {
        defaut
    };
    format!("rgb(\"{valeur}\")")
}

fn longueur_valide(valeur: &str) -> bool {
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

fn largeur_valide(valeur: &str) -> bool {
    valeur.trim() == "auto" || longueur_valide(valeur)
}

fn longueur_ou(valeur: &str, defaut: &str) -> String {
    if longueur_valide(valeur) {
        valeur.trim().to_string()
    } else {
        defaut.to_string()
    }
}

fn largeur_ou(valeur: &str, defaut: &str) -> String {
    if largeur_valide(valeur) {
        valeur.trim().to_string()
    } else {
        defaut.to_string()
    }
}

/// La valeur entre guillemets d'un titre, sans son échappement.
fn echapper_chaine(texte: &str, out: &mut String) {
    for caractere in texte.chars() {
        match caractere {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' | '\r' => out.push(' '),
            autre => out.push(autre),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Les cinq tons ont un nom, et le type inconnu ne casse rien.
    #[test]
    fn les_types_ont_un_nom_et_un_repli() {
        assert_eq!(nom("citation"), "Citation");
        assert_eq!(nom("note"), "Note");
        assert_eq!(nom("important"), "Important");
        assert_eq!(nom("attention"), "Attention");
        assert_eq!(nom("astuce"), "Astuce");
        assert_eq!(nom("inconnu"), "Note");
        assert_eq!(normaliser_type("warning"), "attention");
        assert_eq!(normaliser_type("tip"), "astuce");
    }

    /// La source est courte, et garde le titre entre guillemets.
    #[test]
    fn la_source_est_courte_et_lisible() {
        assert_eq!(source("note", "À retenir"), "{callout note \"À retenir\"}");
        assert_eq!(source("inconnu", ""), "{callout inconnu}");
        assert_eq!(source("", ""), "{callout}");
        assert_eq!(
            source("attention", "Ne pas \"forcer\""),
            "{callout attention \"Ne pas \\\"forcer\\\"\"}"
        );
    }

    /// Le rendu porte les couleurs du ton, le titre échappé et le corps.
    #[test]
    fn le_rendu_porte_le_ton_et_le_titre() {
        let rendu = rendre("attention", "Vigilance", "#quote(block: true)[Un texte.]");

        assert!(rendu.contains("#showybox("));
        assert!(rendu.contains("title: \"Vigilance\""));
        assert!(rendu.contains("border-color: rgb(\"#b23b36\")"));
        assert!(rendu.contains("body-color: rgb(\"#fdf0ef\")"));
        assert!(rendu.contains("#quote(block: true)[Un texte.]"));
        assert!(!rendu.contains("#block("));
    }

    /// Les préréglages du document priment, et une propriété explicite ne
    /// change que ce qu'elle nomme.
    #[test]
    fn un_preset_se_surcharge_sans_perdre_le_reste() {
        let mut presets = presets_defaut();
        presets[1].bordure = "#112233".to_string();
        presets[1].epaisseur = "2pt".to_string();
        let mut proprietes = Proprietes::vide();
        proprietes.noms = vec!["callout".to_string(), "note".to_string()];
        proprietes.poser("bordure", "#445566");
        proprietes.poser("rayon", "0pt");
        proprietes.poser("gras", "false");

        let rendu = rendre_avec("note", "", "Texte.", &presets, &proprietes);

        assert!(rendu.contains("border-color: rgb(\"#445566\")"));
        assert!(rendu.contains("thickness: 2pt"));
        assert!(rendu.contains("radius: 0pt"));
        assert!(rendu.contains("weight: \"regular\""));
        assert!(rendu.contains("body-color: rgb(\"#eef4fb\")"));
    }

    /// La source ne porte que les écarts utiles, et les relit.
    #[test]
    fn une_personnalisation_ecrit_seulement_ses_ecarts() {
        let base = preset_defaut("note");
        let mut style = base.clone();
        style.bordure = "#112233".to_string();
        style.rupture = false;

        let source = source_avec_style("note", "Titre", &style, &base);

        assert!(source.contains("bordure=#112233"));
        assert!(source.contains("rupture=false"));
        assert!(!source.contains("fond=#eef4fb"));
        assert!(!source.contains("largeur="));
    }
}
