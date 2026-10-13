use std::path::{Path, PathBuf};
use std::process::Command;

/// Compile un document Typst en PDF vers une destination choisie.
pub fn compile_to(input: &Path, output: &Path) -> Result<PathBuf, String> {
    run(input, output)?;

    Ok(output.to_path_buf())
}

fn run(input: &Path, output: &Path) -> Result<(), String> {
    if !input.exists() {
        return Err(format!("Le fichier n'existe pas : {}", input.display()));
    }

    let result = Command::new("typst")
        .arg("compile")
        .arg(input)
        .arg(output)
        .output()
        .map_err(|error| format!("Impossible de lancer typst : {error}"))?;

    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        let stdout = String::from_utf8_lossy(&result.stdout);

        return Err(format!(
            "Erreur Typst :\nstderr: {stderr}\nstdout: {stdout}"
        ));
    }

    if !output.exists() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        let stdout = String::from_utf8_lossy(&result.stdout);

        return Err(format!(
            "Typst a réussi mais n'a pas généré de PDF.\nstdout: {stdout}\nstderr: {stderr}"
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Vérifie que l'export produit bien un PDF valide à la destination choisie.
    #[test]
    fn compiles_to_chosen_destination() {
        let dir = std::env::temp_dir().join("typst-writer-export-test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let source = dir.join("document.typ");
        fs::write(&source, "= Titre\n\nDu contenu.\n").unwrap();

        let output = dir.join("ailleurs.pdf");
        let result = compile_to(&source, &output).expect("compilation");

        assert_eq!(result, output, "le PDF doit être à la destination choisie");

        let bytes = fs::read(&output).unwrap();
        assert!(
            bytes.starts_with(b"%PDF"),
            "le fichier exporté doit être un PDF valide"
        );

        let _ = fs::remove_dir_all(&dir);
    }
}
