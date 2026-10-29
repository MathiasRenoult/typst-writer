//! Les polices : celles que Typst embarque, plus celles que le document demande.
//!
//! **Les embarquées, et correctement.** Typst livre ses propres familles, mais
//! `Font::new(data, index)` attend l'index du **visage dans une collection de
//! fichiers** : lui donner le rang du *fichier* ne retient que le premier et
//! laisse tomber tous les autres. C'est ce que faisait l'application — mesuré :
//! **une** police retenue sur les dix-sept embarquées. Tout document était donc
//! composé avec celle-là, quelle que soit celle qu'il demandait. `Font::iter`
//! parcourt tous les visages de chaque fichier, et c'est ce qu'il faut.
//!
//! **Celles du document ensuite.** Un texte qui demande « Galatia SIL » pour le
//! grec ou « Ezra SIL SR » pour l'hébreu — le cas d'un travail de théologie — ne
//! les trouvait pas, et se composait avec ce qui restait. L'aperçu cessait de
//! dire la vérité, et l'export aussi : c'est la promesse entière de
//! l'application qui tombait là.
//!
//! **Pas celles du système en bloc.** Les polices installées pèsent ici plus
//! d'un gigaoctet : les charger toutes reproduirait mot pour mot l'explosion de
//! mémoire dont l'utilisateur avait souffert. Seules sont chargées les familles
//! que le document **nomme**, et c'est `fc-match` qui dit quel fichier leur
//! correspond — le même service que rend le compilateur de référence.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock};

use typst::foundations::Bytes;
use typst::text::{Font, FontBook};

/// Le jeu de polices, partagé par toute l'application.
///
/// Il est construit une fois : parcourir et analyser des fichiers de police
/// coûte des millisecondes par police, et le contenu du système ne change pas
/// pendant qu'on écrit.
pub fn shared() -> MutexGuard<'static, Fonts> {
    static FONTS: OnceLock<Mutex<Fonts>> = OnceLock::new();

    FONTS
        .get_or_init(|| Mutex::new(Fonts::embarked()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Les polices retenues pour composer, et de quoi en ajouter.
pub struct Fonts {
    /// Les familles déjà reconnues, et les octets de leur fichier.
    ///
    /// Gardées, pour qu'une recompilation n'ait pas à relire le disque ni à
    /// redemander à fontconfig ce qu'il a déjà répondu.
    extra: Vec<(String, Vec<u8>)>,

    /// Les visages, dans l'ordre du livre. L'index du livre est la position ici.
    list: Vec<Font>,

    /// Le livre : c'est lui que Typst interroge pour trouver une famille.
    book: FontBook,
}

impl Fonts {
    /// Les seules polices embarquées : de quoi démarrer, et de quoi composer si
    /// le document ne demande rien d'installé.
    fn embarked() -> Self {
        Self::from(Vec::new())
    }

    fn from(extra: Vec<(String, Vec<u8>)>) -> Self {
        // `Font::iter` : **tous** les visages de chaque fichier embarqué.
        let mut list: Vec<Font> = typst_assets::fonts()
            .flat_map(|data| Font::iter(Bytes::new(data.to_vec())))
            .collect();

        for (_, bytes) in &extra {
            list.extend(Font::iter(Bytes::new(bytes.clone())));
        }

        let book = FontBook::from_fonts(list.iter());

        Self { extra, list, book }
    }

    /// Ajoute les familles qu'un texte nomme, si le système les possède.
    ///
    /// La question se pose à chaque compilation : le texte peut nommer une
    /// police installée qu'on n'a jamais rencontrée. Le coût est celui d'un
    /// parcours de chaînes, et d'un appel à fontconfig par nom **nouveau**
    /// seulement.
    pub fn complete(&mut self, root: &Path, texts: &[&str]) {
        let known: Vec<&str> = self.extra.iter().map(|(name, _)| name.as_str()).collect();

        let mut added = Vec::new();

        for family in named_families(texts, root) {
            if known.contains(&family.as_str()) || added.iter().any(|(name, _)| *name == family) {
                continue;
            }

            let Some(bytes) = installed_font(&family) else {
                continue;
            };

            added.push((family, bytes));
        }

        if added.is_empty() {
            return;
        }

        self.extra.extend(added);

        // Le livre est reconstruit **en entier** : ses index sont les positions
        // dans la liste des visages, et c'est cet ordre que Typst utilise.
        *self = Self::from(std::mem::take(&mut self.extra));
    }

    /// Les visages et le livre, tels que le monde Typst les attend.
    pub fn parts(&self) -> (Vec<Font>, FontBook) {
        (self.list.clone(), self.book.clone())
    }

    /// Combien de visages le jeu retient — pour les épreuves et le diagnostic.
    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.list.len()
    }

    /// Les familles de polices connues — celles que la liste des normes
    /// propose, et que Typst sait composer.
    pub fn familles(&self) -> Vec<String> {
        let mut familles: Vec<String> = self
            .list
            .iter()
            .map(|police| police.info().family.clone())
            .collect();

        familles.sort();
        familles.dedup();
        familles
    }

    /// Les familles reconnues du système — pour les épreuves et le diagnostic.
    /// En dehors des épreuves, c'est `complete` qui travaille, et personne n'a
    /// besoin de relire la liste.
    #[cfg(test)]
    pub fn recognised(&self) -> Vec<String> {
        self.extra.iter().map(|(name, _)| name.clone()).collect()
    }
}

/// Les familles de polices qu'un ensemble de textes nomme.
///
/// C'est une **heuristique**, et elle suffit : ce qui est cherché, ce sont les
/// noms cités dans un document — « Times New Roman », « Ezra SIL SR ». Une
/// chaîne qui ne correspond à aucune famille installée est écartée plus loin,
/// par la comparaison avec ce que fontconfig rend.
fn named_families(texts: &[&str], root: &Path) -> Vec<String> {
    let mut names = Vec::new();

    for text in texts {
        names.extend(quoted(text));
    }

    // Les chapitres que l'assemblage inclut sans qu'ils soient ouverts : leur
    // texte n'est nulle part en mémoire, et un document peut très bien demander
    // sa police hébraïque dans le chapitre qu'on n'écrit pas. Le résultat se
    // garde : relire tout le projet à chaque compilation — toutes les trois
    // secondes d'écriture — est un coût pour rien.
    names.extend(noms_des_fichiers_typ(root));

    names.sort_unstable();
    names.dedup();

    names
}

/// Les noms cités par les `.typ` du projet, gardés entre deux compilations.
///
/// Le cache tient tant qu'aucun fichier n'a changé : c'est la date la plus
/// récente des `.typ` qui le dit, et un fichier touché relit le projet.
fn noms_des_fichiers_typ(root: &Path) -> Vec<String> {
    use std::sync::Mutex;
    use std::time::SystemTime;

    static CACHE: Mutex<Vec<(PathBuf, SystemTime, Vec<String>)>> = Mutex::new(Vec::new());

    let fichiers = typ_files(root);
    let plus_recent = fichiers
        .iter()
        .filter_map(|path| std::fs::metadata(path).ok())
        .filter_map(|meta| meta.modified().ok())
        .max()
        .unwrap_or(SystemTime::UNIX_EPOCH);

    if let Ok(caches) = CACHE.lock()
        && let Some((_, garde, noms)) = caches.iter().find(|(racine, _, _)| racine == root)
        && *garde >= plus_recent
    {
        return noms.clone();
    }

    let mut noms = Vec::new();
    for path in &fichiers {
        if let Ok(texte) = std::fs::read_to_string(path) {
            noms.extend(quoted(&texte));
        }
    }

    if let Ok(mut caches) = CACHE.lock() {
        caches.retain(|(racine, _, _)| racine != root);
        caches.push((root.to_path_buf(), plus_recent, noms.clone()));
    }

    noms
}

/// Les chaînes citées d'un texte/// Les chaînes citées d'un texte : ce sont elles qui peuvent être des noms de
/// polices.
fn quoted(text: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut rest = text;

    while let Some(start) = rest.find('"') {
        let after = &rest[start + 1..];

        let Some(end) = after.find('"') else {
            break;
        };

        let value = &after[..end];

        // Un nom de famille tient sur une ligne et n'a rien d'un chemin.
        if !value.is_empty() && value.len() < 64 && !value.contains(['\n', '/', '\\']) {
            names.push(value.to_string());
        }

        rest = &after[end + 1..];
    }

    names
}

/// Les octets d'une famille installée, si le système la possède vraiment.
///
/// `fc-match` rend **toujours** quelque chose : pour un nom inconnu, la police
/// par défaut. C'est donc la famille **rendue** qui est comparée au nom
/// demandé ; c'est ce qui distingue « cette famille est installée » de
/// « fontconfig a fait au mieux ».
fn installed_font(family: &str) -> Option<Vec<u8>> {
    let output = std::process::Command::new("fc-match")
        .args(["-f", "%{family}\t%{file}", family])
        .output()
        .ok()?;

    let answer = String::from_utf8_lossy(&output.stdout);
    let (rendered, file) = answer.split_once('\t')?;

    if family_key(rendered) != family_key(family) {
        return None;
    }

    std::fs::read(file.trim()).ok()
}

/// La forme d'un nom de famille, pour les comparer : la casse, les espaces et
/// les tirets ne distinguent rien — « Times New Roman » et « timesnewroman »
/// désignent la même famille, et « Galatia SIL » s'écrit des deux façons selon
/// qui l'écrit.
fn family_key(name: &str) -> String {
    name.chars()
        .filter(|character| character.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

/// Les fichiers `.typ` d'un projet, récursivement.
fn typ_files(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut pending = vec![root.to_path_buf()];

    while let Some(dir) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };

        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|extension| extension == "typ") {
                found.push(path);
            }
        }
    }

    found
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Les chaînes citées sont des noms candidats, et rien de plus.
    #[test]
    fn quoted_strings_are_the_candidates() {
        let text =
            "#set text(font: \"Times New Roman\")\n#let f = \"Galatia SIL\"\n#figure(\"un mot\")";

        assert_eq!(
            quoted(text),
            vec![
                "Times New Roman".to_string(),
                "Galatia SIL".to_string(),
                "un mot".to_string()
            ]
        );

        // Une chaîne non fermée ne doit pas avaler la suite.
        assert_eq!(quoted("oublie \"ouverture"), Vec::<String>::new());

        // Ni un chemin, ni une chaîne à rallonge : ce ne sont pas des familles.
        assert_eq!(quoted("\"/usr/share/fonts/x.ttf\""), Vec::<String>::new());
        assert_eq!(
            quoted(&format!("\"{}\"", "a".repeat(80))),
            Vec::<String>::new()
        );
    }

    /// Deux écritures d'un même nom de famille se rejoignent.
    #[test]
    fn a_family_name_is_compared_loosely() {
        assert_eq!(family_key("Times New Roman"), family_key("times new roman"));
        assert_eq!(family_key("Galatia SIL"), family_key("GalatiaSIL"));

        assert_ne!(family_key("Galatia SIL"), family_key("Liberation Sans"));
    }

    /// Ce que le système sait rendre, pour les noms d'un document réel.
    ///
    ///     cargo test --release -- --ignored diagnose_fonts --nocapture
    #[test]
    #[ignore]
    fn diagnose_fonts() {
        let texts = vec![
            "#set text(font: \"Times New Roman\")\n#let gr(body) = text(font: \"Galatia SIL\")[#body]\n#let he(body) = text(font: \"Ezra SIL SR\")[#body]\n",
        ];

        let mut shared = shared();

        println!("visages embarqués retenus : {}", shared.len());

        for font in &shared.list {
            println!(
                "  embarquée : {} (visage {})",
                font.info().family,
                font.index()
            );
        }

        shared.complete(Path::new("."), &texts);

        for family in shared.recognised() {
            println!("  famille installée reconnue : {family}");
        }

        println!("visages au total : {}", shared.len());

        println!(
            "familles embarquées : {} visage(s)",
            Fonts::embarked().len()
        );

        for name in [
            "times new roman",
            "galatia sil",
            "ezra sil sr",
            "new computer modern",
        ] {
            println!(
                "  « {name} » dans le jeu : {}",
                shared.book.contains_family(name)
            );
        }
    }

    /// **L'épreuve qui manquait.** Les polices embarquées sont dix-sept ; donner
    /// le rang du fichier comme index de visage n'en retenait qu'une.
    #[test]
    fn every_embedded_face_is_kept() {
        let fonts = Fonts::embarked();

        assert!(
            fonts.len() > 1,
            "une seule police retenue : les visages embarqués sont perdus"
        );

        // Les clés du livre sont **en minuscules** (`FontBook::push`), et c'est
        // ainsi que Typst interroge : la casse d'un nom de famille ne compte pas.
        assert!(
            fonts.book.contains_family("libertinus serif"),
            "la police par défaut de Typst doit être dans le jeu"
        );

        assert!(
            fonts.book.contains_family("new computer modern"),
            "la police embarquée pour les mathématiques doit être là"
        );
    }
}
