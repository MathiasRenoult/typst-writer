//! Les graphiques `{chart}` et leur traduction en primaviz.
//!
//! La source garde des **données CSV** et des paramètres lisibles ; le
//! générateur les transforme en appels primaviz. Les cinq types de départ
//! sont les plus courants : secteurs, barres, lignes, nuage de points et
//! boîtes à moustaches.
//!
//! Le fichier de normes n'intervient pas ici : un graphique est une donnée du
//! document, pas une mesure de mise en page.

use crate::langage::schema::Chart;

/// Les types reconnus par la balise et par le panneau.
pub const TYPES: [&str; 7] = ["pie", "bar", "line", "area", "scatter", "box", "lollipop"];

/// Le type est-il écrit dans une des formes reconnues ?
///
/// Contrairement à [`normaliser_type`], une faute de frappe ne devient pas
/// silencieusement des barres : le parseur compact peut ainsi distinguer un
/// type d'un titre.
pub fn type_connu(type_: &str) -> bool {
    matches!(
        type_.trim().to_lowercase().as_str(),
        "pie"
            | "secteur"
            | "secteurs"
            | "camembert"
            | "bar"
            | "barre"
            | "barres"
            | "line"
            | "ligne"
            | "lignes"
            | "scatter"
            | "points"
            | "nuage"
            | "nuage-de-points"
            | "box"
            | "boxes"
            | "boxplot"
            | "box-plot"
            | "boite"
            | "boîte"
            | "boites"
            | "boîtes"
            | "area"
            | "aire"
            | "lollipop"
            | "sucette"
            | "sucettes"
    )
}

/// Les colonnes attendues, dans l'ordre, pour chaque type.
pub fn colonnes(type_: &str) -> &'static [&'static str] {
    match type_ {
        "pie" | "bar" | "line" | "area" | "lollipop" => &["étiquette", "valeur"],
        "scatter" => &["x", "y"],
        "box" => &["étiquette", "min", "q1", "médiane", "q3", "max"],
        _ => &["étiquette", "valeur"],
    }
}

/// Les formes acceptées dans la balise et dans le panneau.
pub fn normaliser_type(type_: &str) -> &'static str {
    match type_.trim().to_lowercase().as_str() {
        "pie" | "secteur" | "secteurs" | "camembert" => "pie",
        "line" | "ligne" | "lignes" => "line",
        "area" | "aire" => "area",
        "lollipop" | "sucette" | "sucettes" => "lollipop",
        "scatter" | "points" | "nuage" | "nuage-de-points" => "scatter",
        "box" | "boxes" | "boxplot" | "box-plot" | "boite" | "boîte" | "boites" | "boîtes" => {
            "box"
        }
        _ => "bar",
    }
}

/// Un graphique est-il constructible ? Le message dit la première ligne
/// fautive, avec son numéro.
pub fn valider(chart: &Chart) -> Result<(), String> {
    donnees(chart)?;
    if !matches!(chart.courbe.as_str(), "linear" | "smooth" | "catmull-rom") {
        return Err(format!(
            "« {} » n'est pas une interpolation connue (linear, smooth, catmull-rom)",
            chart.courbe
        ));
    }
    if chart.donut && ratio_anneau(&chart.anneau).is_none() {
        return Err(format!(
            "« {} » n'est pas une part de rayon valide (0 à 100 %)",
            chart.anneau
        ));
    }
    Ok(())
}

/// La part du rayon d'un anneau, de 0 à 1.
fn ratio_anneau(valeur: &str) -> Option<f64> {
    let texte = valeur.trim();
    let (nombre, divise) = match texte.strip_suffix('%') {
        Some(nombre) => (nombre, 100.0),
        None => (texte, 1.0),
    };
    nombre
        .trim()
        .parse::<f64>()
        .ok()
        .map(|valeur| valeur / divise)
        .filter(|valeur| (0.0..=0.95).contains(valeur))
}

/// Les données structurées extraites du CSV.
enum Donnees {
    Simple {
        labels: Vec<String>,
        valeurs: Vec<f64>,
    },
    Points(Vec<(f64, f64)>),
    Boites {
        labels: Vec<String>,
        valeurs: Vec<[f64; 5]>,
    },
}

/// Lit le CSV et rend les données prêtes pour primaviz.
fn donnees(chart: &Chart) -> Result<Donnees, String> {
    let type_ = normaliser_type(&chart.type_);
    let lignes = lire_csv(&chart.donnees)?;

    match type_ {
        "scatter" => {
            let (lignes, decalage) = sans_entete(lignes, |ligne| {
                ligne.first().is_some_and(|v| nombre(v).is_err())
                    || ligne.get(1).is_some_and(|v| nombre(v).is_err())
            });
            let mut points = Vec::new();
            for (numero, ligne) in lignes.iter().enumerate() {
                if ligne.len() != 2 {
                    return Err(format!(
                        "ligne {} : le nuage de points attend deux colonnes (x, y)",
                        numero + 1 + decalage
                    ));
                }
                let x = nombre(&ligne[0]).map_err(|_| {
                    format!(
                        "ligne {} : « {} » n'est pas un nombre",
                        numero + 1 + decalage,
                        ligne[0]
                    )
                })?;
                let y = nombre(&ligne[1]).map_err(|_| {
                    format!(
                        "ligne {} : « {} » n'est pas un nombre",
                        numero + 1 + decalage,
                        ligne[1]
                    )
                })?;
                points.push((x, y));
            }
            if points.is_empty() {
                return Err("le graphique n'a aucune donnée".to_string());
            }
            Ok(Donnees::Points(points))
        }
        "box" => {
            let (lignes, decalage) = sans_entete(lignes, |ligne| {
                ligne.iter().skip(1).take(5).any(|v| nombre(v).is_err())
            });
            let mut labels = Vec::new();
            let mut valeurs = Vec::new();
            for (numero, ligne) in lignes.iter().enumerate() {
                if ligne.len() != 6 {
                    return Err(format!(
                        "ligne {} : une boîte attend six colonnes (étiquette, min, q1, médiane, q3, max)",
                        numero + 1 + decalage
                    ));
                }
                let mut cinq = [0.0; 5];
                for (rang, texte) in ligne.iter().skip(1).enumerate() {
                    cinq[rang] = nombre(texte).map_err(|_| {
                        format!(
                            "ligne {} : « {texte} » n'est pas un nombre",
                            numero + 1 + decalage
                        )
                    })?;
                }
                if cinq.windows(2).any(|paire| paire[0] > paire[1]) {
                    return Err(format!(
                        "ligne {} : les cinq valeurs d'une boîte doivent être croissantes (min, q1, médiane, q3, max)",
                        numero + 1 + decalage
                    ));
                }
                labels.push(ligne[0].trim().to_string());
                valeurs.push(cinq);
            }
            if labels.is_empty() {
                return Err("le graphique n'a aucune donnée".to_string());
            }
            Ok(Donnees::Boites { labels, valeurs })
        }
        _ => {
            let (lignes, decalage) = sans_entete(lignes, |ligne| {
                ligne.get(1).is_some_and(|v| nombre(v).is_err())
            });
            let mut labels = Vec::new();
            let mut valeurs = Vec::new();
            for (numero, ligne) in lignes.iter().enumerate() {
                if ligne.len() != 2 {
                    return Err(format!(
                        "ligne {} : ce graphique attend deux colonnes (étiquette, valeur)",
                        numero + 1 + decalage
                    ));
                }
                let valeur = nombre(&ligne[1]).map_err(|_| {
                    format!(
                        "ligne {} : « {} » n'est pas un nombre",
                        numero + 1 + decalage,
                        ligne[1]
                    )
                })?;
                labels.push(ligne[0].trim().to_string());
                valeurs.push(valeur);
            }
            if labels.is_empty() {
                return Err("le graphique n'a aucune donnée".to_string());
            }
            Ok(Donnees::Simple { labels, valeurs })
        }
    }
}

/// Supprime une éventuelle ligne d'en-tête. Une ligne dont une colonne
/// numérique ne l'est pas est une en-tête ; sinon elle reste une donnée.
/// Rend aussi le décalage, pour que les erreurs nomment la ligne du CSV
/// d'origine.
fn sans_entete<F>(lignes: Vec<Vec<String>>, entete: F) -> (Vec<Vec<String>>, usize)
where
    F: Fn(&[String]) -> bool,
{
    if lignes.first().is_some_and(|ligne| entete(ligne)) {
        (lignes.into_iter().skip(1).collect(), 1)
    } else {
        (lignes, 0)
    }
}

/// Lit un CSV : virgule ou point-virgule, guillemets et guillemets doublés.
/// Les lignes vides sont ignorées.
pub fn lire_csv(texte: &str) -> Result<Vec<Vec<String>>, String> {
    let texte = texte.strip_prefix('\u{feff}').unwrap_or(texte);
    let lignes: Vec<&str> = texte
        .lines()
        .filter(|ligne| !ligne.trim().is_empty())
        .collect();
    if lignes.is_empty() {
        return Err("le CSV est vide".to_string());
    }
    let separateur = separateur(lignes[0]);
    let mut out = Vec::new();
    for (numero, ligne) in lignes.iter().enumerate() {
        let champs = decouper(ligne, separateur)
            .map_err(|erreur| format!("ligne {} : {erreur}", numero + 1))?;
        if !champs.is_empty() {
            out.push(champs);
        }
    }
    Ok(out)
}

/// Le séparateur le plus probable de la première ligne, hors guillemets.
fn separateur(ligne: &str) -> char {
    let mut guillemets = false;
    let (mut virgules, mut points_virgules) = (0, 0);
    for caractere in ligne.chars() {
        match caractere {
            '"' => guillemets = !guillemets,
            ',' if !guillemets => virgules += 1,
            ';' if !guillemets => points_virgules += 1,
            _ => {}
        }
    }
    if points_virgules > virgules { ';' } else { ',' }
}

/// Découpe une ligne CSV, en gardant les virgules dans les guillemets.
fn decouper(ligne: &str, separateur: char) -> Result<Vec<String>, String> {
    let mut champs = Vec::new();
    let mut courant = String::new();
    let mut guillemets = false;
    let mut caracteres = ligne.chars().peekable();
    while let Some(caractere) = caracteres.next() {
        match caractere {
            '"' if guillemets && caracteres.peek() == Some(&'"') => {
                let _ = caracteres.next();
                courant.push('"');
            }
            '"' => guillemets = !guillemets,
            c if c == separateur && !guillemets => {
                champs.push(courant.trim().to_string());
                courant.clear();
            }
            c => courant.push(c),
        }
    }
    if guillemets {
        return Err("guillemet non fermé".to_string());
    }
    champs.push(courant.trim().to_string());
    Ok(champs)
}

/// Un nombre décimal, avec la virgule française acceptée.
fn nombre(texte: &str) -> Result<f64, ()> {
    texte
        .trim()
        .replace(',', ".")
        .parse::<f64>()
        .ok()
        .filter(|v| v.is_finite())
        .ok_or(())
}

/// Le texte Typst d'un titre, dans une contenu `[...]`.
fn titre_typst(texte: &str) -> String {
    let mut out = String::new();
    for caractere in texte.chars() {
        match caractere {
            '\\' | '[' | ']' | '#' | '$' | '*' | '_' | '`' | '<' | '>' | '@' => {
                out.push('\\');
                out.push(caractere);
            }
            _ => out.push(caractere),
        }
    }
    out
}

/// Une chaîne Typst entre guillemets.
fn chaine_typst(texte: &str) -> String {
    let mut out = String::with_capacity(texte.len() + 2);
    out.push('"');
    for caractere in texte.chars() {
        if caractere == '\\' || caractere == '"' {
            out.push('\\');
        }
        out.push(caractere);
    }
    out.push('"');
    out
}

/// Un nombre Typst, sans notation inutile.
fn nombre_typst(valeur: f64) -> String {
    if valeur.fract() == 0.0 {
        format!("{valeur:.0}")
    } else {
        format!("{valeur}")
    }
}

/// Le contenu Typst d'un graphique, sans son placement.
fn appel(chart: &Chart) -> Result<String, String> {
    let type_ = normaliser_type(&chart.type_);
    let donnees = donnees(chart)?;
    let mut arguments = Vec::new();
    let fonction;

    match donnees {
        Donnees::Simple { labels, valeurs } => {
            let labels = labels.iter().map(|l| chaine_typst(l)).collect::<Vec<_>>();
            let valeurs = valeurs.iter().map(|v| nombre_typst(*v)).collect::<Vec<_>>();
            arguments.push(format!(
                "(labels: ({}), values: ({}))",
                labels.join(", "),
                valeurs.join(", ")
            ));

            match type_ {
                "pie" => {
                    fonction = "pie-chart";
                    arguments.push(format!("size: {}", taille(&chart.largeur, "8cm")));
                    arguments.push(format!("show-legend: {}", chart.legende));
                    arguments.push(format!("show-percentages: {}", chart.valeurs));
                    if chart.donut {
                        arguments.push("donut: true".to_string());
                        if let Some(ratio) = ratio_anneau(&chart.anneau) {
                            arguments.push(format!("donut-ratio: {ratio}"));
                        }
                    }
                    axe(&mut arguments, "subtitle", &chart.sous_titre);
                }
                "line" => {
                    fonction = "line-chart";
                    arguments.push(format!("width: {}", taille(&chart.largeur, "10cm")));
                    arguments.push(format!("height: {}", taille(&chart.hauteur, "6cm")));
                    arguments.push(format!("show-points: {}", chart.points));
                    arguments.push(format!("show-values: {}", chart.valeurs));
                    arguments.push(format!("fill-area: {}", chart.aire));
                    arguments.push(format!("line-interpolation: \"{}\"", chart.courbe));
                    axe(&mut arguments, "x-label", &chart.x_label);
                    axe(&mut arguments, "y-label", &chart.y_label);
                    axe(&mut arguments, "subtitle", &chart.sous_titre);
                    axes_communs(&mut arguments, chart);
                }
                "area" => {
                    fonction = "area-chart";
                    arguments.push(format!("width: {}", taille(&chart.largeur, "10cm")));
                    arguments.push(format!("height: {}", taille(&chart.hauteur, "6cm")));
                    arguments.push(format!("show-points: {}", chart.points));
                    arguments.push("fill-opacity: 40%".to_string());
                    axe(&mut arguments, "x-label", &chart.x_label);
                    axe(&mut arguments, "y-label", &chart.y_label);
                    axes_communs(&mut arguments, chart);
                }
                "lollipop" => {
                    fonction = "lollipop-chart";
                    arguments.push(format!("width: {}", taille(&chart.largeur, "10cm")));
                    arguments.push(format!("height: {}", taille(&chart.hauteur, "6cm")));
                    arguments.push(format!("show-values: {}", chart.valeurs));
                    axe(&mut arguments, "x-label", &chart.x_label);
                    axe(&mut arguments, "y-label", &chart.y_label);
                }
                "bar" if chart.horizontal => {
                    fonction = "horizontal-bar-chart";
                    arguments.push(format!("width: {}", taille(&chart.largeur, "10cm")));
                    arguments.push(format!("height: {}", taille(&chart.hauteur, "6cm")));
                    arguments.push(format!("show-values: {}", chart.valeurs));
                    axe(&mut arguments, "x-label", &chart.x_label);
                    axe(&mut arguments, "y-label", &chart.y_label);
                }
                "bar" => {
                    fonction = "bar-chart";
                    arguments.push(format!("width: {}", taille(&chart.largeur, "10cm")));
                    arguments.push(format!("height: {}", taille(&chart.hauteur, "6cm")));
                    arguments.push(format!("show-values: {}", chart.valeurs));
                    axe(&mut arguments, "x-label", &chart.x_label);
                    axe(&mut arguments, "y-label", &chart.y_label);
                    axe(&mut arguments, "subtitle", &chart.sous_titre);
                    axes_communs(&mut arguments, chart);
                }
                _ => {
                    fonction = "bar-chart";
                    arguments.push(format!("width: {}", taille(&chart.largeur, "10cm")));
                    arguments.push(format!("height: {}", taille(&chart.hauteur, "6cm")));
                    arguments.push(format!("show-values: {}", chart.valeurs));
                    axe(&mut arguments, "x-label", &chart.x_label);
                    axe(&mut arguments, "y-label", &chart.y_label);
                    axe(&mut arguments, "subtitle", &chart.sous_titre);
                    axes_communs(&mut arguments, chart);
                }
            }
        }
        Donnees::Points(points) => {
            fonction = "scatter-plot";
            let donnees = points
                .iter()
                .map(|(x, y)| format!("({}, {})", nombre_typst(*x), nombre_typst(*y)))
                .collect::<Vec<_>>()
                .join(", ");
            arguments.push(format!("({donnees})"));
            arguments.push(format!("width: {}", taille(&chart.largeur, "10cm")));
            arguments.push(format!("height: {}", taille(&chart.hauteur, "6cm")));
            axe(&mut arguments, "x-label", &chart.x_label);
            axe(&mut arguments, "y-label", &chart.y_label);
            axe(&mut arguments, "subtitle", &chart.sous_titre);
            arguments.push(format!("show-grid: {}", chart.grille));
            arguments.push(format!("show-ticks: {}", chart.graduations));
        }
        Donnees::Boites { labels, valeurs } => {
            fonction = "box-plot";
            let labels = labels.iter().map(|l| chaine_typst(l)).collect::<Vec<_>>();
            let boites = valeurs
                .iter()
                .map(|b| {
                    format!(
                        "(min: {}, q1: {}, median: {}, q3: {}, max: {})",
                        nombre_typst(b[0]),
                        nombre_typst(b[1]),
                        nombre_typst(b[2]),
                        nombre_typst(b[3]),
                        nombre_typst(b[4])
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            arguments.push(format!(
                "(labels: ({}), boxes: ({boites}))",
                labels.join(", ")
            ));
            arguments.push(format!("width: {}", taille(&chart.largeur, "10cm")));
            arguments.push(format!("height: {}", taille(&chart.hauteur, "6cm")));
            arguments.push(format!("show-values: {}", chart.valeurs));
            arguments.push(format!("show-grid: {}", chart.grille));
            axe(&mut arguments, "x-label", &chart.x_label);
            axe(&mut arguments, "y-label", &chart.y_label);
        }
    }

    if !chart.titre.trim().is_empty() {
        arguments.push(format!("title: [{}]", titre_typst(&chart.titre)));
    }

    Ok(format!("{fonction}(\n  {}\n)", arguments.join(",\n  ")))
}

/// Ajoute un argument de contenu Typst s'il n'est pas vide.
fn axe(arguments: &mut Vec<String>, nom: &str, valeur: &str) {
    if !valeur.trim().is_empty() {
        arguments.push(format!("{nom}: [{}]", titre_typst(valeur)));
    }
}

/// Les options communes aux graphiques cartésiens qui les comprennent.
fn axes_communs(arguments: &mut Vec<String>, chart: &Chart) {
    arguments.push(format!("show-ticks: {}", chart.graduations));
    arguments.push(format!("show-minor-grid: {}", chart.grille));
}

/// Une mesure de largeur ou de hauteur ; une valeur vide devient le défaut.
fn taille(valeur: &str, defaut: &str) -> String {
    let valeur = valeur.trim();
    if valeur.is_empty() {
        defaut.to_string()
    } else {
        valeur.to_string()
    }
}

/// Le graphique complet, avec son placement, prêt à entrer dans le Typst.
pub fn rendre(chart: &Chart) -> Result<String, String> {
    let appel = appel(chart)?;
    let figure = format!("#figure(\n  {appel}\n)");
    let place = match chart.placement.as_str() {
        "gauche" => format!("#align(left)[{figure}]"),
        "droite" => format!("#align(right)[{figure}]"),
        "pleine-largeur" => format!("#block(width: 100%)[{figure}]"),
        _ => format!("#align(center)[{figure}]"),
    };
    Ok(format!("{place}\n\n"))
}

/// La source `.tw` d'un graphique, telle que le parseur la relira.
///
/// Tant que le graphique reste dans les réglages simples, la forme concise
/// suffit : le type, le titre, une taille `largeur×hauteur`, un placement,
/// puis `sans-legende` ou `sans-valeurs`. Dès qu'une option plus fine est
/// employée (sous-titre, axes, grille, anneau, courbe…), la source passe à la
/// forme `clé=valeur`, qui reste lisible et ne perd aucun réglage.
pub fn source(chart: &Chart) -> String {
    let avance = !chart.sous_titre.trim().is_empty()
        || !chart.x_label.trim().is_empty()
        || !chart.y_label.trim().is_empty()
        || chart.grille
        || chart.graduations
        || !chart.points
        || chart.aire
        || chart.courbe != "linear"
        || chart.donut
        || chart.horizontal;

    let mut morceaux = if avance {
        let mut morceaux = vec![format!("type={}", normaliser_type(&chart.type_))];
        if !chart.titre.trim().is_empty() {
            morceaux.push(attribut("titre", &chart.titre, true));
        }
        if !chart.sous_titre.trim().is_empty() {
            morceaux.push(attribut("sous-titre", &chart.sous_titre, true));
        }
        if !chart.x_label.trim().is_empty() {
            morceaux.push(attribut("x-label", &chart.x_label, true));
        }
        if !chart.y_label.trim().is_empty() {
            morceaux.push(attribut("y-label", &chart.y_label, true));
        }
        morceaux.push(attribut("largeur", &chart.largeur, false));
        morceaux.push(attribut("hauteur", &chart.hauteur, false));
        morceaux.push(attribut("placement", &chart.placement, false));
        morceaux.push(attribut("legende", &chart.legende.to_string(), false));
        morceaux.push(attribut("valeurs", &chart.valeurs.to_string(), false));
        morceaux.push(attribut("grille", &chart.grille.to_string(), false));
        morceaux.push(attribut(
            "graduations",
            &chart.graduations.to_string(),
            false,
        ));
        morceaux.push(attribut("points", &chart.points.to_string(), false));
        morceaux.push(attribut("aire", &chart.aire.to_string(), false));
        morceaux.push(attribut("courbe", &chart.courbe, true));
        morceaux.push(attribut("donut", &chart.donut.to_string(), false));
        morceaux.push(attribut("anneau", &chart.anneau, false));
        morceaux.push(attribut("horizontal", &chart.horizontal.to_string(), false));
        morceaux
    } else {
        let mut morceaux = vec![normaliser_type(&chart.type_).to_string()];

        if !chart.titre.trim().is_empty() {
            morceaux.push(format!(
                "\"{}\"",
                chart.titre.replace('\\', "\\\\").replace('"', "\\\"")
            ));
        }

        let largeur = if chart.largeur.trim().is_empty() {
            "10cm"
        } else {
            chart.largeur.trim()
        };
        let hauteur = if chart.hauteur.trim().is_empty() {
            "6cm"
        } else {
            chart.hauteur.trim()
        };
        if largeur != "10cm" || hauteur != "6cm" {
            morceaux.push(format!("{largeur}×{hauteur}"));
        }

        if chart.placement != "centre" {
            morceaux.push(
                match chart.placement.as_str() {
                    "pleine-largeur" => "pleine",
                    autre => autre,
                }
                .to_string(),
            );
        }
        if !chart.legende {
            morceaux.push("sans-legende".to_string());
        }
        if !chart.valeurs {
            morceaux.push("sans-valeurs".to_string());
        }
        morceaux
    };

    // Une ligne de balise ne doit pas être confondue avec un titre : la
    // forme avancée cite toujours ses textes.
    if avance {
        morceaux.retain(|morceau| !morceau.is_empty());
    }

    let mut out = format!("{{chart {}}}\n", morceaux.join(" "));
    out.push_str(chart.donnees.trim());
    out.push_str("\n\n");
    out
}

/// La source canonique courte : `{{ch …}}` au lieu de `{chart …}`.
///
/// Le lecteur historique reste la référence pour toutes les options ; cette
/// forme ne change que l'enveloppe de la balise, pas son contenu.
pub fn source_canonique(chart: &Chart) -> String {
    let source = source(chart);
    let Some((entete, reste)) = source.split_once('\n') else {
        return source;
    };
    let entete = entete.trim_end();
    let Some(corps) = entete
        .strip_prefix("{chart")
        .and_then(|corps| corps.strip_suffix('}'))
    else {
        return source;
    };
    format!("{{{{ch{}}}}}\n{reste}", corps)
}

/// Un attribut `clé=valeur`, cité si le texte peut contenir des espaces.
fn attribut(cle: &str, valeur: &str, citer: bool) -> String {
    let valeur = if citer {
        format!("\"{}\"", valeur.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        valeur.trim().to_string()
    };
    format!("{cle}={valeur}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chart(type_: &str, donnees: &str) -> Chart {
        Chart {
            type_: type_.to_string(),
            donnees: donnees.to_string(),
            ..Chart::default()
        }
    }

    #[test]
    fn le_csv_lit_les_virgules_et_les_guillemets() {
        let lignes = lire_csv("\u{feff}region,\"ventes, 2026\"\nNord,30\nSud,25\n").expect("CSV");
        assert_eq!(lignes[0], ["region", "ventes, 2026"]);
        assert_eq!(lignes[1], ["Nord", "30"]);
    }

    #[test]
    fn un_bar_chart_genere_primaviz_avec_ses_donnees() {
        let code = rendre(&chart("bar", "region,ventes\nNord,30\nSud,25\n")).expect("rendu");
        assert!(code.contains("bar-chart("));
        assert!(code.contains("labels: (\"Nord\", \"Sud\")"));
        assert!(code.contains("values: (30, 25)"));
        assert!(code.contains("#align(center)"));
    }

    #[test]
    fn les_cinq_types_sont_reconnus() {
        for type_ in TYPES {
            let code = match type_ {
                "scatter" => rendre(&chart(type_, "x,y\n1,2\n3,4\n")),
                "box" => rendre(&chart(type_, "A,1,2,3,4,5\nB,2,3,4,5,6\n")),
                _ => rendre(&chart(type_, "A,1\nB,2\n")),
            }
            .unwrap_or_else(|erreur| panic!("{type_} : {erreur}"));
            assert!(!code.is_empty(), "le type {type_} produit un appel");
        }
    }

    #[test]
    fn un_csv_fautif_dit_sa_ligne() {
        let erreur = rendre(&chart("scatter", "x,y\n1\n")).expect_err("le CSV est refusé");
        assert!(
            erreur.contains("ligne 2"),
            "le message nomme la ligne : {erreur}"
        );
    }

    #[test]
    fn la_source_compacte_porte_les_options_non_defaut() {
        let mut chart = chart("pie", "A,30\nB,70\n");
        chart.titre = "Répartition".to_string();
        chart.largeur = "12cm".to_string();
        chart.hauteur = "7cm".to_string();
        chart.placement = "pleine-largeur".to_string();
        chart.legende = false;
        chart.valeurs = false;
        let source = source(&chart);
        assert!(
            source.starts_with(
                "{chart pie \"Répartition\" 12cm×7cm pleine sans-legende sans-valeurs}"
            ),
            "{source}"
        );
    }

    #[test]
    fn la_source_se_relit_par_la_balise() {
        let mut chart = chart("pie", "A,30\nB,70\n");
        chart.titre = "Répartition".to_string();
        let source = source(&chart);
        assert!(source.starts_with("{chart pie \"Répartition\""), "{source}");
        assert!(source.contains("A,30"));
        assert!(!source.contains("type="), "{source}");
    }
}
