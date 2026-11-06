//! Le fichier `.tw` transportable : un seul fichier suffit au transfert.
//!
//! C'est la demande 3 de `bug-report-7.md` : « un fichier `.tw` unique
//! soit suffisant pour effectuer le transfert » — bibliographie, normes de
//! citation et paramètres du document **embarqués**, et, à partir du
//! seul `.tw`, tout reconstruire : ce que l'éditeur montre, ce que le
//! rendu compose.
//!
//! ## La structure
//!
//! Le document reste le texte d'always : c'est ce que l'éditeur montre,
//! au début du fichier, sans changement pour qui n'utilise pas le
//! transportable. Les pièces embarquées suivent, dans une **clôture**
//! nommée — ligne `~~~~ tw-portable:N`, sections `@nom`, ligne `~~~~` :
//!
//! ```text
//! = Mon document
//!
//! Du texte, comme toujours.
//!
//! ~~~~ tw-portable:1
//! @meta
//! titre: Ma thèse
//! auteur: Alice
//! @normes
//! <le TOML de mise-en-page.toml>
//! @bibliographie
//! @book{dupont2023, ...}
//! ~~~~
//! ```
//!
//! ## La rétrocompatibilité
//!
//! Un `.tw` sans clôture est un `.tw` d'**avant** : [`lire`] rend le
//! texte entier en `document`, aucune pièce embarquée, et rien ne bouge.
//! Un fichier écrit par une version future porte son numéro ; une
//! version qui ne connaît pas un numéro supérieur **refuse d'écrire**
//! par-dessus (elle lit ce qu'elle connaît) plutôt que de perdre les
//! pièces qu'elle ne voit pas — c'est la règle de toute la maison : une
//! donnée ne se perd jamais.

/// La version du format écrite par cette version de l'app.
pub const VERSION: u32 = 2;

/// La clôture : `~~~~ tw-portable:1` ouvre, `~~~~` ferme.
const OUVERTURE: &str = "~~~~ tw-portable:";
const FERMETURE: &str = "~~~~";

/// Le fichier transportable : le document et ses pièces.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FichierPortable {
    /// La version de la clôture trouvée — 0 s'il n'y en a pas.
    pub version: u32,
    /// Le document : ce que l'éditeur montre.
    pub document: String,
    /// Les métadonnées, une paire clé → valeur par ligne — dans l'ordre
    /// de l'auteur.
    pub meta: Vec<(String, String)>,
    /// Les normes de mise en page, en TOML — `mise-en-page.toml`.
    pub normes: String,
    /// La bibliographie, en BibTeX — `references.bib`.
    pub bibliographie: String,
    /// Le glossaire, en TOML — `glossaire.toml`.
    pub glossaire: String,
}

impl FichierPortable {
    /// Lit un `.tw` : la clôture si elle est là, le texte entier sinon.
    pub fn lire(texte: &str) -> Self {
        let Some((document, version, corps)) = diviser(texte) else {
            return Self {
                document: texte.to_string(),
                ..Default::default()
            };
        };

        let mut document = document.trim_end().to_string();
        if !document.is_empty() {
            document.push('\n');
        }
        let mut fichier = Self {
            version,
            document,
            ..Default::default()
        };

        let mut section = String::new();
        for ligne in corps.lines() {
            // Le corps est indenté de deux — c'est ce qui le fait
            // **replier** par le mécanisme d'indentation de l'éditeur
            // (jalon 3). On retire l'enveloppe, pas davantage :
            // l'indentation interne des notices (BibTeX) se garde.
            let ligne = ligne.strip_prefix("  ").unwrap_or(ligne);

            // Un `@` n'ouvre une section que s'il nomme une section
            // connue : `@book{…}` en bibliographie est une notice, pas un
            // changement de rubrique.
            if let Some(nom) = ligne.strip_prefix('@')
                && matches!(
                    nom.trim(),
                    "meta" | "normes" | "bibliographie" | "glossaire"
                )
            {
                section = nom.trim().to_string();
                continue;
            }
            match section.as_str() {
                "meta" => {
                    if let Some((cle, valeur)) = ligne.split_once(':') {
                        let cle = cle.trim().to_string();
                        // La première valeur gagne : les clés ne se
                        // doublent pas, un doublon est une erreur de main.
                        if !fichier.meta.iter().any(|(c, _)| c == &cle) {
                            fichier.meta.push((cle, valeur.trim().to_string()));
                        }
                    }
                }
                "normes" => {
                    fichier.normes.push_str(ligne);
                    fichier.normes.push('\n');
                }
                "bibliographie" => {
                    fichier.bibliographie.push_str(ligne);
                    fichier.bibliographie.push('\n');
                }
                "glossaire" => {
                    fichier.glossaire.push_str(ligne);
                    fichier.glossaire.push('\n');
                }
                _ => {}
            }
        }

        fichier
    }

    /// Écrit le `.tw` : le document, puis la clôture.
    pub fn ecrire(&self) -> String {
        let mut out = self.document.clone();
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out.push('\n');
        out.push_str(OUVERTURE);
        out.push_str(&VERSION.to_string());
        out.push('\n');

        // Le corps est indenté : c'est la règle de **repli** de l'éditeur
        // (une région s'ouvre quand la ligne suivante descend) — la
        // clôture se replie d'un clic, sans rien changer au vendor.
        if !self.meta.is_empty() {
            out.push_str("  @meta\n");
            for (cle, valeur) in &self.meta {
                out.push_str("  ");
                out.push_str(cle);
                out.push_str(": ");
                out.push_str(valeur);
                out.push('\n');
            }
        }
        if !self.normes.trim().is_empty() {
            out.push_str("  @normes\n");
            for ligne in self.normes.trim_end().lines() {
                out.push_str("  ");
                out.push_str(ligne);
                out.push('\n');
            }
        }
        if !self.bibliographie.trim().is_empty() {
            out.push_str("  @bibliographie\n");
            for ligne in self.bibliographie.trim_end().lines() {
                out.push_str("  ");
                out.push_str(ligne);
                out.push('\n');
            }
        }
        if !self.glossaire.trim().is_empty() {
            out.push_str("  @glossaire\n");
            for ligne in self.glossaire.trim_end().lines() {
                out.push_str("  ");
                out.push_str(ligne);
                out.push('\n');
            }
        }

        out.push_str(FERMETURE);
        out.push('\n');
        out
    }

    /// Un fichier écrit par le futur : on lit, mais on n'écrira pas
    /// par-dessus — les pièces qu'on ne voit pas ne se perdent pas.
    pub fn version_inconnue(&self) -> bool {
        self.version > VERSION
    }
}

/// Sépare le document de la clôture et en lit la version — `None` s'il
/// n'y en a pas.
fn diviser(texte: &str) -> Option<(&str, u32, &str)> {
    let debut = texte.find(OUVERTURE)?;
    let apres = &texte[debut + OUVERTURE.len()..];
    let fin_ligne = apres.find('\n')?;
    let version = apres[..fin_ligne].trim().parse().unwrap_or(0);
    let corps = &apres[fin_ligne + 1..];
    let fin = corps.find(FERMETURE)?;
    Some((&texte[..debut], version, &corps[..fin]))
}

#[cfg(test)]
mod tests {
    use super::FichierPortable;

    /// Le va-et-vient : ce qui s'écrit se relit, pièce pour pièce.
    #[test]
    fn un_fichier_transportable_fait_l_aller_et_le_retour() {
        let mut fichier = FichierPortable {
            document: "= Mon document\n\nDu texte.\n".to_string(),
            normes: "page_de_titre = true\n".to_string(),
            bibliographie: "@book{dupont2023,\n  title = {Un titre},\n}\n".to_string(),
            ..Default::default()
        };
        fichier
            .meta
            .push(("titre".to_string(), "Ma thèse".to_string()));

        let ecrit = fichier.ecrire();
        let relu = FichierPortable::lire(&ecrit);

        assert_eq!(relu.document, fichier.document, "le document se voit");
        assert_eq!(relu.meta, fichier.meta, "les métadonnées suivent");
        assert_eq!(relu.normes, fichier.normes, "les normes suivent");
        assert_eq!(
            relu.bibliographie, fichier.bibliographie,
            "la bibliographie suit"
        );
        assert!(!relu.version_inconnue(), "la version est connue");
    }

    /// Un `.tw` d'avant la clôture : le texte entier est le document, et
    /// rien ne bouge — la rétrocompatibilité est la règle.
    #[test]
    fn un_fichier_d_avant_lit_sans_bouger() {
        let vieux = "= Titre\n\nDu texte avec @dupont2023 et ^[une note].\n";
        let fichier = FichierPortable::lire(vieux);
        assert_eq!(fichier.document, vieux, "le texte entier est le document");
        assert_eq!(fichier.version, 0, "pas de clôture, pas de version");
        assert!(fichier.meta.is_empty(), "aucune pièce");
    }

    /// Les `@` de la bibliographie (`@book{…}`) ne sont pas des
    /// sections : seul un nom de section connu change de rubrique.
    #[test]
    fn les_arobases_de_la_bibliographie_ne_sont_pas_des_sections() {
        let texte = "Du texte.\n\n~~~~ tw-portable:1\n@bibliographie\n@book{ouvrage,\n  title = {Un},\n}\n~~~~\n";
        let fichier = FichierPortable::lire(texte);
        assert!(
            fichier.bibliographie.contains("@book{ouvrage"),
            "la notice est là"
        );
        assert!(fichier.normes.is_empty(), "pas de fuite entre sections");
    }

    /// Un fichier du futur : on lit, mais on refuse d'écrire par-dessus.
    #[test]
    fn le_futur_ne_se_perd_pas() {
        let futur = "Du texte.\n\n~~~~ tw-portable:99\n@meta\ntitre: Futur\n@inconnue\nune pièce que cette version ne voit pas\n~~~~\n";
        let fichier = FichierPortable::lire(futur);
        assert!(fichier.version_inconnue(), "on sait qu'on ne sait pas tout");
    }
}

#[cfg(test)]
mod tests_repli {
    use super::FichierPortable;

    /// La clôture a la forme que le mécanisme de **repli** de l'éditeur
    /// attend : une ligne d'en-tête à la colonne zéro, un corps plus
    /// profond, une fermeture à la colonne zéro — la région se replie
    /// d'un clic, sans rien changer au vendor (jalon 3).
    #[test]
    fn la_cloture_se_replie_par_indentation() {
        let mut fichier = FichierPortable {
            document: "Du texte.\n".to_string(),
            normes: "page_de_titre = true\n".to_string(),
            ..Default::default()
        };
        fichier
            .meta
            .push(("titre".to_string(), "Ma thèse".to_string()));

        let ecrit = fichier.ecrire();
        let lignes: Vec<&str> = ecrit.lines().collect();
        let entete = lignes
            .iter()
            .position(|l| l.starts_with("~~~~ tw-portable:"))
            .expect("l'en-tête de clôture");
        let fermeture = lignes
            .iter()
            .rposition(|l| *l == "~~~~")
            .expect("la fermeture");

        assert_eq!(
            lignes[entete].len() - lignes[entete].trim_start().len(),
            0,
            "l'en-tête est à la colonne zéro"
        );
        assert_eq!(
            lignes[fermeture].len() - lignes[fermeture].trim_start().len(),
            0,
            "la fermeture est à la colonne zéro"
        );
        assert!(
            lignes[entete + 1..fermeture]
                .iter()
                .all(|l| l.is_empty() || l.starts_with(' ')),
            "le corps est plus profond — c'est la région de repli"
        );
    }
}
