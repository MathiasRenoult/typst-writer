#!/usr/bin/env bash
# Retire typst-writer de l'installation utilisateur — exactement ce qui a
# été posé, pas un fichier de plus.
#
#   ./desinstaller.sh
#
# Les documents de l'utilisateur ne sont jamais touchés : ils vivent dans
# le dossier des données, qui reste en place.
set -euo pipefail

base="${XDG_DATA_HOME:-$HOME/.local/share}"
trace="${base}/typst-writer/installation.txt"

if [ ! -f "$trace" ]; then
    echo "Aucune installation tracée à ${trace} : rien à retirer." >&2
    exit 0
fi

echo "==> Désinstallation"
while IFS= read -r fichier; do
    [ -n "$fichier" ] || continue
    if [ -e "$fichier" ]; then
        rm -f "$fichier"
        echo "  retiré : $fichier"
    fi
done < "$trace"
rm -f "$trace"
rmdir "${base}/typst-writer" 2>/dev/null || true

echo "typst-writer est retiré. Vos documents (dossier des données) sont intacts."
