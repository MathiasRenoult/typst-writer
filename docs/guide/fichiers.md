# Les fichiers

## Le projet

Chaque projet vit dans son dossier, sous `~/.typst-writer/projets/` :

    mon-travail/
    ├── document.tw            ce que vous écrivez
    ├── main.typ               l'assemblage, et ses réglages Typst
    ├── mise-en-page.toml      les normes, telles que vous les réglez
    ├── mise-en-page.typ       leur traduction pour Typst — générée
    ├── references.bib         la bibliographie locale
    ├── references-zotero.bib  l'export de la collection Zotero liée
    ├── zotero.json            la collection Zotero liée au projet
    └── chapitres/             les sources `.typ` héritées

`document.tw` et `references.bib` sont les vôtres ; `mise-en-page.typ`
est généré. Les fichiers `.typ` importés y restent — une migration ne
détruit rien de ce qui existe.

## L'application

    ~/.typst-writer/
    ├── session.json   l'état de la dernière session
    ├── themes/        les thèmes : livrés une fois, puis à vous
    ├── corbeille/     les suppressions — rien ne part dans le vide
    └── projets/       vos projets

Les thèmes sont des fichiers TOML, modifiables à la main ou depuis
l'application ; une mise à jour ne réécrit jamais un thème à vous. Les
suppressions partent à la **corbeille** — les projets comme les thèmes —
et un geste qui défait quelque chose **demande son mot** avant de partir :
`Échap` est toujours « Annuler ».

Une seule instance travaille sur un projet à la fois : la seconde
s'ouvre **en lecture seule**, et la raison s'affiche en barre d'état.
Deux écritures ne se dévorent jamais leur texte.

## Exporter

`Ctrl+E` écrit le PDF à côté du projet. L'export suit les normes du
document — papier, marges, typographie, décor — et la bibliographie s'y
imprime selon le style du projet.
