//! La balise `{date:...}` — siècles, millénaires, dates pleines.
//!
//! Nouvelle méthode (2026-11-03), qui remplace la recherche hasardeuse :
//! on **déclare** la date, l'appli la compose.
//!
//! ## Les deux formes
//!
//! - **Chiffres romains** — un romain (majuscules ou minuscules) suivi de
//!   `s` (siècle) ou `m` (millénaire) : `{date:XIXs}`. La règle du
//!   **dernier caractère** : s'il est `m`, on parle de millénaire — compter
//!   des millénaires en milliers ne se fait pas. Un `?` demande la
//!   **forme longue** (le mot + l'époque) ; un `-` en tête indique
//!   **av. J.-C.** : `{date:-IVs?}`. Le premier dit `er`, pas `e` :
//!   `{date:Is}` → « I^er^ ».
//! - **Date pleine** — `yyyy.mm.dd`, suivi du **format d'affichage** en
//!   guillemets : `{date:2026.11.03 "dd/MM/yyyy"}`. Sans format, celui
//!   des normes sert.
//!
//! ## Les normes
//!
//! Les mots sont ceux du panneau des normes : « siècle », « millénaire »,
//! « ap. J.-C. », « av. J.-C. », et le format par défaut des dates
//! pleines. Tokens de format, **syntaxe anglophone internationale** :
//! `yyyy` (an 4), `yy` (an 2), `MM` (mois 2), `M` (mois 1), `dd`
//! (jour 2), `d` (jour 1).

/// Les mots et le format — viennent des normes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Regles {
    /// Le mot du siècle (« siècle », « s. »…).
    pub siecle: String,
    /// Le mot du millénaire.
    pub millenaire: String,
    /// L'époque après (« ap. J.-C. », « EC »…).
    pub epoque_apres: String,
    /// L'époque avant (« av. J.-C. », « AEC »…).
    pub epoque_avant: String,
    /// Le format par défaut des dates pleines.
    pub format: String,
    /// La langue des dates — pour les noms de mois et de jours
    /// (bug-report-8, T3.2). Indépendante de la langue de l'interface.
    pub langue: String,
}

impl Default for Regles {
    fn default() -> Self {
        Self {
            siecle: "siècle".to_string(),
            millenaire: "millénaire".to_string(),
            epoque_apres: "ap. J.-C.".to_string(),
            epoque_avant: "av. J.-C.".to_string(),
            format: "dd/MM/yyyy".to_string(),
            langue: "fr".to_string(),
        }
    }
}

/// Une date déclarée.
#[derive(Debug, Clone, PartialEq)]
pub enum Date {
    /// Un siècle ou un millénaire : `XIXs`, `-IVs?`, `IIIm`…
    Periode {
        /// Le nombre — 19 pour `XIX`.
        nombre: u32,
        /// `true` pour un millénaire (`m`), `false` pour un siècle (`s`).
        millenaire: bool,
        /// `true` si la forme longue est demandée (`?`).
        longue: bool,
        /// `true` si la date est av. J.-C. (`-`).
        avant: bool,
    },
    /// Une date pleine `yyyy.mm.dd`, et son format d'affichage.
    Calendrier {
        an: i32,
        mois: u32,
        jour: u32,
        format: Option<String>,
    },
    /// La date du jour, au moment de la composition.
    Aujourdhui { format: Option<String> },
    /// L'heure de la composition, avec la date.
    Maintenant { format: Option<String> },
}

/// L'heure locale, ou UTC à défaut de fuseau lisible.
///
/// `SOURCE_DATE_EPOCH` fige l'horloge pour une composition reproductible :
/// c'est le même contrat que les outils de build reproductibles, et cela
/// permet à une date dynamique de ne pas changer à chaque essai.
fn maintenant_local() -> time::OffsetDateTime {
    if let Ok(epoch) = std::env::var("SOURCE_DATE_EPOCH")
        && let Ok(secondes) = epoch.trim().parse::<i64>()
        && let Ok(date) = time::OffsetDateTime::from_unix_timestamp(secondes)
    {
        return date;
    }
    time::OffsetDateTime::now_local().unwrap_or_else(|_| time::OffsetDateTime::now_utc())
}

/// Analyse le contenu d'une balise `{date:...}`.
pub fn analyser(spec: &str) -> Option<Date> {
    let spec = spec.trim();
    if spec.is_empty() {
        return None;
    }

    // Les formes dynamiques : la date du jour, ou l'heure de composition.
    let (mot, queue) = spec.split_once(char::is_whitespace).unwrap_or((spec, ""));
    let mot_normalise = mot.to_lowercase().replace('’', "'");
    let format_dynamique = |queue: &str| -> Option<Option<String>> {
        let queue = queue.trim();
        if queue.is_empty() {
            return Some(None);
        }
        let format = queue.strip_prefix('"')?.strip_suffix('"')?;
        (!format.trim().is_empty()).then(|| Some(format.to_string()))
    };
    match mot_normalise.as_str() {
        "aujourd'hui" | "aujourdhui" => {
            return Some(Date::Aujourdhui {
                format: format_dynamique(queue)?,
            });
        }
        "maintenant" => {
            return Some(Date::Maintenant {
                format: format_dynamique(queue)?,
            });
        }
        _ => {}
    }

    // Une période : `-`? romain `s|m` `?`? — le romain en majuscules ou
    // minuscules, et la règle du **dernier caractère** : `s` = siècle,
    // `m` = millénaire (compter des millénaires en milliers ne se fait
    // pas). Le premier dit `er`, pas `e`.
    let (reste, avant) = spec
        .strip_prefix('-')
        .map(|r| (r, true))
        .unwrap_or((spec, false));
    let (reste, longue) = reste
        .strip_suffix('?')
        .map(|r| (r, true))
        .unwrap_or((reste, false));
    let periode = match reste.chars().next_back() {
        Some(c) if c.eq_ignore_ascii_case(&'s') => Some((&reste[..reste.len() - 1], false)),
        Some(c) if c.eq_ignore_ascii_case(&'m') => Some((&reste[..reste.len() - 1], true)),
        // Pas de marqueur : ce n'est pas une période.
        _ => None,
    };
    if let Some((romain, millenaire)) = periode
        && let Some(nombre) = romain_vers_nombre(romain)
    {
        return Some(Date::Periode {
            nombre,
            millenaire,
            longue,
            avant,
        });
    }

    // Une date pleine : `yyyy.mm.dd` ["format…"] — le format, en
    // guillemets, syntaxe anglophone internationale.
    let mut morceaux = spec.splitn(3, '.');
    let an: i32 = morceaux.next()?.parse().ok()?;
    let mois: u32 = morceaux.next()?.parse().ok()?;
    let reste = morceaux.next()?;
    let (jour, format) = match reste.split_once(" \"") {
        Some((jour, format)) => {
            let format = format.strip_suffix('"')?;
            (jour, Some(format.to_string()))
        }
        None => (reste, None),
    };
    let jour: u32 = jour.trim().parse().ok()?;
    if !(1..=12).contains(&mois) || !(1..=31).contains(&jour) {
        return None;
    }
    Some(Date::Calendrier {
        an,
        mois,
        jour,
        format,
    })
}

/// Compose la date — le texte affiché, où le romain est entre marqueurs
/// (`\u{2}` pour la petite capitale, `\u{3}` pour l'exposant) : c'est le
/// même canal que les siècles, substitué après l'échappement.
pub fn rendre(date: &Date, regles: &Regles) -> String {
    match date {
        Date::Periode {
            nombre,
            millenaire,
            longue,
            avant,
        } => {
            let mut out = String::new();
            out.push('\u{2}');
            out.push_str(&nombre_vers_romain(*nombre));
            out.push('\u{2}');
            out.push('\u{3}');
            // Le premier dit `er`, les autres `e`.
            out.push_str(if *nombre == 1 { "er" } else { "e" });
            out.push('\u{3}');
            if *longue {
                out.push(' ');
                out.push_str(if *millenaire {
                    &regles.millenaire
                } else {
                    &regles.siecle
                });
                out.push(' ');
                out.push_str(if *avant {
                    &regles.epoque_avant
                } else {
                    &regles.epoque_apres
                });
            }
            out
        }
        Date::Calendrier {
            an,
            mois,
            jour,
            format,
        } => {
            let format = format
                .as_deref()
                .filter(|f| !f.trim().is_empty())
                .unwrap_or(&regles.format);
            composer_format(*an, *mois, *jour, format, &regles.langue)
        }
        Date::Aujourdhui { format } => {
            let maintenant = maintenant_local();
            let format = format
                .as_deref()
                .filter(|f| !f.trim().is_empty())
                .unwrap_or(&regles.format);
            composer_format(
                maintenant.year(),
                u32::from(maintenant.month() as u8),
                u32::from(maintenant.day()),
                format,
                &regles.langue,
            )
        }
        Date::Maintenant { format } => {
            let maintenant = maintenant_local();
            let format = format
                .as_deref()
                .filter(|f| !f.trim().is_empty())
                .unwrap_or("dd/MM/yyyy HH:mm");
            composer_format_heure(
                maintenant.year(),
                u32::from(maintenant.month() as u8),
                u32::from(maintenant.day()),
                u32::from(maintenant.hour()),
                u32::from(maintenant.minute()),
                format,
                &regles.langue,
            )
        }
    }
}

/// Les tokens de format deviennent les nombres de la date.
/// Le jour de la semaine — 0 = lundi (Sakamoto, sans dépendance).
fn jour_de_la_semaine(an: i32, mois: u32, jour: u32) -> usize {
    const DECALAGES: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let an = if mois < 3 { an - 1 } else { an };
    let jour = an + an / 4 - an / 100 + an / 400 + DECALAGES[(mois - 1) as usize] + jour as i32;
    (jour.rem_euclid(7) as usize + 6) % 7 // lundi = 0
}

/// Les noms de mois et de jours, par langue (bug-report-8, T3.2) — les
/// quatre langues de l'application : français, anglais, allemand, italien.
fn nom_du_mois(mois: u32, langue: &str, abrege: bool) -> String {
    let (noms, noms_courts): (&[&str; 12], &[&str; 12]) = match langue {
        "en" => (
            &[
                "January",
                "February",
                "March",
                "April",
                "May",
                "June",
                "July",
                "August",
                "September",
                "October",
                "November",
                "December",
            ],
            &[
                "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
            ],
        ),
        "de" => (
            &[
                "Januar",
                "Februar",
                "März",
                "April",
                "Mai",
                "Juni",
                "Juli",
                "August",
                "September",
                "Oktober",
                "November",
                "Dezember",
            ],
            &[
                "Jan", "Feb", "Mär", "Apr", "Mai", "Jun", "Jul", "Aug", "Sep", "Okt", "Nov", "Dez",
            ],
        ),
        "it" => (
            &[
                "gennaio",
                "febbraio",
                "marzo",
                "aprile",
                "maggio",
                "giugno",
                "luglio",
                "agosto",
                "settembre",
                "ottobre",
                "novembre",
                "dicembre",
            ],
            &[
                "gen", "feb", "mar", "apr", "mag", "giu", "lug", "ago", "set", "ott", "nov", "dic",
            ],
        ),
        _ => (
            &[
                "janvier",
                "février",
                "mars",
                "avril",
                "mai",
                "juin",
                "juillet",
                "août",
                "septembre",
                "octobre",
                "novembre",
                "décembre",
            ],
            &[
                "janv", "févr", "mars", "avr", "mai", "juin", "juil", "août", "sept", "oct", "nov",
                "déc",
            ],
        ),
    };
    let rang = mois.clamp(1, 12) as usize - 1;
    if abrege {
        noms_courts[rang].to_string()
    } else {
        noms[rang].to_string()
    }
}

fn nom_du_jour(jour: usize, langue: &str, abrege: bool) -> String {
    let (noms, noms_courts): (&[&str; 7], &[&str; 7]) = match langue {
        "en" => (
            &[
                "Monday",
                "Tuesday",
                "Wednesday",
                "Thursday",
                "Friday",
                "Saturday",
                "Sunday",
            ],
            &["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"],
        ),
        "de" => (
            &[
                "Montag",
                "Dienstag",
                "Mittwoch",
                "Donnerstag",
                "Freitag",
                "Samstag",
                "Sonntag",
            ],
            &["Mo", "Di", "Mi", "Do", "Fr", "Sa", "So"],
        ),
        "it" => (
            &[
                "lunedì",
                "martedì",
                "mercoledì",
                "giovedì",
                "venerdì",
                "sabato",
                "domenica",
            ],
            &["lun", "mar", "mer", "gio", "ven", "sab", "dom"],
        ),
        _ => (
            &[
                "lundi", "mardi", "mercredi", "jeudi", "vendredi", "samedi", "dimanche",
            ],
            &["lun", "mar", "mer", "jeu", "ven", "sam", "dim"],
        ),
    };
    let rang = jour.min(6);
    if abrege {
        noms_courts[rang].to_string()
    } else {
        noms[rang].to_string()
    }
}

/// Compose une date et une heure : les mêmes jetons de date, plus `HH`, `H`,
/// `mm`, `m`, `ss` et `s`. Les noms de mois restent ceux de la langue.
fn composer_format_heure(
    an: i32,
    mois: u32,
    jour: u32,
    heure: u32,
    minute: u32,
    format: &str,
    langue: &str,
) -> String {
    let semaine = jour_de_la_semaine(an, mois, jour);
    let jetons: [(&str, String); 16] = [
        ("yyyy", format!("{an:04}")),
        ("yy", format!("{:02}", an.rem_euclid(100))),
        ("MMMM", nom_du_mois(mois, langue, false)),
        ("MMM", nom_du_mois(mois, langue, true)),
        ("MM", format!("{mois:02}")),
        ("dddd", nom_du_jour(semaine, langue, false)),
        ("ddd", nom_du_jour(semaine, langue, true)),
        ("dd", format!("{jour:02}")),
        ("HH", format!("{heure:02}")),
        ("H", heure.to_string()),
        ("mm", format!("{minute:02}")),
        ("M", mois.to_string()),
        ("d", jour.to_string()),
        ("ss", "00".to_string()),
        ("s", "0".to_string()),
        ("m", minute.to_string()),
    ];
    let mut out = String::new();
    let mut reste = format;
    'dehors: while !reste.is_empty() {
        for (jeton, valeur) in &jetons {
            if reste.starts_with(jeton) {
                out.push_str(valeur);
                reste = &reste[jeton.len()..];
                continue 'dehors;
            }
        }
        let caractere = reste.chars().next().expect("non vide");
        out.push(caractere);
        reste = &reste[caractere.len_utf8()..];
    }
    out
}

fn composer_format(an: i32, mois: u32, jour: u32, format: &str, langue: &str) -> String {
    // Les jetons, **syntaxe anglophone internationale**, du plus long au
    // plus court : `yyyy` avant `yy`, `MM` avant `M`, `dd` avant `d` —
    // l'ordre est la règle, pas le hasard d'une table.
    let semaine = jour_de_la_semaine(an, mois, jour);
    let jetons: [(&str, String); 10] = [
        ("yyyy", format!("{an:04}")),
        ("yy", format!("{:02}", an.rem_euclid(100))),
        ("MMMM", nom_du_mois(mois, langue, false)),
        ("MMM", nom_du_mois(mois, langue, true)),
        ("MM", format!("{mois:02}")),
        ("dddd", nom_du_jour(semaine, langue, false)),
        ("ddd", nom_du_jour(semaine, langue, true)),
        ("dd", format!("{jour:02}")),
        ("M", mois.to_string()),
        ("d", jour.to_string()),
    ];
    let mut out = String::new();
    let mut reste = format;
    'dehors: while !reste.is_empty() {
        for (jeton, valeur) in &jetons {
            if reste.starts_with(jeton) {
                out.push_str(valeur);
                reste = &reste[jeton.len()..];
                continue 'dehors;
            }
        }
        let caractere = reste.chars().next().expect("non vide");
        out.push(caractere);
        reste = &reste[caractere.len_utf8()..];
    }
    out
}

/// `XIX` → 19. Minuscules acceptées.
pub fn romain_vers_nombre(romain: &str) -> Option<u32> {
    let mut total = 0u32;
    let mut precedent = 0u32;
    for caractere in romain.chars().rev() {
        let valeur = match caractere.to_ascii_uppercase() {
            'I' => 1,
            'V' => 5,
            'X' => 10,
            'L' => 50,
            'C' => 100,
            'D' => 500,
            'M' => 1000,
            _ => return None,
        };
        if valeur < precedent {
            total -= valeur;
        } else {
            total += valeur;
        }
        precedent = valeur;
    }
    (total > 0).then_some(total)
}

/// 19 → `XIX`.
pub fn nombre_vers_romain(mut nombre: u32) -> String {
    let mut out = String::new();
    for (valeur, marque) in [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ] {
        while nombre >= valeur {
            out.push_str(marque);
            nombre -= valeur;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Les deux formes, telles qu'elles ont été décrites.
    #[test]
    fn une_periode_se_declare_et_se_compose() {
        let regles = Regles::default();

        // XIXs : le siècle, court.
        let d = analyser("XIXs").expect("XIXs");
        assert_eq!(
            d,
            Date::Periode {
                nombre: 19,
                millenaire: false,
                longue: false,
                avant: false
            }
        );
        assert_eq!(rendre(&d, &regles), "\u{2}XIX\u{2}\u{3}e\u{3}");

        // -IVs? : le siècle, long, av. J.-C.
        let d = analyser("-IVs?").expect("-IVs?");
        assert_eq!(
            rendre(&d, &regles),
            "\u{2}IV\u{2}\u{3}e\u{3} siècle av. J.-C."
        );

        // IIIm? : le millénaire, long — la règle du dernier caractère.
        let d = analyser("IIIm?").expect("IIIm?");
        assert_eq!(
            rendre(&d, &regles),
            "\u{2}III\u{2}\u{3}e\u{3} millénaire ap. J.-C."
        );
    }

    /// Le premier dit `er`, pas `e`.
    #[test]
    fn le_premier_dit_er() {
        let regles = Regles::default();
        let d = analyser("Is").expect("Is");
        assert_eq!(rendre(&d, &regles), "\u{2}I\u{2}\u{3}er\u{3}");
        let d = analyser("Im?").expect("Im?");
        assert_eq!(
            rendre(&d, &regles),
            "\u{2}I\u{2}\u{3}er\u{3} millénaire ap. J.-C."
        );
    }

    /// Les romains s'écrivent aussi en minuscules.
    #[test]
    fn les_romains_en_minuscules_passent() {
        let regles = Regles::default();
        let d = analyser("xixs").expect("xixs");
        assert_eq!(rendre(&d, &regles), "\u{2}XIX\u{2}\u{3}e\u{3}");
        let d = analyser("iiim?").expect("iiim?");
        assert_eq!(
            rendre(&d, &regles),
            "\u{2}III\u{2}\u{3}e\u{3} millénaire ap. J.-C."
        );
    }

    /// La date pleine : la valeur, puis le format voulu, en guillemets.
    #[test]
    fn une_date_pleine_se_compose_au_format() {
        let regles = Regles::default();

        let d = analyser("2026.11.03 \"dd/MM/yyyy\"").expect("date");
        assert_eq!(
            d,
            Date::Calendrier {
                an: 2026,
                mois: 11,
                jour: 3,
                format: Some("dd/MM/yyyy".to_string())
            }
        );
        assert_eq!(rendre(&d, &regles), "03/11/2026");

        // Sans format : celui des normes.
        let d = analyser("1789.07.14").expect("date");
        assert_eq!(rendre(&d, &regles), "14/07/1789");

        // Le format se compose comme on le souhaite, tokens anglophones.
        let d = analyser("1789.07.14 \"le d M yyyy\"").expect("date");
        assert_eq!(rendre(&d, &regles), "le 14 7 1789");
        let d = analyser("2026.11.03 \"MM/yyyy\"").expect("date");
        assert_eq!(rendre(&d, &regles), "11/2026");
    }

    /// Les mots viennent des normes.
    #[test]
    fn les_mots_viennent_des_normes() {
        let regles = Regles {
            siecle: "s.".to_string(),
            epoque_apres: "EC".to_string(),
            epoque_avant: "AEC".to_string(),
            ..Default::default()
        };
        let d = analyser("XIXs?").expect("XIXs?");
        assert_eq!(rendre(&d, &regles), "\u{2}XIX\u{2}\u{3}e\u{3} s. EC");
        let d = analyser("-IVs?").expect("-IVs?");
        assert_eq!(rendre(&d, &regles), "\u{2}IV\u{2}\u{3}e\u{3} s. AEC");
    }

    /// Les noms de mois et de jours se localisent — sans que le format
    /// choisi en soit changé (bug-report-8, T3.2).
    #[test]
    fn les_noms_se_localisent_sans_toucher_au_format() {
        let mut regles = Regles::default();

        // Le 14 juillet 1789 était un mardi.
        let d = analyser("1789.07.14 \"dddd d MMMM yyyy\"").expect("date");
        assert_eq!(
            rendre(&d, &regles),
            "mardi 14 juillet 1789",
            "le français, langue par défaut"
        );

        regles.langue = "en".to_string();
        assert_eq!(rendre(&d, &regles), "Tuesday 14 July 1789");
        regles.langue = "de".to_string();
        assert_eq!(rendre(&d, &regles), "Dienstag 14 Juli 1789");
        regles.langue = "it".to_string();
        assert_eq!(rendre(&d, &regles), "martedì 14 luglio 1789");

        // Le format numérique ne bouge pas avec la langue.
        let d = analyser("2026.11.03 \"dd/MM/yyyy\"").expect("date");
        regles.langue = "en".to_string();
        assert_eq!(rendre(&d, &regles), "03/11/2026");

        // Les formes abrégées suivent.
        let d = analyser("2026.11.03 \"ddd MMM\"").expect("date");
        assert_eq!(rendre(&d, &regles), "Tue Nov");
    }

    /// Ce qui n'est pas une date ne se prend pas pour une date.
    #[test]
    fn ce_qui_n_est_pas_une_date_ne_l_est_pas() {
        assert!(analyser("").is_none());
        assert!(analyser("XIXx").is_none());
        assert!(analyser("2026.13.01").is_none());
        assert!(analyser("sans chiffres").is_none());
    }

    /// Les formes dynamiques se lisent, et l'heure porte bien une heure.
    #[test]
    fn les_formes_dynamiques_se_composent() {
        let regles = Regles::default();

        let a = analyser("aujourd'hui").expect("aujourd'hui");
        assert!(matches!(a, Date::Aujourdhui { .. }));
        let texte = rendre(&a, &regles);
        assert_eq!(texte.len(), 10, "la date par défaut : {texte}");

        let m = analyser("maintenant").expect("maintenant");
        assert!(matches!(m, Date::Maintenant { .. }));
        let texte = rendre(&m, &regles);
        assert!(texte.contains(':'), "l'heure est présente : {texte}");
        assert!(texte.len() >= 16, "date et heure : {texte}");

        let m = analyser("maintenant \"HH:mm\"").expect("format d'heure");
        let texte = rendre(&m, &regles);
        assert_eq!(texte.len(), 5, "seule l'heure : {texte}");
        assert!(texte.contains(':'), "{texte}");

        assert!(analyser("aujourd'hui \"dd/MM/yyyy\"").is_some());
        assert!(analyser("maintenant \"format").is_none());
    }

    /// Les romains vont dans les deux sens.
    #[test]
    fn les_romains_vont_et_viennent() {
        assert_eq!(romain_vers_nombre("XIX"), Some(19));
        assert_eq!(romain_vers_nombre("xix"), Some(19));
        assert_eq!(romain_vers_nombre("MCMXCIX"), Some(1999));
        assert_eq!(nombre_vers_romain(19), "XIX");
        assert_eq!(nombre_vers_romain(1999), "MCMXCIX");
    }
}
