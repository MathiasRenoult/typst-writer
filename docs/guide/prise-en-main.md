# Prise en main

## Lancer

L'application s'ouvre sur le dernier projet ouvert, ou sur un projet neuf
au premier lancement. La disposition de l'espace de travail — panneaux,
onglets, modes — se retrouve telle que vous l'avez laissée. Tout ce que
vous écrivez est enregistré automatiquement, une seconde après la
frappe : rien ne se perd à fermer.

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
voient par couvertures, rangés sur des **étagères** que l'on déplace à la
souris. Un bouton « + Nouveau projet ici » crée un projet dans
l'étagère.

## Ce qui se voit tout de suite

- la **barre d'état**, en bas : le compte de mots, les diagnostics du
  compilateur, et ce que l'application a à dire (une erreur, une
  confirmation) — jamais de silence ;
- à gauche, le **panneau de travail** : le sommaire du document, les
  références de la bibliographie, les autres projets ;
- les **touches** se découvrent d'elles-mêmes : `F1` ouvre la carte des
  raccourcis, et les infobulles de la barre d'outils portent leurs
  combinaisons.

## Ouvrir le document, ou un chapitre

`Ctrl+N` ouvre le document d'écriture du projet. `Ctrl+O` charge un
fichier `.typ` existant dans le projet — pour une migration, ou pour un
chapitre hérité. Voir [Les fichiers](fichiers.md).

## En cas de doute

`Échap` ferme toujours ce qui s'est ouvert en dernier. Les fenêtres —
recherche, normes, fiche de référence — se conduisent toutes de même :
`Tabulation` circule, `Entrée` valide. Il n'y a rien à retenir de plus.
