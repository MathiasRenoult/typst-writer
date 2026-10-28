//! Le Document structuré : la source de vérité du langage intermédiaire.
//!
//! L'éditeur enrichi ne manipule pas du texte brut, mais une suite de blocs
//! dont le texte est une suite d'éléments inline. Ce document est la vérité ;
//! le texte sur disque n'est qu'une sérialisation (voir `parse` et `generer`).
//!
//! Deux invariants gouvernent ce schéma :
//!
//! 1. **Round-trip sans perte** : `parse(generer(doc)) == doc`. Le même texte
//!    donne toujours le même Document, et le même Document toujours le même
//!    texte. C'est ce qui rend le fichier lisible à la main et diffable.
//! 2. **Le Typst reste le maître du rendu final.** Tout ce que le langage ne
//!    sait pas exprimer passe par un bloc brut Typst, inséré tel quel dans le
//!    document généré.

/// Un document : une suite ordonnée de blocs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub blocs: Vec<Bloc>,
}

/// Un bloc de niveau document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Bloc {
    /// Un titre de niveau 1 à 6.
    Titre {
        niveau: u8,
        /// Le texte du titre, en éléments inline.
        contenu: Vec<Inline>,
    },
    /// Un paragraphe de texte.
    Paragraphe(Vec<Inline>),
    /// Une citation (bloc de citation).
    Citation(Vec<Inline>),
    /// Une liste à puces.
    ListePuce(Vec<Vec<Inline>>),
    /// Une liste numérotée.
    ListeNumero(Vec<Vec<Inline>>),
    /// Un bloc brut Typst, inséré tel quel dans le document généré.
    Brut(String),
    /// Une formule mathématique : `$$ … $$`, du Typst math brut.
    Formule(String),
    /// Un tableau : `| A | B |`, sa ligne de séparation, ses lignes.
    Tableau {
        /// L'en-tête, cellule par cellule — jamais vide.
        entete: Vec<Vec<Inline>>,
        /// Les lignes du corps, cellule par cellule.
        lignes: Vec<Vec<Vec<Inline>>>,
        /// L'alignement de chaque colonne, dans l'ordre.
        alignements: Vec<Alignement>,
    },
    /// Une image : `![légende](chemin)`.
    Image {
        /// La légende, en éléments inline — peut être vide.
        légende: Vec<Inline>,
        /// Le chemin de l'image, relatif au projet.
        cible: String,
    },
    /// Un bloc **habillé** de propriétés : `{bordure=1pt}` suivi du bloc.
    ///
    /// C'est le point d'arrivée de toutes les propriétés de décor —
    /// bordures, remplissages, dégradés, formes — et il ne dépend d'aucun
    /// bloc en particulier : il habille celui qui suit.
    Décor {
        /// Les propriétés, dans l'ordre écrit.
        proprietes: Proprietes,
        /// Le bloc habillé. Jamais un autre `Décor` : un seul manteau.
        bloc: Box<Bloc>,
    },
}

/// L'alignement d'une colonne de tableau, écrit dans sa ligne de
/// séparation : `---` à gauche, `:-:` au centre, `---:` à droite.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignement {
    Gauche,
    Centre,
    Droite,
}

impl Alignement {
    /// La forme canonique du trait.
    pub fn trait_(&self) -> &'static str {
        match self {
            Alignement::Gauche => "---",
            Alignement::Centre => ":-:",
            Alignement::Droite => "---:",
        }
    }
}

/// Les propriétés d'un bloc : des paires ordonnées, `clé=valeur`.
///
/// La syntaxe est `{largeur=50% bordure=1pt}` — une valeur contenant une
/// espace s'accôle : `remplissage={luma(240)}`. Les clés et les valeurs
/// n'ont ni marqueur ni saut de ligne : ce sont des **noms et des valeurs
/// Typst**, pas du texte, et les balayages du voisinage ne doivent pas s'y
/// méprendre.
///
/// L'ordre est celui de l'écriture : la sérialisation ne trie pas, elle
/// conserve — c'est ce qui rend le round-trip exact.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Proprietes {
    /// Les noms de style — `{encadré plein}`. Ce sont des **intentions** ;
    /// les mesures sont aux normes du document.
    pub noms: Vec<String>,

    /// Les mesures explicites — `{bordure=2pt}`. L'exception, jamais la
    /// règle : la source doit rester du markdown.
    pub paires: Vec<(String, String)>,
}

impl Proprietes {
    /// Sans aucune propriété.
    pub fn vide() -> Self {
        Self::default()
    }

    /// Y a-t-il au moins une propriété ?
    pub fn est_vide(&self) -> bool {
        self.paires.is_empty() && self.noms.is_empty()
    }

    /// La valeur d'une clé, si elle est là.
    #[allow(dead_code)] // API de contrat : éprouvée par les propriétés.
    pub fn obtenir(&self, cle: &str) -> Option<&str> {
        self.paires
            .iter()
            .find(|(c, _)| c == cle)
            .map(|(_, valeur)| valeur.as_str())
    }

    /// Pose une propriété — remplace la valeur si la clé y est déjà.
    pub fn poser(&mut self, cle: impl Into<String>, valeur: impl Into<String>) {
        let cle = cle.into();
        let valeur = valeur.into();

        match self.paires.iter_mut().find(|(c, _)| *c == cle) {
            Some(paire) => paire.1 = valeur,
            None => self.paires.push((cle, valeur)),
        }
    }

    /// Les propriétés sont-elles dans leur forme canonique ?
    #[allow(dead_code)] // API de contrat : éprouvée par les propriétés.
    pub fn est_canonique(&self) -> bool {
        let mut vus = std::collections::HashSet::new();

        let noms_canoniques = self.noms.iter().all(|nom| {
            !nom.is_empty()
                && nom == nom.trim()
                && !nom.contains(['=', '\\', '*', '`', '@', '[', '{', '}', ']', '^', '_'])
                && !nom.contains('\n')
                && vus.insert(nom.as_str())
        });

        if !noms_canoniques {
            return false;
        }

        let mut vues = std::collections::HashSet::new();

        self.paires.iter().all(|(cle, valeur)| {
            !cle.is_empty()
                && cle
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
                && vues.insert(cle.as_str())
                && !valeur.is_empty()
                && valeur == valeur.trim()
                && !valeur.contains('\n')
                && !valeur.contains(['\\', '*', '`', '@', '[', '{', '}', ']', '^', '_', '|', '$'])
        })
    }
}

/// Un élément inline, dans le texte d'un bloc.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Inline {
    /// Du texte nu. Ne contient **jamais** de marqueur : ceux-ci vivent
    /// dans des [`Inline::Litteral`].
    Texte(String),

    /// Des caractères échappés, insérés mot à mot.
    ///
    /// Une barre oblique dans la syntaxe fait un littéral : le caractère qui
    /// suit ne reçoit **aucune interprétation** — ni syntaxique, ni
    /// typographique. C'est la porte de sortie universelle : écrire `\;` en
    /// français donne un point-virgule nu, sans l'espace insécable que la
    /// langue y mettrait.
    Litteral(String),
    /// Du texte en gras.
    Gras(Vec<Inline>),
    /// Du texte en italique.
    Italique(Vec<Inline>),
    /// Du texte en gras et en italique.
    GrasItalique(Vec<Inline>),
    /// Du code inline.
    Code(String),
    /// Du texte coloré.
    Couleur {
        /// La couleur, au format Typst (`red`, `#ff0000`, `rgb(255, 0, 0)`…).
        couleur: String,
        contenu: Vec<Inline>,
    },
    /// Un appel de citation : `@clé`, `@clé[p. 45]`, ou plusieurs références
    /// jointes par `; ` — `@clé[p. 45]; @autre`.
    ///
    /// Un appel ne fait jamais qu'**une** note de bas de page : la mesure le
    /// dit, et c'est elle qui dicte la forme du générateur (voir
    /// [`crate::langage::generer::generer`]).
    Citation(Vec<Reference>),
    /// Une formule mathématique en ligne : `$x^2$` — une bande opaque,
    /// comme le code : ce qui s'y trouve ne ferme rien du voisinage.
    Formule(String),
    /// Une note de bas de page : `^[contenu]`.
    Note(Vec<Inline>),
    /// Un lien : `[texte](https://…)`.
    Lien { texte: Vec<Inline>, cible: String },
}

/// Une référence bibliographique dans un appel de citation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reference {
    /// La clé de l'entrée dans `references.bib`.
    pub cle: String,
    /// La mention de page, écrite telle qu'elle apparaîtra : `p. 45`,
    /// `pp. 12-15`, `chap. 3`…
    pub page: Option<String>,
}

impl Reference {
    /// Une référence sans mention de page.
    pub fn new(cle: impl Into<String>) -> Self {
        Self {
            cle: cle.into(),
            page: None,
        }
    }

    /// La même référence, avec sa mention de page.
    pub fn avec_page(mut self, page: impl Into<String>) -> Self {
        self.page = Some(page.into());
        self
    }
}

impl Document {
    /// Un document vide.
    pub fn vide() -> Self {
        Self { blocs: Vec::new() }
    }

    /// Le document est-il dans sa forme **canonique** ?
    ///
    /// C'est le domaine exact de la garantie de round-trip :
    /// `parse(serialiser(document)) == document` pour tout document canonique,
    /// et pour lui seul. La forme canonique est ce que le sérialiseur écrit et
    /// ce que le parseur relit sans surprise — les autres formes sont tolérées
    /// à la lecture, mais reprises à l'écriture.
    ///
    /// Les règles, une par une :
    ///
    /// 1. un texte nu n'est jamais vide, et ne contient pas de saut de ligne
    ///    (une ligne est une ligne) ;
    /// 2. une emphase est **plate** — elle ne contient pas d'emphase —, son
    ///    contenu n'est pas vide et ne commence ni ne finit par un blanc ;
    ///    le gras-italique se dit en `GrasItalique`, jamais en deux ;
    /// 3. un code inline ne contient ni backtick ni saut de ligne : il est
    ///    littéral, et son délimiteur n'y paraît pas ;
    /// 4. le nom d'une couleur ne contient ni deux-points, ni accolade, ni
    ///    barre oblique ;
    /// 5. une référence a une clé alphanumérique (tirets et underscores
    ///    admis), et sa page, si elle est, est non vide et déjà rognée ;
    /// 6. un paragraphe, une citation de bloc et une liste ne sont pas vides ;
    ///    un titre est de niveau 1 à 6 ;
    /// 7. deux textes nus ne sont jamais adjacents — le parseur les
    ///    rassemble, et deux formes ne peuvent pas avoir la même écriture ;
    /// 8. un lien ne suit jamais immédiatement un appel sans mention de
    ///    page : son crochet ouvrant serait lu comme la page de l'appel —
    ///    `@clé[lien](x)` s'entend `@clé`, page « lien ». Un espace, ou une
    ///    page, les sépare ;
    /// 9. un texte ne suit jamais immédiatement un appel sans page par un
    ///    caractère de clé (lettre, chiffre, tiret, underscore) : il
    ///    grossirait la clé — `@dupont30` suivi de « 2. x » s'écrit
    ///    `@dupont302. x`, qui se relit clé « dupont302 ». Un espace, ou une
    ///    page, les sépare ;
    /// 10. le `; ` d'un appel groupé est **réservé** : un texte dont le
    ///     contenu rogné est « ; » ne sépare jamais deux appels — il serait
    ///     lu comme le séparateur du groupe, et deux formes ne peuvent pas
    ///     avoir la même écriture.
    /// 11. le premier élément d'un paragraphe n'est pas un `Texte` qui
    ///     commence par un marqueur de bloc : la barre de protection qui
    ///     l'écrit se relirait comme un littéral. Le caractère se prend donc
    ///     en `Litteral` — c'est sa forme canonique.
    #[allow(dead_code)] // API de contrat : éprouvée par les propriétés.
    pub fn est_canonique(&self) -> bool {
        self.blocs.iter().all(Bloc::est_canonique)
    }

    /// Le texte visible du document, sans marqueurs ni blocs vides.
    ///
    /// C'est la base du compteur de mots : titres, citations et listes comptent,
    /// tandis que le balisage et les caractères Typst d'un bloc brut ne comptent
    /// pas comme des mots visibles.
    pub fn texte_nu(&self) -> String {
        self.blocs
            .iter()
            .map(Bloc::texte_nu)
            .filter(|texte| !texte.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl Bloc {
    /// Le bloc est-il dans sa forme canonique ? Voir [`Document::est_canonique`].
    #[allow(dead_code)] // API de contrat : éprouvée par les propriétés.
    pub fn est_canonique(&self) -> bool {
        match self {
            Bloc::Titre { niveau, contenu } => {
                (1..=6).contains(niveau) && inline_canonique(contenu)
            }
            Bloc::Paragraphe(contenu) => {
                !contenu.is_empty()
                    && inline_canonique(contenu)
                    && !commence_par_un_marqueur_de_bloc(contenu)
            }
            Bloc::Citation(contenu) => !contenu.is_empty() && inline_canonique(contenu),
            Bloc::ListePuce(items) | Bloc::ListeNumero(items) => {
                !items.is_empty() && items.iter().all(|item| inline_canonique(item))
            }
            // Un bloc brut est pris tel quel : son délimiteur s'adapte à son
            // contenu, tout lui est permis.
            Bloc::Brut(_) => true,
            // La formule est opaque : sa barre de dollars n'y paraît pas.
            Bloc::Formule(texte) => !texte.contains('$') && !texte.trim().is_empty(),
            Bloc::Tableau {
                entete,
                lignes,
                alignements,
            } => {
                // Une en-tête, jamais vide ; autant d'alignements que de
                // colonnes ; chaque ligne a le même compte de cellules —
                // c'est ce qui rend l'écriture univoque.
                !entete.is_empty()
                    && alignements.len() == entete.len()
                    && entete.iter().all(|cellule| inline_canonique(cellule))
                    && lignes.iter().all(|ligne| {
                        ligne.len() == entete.len()
                            && ligne.iter().all(|cellule| inline_canonique(cellule))
                    })
            }
            Bloc::Image { légende, cible } => {
                // La cible suit la règle du lien : non vide, sans saut de
                // ligne. Ses parenthèses s'échappent, comme au lien.
                !cible.is_empty() && !cible.contains('\n') && inline_canonique(légende)
            }
            Bloc::Décor { proprietes, bloc } => {
                // Un seul manteau, jamais deux : un décor dans un décor se
                // rassemble à l'écriture.
                !proprietes.est_vide()
                    && proprietes.est_canonique()
                    && !matches!(bloc.as_ref(), Bloc::Décor { .. })
                    && bloc.est_canonique()
            }
        }
    }
}

impl Inline {
    /// L'élément est-il dans sa forme canonique ? Voir [`Document::est_canonique`].
    #[allow(dead_code)] // API de contrat : éprouvée par les propriétés.
    pub fn est_canonique(&self) -> bool {
        match self {
            Inline::Texte(texte) => {
                // Les marqueurs vivent dans les littéraux : un texte nu n'en
                // porte aucun, c'est ce qui rend l'écriture univoque.
                !texte.is_empty()
                    && !texte.contains('\n')
                    && !texte.contains(['\\', '*', '`', '@', '[', '{', '}', ']', '^', '_', '|', '$'])
            }
            Inline::Litteral(texte) => !texte.is_empty() && !texte.contains('\n'),
            Inline::Gras(contenu) | Inline::Italique(contenu) | Inline::GrasItalique(contenu) => {
                est_emphase_canonique(contenu)
            }
            // Le code est littéral : son délimiteur n'y paraît pas.
            Inline::Code(texte) => !texte.contains('`') && !texte.contains('\n'),
            // La formule aussi : ni sa barre, ni saut de ligne.
            Inline::Formule(texte) => !texte.contains('$') && !texte.contains('\n'),
            Inline::Couleur { couleur, contenu } => {
                // Le nom va jusqu'au premier deux-points : un nom **simple**,
                // lettres, chiffres et quelques signes — `red`, `#ff0000`,
                // `rgb(1, 2, 0)`. Ni marqueur, qui ferait signe aux balayages
                // du voisinage, ni deux-points, qui serait le séparateur.
                !couleur.is_empty()
                    && !couleur.contains([':', '\\', '*', '`', '@', '[', '{', '}', ']', '^', '_', '|', '$'])
                    && !couleur.contains('\n')
                    && inline_canonique(contenu)
            }
            Inline::Citation(references) => {
                !references.is_empty() && references.iter().all(Reference::est_canonique)
            }
            Inline::Note(contenu) => inline_canonique(contenu),
            Inline::Lien { texte, cible } => inline_canonique(texte) && !cible.contains('\n'),
        }
    }

    /// L'élément est-il une emphase ? (Gras, italique, ou les deux.)
    #[allow(dead_code)] // API de contrat : éprouvée par les propriétés.
    pub fn est_emphase(&self) -> bool {
        matches!(
            self,
            Inline::Gras(_) | Inline::Italique(_) | Inline::GrasItalique(_)
        )
    }
}

impl Reference {
    /// La référence est-elle dans sa forme canonique ? Voir
    /// [`Document::est_canonique`].
    #[allow(dead_code)] // API de contrat : éprouvée par les propriétés.
    pub fn est_canonique(&self) -> bool {
        !self.cle.is_empty()
            && self
                .cle
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
            && self
                .page
                .as_ref()
                .is_none_or(|page| !page.is_empty() && !page.contains('\n') && page == page.trim())
    }
}

/// Le texte ouvrirait-il un bloc s'il tombait en tête de ligne ?
///
/// C'est la reconnaissance du sérialiseur, en une seule source : un
/// paragraphe qui commencerait par un marqueur serait relu comme un autre
/// bloc, et sa barre de protection se relirait comme un littéral.
pub fn ouvre_un_bloc(texte: &str) -> bool {
    let debut = texte.trim_start();

    if debut.starts_with('\\') {
        return false;
    }

    // Une image formée, ou une ligne d'attributs : deux marqueurs de plus
    // depuis les blocs et propriétés, avec la même règle qu'aux autres —
    // ce qui n'en est pas un reste du texte.
    if crate::langage::parse::lire_image(debut).is_some()
        || crate::langage::parse::lire_proprietes_ligne(debut).is_some()
        || debut.starts_with('|')
        || debut.starts_with("$$")
    {
        return true;
    }

    let dièses = debut.chars().take_while(|c| *c == '#').count();

    (dièses > 0 && dièses <= 6 && debut[dièses..].starts_with(' '))
        || debut.starts_with('>')
        || debut.starts_with("- ")
        || {
            let chiffres = debut.chars().take_while(|c| c.is_ascii_digit()).count();
            chiffres > 0 && debut[chiffres..].starts_with(". ")
        }
        || debut.starts_with("```")
}

/// Le contenu commence-t-il par un marqueur de bloc ?
fn commence_par_un_marqueur_de_bloc(contenu: &[Inline]) -> bool {
    matches!(&contenu.first(), Some(Inline::Texte(texte)) if ouvre_un_bloc(texte))
}

/// Une suite d'éléments inline est-elle canonique ?
#[allow(dead_code)] // API de contrat : éprouvée par les propriétés.
///
/// Outre la canonicité de chaque élément, deux règles gouvernent la suite :
/// deux textes nus ne s'adjacent pas, et un lien ne colle pas à un appel
/// sans page. Voir [`Document::est_canonique`].
/// Le bord du contenu est-il un blanc ? Il disparaîtrait à la lecture.
fn bord_blanc(element: Option<&Inline>) -> bool {
    match element {
        Some(Inline::Texte(texte)) | Some(Inline::Litteral(texte)) => {
            texte.starts_with(char::is_whitespace) || texte.ends_with(char::is_whitespace)
        }
        _ => false,
    }
}

fn inline_canonique(inline: &[Inline]) -> bool {
    if !inline.iter().all(Inline::est_canonique) {
        return false;
    }

    // Les bords du contenu : `parse` rogne chaque ligne, donc un blanc aux
    // extrémités disparaît à la lecture — et le round-trip avec lui. Le
    // pire est un littéral : `x\ ` se relit `x\`, et c'est un caractère
    // qui change. Le code échappe à la règle : ses backticks protègent son
    // contenu jusqu'au bord.
    if bord_blanc(inline.first()) || bord_blanc(inline.last()) {
        return false;
    }

    for paire in inline.windows(2) {
        let (avant, apres) = (&paire[0], &paire[1]);

        // Deux éléments de même nature se rejoignent à la lecture : jamais
        // adjacents dans la forme canonique.
        let memes_natures = (matches!(avant, Inline::Texte(_))
            && matches!(apres, Inline::Texte(_)))
            || (matches!(avant, Inline::Litteral(_)) && matches!(apres, Inline::Litteral(_)));

        if memes_natures {
            return false;
        }

        if appel_sans_page(avant) {
            // Un lien collé : son crochet serait la page de l'appel.
            if matches!(apres, Inline::Lien { .. }) {
                return false;
            }

            // Un texte collé : sa première lettre grossirait la clé.
            if let Inline::Texte(texte) = apres
                && texte.starts_with(|c: char| c.is_alphanumeric() || c == '-' || c == '_')
            {
                return false;
            }
        }
    }

    // Un « ; » entre deux appels serait le séparateur du groupe.
    for triplet in inline.windows(3) {
        let est_un_point_virgule =
            matches!(&triplet[1], Inline::Texte(texte) if texte.trim() == ";");

        if est_un_point_virgule
            && matches!(triplet[0], Inline::Citation(_))
            && matches!(triplet[2], Inline::Citation(_))
        {
            return false;
        }
    }

    true
}

/// L'élément est-il un appel dont la dernière référence n'a pas de page ?
#[allow(dead_code)] // API de contrat : éprouvée par les propriétés.
fn appel_sans_page(element: &Inline) -> bool {
    match element {
        Inline::Citation(references) => references
            .last()
            .is_some_and(|derniere| derniere.page.is_none()),
        _ => false,
    }
}

/// Une emphase se trouve-t-elle **quelque part** sous ces éléments ?
#[allow(dead_code)] // API de contrat : éprouvée par les propriétés.
fn contient_emphase(inline: &[Inline]) -> bool {
    inline.iter().any(|element| match element {
        Inline::Gras(contenu)
        | Inline::Italique(contenu)
        | Inline::GrasItalique(contenu)
        | Inline::Note(contenu)
        | Inline::Couleur { contenu, .. } => contient_emphase(contenu) || element.est_emphase(),
        Inline::Lien { texte, .. } => contient_emphase(texte),
        _ => false,
    })
}

/// Une emphase canonique : plate, non vide, sans blanc aux bords.
///
/// « Plate » — **aucune** emphase n'y descend, à aucun niveau : le balayage
/// d'une emphase est plat, une étoile de la profondeur la refermerait avant
/// l'heure. Le gras-italique se dit en `GrasItalique`. Les bords se jugent
/// sur les éléments extrêmes : un texte nu détermine le premier et le
/// dernier caractère sérialisés, et un délimiteur (code, note…) les garantit
/// non blancs.
#[allow(dead_code)] // API de contrat : éprouvée par les propriétés.
fn est_emphase_canonique(contenu: &[Inline]) -> bool {
    if contenu.is_empty() || contient_emphase(contenu) || !inline_canonique(contenu) {
        return false;
    }

    let bord_blanc = match (&contenu.first(), &contenu.last()) {
        (Some(Inline::Texte(premier)), _) if premier.starts_with(char::is_whitespace) => true,
        (_, Some(Inline::Texte(dernier))) if dernier.ends_with(char::is_whitespace) => true,
        _ => false,
    };

    !bord_blanc
}

impl Bloc {
    /// Le texte nu d'un bloc, pour le compte de mots et l'outline.
    ///
    /// Le balisage est retiré : seul le texte visible compte. C'est la même
    /// règle que le compteur de mots actuel, appliquée à l'AST plutôt qu'au
    /// source.
    pub fn texte_nu(&self) -> String {
        match self {
            // Un décor n'ajoute pas de texte : il habille celui qui parle.
            Bloc::Décor { bloc, .. } => bloc.texte_nu(),
            // Les cellules se lisent, comme tout texte.
            Bloc::Tableau { entete, lignes, .. } => entete
                .iter()
                .chain(lignes.iter().flatten())
                .map(|cellule| inline_texte_nu(cellule))
                .collect::<Vec<_>>()
                .join(" "),
            // La légende est du texte visible — elle se lit.
            Bloc::Image { légende, .. } => inline_texte_nu(légende),
            Bloc::Titre { contenu, .. } | Bloc::Paragraphe(contenu) | Bloc::Citation(contenu) => {
                inline_texte_nu(contenu)
            }
            Bloc::ListePuce(items) | Bloc::ListeNumero(items) => items
                .iter()
                .map(|item| inline_texte_nu(item))
                .collect::<Vec<_>>()
                .join(" "),
            Bloc::Brut(_) => String::new(),
            Bloc::Formule(texte) => texte.clone(),
        }
    }
}

/// Le texte nu d'une suite d'éléments inline.
pub fn inline_texte_nu(inline: &[Inline]) -> String {
    let mut out = String::new();

    for element in inline {
        match element {
            Inline::Texte(texte) | Inline::Litteral(texte) => out.push_str(texte),
            Inline::Gras(contenu)
            | Inline::Italique(contenu)
            | Inline::GrasItalique(contenu)
            | Inline::Note(contenu)
            | Inline::Couleur { contenu, .. } => out.push_str(&inline_texte_nu(contenu)),
            Inline::Code(texte) | Inline::Formule(texte) => out.push_str(texte),
            Inline::Citation(references) => {
                for (index, reference) in references.iter().enumerate() {
                    if index > 0 {
                        out.push_str("; ");
                    }

                    out.push_str(&reference.cle);

                    if let Some(page) = &reference.page {
                        out.push(' ');
                        out.push_str(page);
                    }
                }
            }
            Inline::Lien { texte, .. } => out.push_str(&inline_texte_nu(texte)),
        }
    }

    out
}

#[cfg(test)]
mod tests {

    /// Les bords blancs ne sont pas canoniques : `parse` les rogne, donc
    /// le round-trip les perdrait. Le pire, un littéral, y changerait de
    /// caractère.
    #[test]
    fn les_bords_blancs_ne_sont_pas_canoniques() {
        use crate::langage::parse;
        use crate::langage::serialiser;

        // Contre-exemples de l'audit, tous déclarés canoniques avant.
        let formes = [
            vec![Inline::Texte(" x".to_string())],
            vec![Inline::Texte("x ".to_string())],
            vec![
                Inline::Texte("x".to_string()),
                Inline::Litteral(" ".to_string()),
            ],
            vec![Inline::Litteral(" ".to_string())],
        ];

        for forme in formes {
            let document = Document {
                blocs: vec![Bloc::Paragraphe(forme.clone())],
            };

            assert!(
                !document.est_canonique(),
                "{forme:?} ne doit pas être canonique"
            );
        }

        // Et la preuve par le vide : une forme proche, mais propre, se
        // relit mot pour mot.
        let document = Document {
            blocs: vec![Bloc::Paragraphe(vec![
                Inline::Texte("x".to_string()),
                Inline::Litteral(" ".to_string()),
                Inline::Texte("y".to_string()),
            ])],
        };

        assert!(document.est_canonique(), "le milieu lui est permis");
        assert_eq!(parse(&serialiser(&document)), document);
    }

    use super::*;

    /// Le texte nu retire le balisage : c'est ce que compte le compteur de
    /// mots et ce que montre l'outline.
    #[test]
    fn texte_nu_retire_le_balisage() {
        let document = Document {
            blocs: vec![Bloc::Paragraphe(vec![
                Inline::Texte("Un ".to_string()),
                Inline::Gras(vec![Inline::Texte("mot".to_string())]),
                Inline::Texte(" et ".to_string()),
                Inline::Italique(vec![Inline::Texte("deux".to_string())]),
            ])],
        };

        assert_eq!(document.blocs[0].texte_nu(), "Un mot et deux");
    }

    /// Le document réunit le texte visible des blocs, sans leur balisage.
    #[test]
    fn document_texte_nu_reunit_les_blocs() {
        let document = Document {
            blocs: vec![
                Bloc::Titre {
                    niveau: 1,
                    contenu: vec![Inline::Texte("Titre".to_string())],
                },
                Bloc::Paragraphe(vec![
                    Inline::Texte("Un ".to_string()),
                    Inline::Gras(vec![Inline::Texte("texte".to_string())]),
                ]),
            ],
        };

        assert_eq!(document.texte_nu(), "Titre Un texte");
    }

    /// Une note compte par son contenu, pas par son marqueur.
    #[test]
    fn une_note_compte_comme_son_contenu() {
        let bloc = Bloc::Paragraphe(vec![
            Inline::Texte("Voir ".to_string()),
            Inline::Note(vec![Inline::Texte("une précision".to_string())]),
        ]);

        assert_eq!(bloc.texte_nu(), "Voir une précision");
    }

    /// Le compteur ne prend pas les commandes d'un bloc Typst brut pour de la prose.
    #[test]
    fn le_typst_brut_ne_gonfle_pas_le_compte_de_mots() {
        let document = Document {
            blocs: vec![
                Bloc::Paragraphe(vec![Inline::Texte("Texte visible".to_string())]),
                Bloc::Brut("#figure(image(\"figure.png\"))".to_string()),
            ],
        };

        assert_eq!(document.texte_nu(), "Texte visible");
    }

    /// Une citation bibliographique compte comme son texte : la clé est un mot.
    #[test]
    fn une_citation_compte_comme_son_texte() {
        let bloc = Bloc::Paragraphe(vec![
            Inline::Texte("Voir ".to_string()),
            Inline::Citation(vec![Reference::new("dupont2023")]),
        ]);

        assert_eq!(bloc.texte_nu(), "Voir dupont2023");
    }

    /// La mention de page se lit à côté de sa clé, et un appel groupé les
    /// rejoint par le point-virgule qui les sépare dans le texte.
    #[test]
    fn une_page_et_un_groupe_se_lisent_comme_ils_s_ecrivent() {
        let bloc = Bloc::Paragraphe(vec![Inline::Citation(vec![
            Reference::new("dupont2023").avec_page("p. 45"),
            Reference::new("martin2020"),
        ])]);

        assert_eq!(bloc.texte_nu(), "dupont2023 p. 45; martin2020");
    }
}
