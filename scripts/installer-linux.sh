#!/usr/bin/env bash
# Installe typst-writer pour l'utilisateur courant — sans sudo, en un geste.
#
#   ./install.sh
#
# Chaque fichier est vérifié par son empreinte (inventaire.txt) avant
# d'être copié : un paquet abîmé s'arrête au lieu de s'installer. Réexécuter
# le script est la mise à jour : il remplace proprement l'installation
# précédente. L'annulation est ./desinstaller.sh.
set -euo pipefail

dossier="$(cd "$(dirname "$0")" && pwd)"
base="${XDG_DATA_HOME:-$HOME/.local/share}"
bin="${HOME}/.local/bin"
applications="${base}/applications"
doc="${base}/doc/typst-writer"
trace="${base}/typst-writer/installation.txt"

echo "==> Vérification de l'intégrité du paquet"
if [ ! -f "$dossier/inventaire.txt" ]; then
    echo "inventaire.txt manque : ce paquet est incomplet, rien n'est installé." >&2
    exit 1
fi
( cd "$dossier" && sha256sum --check --strict --quiet inventaire.txt ) || {
    echo "L'intégrité du paquet n'est pas bonne : rien n'est installé." >&2
    exit 1
}

echo "==> Installation dans ${HOME}/.local"
mkdir -p "$bin" "$applications" "$doc" "${base}/typst-writer"

# Ce qui est copié est tracé fichier par fichier : la désinstallation ne
# cherche rien, elle retire exactement ce qu'elle a posé.
: > "$trace"
installer() {
    local source="$1" cible="$2"
    install -D -m "$3" "$source" "$cible"
    echo "$cible" >> "$trace"
}

installer "$dossier/bin/typst-writer" "$bin/typst-writer" 755
installer "$dossier/share/applications/typst-writer.desktop" "$applications/typst-writer.desktop" 644
for fichier in "$dossier"/share/doc/typst-writer/*; do
    if [ -d "$fichier" ]; then
        for sous in "$fichier"/*; do
            installer "$sous" "$doc/$(basename "$fichier")/$(basename "$sous")" 644
        done
    else
        installer "$fichier" "$doc/$(basename "$fichier")" 644
    fi
done

echo
echo "typst-writer est installé : $bin/typst-writer"
case ":$PATH:" in
    *":${bin}:"*) echo "La commande « typst-writer » est prête." ;;
    *) echo "Ajoutez ${bin} à votre PATH pour lancer « typst-writer »." ;;
esac
echo "Mise à jour : réexécuter ce script. Annulation : ./desinstaller.sh"
