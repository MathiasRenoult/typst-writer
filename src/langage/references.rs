//! Les références bibliques : donnée, syntaxe d'affichage, lecture et écriture.
//!
//! Ce module ne dépend d'aucun plugin et n'embarque aucun texte : il connaît
//! les soixante-six livres canoniques, leurs formes françaises, et les
//! deutérocanoniques et apocryphes que l'auteur peut citer.

/// Une traduction biblique nommée dans une citation.
///
/// Le texte embarqué reste dans le plugin biblique ; cette énumération ne
/// dit que le **nom** que la citation affiche. Une référence sans traduction
/// ne porte aucune mention.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Traduction {
    #[default]
    Lsg,
    Darby,
}

impl Traduction {
    /// Le nom complet de la traduction.
    pub fn nom(&self) -> &'static str {
        match self {
            Traduction::Lsg => "Louis Segond 1910",
            Traduction::Darby => "J.N. Darby",
        }
    }

    /// La forme courte affichée dans une citation : `LSG`, `DARBY`.
    pub fn abreviation(&self) -> &'static str {
        match self {
            Traduction::Lsg => "LSG",
            Traduction::Darby => "DARBY",
        }
    }

    /// Le code accepté dans la source après la barre verticale.
    pub fn code(&self) -> &'static str {
        match self {
            Traduction::Lsg => "lsg",
            Traduction::Darby => "darby",
        }
    }

    /// Lit un code de traduction écrit dans une citation.
    pub fn depuis_code(mot: &str) -> Option<Self> {
        match mot.trim().to_lowercase().as_str() {
            "lsg" | "segond" | "louis_segond_1910" => Some(Traduction::Lsg),
            "darby" | "jnd" | "j.n. darby" => Some(Traduction::Darby),
            _ => None,
        }
    }
}

/// Une référence biblique.
///
/// `livre` est un code OSIS (`Gen`, `John`, `Ps`) : c'est la forme que
/// rendent [`depuis_texte`] et [`code_du_livre`], et celle qu'attendent
/// [`nom_du_livre`] et [`crate::plugins::bible::versets`]. Les bornes
/// `verset_debut` et `verset_fin` sont inclusives ; `verset_fin == u32::MAX`
/// désigne le chapitre entier. `chapitre_fin` vaut `chapitre` pour une
/// référence ordinaire ; plus grand, il désigne une suite de chapitres
/// entiers (`Gn 1-11`).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Reference {
    /// Le code OSIS du livre.
    pub livre: String,
    /// Le premier numéro de chapitre, à partir de 1.
    pub chapitre: u32,
    /// Le dernier numéro de chapitre, inclus ; égal à `chapitre` pour une
    /// référence ordinaire.
    pub chapitre_fin: u32,
    /// Le premier verset de la plage, inclus.
    pub verset_debut: u32,
    /// Le dernier verset de la plage, inclus ; égal au premier pour un seul
    /// verset, `u32::MAX` pour le chapitre entier.
    pub verset_fin: u32,
    /// La traduction explicitement demandée pour cette référence.
    pub traduction: Option<Traduction>,
}

impl Reference {
    /// Une référence à un seul verset.
    pub fn verset(livre: &str, chapitre: u32, verset: u32) -> Self {
        Self {
            livre: livre.to_string(),
            chapitre,
            chapitre_fin: chapitre,
            verset_debut: verset,
            verset_fin: verset,
            traduction: None,
        }
    }

    /// Une plage de versets ; `fin` est ramené au moins à `debut`.
    pub fn plage(livre: &str, chapitre: u32, debut: u32, fin: u32) -> Self {
        Self {
            livre: livre.to_string(),
            chapitre,
            chapitre_fin: chapitre,
            verset_debut: debut,
            verset_fin: fin.max(debut),
            traduction: None,
        }
    }

    /// Une suite de chapitres entiers, du premier au dernier inclus.
    pub fn chapitres(livre: &str, debut: u32, fin: u32) -> Self {
        Self {
            livre: livre.to_string(),
            chapitre: debut,
            chapitre_fin: fin.max(debut),
            verset_debut: 1,
            verset_fin: u32::MAX,
            traduction: None,
        }
    }

    /// La référence couvre-t-elle plusieurs chapitres entiers ?
    pub fn est_plage_de_chapitres(&self) -> bool {
        self.chapitre_fin > self.chapitre
    }

    /// Une clé interne stable, pour les repères Typst et les pages d'un index.
    /// Elle ne dépend pas de la forme d'affichage.
    pub fn cle(&self) -> String {
        format!(
            "{}|{}|{}|{}|{}",
            self.livre, self.chapitre, self.chapitre_fin, self.verset_debut, self.verset_fin
        )
    }
}

/// La façon d'écrire le livre d'une référence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormeLivre {
    /// « Gn », « Jn ».
    FrancaiseCourte,
    /// « Genèse », « Jean ».
    FrancaiseLongue,
    /// « Gen », « John ».
    Osis,
    /// « Gn », « Jn » sans point.
    SansPoint,
    /// Les abréviations de la Traduction œcuménique de la Bible :
    /// « Gn », « 1 S », « 1 Co », « 1 M »…
    Tob,
}

/// Les chiffres des chapitres et des versets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Chiffres {
    Arabes,
    Romains,
}

/// Un livre et ses formes françaises.
struct LivreFr {
    osis: &'static str,
    nom: &'static str,
    abreviation: &'static str,
    fr: &'static str,
}

/// Les livres canoniques, puis les deutérocanoniques et les apocryphes,
/// dans l'ordre où le panneau les propose.
const LIVRES_FR: &[LivreFr] = &[
    LivreFr {
        osis: "Gen",
        nom: "Genèse",
        abreviation: "Gen",
        fr: "Gn",
    },
    LivreFr {
        osis: "Exod",
        nom: "Exode",
        abreviation: "Exod",
        fr: "Ex",
    },
    LivreFr {
        osis: "Lev",
        nom: "Lévitique",
        abreviation: "Lev",
        fr: "Lv",
    },
    LivreFr {
        osis: "Num",
        nom: "Nombres",
        abreviation: "Num",
        fr: "Nb",
    },
    LivreFr {
        osis: "Deut",
        nom: "Deutéronome",
        abreviation: "Deut",
        fr: "Dt",
    },
    LivreFr {
        osis: "Josh",
        nom: "Josué",
        abreviation: "Josh",
        fr: "Js",
    },
    LivreFr {
        osis: "Judg",
        nom: "Juges",
        abreviation: "Judg",
        fr: "Jg",
    },
    LivreFr {
        osis: "Ruth",
        nom: "Ruth",
        abreviation: "Ruth",
        fr: "Rt",
    },
    LivreFr {
        osis: "1Sam",
        nom: "1 Samuel",
        abreviation: "1Sam",
        fr: "1S",
    },
    LivreFr {
        osis: "2Sam",
        nom: "2 Samuel",
        abreviation: "2Sam",
        fr: "2S",
    },
    LivreFr {
        osis: "1Kgs",
        nom: "1 Rois",
        abreviation: "1Kgs",
        fr: "1R",
    },
    LivreFr {
        osis: "2Kgs",
        nom: "2 Rois",
        abreviation: "2Kgs",
        fr: "2R",
    },
    LivreFr {
        osis: "1Chr",
        nom: "1 Chroniques",
        abreviation: "1Chr",
        fr: "1Ch",
    },
    LivreFr {
        osis: "2Chr",
        nom: "2 Chroniques",
        abreviation: "2Chr",
        fr: "2Ch",
    },
    LivreFr {
        osis: "Ezra",
        nom: "Esdras",
        abreviation: "Ezra",
        fr: "Esd",
    },
    LivreFr {
        osis: "Neh",
        nom: "Néhémie",
        abreviation: "Neh",
        fr: "Ne",
    },
    LivreFr {
        osis: "Esth",
        nom: "Esther",
        abreviation: "Esth",
        fr: "Est",
    },
    LivreFr {
        osis: "Job",
        nom: "Job",
        abreviation: "Job",
        fr: "Jb",
    },
    LivreFr {
        osis: "Ps",
        nom: "Psaumes",
        abreviation: "Ps",
        fr: "Ps",
    },
    LivreFr {
        osis: "Prov",
        nom: "Proverbes",
        abreviation: "Prov",
        fr: "Pr",
    },
    LivreFr {
        osis: "Eccl",
        nom: "Ecclésiaste",
        abreviation: "Eccl",
        fr: "Qo",
    },
    LivreFr {
        osis: "Song",
        nom: "Cantique des Cantiques",
        abreviation: "Song",
        fr: "Ct",
    },
    LivreFr {
        osis: "Isa",
        nom: "Ésaïe",
        abreviation: "Isa",
        fr: "Es",
    },
    LivreFr {
        osis: "Jer",
        nom: "Jérémie",
        abreviation: "Jer",
        fr: "Jr",
    },
    LivreFr {
        osis: "Lam",
        nom: "Lamentations",
        abreviation: "Lam",
        fr: "Lm",
    },
    LivreFr {
        osis: "Ezek",
        nom: "Ézéchiel",
        abreviation: "Ezek",
        fr: "Ez",
    },
    LivreFr {
        osis: "Dan",
        nom: "Daniel",
        abreviation: "Dan",
        fr: "Dn",
    },
    LivreFr {
        osis: "Hos",
        nom: "Osée",
        abreviation: "Hos",
        fr: "Os",
    },
    LivreFr {
        osis: "Joel",
        nom: "Joël",
        abreviation: "Joel",
        fr: "Jl",
    },
    LivreFr {
        osis: "Amos",
        nom: "Amos",
        abreviation: "Amos",
        fr: "Am",
    },
    LivreFr {
        osis: "Obad",
        nom: "Abdias",
        abreviation: "Obad",
        fr: "Ab",
    },
    LivreFr {
        osis: "Jonah",
        nom: "Jonas",
        abreviation: "Jonah",
        fr: "Jon",
    },
    LivreFr {
        osis: "Mic",
        nom: "Michée",
        abreviation: "Mic",
        fr: "Mi",
    },
    LivreFr {
        osis: "Nah",
        nom: "Nahoum",
        abreviation: "Nah",
        fr: "Na",
    },
    LivreFr {
        osis: "Hab",
        nom: "Habacuc",
        abreviation: "Hab",
        fr: "Ha",
    },
    LivreFr {
        osis: "Zeph",
        nom: "Sophonie",
        abreviation: "Zeph",
        fr: "So",
    },
    LivreFr {
        osis: "Hag",
        nom: "Aggée",
        abreviation: "Hag",
        fr: "Ag",
    },
    LivreFr {
        osis: "Zech",
        nom: "Zacharie",
        abreviation: "Zech",
        fr: "Za",
    },
    LivreFr {
        osis: "Mal",
        nom: "Malachie",
        abreviation: "Mal",
        fr: "Ml",
    },
    LivreFr {
        osis: "Matt",
        nom: "Matthieu",
        abreviation: "Matt",
        fr: "Mt",
    },
    LivreFr {
        osis: "Mark",
        nom: "Marc",
        abreviation: "Mark",
        fr: "Mc",
    },
    LivreFr {
        osis: "Luke",
        nom: "Luc",
        abreviation: "Luke",
        fr: "Lc",
    },
    LivreFr {
        osis: "John",
        nom: "Jean",
        abreviation: "John",
        fr: "Jn",
    },
    LivreFr {
        osis: "Acts",
        nom: "Actes",
        abreviation: "Acts",
        fr: "Ac",
    },
    LivreFr {
        osis: "Rom",
        nom: "Romains",
        abreviation: "Rom",
        fr: "Rm",
    },
    LivreFr {
        osis: "1Cor",
        nom: "1 Corinthiens",
        abreviation: "1Cor",
        fr: "1Co",
    },
    LivreFr {
        osis: "2Cor",
        nom: "2 Corinthiens",
        abreviation: "2Cor",
        fr: "2Co",
    },
    LivreFr {
        osis: "Gal",
        nom: "Galates",
        abreviation: "Gal",
        fr: "Ga",
    },
    LivreFr {
        osis: "Eph",
        nom: "Éphésiens",
        abreviation: "Eph",
        fr: "Ep",
    },
    LivreFr {
        osis: "Phil",
        nom: "Philippiens",
        abreviation: "Phil",
        fr: "Ph",
    },
    LivreFr {
        osis: "Col",
        nom: "Colossiens",
        abreviation: "Col",
        fr: "Col",
    },
    LivreFr {
        osis: "1Thess",
        nom: "1 Thessaloniciens",
        abreviation: "1Thess",
        fr: "1Th",
    },
    LivreFr {
        osis: "2Thess",
        nom: "2 Thessaloniciens",
        abreviation: "2Thess",
        fr: "2Th",
    },
    LivreFr {
        osis: "1Tim",
        nom: "1 Timothée",
        abreviation: "1Tim",
        fr: "1Tm",
    },
    LivreFr {
        osis: "2Tim",
        nom: "2 Timothée",
        abreviation: "2Tim",
        fr: "2Tm",
    },
    LivreFr {
        osis: "Titus",
        nom: "Tite",
        abreviation: "Titus",
        fr: "Tt",
    },
    LivreFr {
        osis: "Phlm",
        nom: "Philémon",
        abreviation: "Phlm",
        fr: "Phm",
    },
    LivreFr {
        osis: "Heb",
        nom: "Hébreux",
        abreviation: "Heb",
        fr: "Hé",
    },
    LivreFr {
        osis: "Jas",
        nom: "Jacques",
        abreviation: "Jas",
        fr: "Jc",
    },
    LivreFr {
        osis: "1Pet",
        nom: "1 Pierre",
        abreviation: "1Pet",
        fr: "1P",
    },
    LivreFr {
        osis: "2Pet",
        nom: "2 Pierre",
        abreviation: "2Pet",
        fr: "2P",
    },
    LivreFr {
        osis: "1John",
        nom: "1 Jean",
        abreviation: "1John",
        fr: "1Jn",
    },
    LivreFr {
        osis: "2John",
        nom: "2 Jean",
        abreviation: "2John",
        fr: "2Jn",
    },
    LivreFr {
        osis: "3John",
        nom: "3 Jean",
        abreviation: "3John",
        fr: "3Jn",
    },
    LivreFr {
        osis: "Jude",
        nom: "Jude",
        abreviation: "Jude",
        fr: "Jude",
    },
    LivreFr {
        osis: "Rev",
        nom: "Apocalypse",
        abreviation: "Rev",
        fr: "Ap",
    },
    // Les deutérocanoniques et les apocryphes ne sont pas dans le canon
    // protestant, mais l'auteur peut les citer : ils ont un code OSIS, un
    // nom et une abréviation, et restent hors des 66 livres de base.
    LivreFr {
        osis: "Tob",
        nom: "Tobie",
        abreviation: "Tob",
        fr: "Tb",
    },
    LivreFr {
        osis: "Jdt",
        nom: "Judith",
        abreviation: "Jdt",
        fr: "Jdt",
    },
    LivreFr {
        osis: "AddEst",
        nom: "Esther grec",
        abreviation: "AddEst",
        fr: "Est gr",
    },
    LivreFr {
        osis: "1Macc",
        nom: "1 Maccabées",
        abreviation: "1Macc",
        fr: "1 M",
    },
    LivreFr {
        osis: "2Macc",
        nom: "2 Maccabées",
        abreviation: "2Macc",
        fr: "2 M",
    },
    LivreFr {
        osis: "Wis",
        nom: "Sagesse",
        abreviation: "Wis",
        fr: "Sg",
    },
    LivreFr {
        osis: "Sir",
        nom: "Siracide",
        abreviation: "Sir",
        fr: "Si",
    },
    LivreFr {
        osis: "Bar",
        nom: "Baruch",
        abreviation: "Bar",
        fr: "Ba",
    },
    LivreFr {
        osis: "EpJer",
        nom: "Lettre de Jérémie",
        abreviation: "EpJer",
        fr: "Let Jr",
    },
    LivreFr {
        osis: "PrMan",
        nom: "Prière de Manassé",
        abreviation: "PrMan",
        fr: "Pr Man",
    },
    LivreFr {
        osis: "Ps151",
        nom: "Psaume 151",
        abreviation: "Ps151",
        fr: "Ps 151",
    },
    LivreFr {
        osis: "3Macc",
        nom: "3 Maccabées",
        abreviation: "3Macc",
        fr: "3 M",
    },
    LivreFr {
        osis: "4Macc",
        nom: "4 Maccabées",
        abreviation: "4Macc",
        fr: "4 M",
    },
    LivreFr {
        osis: "2Esd",
        nom: "2 Esdras",
        abreviation: "2Esd",
        fr: "2 Esd",
    },
    LivreFr {
        osis: "1Esd",
        nom: "1 Esdras",
        abreviation: "1Esd",
        fr: "1 Esd",
    },
    LivreFr {
        osis: "3Esd",
        nom: "3 Esdras",
        abreviation: "3Esd",
        fr: "3 Esd",
    },
    LivreFr {
        osis: "1En",
        nom: "1 Hénoch",
        abreviation: "1En",
        fr: "1 Hén",
    },
    LivreFr {
        osis: "2En",
        nom: "2 Hénoch",
        abreviation: "2En",
        fr: "2 Hén",
    },
    LivreFr {
        osis: "Jub",
        nom: "Jubilés",
        abreviation: "Jub",
        fr: "Jub",
    },
    LivreFr {
        osis: "TReu",
        nom: "Testament de Ruben",
        abreviation: "TReu",
        fr: "T. Rub.",
    },
    LivreFr {
        osis: "TSim",
        nom: "Testament de Siméon",
        abreviation: "TSim",
        fr: "T. Sim.",
    },
    LivreFr {
        osis: "TLevi",
        nom: "Testament de Lévi",
        abreviation: "TLevi",
        fr: "T. Lévi",
    },
    LivreFr {
        osis: "TJud",
        nom: "Testament de Juda",
        abreviation: "TJud",
        fr: "T. Juda",
    },
    LivreFr {
        osis: "TIss",
        nom: "Testament d'Issachar",
        abreviation: "TIss",
        fr: "T. Iss.",
    },
    LivreFr {
        osis: "TZeb",
        nom: "Testament de Zabulon",
        abreviation: "TZeb",
        fr: "T. Zabulon",
    },
    LivreFr {
        osis: "TDan",
        nom: "Testament de Dan",
        abreviation: "TDan",
        fr: "T. Dan",
    },
    LivreFr {
        osis: "TNaph",
        nom: "Testament de Nephtali",
        abreviation: "TNaph",
        fr: "T. Nephtali",
    },
    LivreFr {
        osis: "TGad",
        nom: "Testament de Gad",
        abreviation: "TGad",
        fr: "T. Gad",
    },
    LivreFr {
        osis: "TAsh",
        nom: "Testament d'Aser",
        abreviation: "TAsh",
        fr: "T. Aser",
    },
    LivreFr {
        osis: "TJos",
        nom: "Testament de Joseph",
        abreviation: "TJos",
        fr: "T. Joseph",
    },
    LivreFr {
        osis: "TBen",
        nom: "Testament de Benjamin",
        abreviation: "TBen",
        fr: "T. Benjamin",
    },
    LivreFr {
        osis: "2Bar",
        nom: "2 Baruch",
        abreviation: "2Bar",
        fr: "2 Ba",
    },
    LivreFr {
        osis: "3Bar",
        nom: "3 Baruch",
        abreviation: "3Bar",
        fr: "3 Ba",
    },
    LivreFr {
        osis: "ApAb",
        nom: "Apocalypse d'Abraham",
        abreviation: "ApAb",
        fr: "Ap. Ab.",
    },
    LivreFr {
        osis: "ApZeph",
        nom: "Apocalypse de Sophonie",
        abreviation: "ApZeph",
        fr: "Ap. So.",
    },
    LivreFr {
        osis: "ApPet",
        nom: "Apocalypse de Pierre",
        abreviation: "ApPet",
        fr: "Ap. Pierre",
    },
    LivreFr {
        osis: "ApPaul",
        nom: "Apocalypse de Paul",
        abreviation: "ApPaul",
        fr: "Ap. Paul",
    },
    LivreFr {
        osis: "ApMos",
        nom: "Apocalypse de Moïse",
        abreviation: "ApMos",
        fr: "Ap. Moïse",
    },
];

/// Les abréviations TOB, par code OSIS.
///
/// La TOB écrit les livres à deux chiffres avec une espace (« 1 S ») et
/// garde des formes propres (« Is », « He », « 1 M »). La table est séparée
/// de `LIVRES_FR` : une forme d'affichage n'est pas un nom de livre.
const ABREVIATIONS_TOB: &[(&str, &str)] = &[
    ("Gen", "Gn"),
    ("Exod", "Ex"),
    ("Lev", "Lv"),
    ("Num", "Nb"),
    ("Deut", "Dt"),
    ("Josh", "Jos"),
    ("Judg", "Jg"),
    ("Ruth", "Rt"),
    ("1Sam", "1 S"),
    ("2Sam", "2 S"),
    ("1Kgs", "1 R"),
    ("2Kgs", "2 R"),
    ("1Chr", "1 Ch"),
    ("2Chr", "2 Ch"),
    ("Ezra", "Esd"),
    ("Neh", "Ne"),
    ("Esth", "Est"),
    ("Job", "Jb"),
    ("Ps", "Ps"),
    ("Prov", "Pr"),
    ("Eccl", "Qo"),
    ("Song", "Ct"),
    ("Isa", "Is"),
    ("Jer", "Jr"),
    ("Lam", "Lm"),
    ("Ezek", "Ez"),
    ("Dan", "Dn"),
    ("Hos", "Os"),
    ("Joel", "Jl"),
    ("Amos", "Am"),
    ("Obad", "Ab"),
    ("Jonah", "Jon"),
    ("Mic", "Mi"),
    ("Nah", "Na"),
    ("Hab", "Ha"),
    ("Zeph", "So"),
    ("Hag", "Ag"),
    ("Zech", "Za"),
    ("Mal", "Ml"),
    ("Matt", "Mt"),
    ("Mark", "Mc"),
    ("Luke", "Lc"),
    ("John", "Jn"),
    ("Acts", "Ac"),
    ("Rom", "Rm"),
    ("1Cor", "1 Co"),
    ("2Cor", "2 Co"),
    ("Gal", "Ga"),
    ("Eph", "Ep"),
    ("Phil", "Ph"),
    ("Col", "Col"),
    ("1Thess", "1 Th"),
    ("2Thess", "2 Th"),
    ("1Tim", "1 Tm"),
    ("2Tim", "2 Tm"),
    ("Titus", "Tt"),
    ("Phlm", "Phm"),
    ("Heb", "He"),
    ("Jas", "Jc"),
    ("1Pet", "1 P"),
    ("2Pet", "2 P"),
    ("1John", "1 Jn"),
    ("2John", "2 Jn"),
    ("3John", "3 Jn"),
    ("Jude", "Jude"),
    ("Rev", "Ap"),
    ("Tob", "Tb"),
    ("Jdt", "Jdt"),
    ("AddEst", "Est gr"),
    ("1Macc", "1 M"),
    ("2Macc", "2 M"),
    ("Wis", "Sg"),
    ("Sir", "Si"),
    ("Bar", "Ba"),
    ("EpJer", "Let Jr"),
    ("PrMan", "Pr Man"),
    ("Ps151", "Ps 151"),
    ("3Macc", "3 M"),
    ("4Macc", "4 M"),
    ("2Esd", "2 Esd"),
    ("1Esd", "1 Esd"),
    ("3Esd", "3 Esd"),
    ("1En", "1 Hén"),
    ("2En", "2 Hén"),
    ("Jub", "Jub"),
    ("TReu", "T. Rub."),
    ("TSim", "T. Sim."),
    ("TLevi", "T. Lévi"),
    ("TJud", "T. Juda"),
    ("TIss", "T. Iss."),
    ("TZeb", "T. Zabulon"),
    ("TDan", "T. Dan"),
    ("TNaph", "T. Nephtali"),
    ("TGad", "T. Gad"),
    ("TAsh", "T. Aser"),
    ("TJos", "T. Joseph"),
    ("TBen", "T. Benjamin"),
    ("2Bar", "2 Ba"),
    ("3Bar", "3 Ba"),
    ("ApAb", "Ap. Ab."),
    ("ApZeph", "Ap. So."),
    ("ApPet", "Ap. Pierre"),
    ("ApPaul", "Ap. Paul"),
    ("ApMos", "Ap. Moïse"),
];

/// Les soixante-six livres du canon protestant, avant les deutérocanoniques
/// et les apocryphes ajoutés à la fin de [`LIVRES_FR`].
const NB_LIVRES_CANON: usize = 66;

/// Les livres hors canon, pour que le panneau biblique puisse les proposer
/// sans dépendre d'une traduction qui ne les contient pas.
pub fn livres_hors_canon() -> Vec<(&'static str, &'static str, &'static str)> {
    LIVRES_FR
        .iter()
        .skip(NB_LIVRES_CANON)
        .map(|livre| (livre.osis, livre.nom, livre.abreviation))
        .collect()
}

/// Le livre appartient-il aux deutérocanoniques ou aux apocryphes ajoutés
/// après les soixante-six livres canoniques ?
pub fn est_hors_canon(osis: &str) -> bool {
    LIVRES_FR
        .iter()
        .skip(NB_LIVRES_CANON)
        .any(|livre| livre.osis == osis)
}

/// L'abréviation TOB d'un livre.
pub fn abreviation_tob(osis: &str) -> Option<&'static str> {
    ABREVIATIONS_TOB
        .iter()
        .find(|(code, _)| *code == osis)
        .map(|(_, abreviation)| *abreviation)
}

/// L'abréviation française d'un livre, telle que la table la porte.
///
/// Une seule source : la colonne `fr` de [`LIVRES_FR`]. La complétion et la
/// lecture d'une référence ne peuvent donc pas diverger sur l'orthographe.
pub fn abreviation_francaise(osis: &str) -> Option<&'static str> {
    LIVRES_FR
        .iter()
        .find(|livre| livre.osis == osis)
        .map(|livre| livre.fr)
}

/// Un nombre en chiffres romains : « III ».
pub fn romain(nombre: u32) -> String {
    const TABLE: [(u32, &str); 7] = [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
    ];
    const PETITS: [(u32, &str); 6] = [
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut reste = nombre;
    let mut sortie = String::new();
    for (valeur, signe) in TABLE {
        while reste >= valeur {
            sortie.push_str(signe);
            reste -= valeur;
        }
    }
    for (valeur, signe) in PETITS {
        while reste >= valeur {
            sortie.push_str(signe);
            reste -= valeur;
        }
    }
    sortie
}

/// Le nombre d'un chapitre ou d'un verset, en chiffres arabes ou romains.
pub fn nombre(n: u32, chiffres: Chiffres) -> String {
    match chiffres {
        Chiffres::Arabes => n.to_string(),
        Chiffres::Romains => romain(n),
    }
}

/// Le nom français d'un livre OSIS, selon la forme demandée.
///
/// Un code inconnu est rendu tel quel : l'affichage ne perd jamais la
/// référence, même si la table ne la connaît pas.
pub fn nom_du_livre(livre: &str, forme: FormeLivre) -> String {
    let Some(l) = LIVRES_FR.iter().find(|l| l.osis == livre) else {
        return livre.to_string();
    };
    match forme {
        FormeLivre::FrancaiseLongue => l.nom.to_string(),
        FormeLivre::FrancaiseCourte => l.fr.to_string(),
        FormeLivre::SansPoint => l.fr.trim_end_matches('.').to_string(),
        FormeLivre::Osis => l.abreviation.to_string(),
        FormeLivre::Tob => abreviation_tob(livre).unwrap_or(l.fr).to_string(),
    }
}

/// Une référence écrite selon les normes : « Jean 3.16 ».
///
/// Le gabarit des normes décide de l'ordre et de l'habillage : par défaut
/// `{livre} {nombres}`, mais il peut devenir `{livre} {chapitre}, {verset}`,
/// `({livre} {nombres})`, etc. Les jetons inconnus restent tels quels — une
/// faute de gabarit se voit, elle ne perd pas la référence.
pub fn afficher(reference: &Reference, normes: &crate::mise_en_page::modele::References) -> String {
    let mut sortie = afficher_sans_mention(reference, normes);
    sortie.push_str(&mention_traduction(reference));
    sortie
}

/// La mention d'une traduction explicite, ou une chaîne vide.
pub fn mention_traduction(reference: &Reference) -> String {
    reference
        .traduction
        .map(|traduction| format!(" ({})", traduction.abreviation()))
        .unwrap_or_default()
}

/// Une référence écrite selon les normes, sans la mention de traduction.
fn afficher_sans_mention(
    reference: &Reference,
    normes: &crate::mise_en_page::modele::References,
) -> String {
    let livre = nom_du_livre(&reference.livre, forme_du_livre(normes.livre));
    let nombres = afficher_nombres(reference, normes);
    let chapitres = afficher_chapitres(reference, normes);
    let versets = afficher_versets(reference, normes);
    let gabarit = if normes.gabarit.trim().is_empty() {
        "{livre} {nombres}"
    } else {
        normes.gabarit.as_str()
    };

    gabarit
        .replace("{livre}", &livre)
        .replace("{nombres}", &nombres)
        .replace("{chapitres}", &chapitres)
        .replace("{versets}", &versets)
        .replace(
            "{chapitre_fin}",
            &nombre(reference.chapitre_fin, chiffres_du_gabarit(normes)),
        )
        .replace(
            "{chapitre}",
            &nombre(reference.chapitre, chiffres_du_gabarit(normes)),
        )
        .replace(
            "{verset_fin}",
            &if reference.verset_fin == u32::MAX {
                String::new()
            } else {
                nombre(reference.verset_fin, chiffres_du_gabarit(normes))
            },
        )
        .replace(
            "{verset}",
            &if reference.verset_fin == u32::MAX && reference.verset_debut == 1 {
                String::new()
            } else {
                nombre(reference.verset_debut, chiffres_du_gabarit(normes))
            },
        )
        .replace("{reference}", &format!("{livre} {nombres}"))
}

/// Les chapitres d'une référence, avec leur éventuelle suite.
fn afficher_chapitres(
    reference: &Reference,
    normes: &crate::mise_en_page::modele::References,
) -> String {
    let chiffres = chiffres_du_gabarit(normes);
    let chapitre = nombre(reference.chapitre, chiffres);
    if reference.est_plage_de_chapitres() {
        format!(
            "{chapitre}{}{}",
            normes.separateur_chapitres,
            nombre(reference.chapitre_fin, chiffres)
        )
    } else {
        chapitre
    }
}

/// Les versets d'une référence, sans son chapitre.
fn afficher_versets(
    reference: &Reference,
    normes: &crate::mise_en_page::modele::References,
) -> String {
    let chiffres = chiffres_du_gabarit(normes);
    if reference.verset_fin == u32::MAX {
        return String::new();
    }
    if reference.verset_fin > reference.verset_debut {
        format!(
            "{}{}{}",
            nombre(reference.verset_debut, chiffres),
            normes.separateur_plage,
            nombre(reference.verset_fin, chiffres)
        )
    } else {
        nombre(reference.verset_debut, chiffres)
    }
}

fn chiffres_du_gabarit(normes: &crate::mise_en_page::modele::References) -> Chiffres {
    match normes.chiffres {
        crate::mise_en_page::modele::Chiffres::Arabes => Chiffres::Arabes,
        crate::mise_en_page::modele::Chiffres::Romains => Chiffres::Romains,
    }
}

/// Le livre tel que les normes le demandent.
pub(crate) fn forme_du_livre(forme: crate::mise_en_page::modele::SystemeAbreviation) -> FormeLivre {
    use crate::mise_en_page::modele::SystemeAbreviation;
    match forme {
        SystemeAbreviation::FrancaiseCourte => FormeLivre::FrancaiseCourte,
        SystemeAbreviation::FrancaiseLongue => FormeLivre::FrancaiseLongue,
        SystemeAbreviation::SansPoint => FormeLivre::SansPoint,
        SystemeAbreviation::Osis => FormeLivre::Osis,
        SystemeAbreviation::Tob => FormeLivre::Tob,
    }
}

/// Les chapitres et versets d'une référence, sans son livre.
///
/// Une suite de chapitres entiers s'écrit « 1 à 11 » ; une plage qui
/// traverse deux chapitres s'écrit « 1.1–2.3 ».
pub fn afficher_nombres(
    reference: &Reference,
    normes: &crate::mise_en_page::modele::References,
) -> String {
    let chiffres = match normes.chiffres {
        crate::mise_en_page::modele::Chiffres::Arabes => Chiffres::Arabes,
        crate::mise_en_page::modele::Chiffres::Romains => Chiffres::Romains,
    };
    let chapitre = nombre(reference.chapitre, chiffres);

    if reference.est_plage_de_chapitres() {
        if reference.verset_debut == 1 && reference.verset_fin == u32::MAX {
            return format!(
                "{chapitre}{}{}",
                normes.separateur_chapitres,
                nombre(reference.chapitre_fin, chiffres)
            );
        }
        return format!(
            "{chapitre}{}{}{}{}{}{}",
            normes.separateur_chapitre_verset,
            reference.verset_debut,
            normes.separateur_plage,
            nombre(reference.chapitre_fin, chiffres),
            normes.separateur_chapitre_verset,
            reference.verset_fin
        );
    }

    if reference.verset_fin == u32::MAX {
        return chapitre;
    }

    if reference.verset_fin > reference.verset_debut {
        format!(
            "{chapitre}{}{}{}{}",
            normes.separateur_chapitre_verset,
            reference.verset_debut,
            normes.separateur_plage,
            reference.verset_fin
        )
    } else {
        format!(
            "{chapitre}{}{}",
            normes.separateur_chapitre_verset, reference.verset_debut
        )
    }
}

/// Un groupe de références : « Jean 3.16 ; Gen 1.1-3 ».
///
/// Le même livre n'est nommé qu'une fois : ses occurrences sont séparées par
/// `separateur_occurrence` (« 2 Ch 5.6, 7.6-12 »).
pub fn afficher_groupe(
    references: &[Reference],
    normes: &crate::mise_en_page::modele::References,
) -> String {
    let elements: Vec<ElementCitation> = references
        .iter()
        .cloned()
        .map(ElementCitation::Reference)
        .collect();
    afficher_elements(&elements, normes)
}

/// Une partie d'une citation : une référence, ou du texte libre entre
/// guillemets dans la balise.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ElementCitation {
    /// Une référence biblique.
    Reference(Reference),
    /// Le texte libre d'une balise, guillemets retirés.
    Texte(String),
}

impl ElementCitation {
    /// La référence, quand cette partie en est une.
    pub fn reference(&self) -> Option<&Reference> {
        match self {
            ElementCitation::Reference(reference) => Some(reference),
            ElementCitation::Texte(_) => None,
        }
    }
}

/// Affiche une citation complète : références groupées par livre et texte
/// libre conservé à sa place.
pub fn afficher_elements(
    elements: &[ElementCitation],
    normes: &crate::mise_en_page::modele::References,
) -> String {
    afficher_elements_avec(elements, normes, |texte| texte.to_string())
}

/// Le cœur de l'affichage : le texte libre passe par `transformer`, ce qui
/// permet à la génération Typst de l'échapper sans changer l'aperçu.
fn afficher_elements_avec(
    elements: &[ElementCitation],
    normes: &crate::mise_en_page::modele::References,
    mut transformer: impl FnMut(&str) -> String,
) -> String {
    let mut morceaux = Vec::new();
    let mut references = Vec::new();

    for element in elements {
        match element {
            ElementCitation::Reference(reference) => references.push(reference.clone()),
            ElementCitation::Texte(texte) => {
                if !references.is_empty() {
                    morceaux.push(afficher_run(&references, normes));
                    references.clear();
                }
                if !texte.trim().is_empty() {
                    morceaux.push(transformer(texte.trim()));
                }
            }
        }
    }
    if !references.is_empty() {
        morceaux.push(afficher_run(&references, normes));
    }

    morceaux.join(normes.separateur_livre.as_str())
}

/// Affiche une suite de références : les livres groupés, les occurrences
/// séparées par une virgule, dans l'ordre demandé.
fn afficher_run(
    references: &[Reference],
    normes: &crate::mise_en_page::modele::References,
) -> String {
    let mut references = references.to_vec();
    if normes.ordre_canonique {
        references.sort_by_key(|reference| {
            (
                rang_du_livre(&reference.livre).unwrap_or(usize::MAX),
                reference.chapitre,
                reference.chapitre_fin,
                reference.verset_debut,
                reference.verset_fin,
            )
        });
    }

    let mut livres: Vec<String> = Vec::new();
    for reference in &references {
        if !livres.iter().any(|livre| livre == &reference.livre) {
            livres.push(reference.livre.clone());
        }
    }

    let mut groupes = Vec::new();
    for livre in livres {
        let mut du_livre: Vec<&Reference> = references
            .iter()
            .filter(|reference| reference.livre == livre)
            .collect();
        du_livre.sort_by_key(|reference| {
            (
                reference.chapitre,
                reference.chapitre_fin,
                reference.verset_debut,
                reference.verset_fin,
            )
        });

        let mut rendu = String::new();
        for (rang, reference) in du_livre.iter().enumerate() {
            if rang > 0 {
                rendu.push_str(&normes.separateur_occurrence);
                rendu.push_str(&afficher_nombres(reference, normes));
                rendu.push_str(&mention_traduction(reference));
            } else {
                rendu.push_str(&afficher(reference, normes));
            }
        }
        groupes.push(rendu);
    }

    // L'ordre canonique a déjà rangé les livres ; sinon le premier livre
    // rencontré garde sa place et les autres suivent leur première apparition.
    groupes.join(normes.separateur_livre.as_str())
}

/// Sépare une éventuelle traduction écrite `|lsg` ou `|darby` à la fin d'un
/// membre. Le reste de la référence est rendu inchangé.
fn separer_traduction(texte: &str) -> (&str, Option<Traduction>) {
    match texte.rsplit_once('|') {
        Some((corps, mot)) if Traduction::depuis_code(mot).is_some() => (
            corps.trim(),
            Some(Traduction::depuis_code(mot).expect("code vérifié")),
        ),
        _ => (texte, None),
    }
}

/// Une référence lue dans un texte : « Jean 3.16 », « Jn 3:16 », « Gn 1 »,
/// « Gn 1-11 ».
///
/// Le livre se dit en entier, en abréviation française ou en code OSIS ; les
/// livres numérotés acceptent « 1Jn » comme « 1 Jean » et « 2 Ch » comme
/// « 2Ch ». Le séparateur de chapitre est `.` (le standard), `:` ou `,`
/// (formes historiques). Un seul nombre désigne le chapitre entier, deux un
/// verset, et `1.1-2.3` traverse les chapitres. `1-11` — deux chapitres
/// séparés par un trait — désigne les chapitres entiers.
pub fn depuis_texte(texte: &str) -> Option<Reference> {
    let (texte, traduction) = separer_traduction(texte.trim());
    let mut reference = depuis_texte_sans_traduction(texte)?;
    reference.traduction = traduction;
    Some(reference)
}

/// Le corps de [`depuis_texte`], une fois la traduction éventuelle retirée.
fn depuis_texte_sans_traduction(texte: &str) -> Option<Reference> {
    let texte = texte.trim();

    // Le livre : le plus long préfixe qui se résout par la table, coupé
    // devant un blanc ou un chiffre. « 1 Samuel » et « 1Jn » passent ainsi.
    let mut coupe = None;
    for (i, c) in texte.char_indices().skip(1) {
        if !(c.is_whitespace() || c.is_ascii_digit()) {
            continue;
        }
        let prefixe = texte[..i].trim();
        if !prefixe.is_empty() && code_du_livre(prefixe).is_some() {
            coupe = Some(i);
        }
    }
    let coupe = coupe?;
    let osis = code_du_livre(texte[..coupe].trim())?;
    let reste = texte[coupe..].trim_start();

    if !reste.starts_with(|c: char| c.is_ascii_digit()) {
        return None;
    }

    lire_nombres(&osis, reste)
}

/// Les nombres d'une référence, une fois le livre reconnu.
fn lire_nombres(livre: &str, reste: &str) -> Option<Reference> {
    if let Some((chapitre, queue)) = reste.split_once('.') {
        let chapitre = chapitre.trim().parse::<u32>().ok()?;
        return lire_versets(livre, chapitre, queue);
    }

    // Les formes historiques : `3:16` et `3,16`. La virgule ne devient un
    // séparateur de chapitre que dans cette position ; ailleurs elle sépare
    // des versets (le groupe s'en charge).
    if let Some((chapitre, queue)) = reste.split_once(':').or_else(|| reste.split_once(',')) {
        let chapitre = chapitre.trim().parse::<u32>().ok()?;
        return lire_versets(livre, chapitre, queue);
    }

    // `Gn 1 à 11` : la forme longue des suites de chapitres.
    if let Some((debut, fin)) = reste.split_once(" à ").or_else(|| reste.split_once(" a ")) {
        let debut = debut.trim().parse::<u32>().ok()?;
        let fin = fin.trim().parse::<u32>().ok()?;
        return Some(Reference::chapitres(livre, debut, fin));
    }

    // Deux nombres séparés par un trait : des chapitres entiers. C'est la
    // demande explicite `Gn 1-11` → « Genèse 1 à 11 ».
    if let Some((debut, fin)) = reste.split_once('-').or_else(|| reste.split_once('–')) {
        let debut = debut.trim().parse::<u32>().ok()?;
        let fin = fin.trim().parse::<u32>().ok()?;
        return Some(Reference::chapitres(livre, debut, fin));
    }

    let chapitre = reste.trim().parse::<u32>().ok()?;
    Some(Reference::chapitres(livre, chapitre, chapitre))
}

/// Les versets d'un chapitre, ou une plage qui traverse les chapitres.
fn lire_versets(livre: &str, chapitre: u32, queue: &str) -> Option<Reference> {
    let queue = queue.trim();
    if queue.is_empty() {
        return Some(Reference::chapitres(livre, chapitre, chapitre));
    }

    // `1.1-2.3` : le premier membre porte le chapitre de départ, le second
    // le chapitre d'arrivée. Un seul chapitre reste un cas ordinaire.
    if let Some((debut, fin)) = queue.split_once('-').or_else(|| queue.split_once('–')) {
        let debut = debut.trim();
        let fin = fin.trim();

        if let Some((chapitre_fin, verset_fin)) = fin.split_once('.') {
            let verset_debut = debut.parse::<u32>().ok()?;
            let chapitre_fin = chapitre_fin.trim().parse::<u32>().ok()?;
            let verset_fin = verset_fin.trim().parse::<u32>().ok()?;
            return Some(Reference {
                livre: livre.to_string(),
                chapitre,
                chapitre_fin,
                verset_debut,
                verset_fin,
                traduction: None,
            });
        }

        let debut = debut.parse::<u32>().ok()?;
        let fin = fin.parse::<u32>().ok()?;
        return Some(Reference::plage(livre, chapitre, debut, fin));
    }

    let verset = queue.parse::<u32>().ok()?;
    Some(Reference::verset(livre, chapitre, verset))
}

/// Lit une **liste de références** dans une balise `{{…}}`.
///
/// Les membres se séparent par `;`. Un membre peut omettre le livre et le
/// chapitre déjà donnés : `{{Jn 3.16 ; 3.17 ; 4.1}}`, ou une liste de
/// versets du même chapitre : `{{Jn 3.16, 17, 19-20}}`. Le suffixe
/// `|long`, `|court`, `|osis` ou `|sanspoint` choisit la forme d'affichage
/// pour cette balise seulement. Un membre entre guillemets est du texte
/// libre, rendu tel quel.
///
/// `None` si aucun membre n'est une référence ; un membre invalide est
/// ignoré, comme le faisait la lecture historique.
pub fn lire_groupe(texte: &str) -> Option<(Vec<Reference>, Option<FormeLivre>)> {
    let (elements, forme) = lire_groupe_complet(texte)?;
    let references: Vec<Reference> = elements
        .into_iter()
        .filter_map(|element| match element {
            ElementCitation::Reference(reference) => Some(reference),
            ElementCitation::Texte(_) => None,
        })
        .collect();
    (!references.is_empty()).then_some((references, forme))
}

/// Lit une balise complète : références et textes libres, dans l'ordre écrit.
pub fn lire_groupe_complet(texte: &str) -> Option<(Vec<ElementCitation>, Option<FormeLivre>)> {
    let texte = texte.trim();
    let (corps, forme) = match texte.rsplit_once('|') {
        Some((corps, mot)) if forme_livre(mot).is_some() => (
            corps.trim(),
            Some(forme_livre(mot).expect("forme vérifiée")),
        ),
        _ => (texte, None),
    };

    let mut sortie = Vec::new();
    let mut livre = String::new();
    let mut chapitre = 0;

    for (debut, fin) in decouper_hors_guillemets(corps, ';') {
        let membre = corps[debut..fin].trim();
        if membre.is_empty() {
            continue;
        }

        if membre.len() >= 2 && membre.starts_with('"') && membre.ends_with('"') {
            sortie.push(ElementCitation::Texte(
                membre[1..membre.len() - 1].to_string(),
            ));
            continue;
        }

        // Une virgule est une liste de versets quand le membre porte déjà un
        // séparateur de chapitre, ou quand la lecture complète échoue
        // (`17, 18`). Sinon elle reste le séparateur historique
        // chapitre/verset (`Jn 3,16`).
        let complet = depuis_texte(membre);
        let morceaux: Vec<&str> =
            if membre.contains(':') || membre.contains('.') || complet.is_none() {
                membre.split(',').collect()
            } else {
                vec![membre]
            };

        for morceau in morceaux {
            let morceau = morceau.trim();
            if morceau.is_empty() {
                continue;
            }

            let reference = if let Some(reference) = depuis_texte(morceau) {
                reference
            } else {
                reference_relative(morceau, &livre, chapitre)?
            };

            livre = reference.livre.clone();
            chapitre = reference.chapitre;
            sortie.push(ElementCitation::Reference(reference));
        }
    }

    sortie
        .iter()
        .any(|element| matches!(element, ElementCitation::Reference(_)))
        .then_some((sortie, forme))
}

/// Découpe un texte sur un séparateur, sans couper à l'intérieur des
/// guillemets. Rend les bornes `[debut, fin)` de chaque morceau.
fn decouper_hors_guillemets(texte: &str, separateur: char) -> Vec<(usize, usize)> {
    let mut morceaux = Vec::new();
    let mut debut = 0;
    let mut dans_guillemets = false;

    for (index, caractere) in texte.char_indices() {
        if caractere == '"' {
            dans_guillemets = !dans_guillemets;
        } else if caractere == separateur && !dans_guillemets {
            morceaux.push((debut, index));
            debut = index + caractere.len_utf8();
        }
    }
    morceaux.push((debut, texte.len()));
    morceaux
}

/// Les références d'une liste, avec **leurs positions** dans le texte.
///
/// C'est ce que le survol consulte pour dire le verset exactement sous la
/// souris, et non le premier de la balise : `{{Jn 3:16, 17}}` a deux
/// positions, comme `{{Jn 3:16 ; Rm 8:1}}`.
pub fn references_situees(texte: &str) -> Vec<(usize, usize, Reference)> {
    let corps = match texte.rsplit_once('|') {
        Some((corps, mot)) if forme_livre(mot).is_some() => corps,
        _ => texte,
    };

    let mut sortie = Vec::new();
    let mut livre = String::new();
    let mut chapitre = 0;

    for (debut_membre, fin_membre) in decouper_hors_guillemets(corps, ';') {
        let brut = &corps[debut_membre..fin_membre];
        let decalage = brut.len() - brut.trim_start().len();
        let membre = brut.trim();
        if membre.is_empty() {
            continue;
        }

        // Même règle que `lire_groupe` : une virgule sépare des versets si le
        // membre porte un deux-points ; sinon elle sépare chapitre et verset.
        let complet = depuis_texte(membre);
        let morceaux: Vec<(usize, usize)> = if membre.contains(':') || complet.is_none() {
            let mut morceaux = Vec::new();
            let mut curseur = 0;
            for morceau in membre.split(',') {
                let debut = curseur + (morceau.len() - morceau.trim_start().len());
                let fin = debut + morceau.trim().len();
                morceaux.push((debut, fin));
                curseur += morceau.len() + 1;
            }
            morceaux
        } else {
            vec![(0, membre.len())]
        };

        for (debut, fin) in morceaux {
            let morceau = &membre[debut..fin];
            if morceau.is_empty() {
                continue;
            }
            let reference = if let Some(reference) = depuis_texte(morceau) {
                reference
            } else if let Some(reference) = reference_relative(morceau, &livre, chapitre) {
                reference
            } else {
                continue;
            };

            livre = reference.livre.clone();
            chapitre = reference.chapitre;
            sortie.push((
                debut_membre + decalage + debut,
                debut_membre + decalage + fin,
                reference,
            ));
        }
    }

    sortie
}

/// La forme d'affichage nommée après la barre verticale.
fn forme_livre(mot: &str) -> Option<FormeLivre> {
    match mot.trim().to_lowercase().as_str() {
        "long" | "longue" | "francaise_longue" => Some(FormeLivre::FrancaiseLongue),
        "court" | "courte" | "francaise_courte" => Some(FormeLivre::FrancaiseCourte),
        "osis" => Some(FormeLivre::Osis),
        "sanspoint" | "sans_point" => Some(FormeLivre::SansPoint),
        "tob" => Some(FormeLivre::Tob),
        _ => None,
    }
}

/// Un membre relatif au livre et au chapitre précédents : `3:17`, `17-18`.
fn reference_relative(texte: &str, livre: &str, chapitre: u32) -> Option<Reference> {
    let (texte, traduction) = separer_traduction(texte.trim());
    let mut reference = reference_relative_sans_traduction(texte, livre, chapitre)?;
    reference.traduction = traduction;
    Some(reference)
}

fn reference_relative_sans_traduction(
    texte: &str,
    livre: &str,
    chapitre: u32,
) -> Option<Reference> {
    if livre.is_empty() || chapitre == 0 {
        return None;
    }

    if let Some((chapitre, queue)) = texte.split_once(':').or_else(|| texte.split_once('.')) {
        let chapitre = chapitre.trim().parse::<u32>().ok()?;
        return lire_versets(livre, chapitre, queue);
    }

    let nombres: Vec<u32> = texte
        .split(['-', '\u{2013}'])
        .map(str::trim)
        .filter(|morceau| !morceau.is_empty())
        .map(|morceau| morceau.parse::<u32>().ok())
        .collect::<Option<Vec<_>>>()?;
    match nombres.as_slice() {
        [verset] => Some(Reference::verset(livre, chapitre, *verset)),
        // Dans un membre relatif, deux nombres restent une plage de versets
        // du chapitre courant : `{{Jn 3.16, 17-18}}`.
        [debut, fin] => Some(Reference::plage(livre, chapitre, *debut, *fin)),
        _ => None,
    }
}

/// Une citation complète avec une forme de livre imposée par la balise.
pub fn afficher_elements_forme(
    elements: &[ElementCitation],
    normes: &crate::mise_en_page::modele::References,
    forme: Option<FormeLivre>,
) -> String {
    afficher_elements_forme_avec(elements, normes, forme, |texte| texte.to_string())
}

/// Une citation rendue en Typst : le texte libre est échappé pour le moteur.
pub fn afficher_elements_typst(
    elements: &[ElementCitation],
    normes: &crate::mise_en_page::modele::References,
    forme: Option<FormeLivre>,
) -> String {
    afficher_elements_forme_avec(elements, normes, forme, echapper_pour_typst)
}

/// Le cœur de l'affichage avec une forme de livre éventuelle.
fn afficher_elements_forme_avec(
    elements: &[ElementCitation],
    normes: &crate::mise_en_page::modele::References,
    forme: Option<FormeLivre>,
    transformer: impl FnMut(&str) -> String,
) -> String {
    let Some(forme) = forme else {
        return afficher_elements_avec(elements, normes, transformer);
    };
    let mut normes = normes.clone();
    normes.livre = match forme {
        FormeLivre::FrancaiseCourte => {
            crate::mise_en_page::modele::SystemeAbreviation::FrancaiseCourte
        }
        FormeLivre::FrancaiseLongue => {
            crate::mise_en_page::modele::SystemeAbreviation::FrancaiseLongue
        }
        FormeLivre::Osis => crate::mise_en_page::modele::SystemeAbreviation::Osis,
        FormeLivre::SansPoint => crate::mise_en_page::modele::SystemeAbreviation::SansPoint,
        FormeLivre::Tob => crate::mise_en_page::modele::SystemeAbreviation::Tob,
    };
    afficher_elements_avec(elements, &normes, transformer)
}

/// Le texte libre tel qu'il s'écrit dans une balise : entre guillemets, sans
/// guillemets doublés. Un guillemet interne est remplacé par deux apostrophes.
pub fn texte_libre_source(texte: &str) -> String {
    format!("\"{}\"", texte.trim().trim_matches('"').replace('"', "''"))
}

/// Écrit une citation dans la **syntaxe source** : une référence par
/// membre, le texte libre entre guillemets, les membres séparés par ` ; `.
/// C'est la forme qui se relit sans perdre le texte libre.
///
/// La source est volontairement écrite dans la forme canonique, et non avec
/// le gabarit d'affichage : un gabarit libre (`({reference})`, par exemple)
/// ne serait pas relu par le parseur.
pub fn ecrire_elements(
    elements: &[ElementCitation],
    _normes: &crate::mise_en_page::modele::References,
) -> String {
    let canonique = crate::mise_en_page::modele::References::default();
    elements
        .iter()
        .map(|element| match element {
            ElementCitation::Reference(reference) => ecrire_reference_source(reference, &canonique),
            ElementCitation::Texte(texte) => texte_libre_source(texte),
        })
        .collect::<Vec<_>>()
        .join(" ; ")
}

/// Une référence dans la syntaxe source : sa forme d'affichage, puis le code
/// de traduction explicite (`|lsg`, `|darby`) s'il y en a un.
fn ecrire_reference_source(
    reference: &Reference,
    normes: &crate::mise_en_page::modele::References,
) -> String {
    let mut source = afficher_sans_mention(reference, normes);
    if let Some(traduction) = reference.traduction {
        source.push('|');
        source.push_str(traduction.code());
    }
    source
}

/// Échappe le texte libre pour Typst : les caractères de balisage deviennent
/// du texte, jamais du code. L'aperçu et la source gardent, eux, le texte nu.
fn echapper_pour_typst(texte: &str) -> String {
    let mut sortie = String::new();
    for caractere in texte.chars() {
        if matches!(
            caractere,
            '\\' | '#'
                | '*'
                | '_'
                | '`'
                | '$'
                | '@'
                | '^'
                | '<'
                | '>'
                | '['
                | ']'
                | '~'
                | '|'
                | '-'
                | '+'
                | '='
                | '/'
        ) {
            sortie.push('\\');
        }
        sortie.push(caractere);
    }
    sortie
}

/// Une référence écrite selon les normes, avec une forme de livre imposée
/// par la balise.
pub fn afficher_groupe_forme(
    references: &[Reference],
    normes: &crate::mise_en_page::modele::References,
    forme: Option<FormeLivre>,
) -> String {
    let elements: Vec<ElementCitation> = references
        .iter()
        .cloned()
        .map(ElementCitation::Reference)
        .collect();
    afficher_elements_forme(&elements, normes, forme)
}

/// Le rang canonique d'un livre, à partir de son code OSIS.
pub fn rang_du_livre(code: &str) -> Option<usize> {
    LIVRES_FR
        .iter()
        .position(|livre| livre.osis.eq_ignore_ascii_case(code))
}

/// Le code OSIS d'un livre nommé en français, en abréviation ou en OSIS.
pub fn code_du_livre(nom: &str) -> Option<String> {
    let normalise = |texte: &str| {
        texte
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>()
            .to_lowercase()
    };
    let cherche = normalise(nom);
    for l in LIVRES_FR {
        if normalise(l.osis) == cherche
            || normalise(l.nom) == cherche
            || normalise(l.abreviation) == cherche
            || normalise(l.fr) == cherche
        {
            return Some(l.osis.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests_des_normes_tob {
    use super::*;

    #[test]
    fn les_abreviations_tob_sont_courtes_et_stables() {
        assert_eq!(nom_du_livre("1Sam", FormeLivre::Tob), "1 S");
        assert_eq!(nom_du_livre("Isa", FormeLivre::Tob), "Is");
        assert_eq!(nom_du_livre("Heb", FormeLivre::Tob), "He");
        assert_eq!(nom_du_livre("1Macc", FormeLivre::Tob), "1 M");
        assert_eq!(nom_du_livre("Jub", FormeLivre::Tob), "Jub");
    }

    #[test]
    fn une_reference_accentuee_se_lit_sans_paniquer() {
        let reference = depuis_texte("Genèse 3:16").expect("le nom accentué se résout");
        assert_eq!(reference.livre, "Gen");

        let (references, _) = lire_groupe("Genèse 3:16, 17").expect("le groupe accentué");
        assert_eq!(references.len(), 2, "les deux versets sont lus");
        assert_eq!(references[1].verset_debut, 17);

        let situees = references_situees("Genèse 3:16, 17");
        assert_eq!(situees.len(), 2, "les positions restent justes");
        assert_eq!(
            &"Genèse 3:16, 17"[situees[1].0..situees[1].1],
            "17",
            "la seconde position ne coupe pas dans un accent"
        );
    }

    #[test]
    fn les_abreviations_tob_nomment_des_livres_connus() {
        for (code, abreviation) in ABREVIATIONS_TOB {
            assert!(
                LIVRES_FR.iter().any(|livre| livre.osis == *code),
                "l'abréviation TOB « {abreviation} » vise un livre absent : {code}"
            );
        }
    }

    #[test]
    fn les_livres_hors_canon_sont_citables() {
        assert!(
            livres_hors_canon()
                .iter()
                .any(|(osis, _, _)| *osis == "1En")
        );
        assert!(
            livres_hors_canon()
                .iter()
                .any(|(osis, _, _)| *osis == "Jub")
        );
        assert_eq!(
            code_du_livre("1 Hénoch").as_deref(),
            Some("1En"),
            "le nom long se résout"
        );
        let reference = depuis_texte("1 Hén 1:1").expect("une référence hors canon");
        assert_eq!(reference.livre, "1En");

        let testament = depuis_texte("T. Rub. 1:1").expect("un Testament des patriarches");
        assert_eq!(testament.livre, "TReu");
    }

    #[test]
    fn la_forme_tob_est_lue_et_affichee() {
        let normes = crate::mise_en_page::modele::References {
            livre: crate::mise_en_page::modele::SystemeAbreviation::Tob,
            ..Default::default()
        };
        assert_eq!(
            afficher(&Reference::verset("1Sam", 1, 1), &normes),
            "1 S 1.1"
        );
        assert_eq!(afficher(&Reference::verset("Isa", 6, 1), &normes), "Is 6.1");

        let (references, forme) = lire_groupe("1 S 1:1|tob").expect("le groupe TOB");
        assert_eq!(forme, Some(FormeLivre::Tob), "la forme est gardée");
        assert_eq!(references[0].livre, "1Sam");
    }
}
