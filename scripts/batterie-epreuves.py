#!/usr/bin/env python3
"""Batterie d'épreuves à HOME jetable — un HOME neuf par test.

Découvre les tests #[ignore] d'un préfixe de module dans le binaire de test
et les lance un par un, chacun dans un HOME neuf sous TMPDIR, avec les
variables d'environnement demandées. C'est la façon canonique de vérifier
une suite applicative à état persistant (verrous, sessions, projets) :
un HOME partagé fait interférer des tests pourtant coupés du système.

Usage :
    python3 scripts/batterie-epreuves.py \
        --prefix app::espace_runtime::epreuves:: \
        --env TYPST_WRITER_TEST_ESPACE=1 \
        --env XDG_CONFIG_HOME={home}/config \
        --timeout 30

Le binaire de test se découvre tout seul (« Executable unittests » de
`cargo test --no-run`) ; --binary court-circuite la découverte. Chaque
variable d'environnement accepte {home} pour le HOME jetable du test.
Après toute exécution, la ligne « test result: » fait foi — un verdict
absent est un échec, jamais une réussite.
"""

import argparse
import os
import re
import subprocess
import sys
import tempfile


def decouvrir_le_binaire() -> str:
    sortie = subprocess.run(
        "cargo test --no-run 2>&1", shell=True, capture_output=True, text=True
    ).stdout
    for ligne in sortie.splitlines():
        if "Executable unittests" in ligne and "(target/" in ligne:
            return ligne[ligne.index("(target/") + 1 : ligne.rindex(")")]
    raise SystemExit("binaire de test introuvable — passer --binary")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--prefix", required=True, help="préfixe des tests, ex. module::epreuves::")
    parser.add_argument("--binary", help="chemin du binaire de test (sinon : découverte)")
    parser.add_argument("--env", action="append", default=[], metavar="CLE=VALEUR")
    parser.add_argument("--timeout", type=int, default=30)
    args = parser.parse_args()

    binaire = args.binary or decouvrir_le_binaire()

    liste = subprocess.run(
        [binaire, "--list", "--ignored"], capture_output=True, text=True, check=True
    ).stdout
    noms = [
        ligne.split(": test")[0]
        for ligne in liste.splitlines()
        if ligne.startswith(args.prefix) and ligne.endswith(": test")
    ]
    if not noms:
        print(f"AUCUN test sous {args.prefix} — motif faux ou exécution non filtrée ?")
        return 1

    echecs = 0
    for nom in noms:
        with tempfile.TemporaryDirectory(dir=os.environ.get("TMPDIR"), prefix="epreuve-") as home:
            environ = os.environ.copy()
            environ["HOME"] = home
            for couple in args.env:
                cle, _, valeur = couple.partition("=")
                environ[cle] = valeur.replace("{home}", home)
            rendu = subprocess.run(
                ["timeout", str(args.timeout), binaire, nom, "--ignored"],
                env=environ,
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
            )
            vert = "test result: ok" in rendu.stdout
            print(("VERT  " if vert else "ECHEC ") + nom)
            if not vert:
                echecs += 1
                print(rendu.stdout[-1500:])

    print(f"{len(noms) - echecs}/{len(noms)} épreuves réussies")
    return 1 if echecs else 0


if __name__ == "__main__":
    sys.exit(main())
