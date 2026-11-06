# Installer typst-writer sous Linux

Un geste, sans sudo, et l'annulation est là d'avance.

## Installer

Téléchargez le paquet `typst-writer-<version>-linux-x86_64.tar.gz`, puis :

    tar -xzf typst-writer-<version>-linux-x86_64.tar.gz
    cd typst-writer-<version>-linux-x86_64
    ./install.sh

L'application va dans `~/.local/bin/typst-writer`, l'entrée de menu dans
`~/.local/share/applications/`. Si `~/.local/bin` n'est pas dans votre
`PATH`, le script vous le dit — c'est la seule manipulation à faire.

## Vérifier le paquet

L'installeur vérifie lui-même chaque fichier par son empreinte
(`inventaire.txt`) avant de copier quoi que ce soit : un paquet abîmé ou
modifié s'arrête en le disant. Vous pouvez aussi vérifier le tarball :

    sha256sum --check typst-writer-<version>-linux-x86_64.sha256

## Mettre à jour

Réexécutez `./install.sh` avec le nouveau paquet : il remplace proprement
l'installation précédente, fichier par fichier. Il n'y a pas de
téléchargement automatique — la mise à jour se fait avec le paquet que
vous avez choisi.

## Désinstaller

    ./desinstaller.sh

Exactement ce qui a été posé est retiré — pas un fichier de plus. **Vos
documents sont intacts** : ils vivent dans le dossier des données
(`~/.typst-writer`), que la désinstallation ne touche pas.

## Où vivent les données

`~/.typst-writer/` — projets, bibliographies, normes, thèmes. Ce chemin
est aussi visible dans l'aide de l'application (`F1`), avec un bouton pour
l'ouvrir.
