#!/usr/bin/env python3
"""Construit le site de documentation depuis `docs/guide/`.

    python3 site/construire.py

Le site se lit dans `site/dist/` — ouvrez `index.html`. Aucune dépendance :
le markdown du manuel est simple, et ce script le dit en entier. Ce qui
n'est pas reconnu reste du texte, comme au langage.

"""

import html
import pathlib
import re
import sys

RACINE = pathlib.Path(__file__).resolve().parent.parent
GUIDE = RACINE / "docs" / "guide"
SORTIE = RACINE / "site" / "dist"

# Le sommaire, dans l'ordre de lecture.
PAGES = [
    ("index", "Le manuel"),
    ("prise-en-main", "Prise en main"),
    ("ecrire", "Écrire"),
    ("bibliographie", "La bibliographie"),
    ("glossaire", "Le glossaire"),
    ("normes", "Les normes"),
    ("recherche", "Rechercher et remplacer"),
    ("raccourcis", "Les raccourcis"),
    ("fichiers", "Les fichiers"),
    ("installation-linux", "Installer sous Linux"),
]


def inline(texte: str) -> str:
    """Le texte enrichi : code, gras, italique, liens — dans cet ordre."""
    # Le code d'abord : une bande opaque, rien ne s'y joue.
    morceaux = re.split(r"(`[^`]+`)", texte)
    sortante = []
    for rang, morceau in enumerate(morceaux):
        if rang % 2 == 1:
            sortante.append(f"<code>{html.escape(morceau[1:-1])}</code>")
            continue

        e = html.escape(morceau)
        e = re.sub(r"\[([^\]]+)\]\(([^)]+)\)", r'<a href="\2">\1</a>', e)
        e = re.sub(r"\*\*([^*]+)\*\*", r"<strong>\1</strong>", e)
        e = re.sub(r"(?<!\*)\*([^*]+)\*(?!\*)", r"<em>\1</em>", e)
        # Les échappements du langage : `\x` est le caractère nu.
        e = re.sub(r"\\(.)", r"\1", e)
        sortante.append(e)

    return "".join(sortante)


def blocs(texte: str) -> str:
    """Le document en blocs : titres, listes, tableaux, code, texte."""
    lignes = texte.splitlines()
    sortie = []
    i = 0

    def ouvert(tag):
        sortie.append(f"<{tag}>")

    while i < len(lignes):
        ligne = lignes[i]

        if not ligne.strip():
            i += 1
            continue

        # Titres.
        m = re.match(r"^(#{1,3}) (.+)$", ligne)
        if m:
            niveau = len(m.group(1))
            sortie.append(f"<h{niveau}>{inline(m.group(2))}</h{niveau}>")
            i += 1
            continue

        # Code indenté (quatre espaces) ou clôturé.
        if ligne.startswith("    ") or ligne.startswith("```"):
            if ligne.startswith("```"):
                i += 1
                contenu = []
                while i < len(lignes) and not lignes[i].startswith("```"):
                    contenu.append(lignes[i])
                    i += 1
                i += 1
            else:
                contenu = []
                while i < len(lignes) and (lignes[i].startswith("    ") or not lignes[i].strip()):
                    contenu.append(lignes[i][4:] if lignes[i].startswith("    ") else "")
                    i += 1
            sortie.append("<pre><code>%s</code></pre>" % html.escape("\n".join(contenu)))
            continue

        # Tableau.
        if ligne.startswith("|") and i + 1 < len(lignes) and re.match(r"^\|[\s:|-]+\|$", lignes[i + 1]):
            entete = [c.strip() for c in ligne.strip().strip("|").split("|")]
            i += 2
            sortie.append("<table><thead><tr>")
            for cellule in entete:
                sortie.append(f"<th>{inline(cellule)}</th>")
            sortie.append("</tr></thead><tbody>")
            while i < len(lignes) and lignes[i].startswith("|"):
                cellules = [c.strip() for c in lignes[i].strip().strip("|").split("|")]
                sortie.append("<tr>" + "".join(f"<td>{inline(c)}</td>" for c in cellules) + "</tr>")
                i += 1
            sortie.append("</tbody></table>")
            continue

        # Citation.
        if ligne.startswith("> "):
            contenu = []
            while i < len(lignes) and lignes[i].startswith("> "):
                contenu.append(lignes[i][2:])
                i += 1
            sortie.append("<blockquote><p>%s</p></blockquote>" % inline(" ".join(contenu)))
            continue

        # Liste.
        if re.match(r"^[-*] |^\d+\. ", ligne):
            ordonnee = ligne[0].isdigit()
            tag = "ol" if ordonnee else "ul"
            ouvert(tag)
            while i < len(lignes) and re.match(r"^[-*] |^\d+\. ", lignes[i]):
                sortie.append(f"<li>{inline(re.sub(r'^([-*] |\d+\. )', '', lignes[i]))}</li>")
                i += 1
            sortie.append(f"</{tag}>")
            continue

        # Paragraphe.
        contenu = []
        while i < len(lignes) and lignes[i].strip() and not re.match(
            r"^(#{1,3} |\||> |[-*] |\d+\. |```|    )", lignes[i]
        ):
            contenu.append(lignes[i])
            i += 1
        sortie.append("<p>%s</p>" % inline(" ".join(contenu)))

    return "\n".join(sortie)


GABARIT = """<!doctype html>
<html lang="fr">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{titre} — Typst Writer</title>
<style>
  :root {{ --texte: #2c2b28; --adouci: #7a766c; --lisiere: #e3ded2; --fond: #faf8f3; --bois: #f2eee5; --accent: #5f7d55; }}
  * {{ box-sizing: border-box; }}
  body {{ margin: 0; font: 16px/1.65 Georgia, "Times New Roman", serif; color: var(--texte); background: var(--fond); }}
  .page {{ display: flex; min-height: 100vh; }}
  nav {{ width: 250px; padding: 2rem 1.5rem; background: var(--bois); border-right: 1px solid var(--lisiere); }}
  nav h1 {{ font-size: 1.05rem; margin: 0 0 1.25rem; }}
  nav a {{ display: block; padding: .35rem 0; color: var(--texte); text-decoration: none; border-bottom: 1px solid var(--lisiere); }}
  nav a:hover {{ color: var(--accent); }}
  nav a.courant {{ color: var(--accent); font-weight: bold; }}
  main {{ flex: 1; padding: 3rem clamp(1.5rem, 6vw, 5rem); max-width: 46rem; }}
  h1, h2, h3 {{ line-height: 1.25; }}
  h2 {{ margin-top: 2.25rem; border-bottom: 1px solid var(--lisiere); padding-bottom: .35rem; }}
  a {{ color: var(--accent); }}
  code {{ background: var(--bois); padding: .1em .35em; border-radius: 4px; font: .9em ui-monospace, monospace; }}
  pre {{ background: var(--bois); padding: 1rem; border-radius: 8px; overflow-x: auto; border: 1px solid var(--lisiere); }}
  pre code {{ background: none; padding: 0; }}
  table {{ border-collapse: collapse; width: 100%; margin: 1rem 0; }}
  th, td {{ border: 1px solid var(--lisiere); padding: .45rem .65rem; text-align: left; }}
  th {{ background: var(--bois); }}
  blockquote {{ margin: 1rem 0; padding: .25rem 1rem; border-left: 3px solid var(--accent); background: var(--bois); }}
</style>
</head>
<body>
<div class="page">
<nav>
<h1>Typst Writer</h1>
{navigation}
</nav>
<main>
{corps}
</main>
</div>
</body>
</html>
"""


def construire() -> None:
    SORTIE.mkdir(parents=True, exist_ok=True)

    navigation = []
    for fichier, titre in PAGES:
        navigation.append(f'<a href="{fichier}.html">{titre}</a>')

    for fichier, titre in PAGES:
        source = GUIDE / f"{fichier}.md"

        if not source.exists():
            print(f"manquant : {source}", file=sys.stderr)
            continue

        corps = blocs(source.read_text())
        nav = "\n".join(
            (
                f'<a href="{f}.html" class="courant">{t}</a>'
                if f == fichier
                else f'<a href="{f}.html">{t}</a>'
            )
            for f, t in PAGES
        )
        page = GABARIT.format(titre=titre, navigation=nav, corps=corps)
        (SORTIE / f"{fichier}.html").write_text(page)
        print(f"construit : {SORTIE / (fichier + '.html')}")


if __name__ == "__main__":
    construire()
