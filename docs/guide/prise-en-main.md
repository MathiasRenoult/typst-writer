# Prise en main

## Lancer

L'application s'ouvre sur le dernier projet ouvert, ou sur un projet neuf
au premier lancement. La disposition de l'espace de travail — panneaux,
onglets, modes — se retrouve telle que vous l'avez laissée. L'application
sauvegarde automatiquement à intervalles d'environ une seconde. À la
fermeture normale, elle tente aussi d'enregistrer tous les documents modifiés ;
si une écriture échoue, elle conserve la fenêtre ouverte et signale le
problème. Une panne ou une interruption brutale peut toujours emporter les
dernières frappes : gardez une copie de vos travaux importants.

## Les trois gestes du quotidien

1. **Écrire** dans le panneau central. Le texte est du `document.tw` —
   la syntaxe est dans [Écrire](ecrire.md).
2. **Voir** à droite : la page se compose dès que l'écriture s'arrête,
   et l'aperçu suit le chapitre sous le curseur. `Ctrl+=`, `Ctrl+-` et
   `Ctrl+0` en gouvernent la taille.
3. **Exporter** en PDF — `Ctrl+E` — quand le document est prêt.

## L'espace de travail

L'éditeur se scinde en **panneaux** : glisser un bord de 8 px divise le
panneau en deux. Chaque panneau a ses **onglets** — un document peut
s'ouvrir dans plusieurs panneaux, avec du texte partagé et des vues
indépendantes (édition, rendu, Typst).

- **shift+clic** sur un projet l'ouvre dans un nouvel onglet ;
- **clic simple** remplace l'onglet actif (sauvegarde avant fermeture) ;
- **glisser un onglet** le déplace d'un panneau à l'autre, le détache
  ou divise le panneau cible ;
- un **point coloré** sur chaque onglet dit le mode : bleu (édition),
  vert (rendu), orange (Typst) ;
- les **panneaux vides** montrent une page d'accueil : Créer, Ouvrir,
  Bibliothèque.

## La bibliothèque

Le bouton « Bibliothèque » ouvre la galerie plein écran : les projets se
voient par couvertures, rangés sur des **étagères** que l'on crée librement,
y compris vides. Les flèches ↑ ↓ d'une étagère la déplacent dans l'ordre de
la galerie ; le bouton « + Nouveau projet » d'une étagère y pose le projet
à naître.

Un **clic** ouvre un projet. **Ctrl+clic** ajoute ou retire un document de
la sélection, **Maj+clic** sélectionne toute la plage entre le dernier
document choisi et celui-ci. Dès qu'une sélection existe, ses actions
rapides paraissent : export PDF, export Typst Writer (`.tw`), changement
des normes (une norme appliquée à tous), export Markdown et export DOCX.
Les exports vont dans un dossier choisi, un fichier par document, et
n'écrasent jamais un fichier existant.

Chaque couverture est la **première page réelle** du projet, toujours
présentée au format A4 — même si le document est composé en Letter, en A5
ou en paysage. Les pages déjà rendues sont mises en cache : la galerie
s'ouvre sans recompiler les projets. Le bouton « Changer la miniature »
accepte une image **PNG ou JPG**, la recadre au centre au format A4 et la
garde avec le projet ; elle remplace alors la première page.

## Ce qui se voit tout de suite

- la **barre d'état**, en bas : le compte de mots, les diagnostics du
  compilateur, et ce que l'application a à dire (une erreur, une
  confirmation) — jamais de silence ;
- à gauche, le **panneau de travail** : un bloc **Repères** en tête (nom
  du document, volume, références, notices, glossaire), puis le sommaire
  en arborescence, les statistiques bibliques, la bibliographie, le
  glossaire et les autres projets. Chaque section se replie ;
- **survol** d'une référence biblique : après un court repos, son verset
  s'affiche dans une boîte au-dessus du curseur, sans touche à maintenir ;
- **cliquer sur une balise biblique** ouvre « La référence cliquée » dans le
  panneau gauche : chaque citation y est séparée avec son livre et son
  chapitre, chacune porte son extrait, et le crayon de sa ligne ouvre une
  fenêtre flottante pour la corriger ;
- les **touches** se découvrent d'elles-mêmes : `F1` ouvre la carte des
  raccourcis, et les infobulles de la barre d'outils portent leurs
  combinaisons.

## Ouvrir le document, ou un chapitre

`Ctrl+N` ouvre le document d'écriture du projet. `Ctrl+O` charge un
fichier `.typ` existant dans le projet — pour une migration, ou pour un
chapitre hérité. Voir [Les fichiers](fichiers.md).

Dans le panneau gauche, **Chapitres** propose le document entier ou une tête
de niveau 1 ; la vue par chapitre ne montre que ses pages, et le sommaire se
réduit à ses têtes. Le document reste entier dans l'éditeur : la vue de
chapitre ne réécrit jamais `document.tw`.

## En cas de doute

`Échap` ferme toujours ce qui s'est ouvert en dernier. Les fenêtres —
recherche, normes, fiche de référence — se conduisent toutes de même :
`Tabulation` circule, `Entrée` valide. Il n'y a rien à retenir de plus.
