use std::path::{Path, PathBuf};
use std::process::Command;

/// Rend chaque page du PDF en image PNG via `pdftoppm` (poppler).
///
/// Les fichiers produits sont nommés `preview-{counter}-{page}.png` dans le
/// même dossier que le PDF : un compteur unique par compilation garantit que
/// le widget image d'Iced recharge bien la nouvelle image (le cache d'images
/// d'Iced est indexé par le chemin du fichier).
///
/// Retourne la liste ordonnée des pages générées.
pub fn render_pdf_to_png(pdf: &Path, counter: usize) -> Result<Vec<PathBuf>, String> {
    let dir = pdf.parent().unwrap_or_else(|| Path::new("."));
    let prefix = dir.join(format!("preview-{counter}"));

    let output = Command::new("pdftoppm")
        .arg("-png")
        .arg("-r")
        .arg("120")
        .arg(pdf)
        .arg(&prefix)
        .output()
        .map_err(|error| format!("Impossible de lancer pdftoppm : {error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        return Err(format!("pdftoppm a échoué : {stderr}"));
    }

    let mut pages = collect_pages(dir, counter);
    pages.sort_by_key(|path| page_number(path, counter).unwrap_or(usize::MAX));
    cleanup_old_previews(dir, counter);

    if pages.is_empty() {
        return Err("pdftoppm n'a produit aucune image".to_string());
    }

    Ok(pages)
}

/// Liste les images `preview-{counter}-{page}.png` du dossier.
fn collect_pages(dir: &Path, counter: usize) -> Vec<PathBuf> {
    let prefix = format!("preview-{counter}-");

    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };

    entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension().and_then(|ext| ext.to_str()) == Some("png")
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with(&prefix))
        })
        .collect()
}

/// Supprime les images de compilations précédentes du dossier.
fn cleanup_old_previews(dir: &Path, counter: usize) {
    let current = format!("preview-{counter}-");

    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for path in entries.flatten().map(|entry| entry.path()) {
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };

        if name.starts_with("preview-") && name.ends_with(".png") && !name.starts_with(&current) {
            let _ = std::fs::remove_file(&path);
        }
    }
}

/// Extrait le numéro de page d'un nom `preview-{counter}-{page}.png`.
fn page_number(path: &Path, counter: usize) -> Option<usize> {
    let name = path.file_name()?.to_str()?;
    let digits = name
        .strip_prefix(&format!("preview-{counter}-"))?
        .strip_suffix(".png")?;

    digits.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Vérifie que le pipeline de rendu produit bien une image PNG par page
    /// et que le tri des pages est correct (y compris au-delà de 9 pages).
    #[test]
    fn renders_pdf_pages_in_order() {
        let dir = std::env::temp_dir().join("typst-writer-preview-test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        // Document de 11 pages (sauts explicites) pour vérifier le tri
        // des images au-delà de 9 pages.
        let mut content = String::from("#set page(paper: \"a6\")\n");
        for i in 1..=11 {
            if i > 1 {
                content.push_str("#pagebreak()\n");
            }
            content.push_str(&format!("= Page {i}\n\nContenu de la page {i}.\n"));
        }

        let source = dir.join("test.typ");
        fs::write(&source, content).unwrap();

        let pdf = crate::typst::compiler::compile(&source).expect("compilation typst");
        let pages = render_pdf_to_png(&pdf, 0).expect("rendu pdftoppm");

        assert_eq!(pages.len(), 11, "une image par page");
        assert_eq!(page_number(&pages[0], 0), Some(1));
        assert_eq!(page_number(&pages[9], 0), Some(10));
        assert_eq!(page_number(&pages[10], 0), Some(11));

        let bytes = fs::read(&pages[0]).unwrap();
        assert!(
            bytes.starts_with(b"\x89PNG"),
            "le rendu doit être un PNG valide"
        );

        // Une seconde compilation avec un nouveau compteur doit remplacer
        // les images de la précédente.
        let pages = render_pdf_to_png(&pdf, 1).expect("second rendu");
        assert_eq!(pages.len(), 11);
        assert!(!dir.join("preview-0-1.png").exists());

        let _ = fs::remove_dir_all(&dir);
    }
}
