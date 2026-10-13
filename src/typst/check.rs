//! Vérification du document par le compilateur Typst officiel.
//!
//! Contrairement au parseur `typst-syntax` — qui ne fait que de l'analyse
//! syntaxique (délimiteurs, structure du markup) — le binaire `typst compile`
//! évalue réellement le document : fonctions inconnues, types incorrects,
//! arguments manquants, imports introuvables... Ses erreurs sont remontées en
//! direct pour l'indicateur de la barre d'état et le bandeau du panneau de
//! prévisualisation, avec un lien vers la ligne concernée.
//!
//! La vérification écrit un fichier de travail caché dans le même dossier que
//! le document actif : les imports et images relatifs s'y résolvent comme à
//! l'export, donc les erreurs signalées sont exactement celles de l'export.

use std::path::Path;
use std::process::Command;

/// Nom du fichier de travail utilisé pour la vérification.
const CHECK_SOURCE: &str = ".typst-writer-check.typ";

/// Nom du PDF de vérification (écrasé à chaque passe).
const CHECK_PDF: &str = ".typst-writer-check.pdf";

/// Un diagnostic du compilateur (erreur ou avertissement).
#[derive(Debug, Clone)]
pub struct CheckError {
    pub message: String,

    /// Ligne du diagnostic (1-based), si le compilateur l'a précisée.
    pub line: Option<usize>,

    /// Colonne du diagnostic (1-based), si le compilateur l'a précisée.
    pub column: Option<usize>,
}

/// Résultat d'une vérification.
#[derive(Debug, Clone, Default)]
pub struct CheckOutcome {
    /// Contenu vérifié : permet de ne relancer que si le document a changé.
    pub content: String,

    pub errors: Vec<CheckError>,
    pub warnings: Vec<CheckError>,
}

/// Vérifie un contenu dans le dossier donné (celui du document actif).
pub fn check(dir: &Path, content: &str) -> CheckOutcome {
    let source = dir.join(CHECK_SOURCE);
    let pdf = dir.join(CHECK_PDF);

    let write_error = |message: String| CheckOutcome {
        content: content.to_string(),
        errors: vec![CheckError {
            message,
            line: None,
            column: None,
        }],
        warnings: Vec::new(),
    };

    // Le dossier peut ne pas encore exister (document jamais sauvé).
    if let Err(error) = std::fs::create_dir_all(dir) {
        return write_error(format!(
            "Impossible de créer le dossier de vérification : {error}"
        ));
    }

    if let Err(error) = std::fs::write(&source, content) {
        return write_error(format!(
            "Impossible d'écrire le fichier de vérification : {error}"
        ));
    }

    let output = match Command::new("typst")
        .arg("compile")
        .arg(&source)
        .arg(&pdf)
        .output()
    {
        Ok(output) => output,
        Err(error) => {
            return write_error(format!("Impossible de lancer typst : {error}"));
        }
    };

    let stderr = String::from_utf8_lossy(&output.stderr);
    let (errors, warnings) = parse_diagnostics(&stderr);

    if errors.is_empty() && warnings.is_empty() && !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);

        return write_error(format!("La compilation a échoué :\n{stderr}{stdout}"));
    }

    CheckOutcome {
        content: content.to_string(),
        errors,
        warnings,
    }
}

/// Extrait les diagnostics de la sortie du compilateur.
///
/// Format Typst :
/// ```text
/// error: unknown variable: foo
///   ┌─ .typst-writer-check.typ:3:1
///   │
/// 3 │ #foo()
///   │  ^^^
/// ```
fn parse_diagnostics(text: &str) -> (Vec<CheckError>, Vec<CheckError>) {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut pending: Option<(bool, String, Option<(usize, usize)>)> = None;

    let mut flush = |pending: &mut Option<(bool, String, Option<(usize, usize)>)>| {
        if let Some((is_error, message, position)) = pending.take() {
            let diagnostic = CheckError {
                message,
                line: position.map(|(line, _)| line),
                column: position.map(|(_, column)| column),
            };

            if is_error {
                errors.push(diagnostic);
            } else {
                warnings.push(diagnostic);
            }
        }
    };

    for line in text.lines() {
        if let Some(message) = line.strip_prefix("error: ") {
            flush(&mut pending);
            pending = Some((true, message.to_string(), None));
        } else if let Some(message) = line.strip_prefix("warning: ") {
            flush(&mut pending);
            pending = Some((false, message.to_string(), None));
        } else if let Some(position) = parse_position(line) {
            if let Some((_, _, slot)) = pending.as_mut() {
                *slot = Some(position);
            }
        }
    }

    flush(&mut pending);

    (errors, warnings)
}

/// Extrait `(ligne, colonne)` d'une ligne `┌─ fichier:ligne:colonne`.
fn parse_position(line: &str) -> Option<(usize, usize)> {
    let after = line.split("┌─ ").nth(1)?;

    let mut parts = after.rsplitn(3, ':');
    let column = parts.next()?.trim().parse().ok()?;
    let line = parts.next()?.trim().parse().ok()?;

    Some((line, column))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn parses_compiler_diagnostics() {
        let stderr = "\
error: unknown variable: foo
  ┌─ .typst-writer-check.typ:3:1
  │
3 │ #foo()
  │  ^^^

warning: unused label
  ┌─ .typst-writer-check.typ:7:5
  │";

        let (errors, warnings) = parse_diagnostics(stderr);

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "unknown variable: foo");
        assert_eq!(errors[0].line, Some(3));
        assert_eq!(errors[0].column, Some(1));

        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].message, "unused label");
        assert_eq!(warnings[0].line, Some(7));
        assert_eq!(warnings[0].column, Some(5));
    }

    /// Le vrai compilateur détecte les erreurs sémantiques que le parseur
    /// syntaxique ne voit pas (fonction inconnue).
    #[test]
    fn real_compiler_reports_semantic_errors() {
        let dir = std::env::temp_dir().join("typst-writer-check-test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let outcome = check(&dir, "= Titre\n\n#foo()\n");

        assert_eq!(outcome.errors.len(), 1, "une erreur sémantique attendue");
        assert_eq!(outcome.errors[0].message, "unknown variable: foo");
        assert_eq!(outcome.errors[0].line, Some(3));

        let _ = fs::remove_dir_all(&dir);
    }

    /// Un document valide passe sans erreur ni avertissement.
    #[test]
    fn clean_document_passes() {
        let dir = std::env::temp_dir().join("typst-writer-check-ok");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let outcome = check(&dir, "= Titre\n\nDu contenu valide.\n");

        assert!(outcome.errors.is_empty(), "aucune erreur attendue");
        assert!(outcome.warnings.is_empty(), "aucun avertissement attendu");

        let _ = fs::remove_dir_all(&dir);
    }
}
