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
graphiques, callouts — se règlent dans le panneau des plugins, mais ils
appartiennent au **même fichier de normes** que la mise en page :
`mise-en-page.toml` les emporte avec lui. Quand un document est ouvert,
l'interface, l'éditeur et les plugins prennent donc sa mise en page et son
paramétrage de plugins.

Le nom de ce fichier est affiché dans la **barre d'état**, sous le nom du
document : `Norme : mise-en-page.toml`. Quand les réglages appliqués
correspondent exactement à une norme de la bibliothèque, son nom suit :
`Norme : mise-en-page.toml · Thèse`. Une norme personnalisée n'affiche que
le fichier ; une norme absente est dite `(absente)`.

Les deux panneaux ont un bouton de bascule : **Plugins** dans les normes,
**Normes** dans les plugins. La copie de travail reste la même pendant la
bascule — une valeur saisie n'est pas rechargée depuis le disque.

## Fichiers de normes

La fenêtre des normes porte l'onglet **Fichiers de normes** (bouton
« Fichiers… »). Il gère la bibliothèque interne :

- **Charger** met le fichier dans la fenêtre, sans toucher au document ;
- **Appliquer au document** écrit son `mise-en-page.toml`, son gabarit
  Typst et met à jour les plugins ;
- **Importer un fichier…** ajoute un `.toml` à la bibliothèque ; le nom du
  fichier devient le nom de la norme, suffixé si ce nom est déjà pris ;
- **Exporter** écrit une copie `.toml` sans jamais écraser un fichier
  existant ;
- **Supprimer** retire l'entrée de la bibliothèque ; le geste demande son
  mot, car un fichier de normes ne se reconstitue pas à l'identique.

Une norme importée reste une **norme interne** : elle est proposée à la
création d'un projet comme les normes livrées. Le nom de la copie et le
bouton d'enregistrement créent une nouvelle entrée à partir des réglages
courants.

Les champs numériques se règlent à la **molette** ou aux **flèches** ▲▼
— la valeur garde son unité : `11pt` devient `12pt`, `2cm` devient `3cm`.

## Mise en page

Le papier (format, orientation), les marges, les numéros de page, le
texte courant (police, taille, interligne, retrait, justification,
langue), les titres (police, tailles, gras, casse, numérotation) et les
couleurs.

Les choix finis sont des **listes déroulantes** ; chaque paramètre porte
son **infobulle** — laisser la souris dit ce qu'il règle et sous quelle
forme l'écrire. Toute mesure Typst est acceptée : `2cm`, `1in`, `24pt`.

Deux champs donnent une **police prioritaire par script** : « Hébreu » et
« Grec ». Ils ne s'appliquent qu'aux caractères de ce script — le latin
garde la police du texte. Par exemple `Ezra SIL SR` pour l'hébreu et
`Galatia SIL` pour le grec. Laisser un champ vide conserve la police de
base et son repli habituel ; le document existant ne change donc pas tant
que l'auteur n'a rien choisi.

Les **blocs de citation** (`>`) ont leur propre section : police, taille,
couleur, retrait à gauche et italique. Chaque champ vide laisse le défaut
de Typst ; le décalage s'écrit comme une mesure (`1em`, `0.8cm`).

Les tableaux titrés suivent le style des légendes de figures — gras ou
italique — et ont leur propre numérotation, indépendante des figures. Le
titre du tableau s'écrit `{{t "Titre"}}` au-dessus de la grille.

## Typographie

La composition du texte réglée **par langue** : un défaut — `française`,
`anglaise`, `aucune` — puis les réglages fins, un par un.

En français : une espace insécable devant `; : ! ?` et les guillemets,
normale après ; en anglais : la ponctuation colle. En toute langue, la
virgule et le point ne s'ouvrent jamais — `3.14` tient. Les points de
suspension ne reçoivent jamais d'espace après eux : `mot…suite`. Les guillemets se
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

Le panneau des plugins est une seconde vue de la même fenêtre, et ses
valeurs font partie du **fichier de normes** du document. Il s'ouvre par
le bouton **Plugins** du panneau des normes, ou par le bouton **Normes**
pour revenir. Le rail des onglets défile et revient à la ligne : un onglet
ajouté ne peut plus être rogné par le bas. Il porte six onglets :

- **Bibliographie** — le style des citations et de la liste finale,
  l'import d'un fichier `.csl` et les styles installés dans Zotero ;
- **Biblique** — la traduction consultée (LSG, Darby ou une version ajoutée
  depuis [les versions bibliques](bibles.md)), l'ordre de la
  liste (apparition ou canonique), la forme des livres (abréviation, nom
  complet, OSIS, sans point ou TOB), les chiffres arabes ou romains, les
  séparateurs, le gabarit de citation, l'ordre canonique et l'index des
  citations (pages, regroupement, canon, forme, colonnes). Le canon choisi
  sépare les livres canoniques des livres non canoniques ; « Tout le corpus »
  garde une seule liste. Un index entièrement canonique n'affiche pas le
  titre « Livres canoniques », devenu implicite.
- **Biblique — les livres hors canon** : Tobie, Judith, Sagesse, Siracide,
  Baruch, les Maccabées, les Esdras additionnels, Hénoch, les Jubilés, les
  Testaments des douze patriarches et les principales apocalypses sont
  proposés dans le panneau d'insertion. Leurs références se composent et
  s'indexent, sans texte de traduction embarqué ;
- **Glossaire** — un renvoi vers le panneau du glossaire, où se gèrent les
  termes ;
- **Graphiques** — un renvoi vers la fenêtre d'insertion, qui porte les
  données ;
- **Personnes** — le registre des personnes citées, les formes du nom
  (complet, simple, alias) utilisées par les balises sans expression et par
  `auto`, à la première mention et aux suivantes, la forme de l'index, la forme des dates
  (naissance et mort, naissance, mort ou texte libre), leur affichage à la
  première mention seulement, les pages, les initiales, la qualité et les
  colonnes. Le bouton ouvre la fenêtre du registre et le panneau d'insertion
  est accessible par le raccourci de la balise.

Quand un réglage de plugin a été modifié et que le document porte une
norme de la bibliothèque, le pied du panneau propose **Écraser « … »** :
la bibliothèque reçoit la nouvelle version, sans quitter les plugins.
**Appliquer** reste le geste qui écrit `mise-en-page.toml` dans le
document.

Une balise biblique peut choisir sa propre forme avec `|long`, `|court`,
`|osis` ou `|sanspoint` ; elle gagne sur le réglage du projet. Une
référence peut aussi nommer sa traduction avec `|lsg`, `|darby` ou le code
d'une version ajoutée (par exemple `|vulgate`) ; elle est alors affichée dans
la citation.

## Les titres par niveau

Les tailles se règlent par niveau. La graisse, l'italique, le souligné,
l'espacement et la couleur se règlent aussi par niveau ; le réglage global
reste le défaut. La casse présentée dans la fenêtre est un réglage global.

Voir [Écrire](ecrire.md).
