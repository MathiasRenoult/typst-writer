# Les fichiers

## Le projet

Chaque projet vit dans son dossier, sous `~/.typst-writer/projets/`
(sous Windows : `%APPDATA%\typst-writer\projets\`) :

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
    ├── etageres.json  le classement des projets sur les étagères
    ├── themes/        les thèmes : livrés une fois, puis à vous
    ├── corbeille/     les suppressions — rien ne part dans le vide
    └── projets/       vos projets

Les thèmes sont des fichiers TOML, modifiables à la main ou depuis
l'application ; une mise à jour ne réécrit jamais un thème à vous. Les
suppressions partent à la **corbeille** — les projets comme les thèmes —
et un geste qui défait quelque chose **demande son mot** avant de partir :
`Échap` est toujours « Annuler ».

Les projets se classent sur des **étagères** — glisser une carte de
projet d'une étagère à l'autre. Le classement vit dans `etageres.json`.

Les instances récentes utilisent une ancre de verrouillage
`.verrou-natif`, qui reste dans le projet même après fermeture. Ne la
supprimez pas pendant que l'application est ouverte. Si le projet est déjà
utilisé, une autre instance l'ouvre en lecture seule.

Ne travaillez pas simultanément sur un projet avec une ancienne version de
Typst Writer : elle ne respecte pas le verrou natif. La protection a été
éprouvée sous Linux ; Windows et macOS restent à vérifier. Après un arrêt
brutal, un ancien marqueur `.verrou` peut demander une intervention :
fermez toutes les instances avant de retirer ce marqueur, jamais l'ancre
native. Conservez des copies de vos travaux importants.

## Exporter

`Ctrl+E` écrit le PDF à côté du projet. L'export suit les normes du
document — papier, marges, typographie, décor — et la bibliographie s'y
imprime selon le style du projet. Le panneau Projets porte aussi :

- **Exporter en Markdown…** : le document au canonique — les notes
  `^[…]` et citations `@clé[…]` y restent lisibles, quoique non
  standard ;
- **Exporter en DOCX…** : par LibreOffice. Titres, tableaux, citations et
  images y survivent ; **les notes deviennent du texte `[note: …]`** —
  c'est la limite annoncée. Sans LibreOffice, le geste dit son remède
  (installer LibreOffice, ou exporter en Markdown).

## Faire voyager un projet

**Exporter le projet (archive)…** écrit une archive unique : la source,
les normes, les styles, les bibliographies et les ressources, avec un
manifeste qui dit l'inventaire — et **ce qui manque déjà** (les images
appelées mais absentes, par exemple).

**Importer une sauvegarde…** restitue le projet, octet pour octet. Rien
n'est jamais écrasé sans choix : la moindre collision pose la question,
et ce qui est remplacé part à la corbeille — d'où il se retrouve. Un
import interrompu laisse la destination exactement comme avant.

## Versions et changements extérieurs

Avant chaque remplacement, la version qui se trouvait sur le disque est
conservée dans le dossier `sauvegardes/` du projet, à côté du document.
Une copie porte le nom du document suivi de son heure
(`document.tw.1762000000`) ; les copies d'un même document sont espacées
d'au moins une minute et les vingt dernières sont gardées. Le gestionnaire
de fichiers y affiche la date lisible de chacune.

**Restaurer une version** :

1. Fermez Typst Writer — le projet ne doit pas être ouvert.
2. Ouvrez `sauvegardes/` à côté de votre document.
3. Choisissez la copie voulue, d'après sa date, et copiez-la à la place du
   document : `document.tw.1762000000` devient `document.tw`.
4. Rouvrez le projet.

Si le fichier a été modifié **hors de l'application** — synchronisation,
autre éditeur — l'application ne l'écrase jamais en silence : elle le dit,
garde vos frappes en mémoire, et demande votre décision. Confirmer écrit
votre texte et conserve la version du disque dans `sauvegardes/` ; `Échap`
laisse tout en place. Une sauvegarde impossible — permission refusée,
disque plein — n'écrit rien non plus : le texte reste dans l'application,
et la version précédente est déjà conservée.

Si l'application a été **interrompue en pleine écriture** — panne,
plantage — la version en cours se retrouve elle aussi dans `sauvegardes/`
à la réouverture du document, datée de son écriture : elle se restaure
comme les autres.
