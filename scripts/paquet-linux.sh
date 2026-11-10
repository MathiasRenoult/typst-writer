#!/usr/bin/env bash
# Fabrique le paquet Linux de typst-writer : un tarball installable par
# l'utilisateur, avec son inventaire vérifié (intégrité) et son annulation.
#
#   bash scripts/paquet-linux.sh
#
# Produit dans dist/ :
#   typst-writer-<version>-linux-x86_64.tar.gz   le paquet
#   typst-writer-<version>-linux-x86_64.sha256   son empreinte
#
# Le paquet se vérifie par l'inventaire de ses fichiers et `ldd` — jamais
# en lançant le binaire : une application qui s'ouvre pendant la vérification
# pourrait toucher aux vrais documents de l'utilisateur.
set -euo pipefail

racine="$(cd "$(dirname "$0")/.." && pwd)"
cd "$racine"

version="$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)"
arch="$(uname -m)"
nom="typst-writer-${version}-linux-${arch}"
dossier="dist/${nom}"

echo "==> Build release"
cargo build --release

echo "==> Assemblage de ${nom}"
rm -rf "$dossier"
mkdir -p "$dossier/bin" \
         "$dossier/share/applications" \
         "$dossier/share/doc/typst-writer"

cp target/release/typst-writer "$dossier/bin/typst-writer"
cp README.md CHANGELOG.md "$dossier/share/doc/typst-writer/"
cp -r docs/guide "$dossier/share/doc/typst-writer/guide"
if [ -f LICENSE ]; then
    cp LICENSE "$dossier/share/doc/typst-writer/"
fi

# L'entrée de menu : sans icône inventée — le dessin appartient à
# l'application quand elle en aura un.
cat > "$dossier/share/applications/typst-writer.desktop" <<'BUREAU'
[Desktop Entry]
Type=Application
Name=Typst Writer
Comment=Écrire en Typst, avec le rendu du document final
Exec=typst-writer
Terminal=false
Categories=Office;TextEditor;
BUREAU

cp scripts/installer-linux.sh "$dossier/install.sh"
cp scripts/desinstaller-linux.sh "$dossier/desinstaller.sh"
chmod +x "$dossier/install.sh" "$dossier/desinstaller.sh"

# L'inventaire : chaque fichier, son empreinte — c'est ce que l'installeur
# vérifie avant de copier, et ce que la désinstallation retrouve.
echo "==> Inventaire (intégrité)"
(
    cd "$dossier"
    find . -type f ! -name 'inventaire.txt' -print0 \
        | sort -z \
        | xargs -0 sha256sum > inventaire.txt
)

echo "==> Tarball"
tar -C dist -czf "dist/${nom}.tar.gz" "$nom"
( cd dist && sha256sum "${nom}.tar.gz" > "${nom}.sha256" )

echo
echo "Paquet : dist/${nom}.tar.gz"
echo "Empreinte : dist/${nom}.sha256"
echo "Installation : tar -xzf dist/${nom}.tar.gz && ./${nom}/install.sh"
