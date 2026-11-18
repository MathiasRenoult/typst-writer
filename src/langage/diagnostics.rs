//! Le modèle des diagnostics : ce que le document, le pipeline, le
//! moteur et le PDF ont à dire à l'auteur.
//!
//! Il vivait chez le moteur Typst alors que son producteur majoritaire est
//! le pipeline (feuille de route, phase 1, point 1.3).
use std::path::PathBuf;

/// L'origine d'un diagnostic : le chemin du document jusqu'au PDF, étape
/// par étape. La feuille de route (phase 1, point 1.2) veut les cinq
/// séparées — jamais une seule liste muette.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origine {
    /// La lecture du `.tw` : son analyse, sa migration, ses directives.
    Analyse,
    /// La transformation du `.tw` en code Typst.
    Transformation,
    /// La compilation du code Typst.
    CompilationTypst,
    /// La production du fichier PDF.
    ExportPdf,
    /// Le rendu des pages du PDF à l'écran.
    RenduPdf,
}

impl Origine {
    /// Le nom que l'interface dit à l'auteur.
    pub fn libelle(&self) -> &'static str {
        match self {
            Origine::Analyse => "analyse du .tw",
            Origine::Transformation => "transformation en Typst",
            Origine::CompilationTypst => "compilation Typst",
            Origine::ExportPdf => "export PDF",
            Origine::RenduPdf => "rendu PDF",
        }
    }
}

/// Un diagnostic (erreur ou avertissement) positionné dans le document.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub message: String,

    /// Ligne du diagnostic (1-based), si le compilateur l'a précisée.
    pub line: Option<usize>,

    /// Colonne du diagnostic (1-based), si le compilateur l'a précisée.
    pub column: Option<usize>,

    /// L'étape du chemin documentaire qui l'a produit.
    pub origine: Origine,

    /// Le fichier où se situe le diagnostic, résolu sur le disque.
    ///
    /// Le document compilé est l'assemblage, qui `#include` les chapitres : un
    /// diagnostic peut donc tomber dans n'importe lequel. Sans ce champ, une
    /// erreur dans un chapitre serait signalée sans qu'on sache où la
    /// corriger — et « sauter à l'erreur » n'aurait aucun sens.
    pub file: Option<PathBuf>,
}

impl Diagnostic {
    /// Ce diagnostic concerne-t-il le document actuellement ouvert ?
    ///
    /// Un diagnostic sans fichier (erreur de rendu PDF, avertissement global)
    /// est rattaché au document courant : c'est celui que l'écran montre. Un
    /// diagnostic venu d'un chapitre inclus ne concerne que ce chapitre.
    pub fn concerne(&self, ouvert: Option<&std::path::Path>) -> bool {
        match (&self.file, ouvert) {
            (Some(fichier), Some(ouvert)) => crate::project::same_file(fichier, ouvert),
            (Some(_), None) => false,
            (None, _) => true,
        }
    }

    /// Le diagnostic vient-il du fichier généré ? Alors il parle du
    /// document source : le générateur garde les numéros de ligne
    /// (feuille de route, phase 2, point 2.5), et la position reste
    /// valable dans le `.tw` que l'auteur écrit.
    pub fn vers_la_source(self, source: &std::path::Path) -> Self {
        let est_genere = self
            .file
            .as_deref()
            .and_then(|f| f.file_name())
            .is_some_and(|n| n == crate::langage::pipeline::GENERATED_FILE);
        if est_genere {
            Diagnostic {
                file: Some(source.to_path_buf()),
                ..self
            }
        } else {
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// La correspondance de positions (feuille de route, phase 2, point 2.5) :
    /// le diagnostic du fichier généré parle du document source, aux mêmes
    /// numéros de ligne.
    #[test]
    fn le_du_genere_parle_du_source() {
        let genere = std::path::PathBuf::from("/projet/document.generated.typ");
        let source = std::path::PathBuf::from("/projet/chapitres/01.tw");
        let diagnostic = Diagnostic {
            origine: Origine::CompilationTypst,
            message: "quelque chose".to_string(),
            line: Some(4),
            column: Some(2),
            file: Some(genere),
        }
        .vers_la_source(&source);
        assert_eq!(
            diagnostic.file,
            Some(source.clone()),
            "le fichier est celui de l'auteur"
        );
        assert_eq!(diagnostic.line, Some(4), "la ligne tient");

        // Un diagnostic d'un vrai fichier le garde.
        let biblio = std::path::PathBuf::from("/projet/locale.bib");
        let diagnostic = Diagnostic {
            origine: Origine::Transformation,
            message: "autre".to_string(),
            line: None,
            column: None,
            file: Some(biblio.clone()),
        }
        .vers_la_source(&source);
        assert_eq!(
            diagnostic.file,
            Some(biblio),
            "un vrai fichier reste le sien"
        );
    }
}
