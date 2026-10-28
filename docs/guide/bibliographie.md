# La bibliographie

## Les références du projet

Le panneau de gauche liste les références de la bibliographie — par
**auteur et par titre**, jamais par clé — séparées par un filet, l'auteur
au premier registre, l'ouvrage au second. Un clic sur la ligne insère
`@cle` au curseur.

## La fiche de référence

Le crayon d'une ligne ouvre sa fiche ; **Nouvelle référence** en crée une
(la clé se numérote d'elle-même, et ne se réutilise jamais). La fiche
porte : le type de document en liste (livre, article, chapitre, actes,
thèse, rapport, en ligne, autre), le nom, le prénom, le titre, le titre
court, l'année, **l'éditeur** et **le lieu**.

La fiche se conduit au clavier : `Tabulation` circule dans les champs,
`Maj+Tabulation` revient, `Entrée` enregistre, `Échap` ferme sans rien
écrire. Une notice neuve est jugée par le parseur bibliographique avant
d'être acceptée.

## Les deux sources

- une **bibliothèque locale** (`references.bib`) s'écrit : la fiche
  réécrit les champs de son entrée, et rien d'autre ;
- une collection **Zotero** liée se lit **seulement** — c'est Zotero qui
  la maîtrise, et la synchronisation reprendrait de toute façon ses
  valeurs.

## Lier un projet à Zotero

Le bouton « Lier à Zotero… » liste les **collections** de la bibliothèque
Zotero ouverte — leur nom, leur chemin (« Thèse / Chapitre 2 »), leur
nombre de notices. Une collection choisie s'exporte dans le projet, et le
bouton « Synchroniser » reprend l'export quand la collection bouge.

La liaison vit dans `zotero.json`, à côté du projet ; « Délier » la
retire. Pour que la liaison trouve Zotero, l'application peut poser elle-
même le réglage nécessaire — le panneau le propose d'un clic, et la note
dit ce qui reste à faire.

## La bibliographie imprimée

Le style du projet est `chicago-notes` : les appels deviennent des notes
de bas de page, et la bibliographie s'imprime à la fin du document. Un
style défini explicitement dans `main.typ` reste prioritaire.
