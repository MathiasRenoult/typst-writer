//! Les exports du document : Markdown, HTML, DOCX.
//!
//! Décision de `agent/Pré-Beta/experience-docx-markdown-2026-11.md` :
//
//! - le **Markdown** reprend le document au canonique — le dialecte `.tw`
//!   est du Markdown étendu, la fidélité est proche de l'identité ; les
//!   notes `^[…]` et les citations `@clé[…]` y restent lisibles, quoique
//!   non standard ;
//  - le **DOCX** passe par le HTML puis LibreOffice : titres, tableaux,
//    citations et images y survivent, **les notes deviennent du texte
//    `[note: …]`** — c'est la limite annoncée, verrouillée par un test ;
//  - l'import DOCX n'est pas livré : les notes perdraient leur identité.

use crate::langage::parse::parse;
use crate::langage::serialiser::serialiser;
use std::path::Path;

/// Le document, en Markdown : le texte canonique du dialecte.
pub fn vers_markdown(source: &str) -> String {
    serialiser(&parse(source))
}

/// Le document, en HTML — la forme que la chaîne DOCX mangera.
///
/// Ce mapping est volontairement simple : ce qui ne s'y dit pas se dit
/// dans le texte (les notes deviennent `[note: …]`), jamais perdu en
/// silence.
pub fn vers_html(source: &str) -> String {
    let mut html = String::from("<html><body>\n<meta charset=\"utf-8\">\n");
    let mut lignes = source.lines().peekable();
    while let Some(ligne) = lignes.next() {
        let nu = ligne.trim();
        if nu.is_empty() {
            continue;
        }
        if let Some(reste) = nu.strip_prefix("## ") {
            html.push_str(&format!("<h2>{}</h2>\n", texte_html(reste)));
        } else if let Some(reste) = nu.strip_prefix("# ") {
            html.push_str(&format!("<h1>{}</h1>\n", texte_html(reste)));
        } else if nu.starts_with('|') {
            html.push_str("<table>\n");
            html.push_str(&rangee_html(nu, true));
            while let Some(suite) = lignes.peek() {
                let s = suite.trim();
                if !s.starts_with('|') {
                    break;
                }
                if !s.contains("---") {
                    html.push_str(&rangee_html(s, false));
                }
                lignes.next();
            }
            html.push_str("</table>\n");
        } else if nu.starts_with("![") {
            let (legende, cible) = decouper_image(nu);
            html.push_str(&format!(
                "<figure><img src=\"{}\"/><figcaption>{}</figcaption></figure>\n",
                texte_html(&cible),
                texte_html(&legende)
            ));
        } else {
            html.push_str(&format!("<p>{}</p>\n", texte_html(nu)));
        }
    }
    html.push_str("</body></html>\n");
    html
}

/// Le document, en DOCX, via le HTML puis LibreOffice.
///
/// Sans LibreOffice, l'erreur dit le geste : installer LibreOffice, ou
/// exporter en Markdown.
pub fn vers_docx(source: &str, cible: &Path) -> Result<(), String> {
    if !soffice_disponible() {
        return Err(message_absence());
    }

    let atelier = std::env::temp_dir().join(format!(
        "tw-export-docx-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&atelier)
        .map_err(|e| format!("Le dossier de travail ne se crée pas : {e}."))?;
    let entree = atelier.join("document.html");
    std::fs::write(&entree, vers_html(source))
        .map_err(|e| format!("Le HTML intermédiaire ne s'écrit pas : {e}."))?;

    let profil = atelier.join("profil");
    std::fs::create_dir_all(&profil)
        .map_err(|e| format!("Le profil LibreOffice ne se crée pas : {e}."))?;
    let resultat = std::process::Command::new("soffice")
        .arg(format!("-env:UserInstallation=file://{}", profil.display()))
        .arg("--headless")
        .arg("--infilter=HTML (StarWriter)")
        .arg("--convert-to")
        .arg("docx:Office Open XML Text")
        .arg("--outdir")
        .arg(&atelier)
        .arg(&entree)
        .output()
        .map_err(|e| format!("LibreOffice ne se lance pas : {e}."))?;
    if !resultat.status.success() {
        return Err(format!(
            "LibreOffice a refusé la conversion : {}",
            String::from_utf8_lossy(&resultat.stderr)
        ));
    }

    let produit = atelier.join("document.docx");
    if !produit.is_file() {
        return Err("LibreOffice n'a rien produit : la conversion a échoué en silence.".into());
    }
    let octets =
        std::fs::read(&produit).map_err(|e| format!("Le DOCX produit ne se lit pas : {e}."))?;
    let _ = std::fs::remove_dir_all(&atelier);
    crate::project::ecrire_atomique_octets(cible, &octets)
        .map_err(|e| format!("{} ne s'écrit pas : {e}.", cible.display()))
}

/// LibreOffice est-il là ?
pub fn soffice_disponible() -> bool {
    std::process::Command::new("soffice")
        .arg("--version")
        .output()
        .is_ok()
}

/// Le geste à faire quand LibreOffice manque : un état d'erreur propose
/// son remède, jamais seulement son nom.
pub fn message_absence() -> String {
    "L'export DOCX a besoin de LibreOffice, absent de cette machine. \
     Installez LibreOffice, ou exportez en Markdown — le document y va \
     fidèlement."
        .to_string()
}

fn rangee_html(ligne: &str, entete: bool) -> String {
    let cellules: Vec<&str> = ligne.trim_matches('|').split('|').map(str::trim).collect();
    let ouvrant = if entete { "th" } else { "td" };
    let mut rangee = String::from("<tr>");
    for cellule in &cellules {
        rangee.push_str(&format!("<{ouvrant}>{}</{ouvrant}>", texte_html(cellule)));
    }
    rangee.push_str("</tr>\n");
    rangee
}

fn decouper_image(ligne: &str) -> (String, String) {
    let sans = ligne.trim_start_matches("![").trim_end_matches(']');
    if let Some(ouvrante) = sans.find("](") {
        (
            sans[..ouvrante].to_string(),
            sans[ouvrante + 2..].to_string(),
        )
    } else {
        (String::new(), sans.to_string())
    }
}

/// Le texte nu, échappé pour le HTML — et les notes y deviennent ce
/// qu'elles sont dans une chaîne sans note de bas de page : du texte.
fn texte_html(texte: &str) -> String {
    let mut sorti = String::new();
    let mut reste = texte;
    while let Some(ouvrante) = reste.find("^[") {
        echapper(&mut sorti, &reste[..ouvrante]);
        reste = &reste[ouvrante + 2..];
        match reste.find(']') {
            Some(fermante) => {
                sorti.push_str("[note: ");
                echapper(&mut sorti, &reste[..fermante]);
                sorti.push(']');
                reste = &reste[fermante + 1..];
            }
            None => break,
        }
    }
    echapper(&mut sorti, reste);
    sorti
}

fn echapper(sorti: &mut String, texte: &str) {
    for caractere in texte.chars() {
        match caractere {
            '&' => sorti.push_str("&amp;"),
            '<' => sorti.push_str("&lt;"),
            '>' => sorti.push_str("&gt;"),
            _ => sorti.push(caractere),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = "\
# Titre un\n\
\n\
Un paragraphe avec du texte et une note^[la note attachée] et une citation @dupont2023[p. 45].\n\
\n\
## Titre deux\n\
\n\
| Colonne A | Colonne B |\n\
| --- | --- |\n\
| cellule A1 | cellule B1 |\n\
\n\
![la légende de la figure](images/fig.png)\n";

    /// Le Markdown est le document canonique : il se relit au même
    /// document — c'est le contrat, pas une copie qui pourrait dériver.
    #[test]
    fn le_markdown_se_relit_au_meme_document() {
        let markdown = vers_markdown(FIXTURE);
        assert_eq!(
            parse(&markdown),
            parse(FIXTURE),
            "le Markdown exporté redonne le même document"
        );
    }

    /// La limite annoncée du DOCX, verrouillée : les notes deviennent du
    /// texte `[note: …]` — jamais perdues, jamais prétendues notes.
    #[test]
    fn les_notes_du_html_deviennent_du_texte_dit() {
        let html = vers_html(FIXTURE);
        assert!(html.contains("[note: la note attachée]"), "{html}");
        assert!(
            !html.contains("footnote"),
            "le HTML ne promet pas de note de bas de page"
        );
    }

    /// L'état d'erreur propose son remède : LibreOffice, ou Markdown.
    #[test]
    fn l_absence_de_libreoffice_dit_le_geste() {
        let message = message_absence();
        assert!(message.contains("LibreOffice"), "{message}");
        assert!(message.contains("Markdown"), "{message}");
    }
}
