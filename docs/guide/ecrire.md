# Écrire

Le document se compose dans `document.tw` : une syntaxe markdown, lisible
et comparable ligne à ligne. Ce qui suit est l'état actuel de cette
syntaxe.

## La structure

| Ce qu'on écrit | Ce que c'est |
| --- | --- |
| `# Titre`, `## Sous-titre` | un titre, de niveau 1 à 6 |
| une suite de lignes | un paragraphe ; les lignes se joignent par une espace |
| `> texte` | une citation de bloc |
| `- élément` | une liste à puces |
| `1. élément` | une liste numérotée |
| ```` ```typst ```` … ```` ``` ```` | un bloc Typst brut, inséré tel quel |
| trois lignes vides consécutives | un saut de page |
| `// texte` | un commentaire — conservé dans le `.tw`, ignoré au rendu |
| `{bibliographie}` | la bibliographie du projet, à cet endroit |
| `{{index}}` | l'index des citations bibliques, recalculé au rendu |

Une **ligne vide** sépare les blocs. Ce qui n'est pas reconnu reste du
texte — un fichier abîmé ne bloque jamais l'ouverture.

## Le texte

| Ce qu'on écrit | Ce que c'est |
| --- | --- |
| `**texte**` | le gras |
| `*texte*` ou `_texte_` | l'italique |
| `***texte***` | le gras et l'italique |
| `__texte__` | le souligné |
| `~~texte~~` | le barré |
| `` `code` `` | du code inline — une bande opaque : rien ne s'y ouvre ni ne s'y ferme |
| `[texte](https://…)` | un lien |
| `^[texte]` | une note libre, en bas de page |

## Les citations

`@cle` appelle une référence de la bibliographie ; `@cle[p. 45]` porte la
page. Plusieurs appels d'un geste — `@dupont[p. 45]; @martin` — ne font
jamais qu'**une** note de bas de page, avec les deux mentions.

L'appel est un **glyphe numéroté** — ¹, ² — dans le fil du texte, un
caractère entier : il se déplace, se sélectionne et se supprime d'un seul
geste. **Cliquer dessus** ouvre le composeur de notes ; les flèches
parcourent la note puis les clés, `Échap` abandonne sans rien écrire.

Voir [La bibliographie](bibliographie.md).

## Les références bibliques

Une référence s'écrit entre doubles accolades : `{{Jn 3.16}}`. La forme
standard est le **nom abrégé suivi de `chapitre.verset`** ; le livre peut
aussi être nommé en entier ou en code OSIS, et les livres numérotés
acceptent `1Jn` comme `1 Jean`. Les formes historiques `Jn 3:16` et
`Jn 3,16` restent lues.

Le bouton **Référence biblique** de la barre supérieure ouvre le panneau
d'insertion : on y choisit un livre, un chapitre et des versets (`16`,
`16-18`, `16, 17`), on ajoute autant de références que nécessaire, puis on
insère **une seule** balise qui les porte toutes. Le champ chapitre accepte
une suite de chapitres entiers, `1-11`. Un champ **texte libre** ajoute une
mention à la fin de la citation : elle est écrite entre guillemets et
rendue telle quelle, sans devenir une référence.

    {{Genèse 3.15 ; Marc 2.8 ; Jean 3.16}}
    {{Gn 1-11}}                 → Genèse 1 à 11
    {{Jn 3.16 ; "cf. aussi" ; Gn 1.1}}

C'est aussi la syntaxe à connaître pour écrire une citation multiple à la
main : le point-virgule sépare les membres, la virgule prolonge le membre
courant (`{{Jn 3.16, 17 ; Rm 8.1}}`). Les formes relatives sont lues :
`{{Jn 3.16 ; 17 ; 4.1}}`. Le texte libre se place où l'on veut entre les
références, à condition d'être entre guillemets ; un point-virgule dans les
guillemets ne coupe pas la citation.

Un même livre n'est nommé qu'une fois : ses occurrences sont réunies et
séparées par une virgule. L'ordre des livres suit leur première apparition,
sauf si la norme **Ranger les références dans l'ordre canonique** est
cochée dans l'onglet **Biblique** des plugins. La norme s'applique au
rendu ; une balise déjà écrite garde son ordre dans la source, et le
panneau d'insertion produit directement l'ordre canonique.

    {{2 Ch 5.6 ; Js 1.9 ; 2 Ch 7.6-12}}
    → Josué 1.9 ; 2 Chroniques 5.6, 7.6-12   (forme longue, ordre canonique)
    → 2Ch 5.6, 7.6-12 ; Js 1.9               (forme courte, ordre du texte)

La forme d'affichage se choisit pour la balise avec un suffixe ; le panneau
d'insertion propose les mêmes choix sans modifier les normes du projet :

- `|long` — le nom complet, « Jean 3.16 » ;
- `|court` — l'abréviation française, « Jn 3.16 » ;
- `|osis` — le code OSIS, « John 3.16 » ;
- `|sanspoint` — l'abréviation sans point.

Sans suffixe, c'est la forme réglée pour le projet qui s'applique. Le
panneau des plugins (onglet **Biblique**) choisit aussi les chiffres des
chapitres et des versets, les séparateurs et l'ordre canonique.

Une référence peut nommer sa **traduction** : `{{Jn 3.16|lsg}}` rend
`Jn 3.16 (LSG)`, `{{Jn 3.16|darby}}` rend `Jn 3.16 (DARBY)`. La
traduction se met sur chaque référence, avant le suffixe de forme de la
balise s'il y en a un (`{{Jn 3.16|lsg|long}}`). Sans précision, aucune
mention de traduction n'est ajoutée à la citation.

### Composer sa syntaxe

Le champ **Gabarit d'une référence** de l'onglet **Biblique** décrit la
forme complète d'une référence, indépendamment de la balise source qui
reste `{{…}}`. Les jetons disponibles sont `{livre}`, `{chapitres}`,
`{versets}`, `{nombres}`, `{chapitre}`, `{chapitre_fin}`, `{verset}`,
`{verset_fin}` et `{reference}`. Par exemple :

    {livre} {chapitre}, {verset}       →  Jn 3, 16
    ({reference})                     →  (Jn 3.16)
    {chapitres}.{versets}              →  1.18

Un gabarit vide revient au standard `{livre} {nombres}`. Les formes
longues, courtes, OSIS et sans point, les chiffres romains et les
séparateurs se combinent avec ce gabarit.

### L'index des citations

L'index s'insère avec `{{index}}` ou l'icône de la barre supérieure. Dans
l'onglet **Biblique**, on peut :

- **montrer les pages** où chaque référence apparaît, avec la liste des
  pages séparées par des virgules ;
- **regrouper les références sous leur livre** ;
- choisir la **forme des livres dans l'index** (héritée, longue, abrégée,
  OSIS ou sans point) ;
- choisir le **nombre de colonnes** de l'index (1 à 4).

Avec le regroupement, l'index ressemble à :

    Genèse :
    - 1.18 (p. 2, 5, 6, 18)
    - 3.15 (p. 4, 5)

Les pages sont demandées à Typst au moment de la composition : ce sont
les pages réelles du document, pas une estimation de l'éditeur. Une
citation placée dans une note est signalée par le suffixe `n` (`6n`).

Le survol d'une référence affiche son extrait après un court repos, sans
touche à maintenir ; la bulle est placée au-dessus du curseur et reste
dans la fenêtre. Tant qu'une fenêtre flottante est ouverte, le texte ne
répond ni au survol ni au clic : le pointeur ne fait que désigner la
fenêtre. Cliquer sur une balise ouvre **La référence cliquée** dans
le panneau gauche : chaque citation y est séparée, avec son livre et son
chapitre (ou sa suite de chapitres), et chacune porte **son propre extrait
biblique**. Le crayon d'une ligne ouvre une **fenêtre flottante**
(livre, chapitre, versets) ; **Enregistrer** réécrit la balise entière en
conservant son texte libre et sa forme. Chercher une référence dans les
statistiques donne le clavier au champ de recherche, sans écrire dans
l'éditeur. La section **Statistiques bibliques** liste les références du
document, permet de les chercher, de les trier dans l'ordre du document ou
dans l'ordre canonique, et d'aller directement à leur citation d'un clic.

## Le glossaire

`{glo:cle}` appelle un terme du glossaire : la forme se développe à la
première occurrence, les reprises gardent le sigle. Les variantes
`pluriel` et `majuscule` s'écrivent après la clé — `{glo:sdo pluriel}`,
`{glo:sdo majuscule}`. La liste finale se pose d'une ligne
`{glossaire}`, ou `{glossaire: tout}` pour y montrer aussi les termes
jamais cités.

Voir [Le glossaire](glossaire.md).

## Les images

    ![La légende de la planche](figs/planche.png)

Une image avec légende compose une **figure numérotée** ; sans légende —
`![](planche.png)` — c'est l'image seule. Le chemin est relatif au projet.
Lorsqu'une image extérieure est importée depuis l'application, elle est copiée
dans `images/` ; si un fichier porte déjà ce nom, la copie reçoit un suffixe
(`figure-2.png`, par exemple) plutôt que de remplacer l'image existante.

La légende peut se séparer de l'alt par une barre :

    ![description](figs/planche.png | La vraie légende)

C'est la légende qui paraît dans la figure numérotée.

## Les formules

En ligne, entre dollars : `Soit $x^2$ dans le texte.` — une bande
opaque comme le code : rien ne s'y ouvre ni ne s'y ferme.

À part, sur ses lignes :

    $$
    x^2 + y^2 = z^2
    $$

Le contenu est du **math Typst**, mot pour mot. Une barre de dollars
dans le texte s'échappe : `\$`.

## Les tableaux

La forme markdown, avec sa ligne de séparation qui dit l'alignement :

    | En-tête | Valeur |
    | --- | :-: |
    | a | 1 |
    | b | 2 |

`---` aligne à gauche, `:-:` au centre, `---:` à droite. Les cellules
portent du texte ordinaire — emphase, code, citations ; une barre dedans
s'échappe : `\|`. Sans la ligne de séparation, ce n'est pas un tableau :
la ligne reste du texte.

Une ligne qui porte plus de cellules que l'en-tête **élargit le tableau**
au lieu de perdre la fin de la ligne ; les cellules Unicode se relisent
telles quelles. Vérifiez tout de même le rendu d'un tableau irrégulier
dans le PDF avant de le remettre.

Le décor l'habille comme tout bloc — `{encadré}` prend sa tenue dans les
normes.

## Les graphiques

Un graphique se déclare par un bloc `{graphique}` suivi de ses données :

    {graphique type=barre titre="Évolution"}
    Jan | 10
    Fév | 20
    Mar | 15

Chaque ligne est `libellé | valeur`. Le `type` dit la forme : `barre`
(par défaut), `ligne` ou `secteur`. Le `titre` est facultatif — il devient
la légende de la figure.

Les graphiques utilisent le paquet Typst
[cetz](https://typst.app/universe/package/cetz). Sa résolution dans
l'application doit être vérifiée avant de compter sur ce rendu.

## Le décor : l'intention, jamais la mesure

**La source dit l'intention, les normes disent la mesure.** Par défaut,
un bloc paraît comme les normes du document le disent — rien à écrire.

Quand un bloc doit paraître autrement, un **nom** le précède :

    {encadré}
    > une note encadrée

`encadré` prend dans les normes sa couleur de bordure, son épaisseur, son
rayon et sa marge ; `plein` y prend son remplissage. Les noms se
combinent — `{encadré plein}`. La mesure reste possible pour l'exception
— `{encadré rayon=0pt}` — et elle gagne sur les normes. Voir
[Les normes](normes.md).

## L'échappement

Une barre oblique échappe le caractère qui suit : `\*`, `\@`, `\{`
restent du texte, sans aucune interprétation — ni syntaxique, ni
typographique. C'est la porte de sortie universelle : écrire `\;` en
français donne un point-virgule nu, sans l'espace insécable que la langue
y mettrait.

## Ce qui passe par le bloc brut

Le Typst à la main reste toujours possible — c'est la porte de sortie
pour ce que la couche ne couvre pas :

    ```typst
    #figure(
      table(columns: 2, [a], [b]),
      caption: [Un tableau],
    )
    ```

Ce qui s'y trouve va dans le document compilé, mot pour mot.
