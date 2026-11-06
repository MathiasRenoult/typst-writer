# Utiliser Typst Writer

Cette documentation dit comment utiliser l'éditeur au quotidien : écrire,
mettre en page, citer, chercher, exporter. Ce qui est ici décrit
**l'application telle qu'elle est** ; les choix de conception sont ailleurs,
dans `docs/design/`, et ne regardent pas l'usage.

## Au sommaire

1. [Prise en main](prise-en-main.md) — lancer, écrire, voir, exporter.
2. [Écrire](ecrire.md) — la syntaxe de `document.tw`, de la ligne au graphique.
3. [La bibliographie](bibliographie.md) — les références, la fiche, Zotero.
4. [Le glossaire](glossaire.md) — les termes, leurs formes, la liste finale.
5. [Les normes](normes.md) — mise en page, typographie, décor, en-tête et pied.
6. [Rechercher et remplacer](recherche.md) — les expressions régulières.
7. [Les raccourcis](raccourcis.md) — le clavier, entier.
8. [Les fichiers](fichiers.md) — où tout vit, ce qu'on en garde, ce qui
   voyage.
9. [Installer sous Linux](installation-linux.md) — le paquet, d'un geste.

## Le principe de l'écriture

Un document se compose dans une couche simple, `document.tw` : du markdown
qu'on lit et qu'on compare ligne à ligne. Le Typst reste le maître du
rendu — l'aperçu est celui du document final, compilé par le vrai
compilateur — et tout ce que la couche ne dit pas encore passe par un
bloc brut, sans jamais se perdre.

La mise en page ne se porte pas sur les épaules de l'écrivain : les
mesures vivent dans les normes du document, et la source dit
l'intention. Voir [Écrire](ecrire.md) et [Les normes](normes.md).

## L'espace de travail

L'éditeur se scinde en panneaux divisibles, chacun avec ses onglets. Le
même document peut s'ouvrir dans plusieurs vues — texte partagé, aperçus
indépendants. La disposition se retrouve au lancement. La bibliothèque
de projets se parcourt en galerie plein écran. Voir
[Prise en main](prise-en-main.md).
