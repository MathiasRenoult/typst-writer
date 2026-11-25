//! L'index des personnes citées : ses entrées, ses groupes et son rendu Typst.
//!
//! L'index est recalculé à partir du texte canonique : chaque balise
//! `{{p clé}}` est une mention explicite. Le registre `personnes.toml` fournit
//! le nom, les dates et la qualité ; il ne sert jamais à reconnaître un nom
//! dans la prose.
//!
//! Les pages ne sont pas calculées ici : Typst les connaît au moment de la
//! composition. Le rendu paginé pose donc des repères `tw-personne` autour des
//! mentions et laisse une `#context` les interroger.

use std::collections::BTreeMap;

use crate::mise_en_page::modele::{FormePersonne, IndexPersonnes as Configuration};
use crate::project::personnes::{EntreePersonne, Personnes};

/// Une entrée de l'index : une personne et son nombre de mentions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntreeIndexPersonne {
    /// La clé du registre, ou la clé brute quand le registre l'ignore.
    pub cle: String,
    /// Le nom affiché.
    pub nom: String,
    /// La clé de classement.
    pub tri: String,
    /// Les dates, si le registre les donne.
    pub dates: Option<String>,
    /// La qualité, si le registre la donne.
    pub qualite: Option<String>,
    /// Le nombre de mentions dans le document.
    pub occurrences: usize,
}

/// Un groupe alphabétique de l'index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupeIndexPersonne {
    /// L'initiale du groupe ; `None` pour les entrées qui n'en ont pas.
    pub initiale: Option<char>,
    /// Les entrées du groupe, déjà triées.
    pub entrees: Vec<EntreeIndexPersonne>,
}

/// L'index préparé pour un document.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IndexPersonnes {
    /// Les groupes alphabétiques, ou un seul groupe sans initiale quand le
    /// regroupement est désactivé.
    pub groupes: Vec<GroupeIndexPersonne>,
    /// Les pages sont-elles montrées ?
    pub afficher_pages: bool,
    /// Les entrées sont-elles groupées sous leur initiale ?
    pub grouper_par_initiale: bool,
    /// Le nombre de colonnes du rendu (1 à 4).
    pub colonnes: usize,
    /// Les dates sont-elles affichées ?
    pub afficher_dates: bool,
    /// La qualité est-elle affichée ?
    pub afficher_qualite: bool,
}

/// L'expression d'affichage écrite après `|` dans une balise de personne.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpressionPersonne {
    /// Une forme explicitement simple (`|simple`).
    Simple,
    /// Une forme nommée : `complet`, `simple` ou `alias`.
    Forme(FormePersonne),
    /// Une liste de champs : `nom+dates`, `naissance+mort`.
    Champs(Vec<ChampPersonne>),
    /// Les règles de première mention et de reprise des normes. C'est aussi
    /// le sens d'une balise sans expression.
    Auto,
    /// Un texte libre historique, affiché tel quel.
    Texte,
}

/// Un champ affichable dans une expression de personne.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChampPersonne {
    NomComplet,
    NomSimple,
    Alias,
    Naissance,
    Mort,
    Dates,
    Qualite,
}

impl ChampPersonne {
    /// Reconnaît un jeton de champ. Les synonymes restent courts et lisibles.
    fn depuis_jeton(jeton: &str) -> Option<Self> {
        match jeton {
            "nom" | "nom_complet" | "nom complet" | "complet" => Some(Self::NomComplet),
            "nom_simple" | "nom simple" | "nom court" | "simple" => Some(Self::NomSimple),
            "alias" | "plume" => Some(Self::Alias),
            "naissance" | "ne" | "date de naissance" => Some(Self::Naissance),
            "mort" | "deces" | "décès" | "date de mort" => Some(Self::Mort),
            "dates" | "date" | "dates de naissance et de mort" => Some(Self::Dates),
            "qualite" | "qualité" => Some(Self::Qualite),
            _ => None,
        }
    }
}

/// Interprète le corps d'une balise `{{p clé|…}}`.
///
/// La forme vide applique les règles de première mention et de reprise des
/// normes, comme `auto`. Les mots `complet`, `simple`, `alias` et `auto` sont
/// réservés ; une suite de jetons connus est une liste de champs ; tout le
/// reste demeure un texte libre, pour ne pas casser les anciennes balises.
pub fn expression_personne(contenu: &[super::schema::Inline]) -> ExpressionPersonne {
    if contenu.is_empty() {
        return ExpressionPersonne::Auto;
    }

    let texte = super::schema::inline_texte_nu(contenu);
    let texte = texte.trim();
    if texte.is_empty() {
        return ExpressionPersonne::Auto;
    }

    match texte.to_lowercase().as_str() {
        "complet" | "nom_complet" | "nom" => {
            return ExpressionPersonne::Forme(FormePersonne::Complet);
        }
        "simple" | "nom_simple" | "nom simple" | "nom court" => {
            return ExpressionPersonne::Simple;
        }
        "alias" | "plume" => return ExpressionPersonne::Forme(FormePersonne::Alias),
        "auto" => return ExpressionPersonne::Auto,
        _ => {}
    }

    let jetons: Vec<&str> = texte
        .split(['+', ',', ';'])
        .map(str::trim)
        .filter(|jeton| !jeton.is_empty())
        .collect();
    if !jetons.is_empty()
        && jetons
            .iter()
            .all(|jeton| ChampPersonne::depuis_jeton(&jeton.to_lowercase()).is_some())
    {
        return ExpressionPersonne::Champs(
            jetons
                .iter()
                .filter_map(|jeton| ChampPersonne::depuis_jeton(&jeton.to_lowercase()))
                .collect(),
        );
    }

    ExpressionPersonne::Texte
}

/// Compose les champs demandés par une expression explicite.
pub fn rendre_champs(
    champs: &[ChampPersonne],
    entree: &EntreePersonne,
    config: &Configuration,
) -> String {
    let mut morceaux = Vec::new();

    for champ in champs {
        match champ {
            ChampPersonne::NomComplet => morceaux.push(entree.nom_affiche().to_string()),
            ChampPersonne::NomSimple => morceaux.push(entree.nom_simple_affiche().to_string()),
            ChampPersonne::Alias => morceaux.push(entree.alias_affiche().to_string()),
            ChampPersonne::Naissance => {
                if let Some(date) = entree.naissance_affichee() {
                    morceaux.push(date.to_string());
                }
            }
            ChampPersonne::Mort => {
                if let Some(date) = entree.mort_affichee() {
                    morceaux.push(date.to_string());
                }
            }
            ChampPersonne::Dates => {
                if let Some(dates) = entree.dates_pour(config.dates_forme) {
                    morceaux.push(format!("({dates})"));
                }
            }
            ChampPersonne::Qualite => {
                if let Some(qualite) = entree.qualite_affichee() {
                    morceaux.push(qualite.to_string());
                }
            }
        }
    }

    morceaux.join(" ")
}

impl IndexPersonnes {
    /// Construit l'index à partir du document et du registre.
    pub fn construire(
        document: &super::schema::Document,
        personnes: &Personnes,
        config: &Configuration,
    ) -> Self {
        let texte = super::serialiser::serialiser(document);
        let mut occurrences: BTreeMap<String, usize> = BTreeMap::new();
        let mut reste = texte.as_str();

        // Le sérialiseur écrit toujours la forme courte `{{p clé|…}}`.
        while let Some(debut) = reste.find("{{p ") {
            let apres = &reste[debut + 4..];
            let Some(fin) = apres.find("}}") else {
                break;
            };

            let interieur = &apres[..fin];
            let (tete, _) = interieur.split_once('|').unwrap_or((interieur, ""));
            let cle = tete.trim();
            if EntreePersonne::cle_valide(cle) {
                *occurrences.entry(cle.to_string()).or_insert(0) += 1;
            }

            reste = &apres[fin + 2..];
        }

        let mut entrees: Vec<EntreeIndexPersonne> = occurrences
            .into_iter()
            .map(|(cle, occurrences)| {
                let entree = personnes.entree(&cle);
                EntreeIndexPersonne {
                    nom: entree
                        .map(|entree| entree.nom_pour(config.forme_index).to_string())
                        .unwrap_or_else(|| cle.clone()),
                    tri: entree
                        .map(|entree| entree.cle_de_tri().to_string())
                        .unwrap_or_else(|| cle.clone()),
                    dates: entree.and_then(|entree| entree.dates_pour(config.dates_forme)),
                    qualite: entree
                        .and_then(EntreePersonne::qualite_affichee)
                        .map(str::to_string),
                    cle,
                    occurrences,
                }
            })
            .collect();

        entrees.sort_by(|a, b| {
            a.tri
                .to_lowercase()
                .cmp(&b.tri.to_lowercase())
                .then_with(|| a.cle.cmp(&b.cle))
        });

        let groupes = if config.grouper_par_initiale {
            let mut groupes: Vec<GroupeIndexPersonne> = Vec::new();
            for entree in entrees {
                let initiale = initiale(&entree.tri);
                if let Some(groupe) = groupes.iter_mut().find(|g| g.initiale == initiale) {
                    groupe.entrees.push(entree);
                } else {
                    groupes.push(GroupeIndexPersonne {
                        initiale,
                        entrees: vec![entree],
                    });
                }
            }
            groupes
        } else {
            vec![GroupeIndexPersonne {
                initiale: None,
                entrees,
            }]
        };

        Self {
            groupes,
            afficher_pages: config.afficher_pages,
            grouper_par_initiale: config.grouper_par_initiale,
            colonnes: config.colonnes.clamp(1, 4),
            afficher_dates: config.afficher_dates,
            afficher_qualite: config.afficher_qualite,
        }
    }

    /// L'index est-il vide ?
    pub fn est_vide(&self) -> bool {
        self.groupes.iter().all(|groupe| groupe.entrees.is_empty())
    }

    /// Rend l'index en Typst.
    pub fn rendre_typst(&self) -> String {
        let mut out = String::from("= Index des personnes\n\n");

        if self.est_vide() {
            out.push_str("_Aucune personne citée._\n\n");
            return out;
        }

        let mut corps = String::new();
        if self.afficher_pages {
            corps.push_str("#context {\n");
            corps.push_str("  let mentions = query(<tw-personne>)\n");
            corps.push_str("  let notes = query(<tw-personne-note>)\n");
            corps.push_str("  let occurrences = key => mentions.filter(x => x.value == key)\n");
            corps.push_str("  let occurrences_notes = key => notes.filter(x => x.value == key)\n");
            corps.push_str(
                "  let pages = key => (occurrences(key).map(x => (x.location().page(), false)) + occurrences_notes(key).map(x => (x.location().page(), true))).sorted(key: p => p.at(0)).dedup().map(p => str(p.at(0)) + if p.at(1) { \"n\" } else { \"\" }).join(\", \")\n",
            );
            corps.push_str(
                "  let present = key => occurrences(key).len() + occurrences_notes(key).len()\n",
            );

            for groupe in &self.groupes {
                if let Some(initiale) = groupe.initiale {
                    corps.push_str(&format!("  [== {initiale}\n\n]\n"));
                }
                for entree in &groupe.entrees {
                    let cle = echapper_chaine_typst(&entree.cle);
                    corps.push_str(&format!(
                        "  [- {}#if present(\"{cle}\") > 0 [(p. #pages(\"{cle}\"))]]\n",
                        description(entree, self.afficher_dates, self.afficher_qualite)
                    ));
                }
                corps.push('\n');
            }
            corps.push_str("}\n\n");
        } else {
            for groupe in &self.groupes {
                if let Some(initiale) = groupe.initiale {
                    corps.push_str(&format!("== {initiale}\n\n"));
                }
                for entree in &groupe.entrees {
                    corps.push_str("- ");
                    corps.push_str(&description(
                        entree,
                        self.afficher_dates,
                        self.afficher_qualite,
                    ));
                    if entree.occurrences > 1 {
                        corps.push_str(&format!(" ({} mentions)", entree.occurrences));
                    }
                    corps.push('\n');
                }
                corps.push('\n');
            }
        }

        if self.colonnes > 1 {
            out.push_str(&format!("#columns({})[\n", self.colonnes));
            out.push_str(&corps);
            out.push_str("]\n\n");
        } else {
            out.push_str(&corps);
        }

        out
    }
}

/// La description d'une entrée : nom, dates et qualité.
fn description(entree: &EntreeIndexPersonne, dates: bool, qualite: bool) -> String {
    let mut description = echapper_contenu_typst(&entree.nom);

    if dates && let Some(dates) = &entree.dates {
        description.push_str(" (");
        description.push_str(&echapper_contenu_typst(dates));
        description.push(')');
    }
    if qualite && let Some(qualite) = &entree.qualite {
        description.push_str(" — ");
        description.push_str(&echapper_contenu_typst(qualite));
    }

    description
}

/// L'initiale d'un nom, sans accent et en capitale.
fn initiale(texte: &str) -> Option<char> {
    let premier = texte
        .trim()
        .trim_start_matches(|c: char| !c.is_alphanumeric())
        .chars()
        .next()?;

    Some(premier.to_uppercase().next().unwrap_or(premier))
}

/// Les repères Typst posés auprès d'une mention. Un repère par mention, même
/// dans une note.
pub fn reperes_typst(cle: &str, dans_note: bool) -> String {
    let label = if dans_note {
        "tw-personne-note"
    } else {
        "tw-personne"
    };
    format!("#metadata(\"{}\") <{label}>", echapper_chaine_typst(cle))
}

/// Échappe une chaîne pour un littéral Typst entre guillemets.
fn echapper_chaine_typst(texte: &str) -> String {
    texte.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Échappe le texte d'une entrée pour qu'il reste du texte Typst.
fn echapper_contenu_typst(texte: &str) -> String {
    let mut sortie = String::new();
    for caractere in texte.chars() {
        if matches!(
            caractere,
            '\\' | '#' | '*' | '_' | '`' | '$' | '@' | '^' | '<' | '>' | '[' | ']' | '~' | '|'
        ) {
            sortie.push('\\');
        }
        sortie.push(caractere);
    }
    sortie
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mise_en_page::modele::IndexPersonnes as Configuration;

    fn registre() -> Personnes {
        let mut personnes = Personnes::default();
        personnes.definir(EntreePersonne {
            cle: "augustin".into(),
            nom: "Augustin d'Hippone".into(),
            nom_simple: Some("Augustin".into()),
            alias: Some("Saint Augustin".into()),
            tri: Some("Augustin".into()),
            dates: Some("354-430".into()),
            naissance: Some("354".into()),
            mort: Some("430".into()),
            qualite: Some("évêque d'Hippone".into()),
            ..Default::default()
        });
        personnes.definir(EntreePersonne {
            cle: "aquin".into(),
            nom: "Thomas d'Aquin".into(),
            tri: Some("Aquin".into()),
            dates: Some("1225-1274".into()),
            ..Default::default()
        });
        personnes
    }

    #[test]
    fn les_mentions_explicites_sont_comptees_et_triees() {
        let document = crate::langage::parse::parse(
            "{{p augustin}} puis {{p augustin|Augustin}} et {{p aquin}}.\n",
        );
        let index = IndexPersonnes::construire(&document, &registre(), &Configuration::default());

        assert_eq!(index.groupes.len(), 1);
        assert_eq!(index.groupes[0].initiale, Some('A'));
        assert_eq!(index.groupes[0].entrees[0].cle, "aquin");
        assert_eq!(index.groupes[0].entrees[0].nom, "Thomas d'Aquin");
        assert_eq!(index.groupes[0].entrees[1].cle, "augustin");
        assert_eq!(index.groupes[0].entrees[1].occurrences, 2);
    }

    #[test]
    fn les_pages_se_demandent_a_typst() {
        let document = crate::langage::parse::parse("{{p augustin}}\n\n{{index-personnes}}\n");
        let index = IndexPersonnes::construire(&document, &registre(), &Configuration::default());
        let typst = index.rendre_typst();

        assert!(typst.contains("#context {"), "{typst}");
        assert!(typst.contains("query(<tw-personne>)"), "{typst}");
        assert!(typst.contains("Augustin d'Hippone"), "{typst}");
        assert!(typst.contains("(p. #pages(\"augustin\"))"), "{typst}");
    }

    #[test]
    fn sans_pages_l_index_reste_statique() {
        let document = crate::langage::parse::parse("{{p augustin}}\n\n{{p augustin}}\n");
        let config = Configuration {
            afficher_pages: false,
            ..Default::default()
        };
        let index = IndexPersonnes::construire(&document, &registre(), &config);
        let typst = index.rendre_typst();

        assert!(!typst.contains("#context"), "{typst}");
        assert!(typst.contains("- Augustin d'Hippone (354-430)"), "{typst}");
        assert!(typst.contains("(2 mentions)"), "{typst}");
    }

    #[test]
    fn sans_regroupement_une_seule_liste_suffit() {
        let document = crate::langage::parse::parse("{{p augustin}} {{p aquin}}\n");
        let config = Configuration {
            grouper_par_initiale: false,
            ..Default::default()
        };
        let index = IndexPersonnes::construire(&document, &registre(), &config);
        let typst = index.rendre_typst();

        assert!(!typst.contains("== A"), "{typst}");
        assert!(typst.contains("Augustin d'Hippone"), "{typst}");
        assert!(typst.contains("Thomas d'Aquin"), "{typst}");
    }

    #[test]
    fn l_index_suit_la_forme_du_nom_et_des_dates() {
        let document = crate::langage::parse::parse("{{p augustin}}\n");
        let config = Configuration {
            forme_index: crate::mise_en_page::modele::FormePersonne::Simple,
            dates_forme: crate::mise_en_page::modele::FormeDatesPersonne::Mort,
            ..Default::default()
        };
        let index = IndexPersonnes::construire(&document, &registre(), &config);
        let entree = &index.groupes[0].entrees[0];

        assert_eq!(entree.nom, "Augustin");
        assert_eq!(entree.dates.as_deref(), Some("430"));
        let typst = index.rendre_typst();
        assert!(typst.contains("Augustin (430)"), "{typst}");
        assert!(!typst.contains("Augustin d'Hippone"), "{typst}");
    }

    #[test]
    fn une_cle_inconnue_reste_visible_dans_l_index() {
        let document = crate::langage::parse::parse("{{p inconnu}}\n");
        let index =
            IndexPersonnes::construire(&document, &Personnes::default(), &Configuration::default());

        assert_eq!(index.groupes[0].entrees[0].nom, "inconnu");
        assert_eq!(index.groupes[0].entrees[0].cle, "inconnu");
    }
}
