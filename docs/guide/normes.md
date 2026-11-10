# Les normes

La fenêtre des normes — `Ctrl+P`, ou le bouton de la barre — porte les
réglages de mise en page du document : **Métadonnées**, **Page**,
**Texte**, **Titres**, **Figures et dates**, **En-tête et pied**,
**Typographie**, **Couleurs et décor**, **Add-ons**. Ce qu'elle montre est
une copie de travail : seul « Appliquer » touche le projet, et les valeurs
fautives sont dites champ par champ avant que quoi que ce soit ne soit
écrit.

Les normes **activent ou désactivent** les add-ons. Les paramètres propres
à un plugin — style bibliographique, références bibliques, glossaire,
graphiques — ont leur **panneau séparé**, ouvert par le bouton de la barre
d'outils « Paramètres des plugins ». C'est la même séparation que dans le
document : les normes disent ce qui est actif, le plugin dit comment il se
comporte.

Les champs numériques se règlent à la **molette** ou aux **flèches** ▲▼
— la valeur garde son unité : `11pt` devient `12pt`, `2cm` devient `3cm`.

## Mise en page

Le papier (format, orientation), les marges, les numéros de page, le
texte courant (police, taille, interligne, retrait, justification,
langue), les titres de chapitres (police, tailles, gras, casse,
numérotation) et les couleurs.

Les choix finis sont des **listes déroulantes** ; chaque paramètre porte
son **infobulle** — laisser la souris dit ce qu'il règle et sous quelle
forme l'écrire. Toute mesure Typst est acceptée : `2cm`, `1in`, `24pt`.

## Typographie

La composition du texte réglée **par langue** : un défaut — `française`,
`anglaise`, `aucune` — puis les réglages fins, un par un.

En français : une espace insécable devant `; : ! ?` et les guillemets,
normale après ; en anglais : la ponctuation colle. En toute langue, la
virgule et le point ne s'ouvrent jamais — `3.14` tient. Les guillemets se
choisissent par niveau d'imbrication : `«»`, `""`, `''`.

Le code inline, les blocs bruts et les caractères échappés ne reçoivent
aucune règle.

## Décor

Les mesures que la source ne porte jamais : **couleur de bordure,
épaisseur, rayon des coins, marge intérieure, remplissage**. C'est là
qu'un `{encadré}` prend sa tenue ; changer une valeur ici change le
document entier, sans toucher à une ligne.

## En-tête et pied de page

Le contenu de l'en-tête et du pied de page se **rédige librement** :
chacun a trois emplacements (gauche, centre, droite). La numérotation
des pages se choisit par forme :

| Forme | Rendu |
| --- | --- |
| `1`, `2`, `3` | chiffres arabes |
| `a`, `b`, `c` | minuscules |
| `A`, `B`, `C` | majuscules |
| `p. 1`, `p. 2` | avec préfixe |
| `page 1 sur 12` | avec total |
| `1/12` | fraction |

La numérotation s'active ou non, et se positionne où l'on veut.

La saisie prévoit aussi les champs `{document}`, `{date}`, `{auteur}`,
`{projet}`, `{titre}`, `{page}` et `{pages}`. Ils sont résolus par le
moteur Typst ; vérifiez tout de même leur place dans le PDF exporté.

## Les plugins

Le panneau des plugins est distinct des normes. Il porte quatre onglets :

- **Bibliographie** — le style des citations et de la liste finale,
  l'import d'un fichier `.csl` et les styles installés dans Zotero ;
- **Biblique** — la forme des livres (abréviation, nom complet, OSIS,
  sans point), les chiffres arabes ou romains, les séparateurs, le gabarit
  de citation, l'ordre canonique et l'index des citations (pages,
  regroupement, forme, colonnes) ;
- **Glossaire** — un renvoi vers le panneau du glossaire, où se gèrent les
  termes ;
- **Graphiques** — un renvoi vers la fenêtre d'insertion, qui porte les
  données.

Une balise biblique peut choisir sa propre forme avec `|long`, `|court`,
`|osis` ou `|sanspoint` ; elle gagne sur le réglage du projet. Une
référence peut aussi nommer sa traduction avec `|lsg` ou `|darby` ; elle
est alors affichée dans la citation.

## Les titres par niveau

Les tailles se règlent par niveau. La graisse, l'italique, le souligné,
l'espacement et la couleur se règlent aussi par niveau ; le réglage global
reste le défaut. La casse présentée dans la fenêtre est un réglage global.

Voir [Écrire](ecrire.md).
