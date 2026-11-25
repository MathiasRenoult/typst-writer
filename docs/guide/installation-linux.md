# Installer typst-writer sous Linux

La version 0.6.3 est publiée **sans paquet binaire ni artefact de test**.
Le code source est la référence.

## Compiler depuis les sources

Une chaîne Rust à jour suffit :

    git clone https://github.com/MathiasRenoult/typst-writer.git
    cd typst-writer
    cargo build --release
    ./target/release/typst-writer

L'application demande Linux avec Wayland ou X11 et un pilote graphique
récent. L'assistance à la frappe est facultative.

## Paquets des versions précédentes

Les versions antérieures fournissaient un paquet Linux installable sans
`sudo`, avec son empreinte et son désinstallateur. Ces paquets restent
disponibles dans leurs versions GitHub. Une fois installés, ils placent
l'application dans `~/.local/bin/typst-writer` et l'entrée de menu dans
`~/.local/share/applications/`. La désinstallation ne touche jamais aux
documents.

## Où vivent les données

`~/.typst-writer/` — projets, bibliographies, normes, thèmes. Ce chemin
est aussi visible dans l'aide de l'application (`F1`), avec un bouton pour
l'ouvrir.
