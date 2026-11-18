#!/usr/bin/env python3
"""Convertit un fichier biblique courant vers le JSON de Typst Writer.

Formats lus :

- un JSON complet, au format de `~/.typst-writer/bibles/` ;
- un JSONL de versets :
  {"osis": "Gen", "nom": "Genèse", "abreviation": "Gn",
   "chapitre": 1, "verset": 1, "texte": "Au commencement…"}
- un JSONL qui contient un objet complet (une seule ligne avec `livres`) ;
- un USFM simple : `\\id`, `\\h`, `\\c`, `\\v`, avec les marqueurs de
  paragraphe ignorés.

Le script ne télécharge rien. Il valide les identifiants et les livres, puis
écrit un fichier que l'application peut installer. Les numéros de versets
manquants sont conservés par une chaîne vide ; l'application garde donc la
numérotation de l'édition.

Exemple :

    python3 scripts/convertir-version-biblique.py \
        --entree ma-bible.usfm --id vulgate --nom "Vulgate" \
        --sortie vulgate.json

Puis, dans Typst Writer : Statistiques bibliques → + → choisir
`vulgate.json`.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any, NoReturn

RESERVES = {"lsg", "darby", "segond", "louis_segond_1910", "jnd"}
# Mêmes règles que `VersionBiblique::valider` (src/langage/versions.rs) :
# minuscules, chiffres, tirets, points et soulignés ; premier caractère
# alphanumérique, sans limite de longueur.
IDENTIFIANT = re.compile(r"^[a-z0-9][a-z0-9._-]*$")
OSIS = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]*$")

# Les codes USFM à trois lettres ne sont pas les codes OSIS attendus par
# l'application (`GEN` doit devenir `Gen`, `1SA` doit devenir `1Sam`).
USFM_OSIS = {
    "GEN": "Gen", "EXO": "Exod", "LEV": "Lev", "NUM": "Num", "DEU": "Deut",
    "JOS": "Josh", "JDG": "Judg", "RUT": "Ruth", "1SA": "1Sam", "2SA": "2Sam",
    "1KI": "1Kgs", "2KI": "2Kgs", "1CH": "1Chr", "2CH": "2Chr", "EZR": "Ezra",
    "NEH": "Neh", "EST": "Esth", "JOB": "Job", "PSA": "Ps", "PRO": "Prov",
    "ECC": "Eccl", "SNG": "Song", "ISA": "Isa", "JER": "Jer", "LAM": "Lam",
    "EZK": "Ezek", "DAN": "Dan", "HOS": "Hos", "JOL": "Joel", "AMO": "Amos",
    "OBA": "Obad", "JON": "Jonah", "MIC": "Mic", "NAM": "Nah", "HAB": "Hab",
    "ZEP": "Zeph", "HAG": "Hag", "ZEC": "Zech", "MAL": "Mal", "MAT": "Matt",
    "MRK": "Mark", "LUK": "Luke", "JHN": "John", "ACT": "Acts", "ROM": "Rom",
    "1CO": "1Cor", "2CO": "2Cor", "GAL": "Gal", "EPH": "Eph", "PHP": "Phil",
    "COL": "Col", "1TH": "1Thess", "2TH": "2Thess", "1TI": "1Tim",
    "2TI": "2Tim", "TIT": "Titus", "PHM": "Phlm", "HEB": "Heb", "JAS": "Jas",
    "1PE": "1Pet", "2PE": "2Pet", "1JN": "1John", "2JN": "2John",
    "3JN": "3John", "JUD": "Jude", "REV": "Rev",
}


def erreur(message: str) -> NoReturn:
    print(f"Erreur : {message}", file=sys.stderr)
    raise SystemExit(1)


def valider(version: dict[str, Any]) -> dict[str, Any]:
    """Vérifie les invariants que l'application vérifiera aussi."""
    identifiant = str(version.get("id", "")).strip().lower()
    if not IDENTIFIANT.fullmatch(identifiant) or identifiant in RESERVES:
        erreur(f"identifiant invalide ou réservé : {identifiant!r}")
    version["id"] = identifiant

    nom = str(version.get("nom", "")).strip()
    if not nom:
        erreur("le champ 'nom' est vide")
    version["nom"] = nom

    version["langue"] = str(version.get("langue", "")).strip()

    version["source"] = str(version.get("source", "")).strip()
    livres = version.get("livres")
    if not isinstance(livres, list) or not livres:
        erreur("la version doit contenir au moins un livre")
    livres = list(livres)

    osis_vus: set[str] = set()
    for livre in livres:
        osis = str(livre.get("osis", "")).strip()
        if not OSIS.fullmatch(osis):
            erreur(f"code OSIS invalide : {osis!r}")
        if osis in osis_vus:
            erreur(f"le livre {osis!r} apparaît deux fois")
        osis_vus.add(osis)
        for champ in ("nom", "abreviation"):
            if not str(livre.get(champ, "")).strip():
                erreur(f"le livre {osis!r} n'a pas de {champ}")
        chapitres = livre.get("chapitres")
        if not isinstance(chapitres, list) or not chapitres:
            erreur(f"le livre {osis!r} n'a aucun chapitre")
        for numero, chapitre in enumerate(chapitres, start=1):
            if not isinstance(chapitre, list) or not chapitre:
                erreur(f"{osis} {numero} n'a aucun verset")
            if any(not isinstance(verset, str) for verset in chapitre):
                erreur(f"{osis} {numero} contient autre chose que du texte")
    return version


def depuis_json(objet: Any) -> dict[str, Any]:
    if not isinstance(objet, dict):
        erreur("le JSON doit être un objet")
    return objet


def depuis_jsonl(texte: str) -> dict[str, Any]:
    lignes = [json.loads(ligne) for ligne in texte.splitlines() if ligne.strip()]
    complets = [ligne for ligne in lignes if isinstance(ligne, dict) and "livres" in ligne]
    if complets:
        if len(complets) > 1:
            erreur("plusieurs objets complets dans le JSONL")
        return valider(complets[0])

    livres: dict[str, dict[str, Any]] = {}
    chapitres: dict[str, dict[int, dict[int, str]]] = {}
    for ligne in lignes:
        if not isinstance(ligne, dict):
            erreur("chaque ligne JSONL doit être un objet")
        osis = str(ligne.get("osis", "")).strip()
        if not osis:
            erreur("une ligne de verset n'a pas de code OSIS")
        chapitre = int(ligne.get("chapitre", 0))
        verset = int(ligne.get("verset", 0))
        if chapitre < 1 or verset < 1:
            erreur(f"{osis} : chapitre et verset doivent être positifs")
        meta = livres.setdefault(
            osis,
            {
                "osis": osis,
                "nom": str(ligne.get("nom", osis)).strip(),
                "abreviation": str(ligne.get("abreviation", osis[:3])).strip(),
            },
        )
        if valeur := str(ligne.get("nom", "")).strip():
            meta["nom"] = valeur
        if valeur := str(ligne.get("abreviation", "")).strip():
            meta["abreviation"] = valeur
        chapitres.setdefault(osis, {}).setdefault(chapitre, {})[verset] = str(
            ligne.get("texte", "")
        )

    sortie_livres = []
    for osis, meta in livres.items():
        par_chapitre = chapitres[osis]
        nombre_chapitres = max(par_chapitre)
        chapitres_listes = []
        for numero in range(1, nombre_chapitres + 1):
            versets = par_chapitre.get(numero, {})
            nombre_versets = max(versets) if versets else 1
            chapitres_listes.append(
                [versets.get(verset, "") for verset in range(1, nombre_versets + 1)]
            )
        sortie_livres.append({**meta, "chapitres": chapitres_listes})

    return {
        "id": "",
        "nom": "",
        "langue": "",
        "source": "",
        "livres": sortie_livres,
    }


def depuis_usfm(texte: str) -> dict[str, Any]:
    """Lit le sous-ensemble USFM du texte biblique courant.

    Les notes `\\f`, les titres de section et les marqueurs de paragraphe sont
    ignorés. Une ligne de continuation après `\\v` prolonge le verset.
    """
    livres: list[dict[str, Any]] = []
    livre: dict[str, Any] | None = None
    chapitre_actuel: list[str] | None = None
    verset_actuel: int | None = None

    def terminer_le_verset() -> None:
        nonlocal verset_actuel
        if verset_actuel is None or chapitre_actuel is None or livre is None:
            return
        while len(chapitre_actuel) < verset_actuel:
            chapitre_actuel.append("")
        verset_actuel = None

    for brute in texte.splitlines():
        ligne = brute.strip()
        if not ligne:
            continue
        marqueur = re.match(r"\\([a-z0-9]+)\s*(.*)", ligne)
        if not marqueur:
            if verset_actuel is not None and chapitre_actuel is not None:
                chapitre_actuel[verset_actuel - 1] += (" " + ligne).rstrip()
            continue
        code, contenu = marqueur.group(1), marqueur.group(2).strip()
        if code == "id":
            terminer_le_verset()
            osis_brut = contenu.split()[0] if contenu else "Inconnu"
            osis = USFM_OSIS.get(osis_brut.upper(), osis_brut)
            livre = {
                "osis": osis,
                "nom": osis,
                "abreviation": osis[:3],
                "chapitres": [],
            }
            livres.append(livre)
            chapitre_actuel = None
        elif code == "h" and livre is not None:
            livre["nom"] = contenu or livre["nom"]
            livre["abreviation"] = contenu[:3] or livre["abreviation"]
        elif code == "c" and livre is not None:
            terminer_le_verset()
            numero = 0
            try:
                numero = int(contenu.split()[0])
            except (ValueError, IndexError):
                erreur(f"chapitre USFM illisible : {ligne!r}")
            while len(livre["chapitres"]) < numero:
                livre["chapitres"].append([])
            chapitre = livre["chapitres"][numero - 1]
            chapitre_actuel = chapitre
        elif code == "v" and chapitre_actuel is not None:
            terminer_le_verset()
            chapitre = chapitre_actuel
            morceau = contenu.split(maxsplit=1)
            verset_actuel = 0
            try:
                verset_actuel = int(morceau[0])
            except (ValueError, IndexError):
                erreur(f"verset USFM illisible : {ligne!r}")
            while len(chapitre) < verset_actuel:
                chapitre.append("")
            chapitre[verset_actuel - 1] = morceau[1] if len(morceau) > 1 else ""
        # Les autres marqueurs (\p, \q, \s, \f…) ne portent pas le texte
        # canonique du verset : ils sont ignorés volontairement.
    terminer_le_verset()

    if not livres:
        erreur("aucun livre USFM trouvé (\\id manquant)")
    return {
        "id": "",
        "nom": "",
        "langue": "",
        "source": "",
        "livres": livres,
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--entree", required=True, type=Path)
    parser.add_argument("--sortie", type=Path)
    parser.add_argument("--format", choices=["auto", "json", "jsonl", "usfm"], default="auto")
    parser.add_argument("--id", default="")
    parser.add_argument("--nom", default="")
    parser.add_argument("--langue", default="fr")
    parser.add_argument("--source", default="")
    args = parser.parse_args()

    if not args.entree.is_file():
        erreur(f"fichier introuvable : {args.entree}")
    texte = args.entree.read_text(encoding="utf-8")
    extension = args.entree.suffix.lower()
    format_lu = args.format
    if format_lu == "auto":
        format_lu = "usfm" if extension in {".usfm", ".sfm"} else "jsonl" if extension == ".jsonl" else "json"

    if format_lu == "json":
        version = depuis_json(json.loads(texte))
    elif format_lu == "jsonl":
        version = depuis_jsonl(texte)
    else:
        version = depuis_usfm(texte)

    if not version.get("id"):
        version["id"] = args.id
    if not version.get("nom"):
        version["nom"] = args.nom
    if not version.get("langue"):
        version["langue"] = args.langue
    if not version.get("source"):
        version["source"] = args.source
    version = valider(version)

    jeton = json.dumps(version, ensure_ascii=False, indent=2) + "\n"
    if args.sortie:
        args.sortie.write_text(jeton, encoding="utf-8")
        print(
            f"{args.sortie} : {version['id']} — {len(version['livres'])} livre(s), "
            f"{sum(len(livre['chapitres']) for livre in version['livres'])} chapitre(s)."
        )
    else:
        print(jeton, end="")


if __name__ == "__main__":
    main()
