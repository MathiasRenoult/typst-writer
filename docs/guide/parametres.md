# Les paramètres de l'application

La page **Paramètres** règle l'application elle-même : la barre supérieure,
l'éditeur et son apparence. Elle ne touche **jamais** au document ni à ses
normes — le papier, les marges, les titres et les citations se règlent dans
[Les normes](normes.md), avec `Ctrl+P`.

Ouvrez-la par l'icône d'engrenage à droite de la barre, par `Ctrl+,`, ou par
la palette de commandes (`Ctrl+K`, puis « paramètres »).

## La barre supérieure

La barre se lit en trois domaines, de gauche à droite :

- **Projet et document** : la bibliothèque des projets, le menu `Projet`
  (création, archives, imports et exports), l'ouverture du document, l'export
  PDF et la fermeture de l'onglet actif ;
- **Écriture** : les boutons de mise en page et le menu `Insérer` ;
- **Document et application** : les commandes, les normes, les paramètres des
  plugins, le panneau de travail, le mode focus et cette page.

Deux réglages retirent les gestes d'écriture de la barre :

- **Boutons de mise en page** : gras, italique, titres, listes, citation de
  bloc et couleurs. Les masquer convient à qui écrit déjà le markdown ;
- **Menu Insérer** : sommaire, notes, citations, images, tableaux, graphiques,
  sections et extensions.

Les raccourcis et la palette de commandes restent actifs quand ces boutons
sont masqués. La note de bas de page, la citation bibliographique et le bloc
Typst brut restent dans le menu `Insérer` : la mise en page peut disparaître
sans emporter l'insertion.

## L'éditeur

L'affichage des **numéros de ligne** est une préférence de lecture, pas une
propriété du document : elle vaut pour tous les projets et survit au
redémarrage. Elle est masquée par défaut en prose.

## L'apparence

Le thème de l'interface, le mode d'apparence (système, clair ou sombre), le
chargement d'un thème, son édition et sa suppression vivent ici. Les thèmes
ne changent que l'application ; ils ne modifient pas la mise en page du
document.

## Les données

Le panneau **Données** nomme le dossier local où vivent les projets, les
sauvegardes, les thèmes et la session de travail. Le bouton l'ouvre dans le
gestionnaire de fichiers ; rien n'est envoyé ailleurs.

## Ce qui est retenu

Les deux réglages de la barre, les numéros de ligne, le thème et le mode
d'apparence sont enregistrés dans la **session de travail**, avec la
disposition des panneaux. Ils se retrouvent au prochain lancement, sans être
écrits dans le projet ni dans ses normes.
