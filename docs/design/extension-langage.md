# Étendre le langage — blocs et propriétés

> Comment ajouter des tableaux, des images, des formules, des dégradés,
> des bordures, des formes… sans jamais défaire ce qui tient.

La grammaire de `document.tw` est **gelée** (voir `langage-intermediaire.md`).
Elle ne l'est pas pour toujours : elle l'est pour que chaque ajout soit
**délibéré**. Ce document est la procédure de cet ajout — et la première
brique qu'il a produite, l'image et le décor, qui l'ont éprouvée.

## Les deux natures d'extension

Un ajout est toujours l'un des deux, et jamais les deux à la fois :

1. **Un bloc** — une construction qui a sa place dans la structure du
   document : un tableau, une image, une formule, une page de garde.
   Un bloc est une **ligne ou un groupe de lignes** reconnaissable, avec
   une forme canonique, un parseur, un générateur, et un genre d'édition.
2. **Une propriété** — un décor qui s'applique à un bloc quelconque :
   une bordure, un remplissage, un dégradé, un rayon, une forme. Les
   propriétés ne sont pas des blocs : elles **habitent** ceux qui existent.

Cette distinction commande tout le reste : les propriétés ont été faites
**une fois pour toutes** (`Proprietes` et `Bloc::Décor`), et chaque bloc
nouveau en hérite sans rien y toucher.

## La syntaxe des propriétés

La ligne d'attributs précède le bloc qu'elle habille :

    {bordure=1pt rayon=4pt}
    > une citation encadrée

    {largeur=50% remplissage={gradient.linear(#blue, #red)}}
    ![La planche](figs/planche.png)

Les paires sont `clé=valeur`, séparées par des espaces ; une valeur qui
porte une espace s'accôle entre accolades. Les clés sont des noms
(`lettres, chiffres, tirets, soulignements`) et les valeurs des
**expressions Typst** — pas du texte. Ni clé ni valeur ne portent de
marqueur ni de saut de ligne : les balayages du voisinage ne doivent pas
s'y méprendre, au même titre que le nom d'une couleur.

Les manteaux emboîtés se fondent en un seul à l'écriture — un seul
manteau par bloc, c'est la forme canonique. Une ligne d'attributs qui ne
trouve pas de bloc à habiller **reste du texte** : ce qui ne se lit pas
comme une construction n'en est pas une.

## Le vocabulaire du décor

Les propriétés deviennent les arguments d'un `#block(…)` Typst. Chaque
nom français a son argument :

| Propriété | Argument Typst | Valeur |
|---|---|---|
| `largeur` | `width` | `50%`, `12cm`, `auto` |
| `hauteur` | `height` | `4cm`, `auto` |
| `bordure` | `stroke` | `1pt`, `red`, `(paint: gray, thickness: 1pt)` |
| `remplissage` | `fill` | `luma(240)`, `#ff0000`, `gradient.linear(…)` |
| `rayon` | `radius` | `4pt`, `(top-left: 4pt)` |
| `marge` | `inset` | `8pt`, `(x: 4pt, y: 8pt)` |
| `marge_ext` | `outset` | `2pt` |
| `espace` | `spacing` | `1em` |

Une clé que la table ne connaît pas passe **telle quelle** : le Typst
reste le maître du rendu, et c'est lui qui dira s'il la comprend. C'est
la même règle que les blocs bruts — l'intermédiaire ne bloque jamais ce
qu'il ne sait pas dire.

## La recette pour un bloc nouveau

Trouvée en ajoutant l'image, vérifiée par elle. Sept endroits, un seul
ordre — et le générateur au hasard qui juge la dernière étape :

1. **`schema.rs`** — la variante de `Bloc` avec sa forme canonique
   (`Bloc::est_canonique`), et son texte visible (`Bloc::texte_nu`).
2. **`parse.rs`** — la lecture de la ligne, dans la boucle de `parse`.
   Une forme qui n'est pas reconnue **reste du texte** : jamais d'erreur,
   jamais de panne — le parseur ne renvoie rien d'autre que du sens.
3. **`serialiser.rs`** — l'écriture de la forme canonique, dans
   `serialiser`. L'écriture est déterministe : ce qui se relit doit se
   réécrire à l'identique.
4. **Les remparts** — `parse::ouvre_un_bloc` (les lignes) et
   `schema::ouvre_un_bloc` (la tête de paragraphe) : le nouveau marqueur
   s'inscrit dans les deux, pour qu'un paragraphe qui commence par lui se
   protège d'une barre, jamais d'un malentendu.
5. **`generer.rs`** — la forme Typst. Les valeurs sont des expressions
   Typst ; le composé suit les normes du document.
6. **`blocs.rs`** — le genre d'édition, et sa place dans `blocs()` :
   un bloc se déplace **avec son manteau**, et une forme qui est sa
   donnée (l'image) ne change pas de nature.
7. **Le générateur au hasard** — `bloc_au_hasard` produit la forme, et
   `le_round_trip_tient_sur_tout_ast_canonique` juge par centaines de
   documents. **Une épreuve qui crie est la recette qui marche** : c'est
   elle qui a trouvé que les légendes portaient des liens, et que la
   cible d'une image s'échappe comme celle d'un lien.

## Ce que l'image a appris

- **Le lecteur de lien est la leçon de toutes les formes** : `![…](…)`
  est le lien à l'`!` près, et il hérite de l'imbrication des crochets,
  des échappements et de l'opacité du code. Une forme nouvelle qui
  ressemble à une forme ancienne **s'appuie sur son lecteur**, elle ne
  l'imite pas.
- **On lit depuis la fin** quand la donnée le permet : la cible n'a ni
  parenthèse ni saut de ligne, donc le délimiteur se trouve sans
  ambiguïté — une légende qui contient un lien ne coupe plus rien.
- **La cible s'échappe** comme au lien : les dix marqueurs s'écrivent
  avec leur barre, et la lecture les retire. Oublier la moitié du
  voyage — échapper sans déséchapper — se voit au round-trip, jamais
  avant.

## La route

1. **Fait** — `Proprietes`, `Bloc::Décor`, `Bloc::Image` : le décor de
   tout bloc (bordures, remplissages, dégradés, rayons, formes) et la
   première brique de contenu.
2. **Le tableau** — `| A | B |` et sa ligne de séparation, cellules en
   éléments inline. Le décor lui donne déjà ses bordures et son
   remplissage ; le bloc lui donne sa grille.
3. **La formule** — `$…$` en ligne et `$$…$$` en bloc, bandes opaques
   comme le code : ce qui s'y trouve ne ferme rien du voisinage.
4. **Les formes** — `forme=…` en propriété : le `#block(…)` des
   propriétés est déjà une boîte ; une forme n'est qu'un dessin de plus.
5. **L'image en ligne** — `![](…)` dans le texte d'un paragraphe, quand
   la figure par bloc ne suffit pas.

Chaque étape suit la recette, et la recette ne change pas : c'est tout
son objet.

## Ce qui ne changera jamais

Les invariants de `langage-intermediaire.md` sont intacts, et chaque
extension les tient :

1. le Document structuré est la source de vérité ;
2. `parse(serialiser(document)) == document` pour tout document
   canonique — éprouvé au hasard, pour les formes neuves comme pour
   les anciennes ;
3. ce qui n'est pas reconnu reste du texte, ou du bloc brut Typst ;
4. le même Document génère toujours le même corps Typst ;
5. les échappements sont uniformes, et un échappement fait un littéral ;
6. une migration ne détruit rien.
