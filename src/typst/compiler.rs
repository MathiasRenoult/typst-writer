use std::path::PathBuf;
use std::process::Command;

pub fn compile(input: &PathBuf) -> Result<PathBuf, String> {
    if !input.exists() {
        return Err(format!("Le fichier n'existe pas : {}", input.display()));
    }

    let output = Command::new("typst")
        .arg("compile")
        .arg(input)
        .output()
        .map_err(|error| format!("Impossible de lancer typst : {error}"))?;

    if output.status.success() {
        let pdf = input.with_extension("pdf");

        if pdf.exists() {
            return Ok(pdf);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        return Err(format!(
            "Typst a réussi mais n'a pas généré de PDF.\nstdout: {stdout}\nstderr: {stderr}"
        ));
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    Err(format!(
        "Erreur Typst :\nstderr: {stderr}\nstdout: {stdout}"
    ))
}
