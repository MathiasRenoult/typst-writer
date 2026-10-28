//! Le client de l'API locale de Zotero.
//!
//! Zotero 7 expose, sur `127.0.0.1:23119`, une implémentation locale de son
//! API web : on y lit les collections, et on y exporte les notices d'une
//! collection en BibTeX. Tout est en lecture : Zotero reste le maître de sa
//! bibliothèque.
//!
//! Rien de tout cela n'est obligatoire. Zotero peut être éteint, son API
//! désactivée, sa réponse illisible : chaque cas a sa place dans
//! [`Erreur`], et jamais une panne ne devient un silence.

use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

/// L'adresse de l'API locale de Zotero.
const ADRESSE: &str = "http://127.0.0.1:23119/api";

/// Le temps au-delà duquel un Zotero qui ne répond pas est déclaré muet.
const PATIENCE: Duration = Duration::from_secs(5);

/// Ce que la liaison à Zotero peut rencontrer.
#[derive(Debug, Clone)]
pub enum Erreur {
    /// Zotero ne répond pas : éteint, ou son API désactivée.
    Injoignable(String),

    /// Zotero répond, mais son API locale est fermée : c'est un réglage,
    /// et il se reprend — même sans redémarrer.
    ApiFermee,

    /// Zotero a répondu, mais pas d'une façon qu'on puisse lire.
    ReponseInvalide(String),
}

impl std::fmt::Display for Erreur {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Erreur::Injoignable(_) => write!(
                f,
                "Zotero ne répond pas — est-il lancé ? \
                 (Réglages → Avancé → « Permettre aux autres applications de communiquer »)"
            ),
            Erreur::ApiFermee => write!(
                f,
                "L'API locale de Zotero est fermée : Réglages → Avancé → \
                 « Permettre aux autres applications sur cet ordinateur de communiquer avec Zotero ». \
                 La case prise, tout fonctionne — sans même redémarrer."
            ),
            Erreur::ReponseInvalide(motif) => {
                write!(f, "La réponse de Zotero est illisible : {motif}")
            }
        }
    }
}

/// Une collection Zotero, telle qu'il faut la montrer pour la choisir.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Collection {
    /// Sa clé — l'identifiant stable, celui qui lie le projet.
    pub cle: String,

    /// Son nom.
    pub nom: String,

    /// La clé de la collection qui la contient, s'il y en a une.
    pub parent: Option<String>,

    /// Le nombre de notices qu'elle contient, directement.
    pub notices: usize,
}

/// Toutes les collections de la bibliothèque, dans l'ordre de Zotero.
pub fn collections() -> Result<Vec<Collection>, Erreur> {
    collections_depuis(ADRESSE)
}

/// Toutes les collections, depuis une adresse — celle des tests.
pub fn collections_depuis(base: &str) -> Result<Vec<Collection>, Erreur> {
    let mut collections = Vec::new();
    let mut depart = 0usize;

    // Cent par cent, tant qu'il en arrive : l'API a une limite de page,
    // et une bibliothèque de collectionneur la dépasse sans peine.
    loop {
        let chemin = format!("/users/0/collections?start={depart}&limit=100");
        let corps = requete(base, &chemin)?;
        let page: Vec<Collection> = lecture_collections(&corps)?;

        // Une page plus courte que la limite est la dernière : le compte
        // est celui des **items**, jamais celui des octets du corps.
        depart += page.len();
        let fini = page.len() < 100;
        collections.extend(page);

        if fini {
            break;
        }
    }

    Ok(collections)
}

/// Les notices d'une collection, en BibTeX : ce que le projet éprouvera.
pub fn bibtex_depuis(base: &str, cle: &str) -> Result<String, Erreur> {
    let mut sortie = String::new();
    let mut depart = 0usize;

    loop {
        let chemin = format!(
            "/users/0/collections/{cle}/items?format=bibtex&start={depart}&limit=100"
        );
        let page = requete(base, &chemin)?;

        // Une entrée BibTeX s'ouvre par `@` en tête de ligne : les compter
        // dit si la page est entière — jamais la longueur du corps, qui
        // dépend de la longueur des titres.
        let entrees = page.lines().filter(|ligne| ligne.starts_with('@')).count();
        let fini = page.trim().is_empty() || entrees < 100;
        sortie.push_str(&page);
        if !sortie.ends_with('\n') {
            sortie.push('\n');
        }

        if fini {
            break;
        }

        depart += 100;
    }

    Ok(sortie)
}

/// Le BibTeX d'une collection, depuis l'adresse par défaut.
pub fn bibtex(cle: &str) -> Result<String, Erreur> {
    bibtex_depuis(ADRESSE, cle)
}

/// Les collections lues dans une réponse de Zotero.
fn lecture_collections(corps: &str) -> Result<Vec<Collection>, Erreur> {
    let valeur: serde_json::Value = serde_json::from_str(corps)
        .map_err(|erreur| Erreur::ReponseInvalide(format!("JSON : {erreur}")))?;

    let entrees = valeur.as_array().ok_or_else(|| {
        Erreur::ReponseInvalide("une liste de collections était attendue".to_string())
    })?;

    let mut collections = Vec::with_capacity(entrees.len());

    for entree in entrees {
        let donnees = &entree["data"];
        let Some(cle) = donnees["key"].as_str() else {
            continue;
        };
        let Some(nom) = donnees["name"].as_str() else {
            continue;
        };

        // `parentCollection` est `false` en haut, une clé sinon.
        let parent = donnees["parentCollection"]
            .as_str()
            .map(|cle| cle.to_string());

        let notices = entree["meta"]["numItems"].as_u64().unwrap_or(0) as usize;

        collections.push(Collection {
            cle: cle.to_string(),
            nom: nom.to_string(),
            parent,
            notices,
        });
    }

    Ok(collections)
}

/// Un aller simple vers l'API locale de Zotero, et son corps de réponse.
fn requete(base: &str, chemin: &str) -> Result<String, Erreur> {
    let (hote, port) = hote_et_port(base)?;

    let adresse = (hote.as_str(), port)
        .to_socket_addrs()
        .map_err(|erreur| Erreur::Injoignable(erreur.to_string()))?
        .next()
        .ok_or_else(|| {
            Erreur::Injoignable("aucune adresse pour Zotero".to_string())
        })?;

    let mut flux = TcpStream::connect_timeout(&adresse, PATIENCE)
        .map_err(|erreur| Erreur::Injoignable(erreur.to_string()))?;
    flux.set_read_timeout(Some(PATIENCE))
        .map_err(|erreur| Erreur::Injoignable(erreur.to_string()))?;
    flux.set_write_timeout(Some(PATIENCE))
        .map_err(|erreur| Erreur::Injoignable(erreur.to_string()))?;

    let demande = format!(
        "GET {chemin} HTTP/1.1\r\n\
         Host: {hote}:{port}\r\n\
         Accept: application/json, text/plain, */*\r\n\
         Connection: close\r\n\r\n"
    );
    flux.write_all(demande.as_bytes())
        .map_err(|erreur| Erreur::Injoignable(erreur.to_string()))?;

    let mut reponse = String::new();
    flux.read_to_string(&mut reponse)
        .map_err(|erreur| Erreur::Injoignable(erreur.to_string()))?;

    corps_de(&reponse)
}

/// L'hôte et le port d'une adresse `http://hôte:port/…`.
fn hote_et_port(base: &str) -> Result<(String, u16), Erreur> {
    let sans_schema = base
        .strip_prefix("http://")
        .or_else(|| base.strip_prefix("https://"))
        .unwrap_or(base);

    let autorite = sans_schema.split('/').next().unwrap_or(sans_schema);

    match autorite.rsplit_once(':') {
        Some((hote, port)) => {
            let port = port.parse().map_err(|_| {
                Erreur::ReponseInvalide(format!("port illisible : {port}"))
            })?;

            Ok((hote.to_string(), port))
        }
        None => Ok((autorite.to_string(), 80)),
    }
}

/// Le corps d'une réponse HTTP : statut, en-têtes, et ce qu'ils portent.
fn corps_de(reponse: &str) -> Result<String, Erreur> {
    let Some((entetes, corps)) = reponse.split_once("\r\n\r\n") else {
        return Err(Erreur::ReponseInvalide("réponse tronquée".to_string()));
    };

    let statut = entetes
        .lines()
        .next()
        .and_then(|ligne| ligne.split_whitespace().nth(1))
        .and_then(|code| code.parse::<u16>().ok())
        .ok_or_else(|| Erreur::ReponseInvalide("statut illisible".to_string()))?;

    if !(200..300).contains(&statut) {
        // Le cas du monde réel : Zotero tourne, mais son API locale est
        // un réglage, et il est éteint. Le corps le dit mot pour mot.
        if corps.contains("Local API is not enabled") || statut == 403 {
            return Err(Erreur::ApiFermee);
        }

        return Err(Erreur::ReponseInvalide(format!("statut {statut}")));
    }

    // `Connection: close` fait que le corps arrive entier ; si Zotero
    // morcelle, on remet les morceaux bout à bout.
    if entetes.to_ascii_lowercase().contains("transfer-encoding: chunked") {
        return Ok(morceaux(corps));
    }

    Ok(corps.to_string())
}

/// Un corps en morceaux, remis dans l'ordre.
fn morceaux(corps: &str) -> String {
    let mut sortie = String::new();
    let mut reste = corps;

    while let Some((taille, suite)) = reste.split_once("\r\n") {
        let taille = usize::from_str_radix(taille.trim(), 16).unwrap_or(0);
        if taille == 0 {
            break;
        }

        let morceau: String = suite.chars().take(taille).collect();
        sortie.push_str(&morceau);

        // Le morceau fini, son séparateur aussi. La taille est en
        // **caractères** : le reste se découpe au même cran, jamais au
        // nombre d'octets — un accent y coupait en deux.
        let suite = {
            let octets = suite.len();
            let pris: String = suite.chars().take(taille).collect();
            &suite[pris.len()..octets]
        };
        reste = suite.strip_prefix("\r\n").unwrap_or(suite);
    }

    sortie
}

#[cfg(test)]
mod tests {

    /// Le cas du monde réel : Zotero tourne, son API locale est fermée.
    /// Le remède se dit, pas un statut nu.
    #[test]
    fn une_api_fermee_dit_comment_l_ouvrir() {
        let base = serveur2(
            "HTTP/1.1 403 Forbidden\r\nConnection: close\r\n\r\n",
            "Local API is not enabled",
        );

        let erreur = collections_depuis(&base).expect_err("fermée");

        assert!(matches!(erreur, Erreur::ApiFermee), "vu : {erreur:?}");
        assert!(
            erreur.to_string().contains("Permettre aux autres applications"),
            "le remède se dit : {erreur}"
        );
    }

    use super::*;
    use std::io::Write;
    use std::net::TcpListener;

    /// Un serveur qui répond une fois, puis s'arrête : l'API de Zotero,
    /// telle qu'un test peut la fabriquer.
    fn serveur(reponse: &'static str) -> String {
        let auditeur = TcpListener::bind("127.0.0.1:0").expect("écoute");
        let adresse = auditeur.local_addr().expect("adresse");

        std::thread::spawn(move || {
            if let Ok((mut flux, _)) = auditeur.accept() {
                let mut demande = [0u8; 4096];
                let _ = std::io::Read::read(&mut flux, &mut demande);
                let _ = flux.write_all(reponse.as_bytes());
            }
        });

        format!("http://{}", adresse)
    }

    const REPONSE: &str = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n";

    /// Les collections se lisent, avec leur nom, leur parent et leurs notices.
    #[test]
    fn les_collections_se_lisent() {
        let corps = r#"[
            {"key": "AAAA1111", "meta": {"numItems": 12},
             "data": {"key": "AAAA1111", "name": "Thèse", "parentCollection": false}},
            {"key": "BBBB2222", "meta": {"numItems": 3},
             "data": {"key": "BBBB2222", "name": "Chapitre 2", "parentCollection": "AAAA1111"}}
        ]"#;
        let base = serveur2(REPONSE, corps);

        let collections = collections_depuis(&base).expect("collections");

        assert_eq!(collections.len(), 2);
        assert_eq!(collections[0].nom, "Thèse");
        assert_eq!(collections[0].parent, None);
        assert_eq!(collections[0].notices, 12);
        assert_eq!(collections[1].parent.as_deref(), Some("AAAA1111"));
    }

    /// Une bibliothèque plus large que la page : on lit la suite.
    #[test]
    fn les_collections_se_paginent() {
        // Cent-une collections : la première page seule ne suffit pas.
        let mut premiere = String::from("[");
        for numero in 0..100 {
            if numero > 0 {
                premiere.push(',');
            }
            premiere.push_str(&format!(
                r#"{{"key": "K{numero:04}", "meta": {{"numItems": 0}}, "data": {{"key": "K{numero:04}", "name": "n{numero}", "parentCollection": false}}}}"#
            ));
        }
        premiere.push(']');

        let suite = r#"[{"key": "KDERN", "meta": {"numItems": 7}, "data": {"key": "KDERN", "name": "Dernière", "parentCollection": false}}]"#;

        let auditeur = TcpListener::bind("127.0.0.1:0").expect("écoute");
        let adresse = auditeur.local_addr().expect("adresse");

        std::thread::spawn(move || {
            for corps in [premiere, suite.to_string()] {
                let Ok((mut flux, _)) = auditeur.accept() else {
                    return;
                };
                let mut demande = [0u8; 4096];
                let _ = std::io::Read::read(&mut flux, &mut demande);
                let entete = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    corps.len()
                );
                let _ = flux.write_all(entete.as_bytes());
                let _ = flux.write_all(corps.as_bytes());
            }
        });

        let collections = collections_depuis(&format!("http://{}", adresse)).expect("collections");
        assert_eq!(collections.len(), 101);
        assert_eq!(collections[100].nom, "Dernière");
    }

    /// Zotero éteint n'est pas une erreur de code : c'est un état, et le
    /// message dit ce qu'il en est.
    #[test]
    fn un_zotero_eteint_se_reconnait() {
        // Un port qu'on tient fermé : la connexion est refusée.
        let auditeur = TcpListener::bind("127.0.0.1:0").expect("écoute");
        let adresse = auditeur.local_addr().expect("adresse");
        drop(auditeur);

        let erreur = collections_depuis(&format!("http://{}", adresse)).expect_err("éteint");

        assert!(matches!(erreur, Erreur::Injoignable(_)), "vu : {erreur:?}");
        assert!(erreur.to_string().contains("Zotero ne répond pas"));
    }

    /// Une réponse qui n'est pas du JSON se dit, sans emporter le reste.
    #[test]
    fn une_reponse_invalide_se_dit() {
        let base = serveur2("HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n", "pas du json");
        let erreur = collections_depuis(&base).expect_err("illisible");
        assert!(matches!(erreur, Erreur::ReponseInvalide(_)));

        let base = serveur2("HTTP/1.1 500 Panne\r\nConnection: close\r\n\r\n", "");
        let erreur = collections_depuis(&base).expect_err("statut 500");
        assert!(erreur.to_string().contains("statut 500"));
    }

    /// Un corps morcelé se remet entier.
    #[test]
    fn un_corps_morcelle_se_remet() {
        let reponse = "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n\
                       4\r\n{\"a\"\r\n4\r\n: 1}\r\n0\r\n\r\n";
        let base = serveur2(reponse, "");
        let corps = requete(&base, "/users/0/collections").expect("corps");
        assert_eq!(corps, "{\"a\": 1}");
    }

    /// Un serveur qui répond à la fois l'en-tête et le corps donnés.
    fn serveur2(entete: &str, corps: &str) -> String {
        let auditeur = TcpListener::bind("127.0.0.1:0").expect("écoute");
        let adresse = auditeur.local_addr().expect("adresse");
        let reponse = format!("{entete}{corps}");

        std::thread::spawn(move || {
            if let Ok((mut flux, _)) = auditeur.accept() {
                let mut demande = [0u8; 4096];
                let _ = std::io::Read::read(&mut flux, &mut demande);
                let _ = flux.write_all(reponse.as_bytes());
            }
        });

        format!("http://{}", adresse)
    }

    /// L'export BibTeX d'une collection en vient, page après page.
    #[test]
    fn l_export_bibtex_se_lit() {
        let base = serveur2(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\n",
            "@book{a, title = {A}}\n",
        );

        let bibtex = bibtex_depuis(&base, "AAAA1111").expect("bibtex");
        assert_eq!(bibtex, "@book{a, title = {A}}\n");
    }
}
