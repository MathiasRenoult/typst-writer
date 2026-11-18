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
| `Tab` / `Maj+Tab` | enfoncer / remonter la ligne de liste sous le curseur |
| ```` ```typst ```` … ```` ``` ```` | un bloc Typst brut, inséré tel quel |
| trois lignes vides consécutives | un saut de page |
| `// texte` | un commentaire — conservé dans le `.tw`, ignoré au rendu |
| `{{bib}}` ou `{{bibliographie}}` | la bibliographie du projet, à cet endroit |
| `{{idx}}` ou `{{index-biblique}}` | l'index des citations bibliques, recalculé au rendu |
| `{{s}}` ou `{{sommaire}}` | la table des matières, à cet endroit |
| `{{garde}}` … `{{/garde}}` | une page de garde, avant le sommaire |

Une **ligne vide** sépare les blocs. Ce qui n'est pas reconnu reste du
texte — un fichier abîmé ne bloque jamais l'ouverture.

## Les chapitres

Une **tête de niveau 1** (`# Titre`) ouvre un chapitre : elle emporte tout
ce qui la suit jusqu'à la tête de niveau 1 suivante. Les têtes de niveau 2
et au-delà restent des sections du chapitre.

Le panneau de gauche porte une section **Chapitres**. **Document entier**
montre tout le rendu ; un chapitre choisi ne montre que ses pages, et le
**Sommaire** se réduit à ses têtes. Le retour au document entier se fait
par la même ligne. La source, elle, reste entière dans l'éditeur : choisir
un chapitre ne réécrit ni ne tronque jamais `document.tw`.

Les chapitres viennent du sommaire composé : leurs pages sont celles du
PDF réel, pas une estimation. Si le document n'a aucune tête de niveau 1,
la section n'apparaît pas — tout le document reste alors la seule vue.

Les balises de fin ont une forme canonique courte : `{{s}}`, `{{bib}}`,
`{{gs}}`, `{{idx}}`. Les noms longs (`{{sommaire}}`, `{{bibliographie}}`,
`{{glossaire-section}}`, `{{index-biblique}}`) et les anciennes formes
(`[[TOC]]`, `{bibliographie}`, `{glossaire}`, `{{index}}`) restent lus,
mais les nouveaux gestes écrivent la forme courte.

## Les pages de garde

Une page de garde se délimite par deux lignes :

```
{{garde}}
# Dédicace

À ceux qui cherchent.
{{/garde}}
```

Son contenu est composé **avant le sommaire**, sur une page **sans numéro**,
et il n'entre pas dans la table des matières — même si la page contient des
titres. Placez les blocs en tête par convention, même si leur contenu est
toujours composé avant le sommaire. On peut répéter le bloc pour plusieurs
pages : chacune commence sur sa propre page.
Le menu `..`, sous « Page de garde », écrit le bloc et place le curseur à
l'intérieur.

La page de titre des normes suit la même règle : elle est composée sans
numéro et n'ajoute aucune entrée au sommaire. Si les normes en composent
déjà une, elle précède les pages de garde ; désactivez-la pour la remplacer.

## Les métadonnées

Une métadonnée du projet s'appelle par `{{m clé}}` : `{{m auteur}}` affiche
la valeur d'`auteur` réglée dans les normes. Les métadonnées existantes sont
aussi proposées dans le menu `..` sous « Métadonnée · clé ».

## Les blocs canoniques

| Ce qu'on écrit | Ce que c'est |
| --- | --- |
| `{{e type=note titre="À retenir"\|texte}}` | un encadré et son contenu, sur une ligne |
| `{{gr type=barre titre="Ventes"}}` puis `Nord \| 30` | un graphique simple et ses données |
| `{{ch type=pie titre="Répartition"}}` puis le CSV | un graphique primaviz |
| `{{t}}` puis la grille Markdown | un tableau explicite |

Les paramètres gardent leurs noms longs (`type`, `titre`, `couleur`) ; seule
la balise est courte. Les panneaux de graphique et d'encadré restent
disponibles pour les réglages fins.

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
| `{{c red\|texte}}` | un passage teinté (forme canonique courte) |
| `{{d 1789.07.14}}` | une date composée par les normes |
| `{{d aujourd'hui}}` | la date du jour, au moment de la composition |
| `{{d maintenant}}` | la date et l'heure de la composition |
| `{{g clé}}` | un terme du glossaire |
| `{{m auteur}}` | une métadonnée du projet |

Toutes ces mises en forme sont aussi des commandes de la palette
(`Ctrl+K` ou `/`) : gras, italique, souligné, barré, code, formule, titres,
listes, citation, note et couleur. Les teintes orange, verte, rouge, bleue,
violette, grise et noire y sont directes, et « dernière couleur choisie »
réapplique la couleur courante.

`aujourd'hui` et `maintenant` sont évalués au moment où le document est
composé, pas à chaque frappe. Pour une composition reproductible — même
sortie à chaque essai —, la variable d'environnement `SOURCE_DATE_EPOCH`
fixe l'horloge.

La note reste elle aussi visible dans sa source : le panneau d'insertion écrit
`^[texte]`, et le rendu en fait une note de bas de page. Rien n'est remplacé
par un signe dans le fichier. Une note peut contenir une citation
(`^[Voir @dupont2023[p. 45].]`) : elle est composée dans la note, pas à la
place de la note.

## Les citations

`@cle` appelle une référence de la bibliographie ; `@cle[p. 45]` porte la
page. Plusieurs appels d'un geste — `@dupont[p. 45]; @martin` — ne font
jamais qu'**une** note de bas de page, avec les deux mentions. Une citation
peut être collée au mot qu'elle appuie (`famille@grudem…[p. 981]`) ou suivre
un guillemet fermant ; elle reste une citation Typst. Une adresse
électronique (`user@example.com`) reste du texte.

La citation reste **visible dans sa source** : l'éditeur ne la remplace
jamais par un glyphe numéroté. C'est un texte comme un autre — on le
déplace, on le sélectionne et on le corrige caractère par caractère, et
`@dupont2023` reste lisible dans le document. La numérotation et la note
sont produites au rendu, pas dans le fichier.

Le bouton citation de la barre supérieure ouvre le composeur à l'endroit du
curseur. Il sert aussi à réécrire une citation existante lorsque le curseur
s'y trouve. Tant que l'on n'a pas validé, le document n'est pas modifié ;
`Échap` ne laisse rien. Taper `@` ouvre le même composeur : un `@` isolé
n'est donc plus seulement du texte, et un `@` collé à un mot reste une
adresse électronique.

`Tab` après `@exemple` ouvre aussi le menu contextuel, posé près du curseur :
le document reste visible derrière lui, et la source n'est pas modifiée. Le
menu permet de choisir la référence, de saisir sa page (`p. 45`,
`pp. 12-15`…), puis d'ajouter une autre référence à la même citation. `Tab`
fait avancer d'une étape, `Entrée` insère, `Échap` abandonne sans rien
écrire ; un clic dans le texte ferme le menu et rend la main à l'éditeur.

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

Dans le panneau d'insertion, le livre se choisit dans une **liste
déroulante cherchable** : un seul champ porte la recherche et la sélection,
et seuls les livres réels peuvent être choisis. Le livre, le chapitre et
les versets laissés dans les champs sont pris en compte au moment d'insérer,
même sans cliquer sur **Ajouter** ; un nom de livre tapé sans sélection peut
aussi être résolu s'il désigne un livre sans ambiguïté.

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
- `|sanspoint` — l'abréviation sans point ;
- `|tob` — les abréviations de la TOB, « 1 S 1.1 », « Is 6.1 ».

Sans suffixe, c'est la forme réglée pour le projet qui s'applique. Le
panneau des plugins (onglet **Biblique**) choisit aussi les chiffres des
chapitres et des versets, les séparateurs et l'ordre canonique.

Les livres deutérocanoniques et apocryphes sont citables dans la même
syntaxe : Tobie, Judith, Sagesse, Siracide, Baruch, les Maccabées, les
Psaumes additionnels, Hénoch, les Jubilés, les Testaments des douze
patriarches et les principales apocalypses. Leurs références se composent
et s'indexent ; les deux traductions embarquées ne fournissent pas leur
texte, mais une version ajoutée peut le fournir (voir
[les versions bibliques](bibles.md)).

Une référence peut nommer sa **traduction** : `{{Jn 3.16|lsg}}` rend
`Jn 3.16 (LSG)`, `{{Jn 3.16|darby}}` rend `Jn 3.16 (DARBY)`. Une version
ajoutée s'appelle par son code, par exemple `{{Tob 1.1|vulgate}}`. La
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

Les indications de corpus — **Livres canoniques** et **Livres non
canoniques** — sont des titres de niveau 2, suivis d'un retour à la ligne.
Un titre de livre ne reste jamais seul en bas d'une colonne : il est
solidaire de sa première référence, qui le suit dans la même colonne.

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

`{{g cle}}` appelle un terme du glossaire : la forme se développe à la
première occurrence, les reprises gardent le sigle. Les variantes
`pluriel` et `majuscule` s'écrivent après la clé — `{{g sdo pluriel}}`,
`{{g sdo majuscule}}`. La forme historique `{glo:cle}` reste lue. La liste
finale se pose d'une ligne `{{gs}}`, ou `{{gs tout}}` pour y montrer aussi
les termes jamais cités ; `{glossaire}` et `{glossaire: tout}` restent lus.

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

## Les liens internes

Un titre, un sous-titre ou une figure reçoit une **ancre** stable. Un lien
interne s'écrit `[[référence|texte]]` :

    [[titre-introduction|voir l'introduction]]
    [[figure-planche|la planche]]

Taper `[[` ouvre la recherche des éléments linkables du document : la liste
se filtre à la frappe, `Tab` ou `Entrée` insère l'ancre choisie, et le
curseur attend le texte du lien. Les ancres sont fabriquées à partir du
titre ou de la légende ; deux éléments qui portent le même texte reçoivent
un suffixe (`…-2`, `…-3`). Une ancre ne contient que lettres, chiffres,
tirets, points, deux-points et soulignés. Si un titre est renommé, les liens
qui visaient son ancienne ancre ne trouvent plus leur cible : le panneau
gauche les signale sous **liens internes à vérifier**, et un clic sélectionne
le lien dans la source.

Quand le texte du lien est un mot clé **entre guillemets**, il est remplacé
au rendu par la numérotation réelle du document :

    [[titre-adoption-coutume|"chapitre"]]   → chapitre 2.3
    [[figure-planche|"figure"]]             → Figure 4
    [[titre-adoption-coutume|"page"]]       → page 7

`"figure"` s'applique à une figure numérotée, c'est-à-dire une image avec
légende ; pour une image sans légende, `"page"` reste disponible.

`"section"` et `"titre"` sont des synonymes de `"chapitre"` ; `"image"` est
un synonyme de `"figure"`. Le numéro du chapitre suit la numérotation des
titres des normes, le préfixe de la figure suit le libellé des légendes, et
la page suit la numérotation des pages. Le texte entre guillemets est
volontairement une convention : un lien ordinaire vers un mot « page » reste
possible sans guillemets.

Les ancres complètes s'écrivent `titre-adoption-coutume` ou
`figure-planche`. Quand la forme courte est unique dans le document,
`adoption-coutume` suffit et le lien est résolu vers la bonne ancre ; si un
titre et une figure partagent la même forme courte, l'ancre complète reste
nécessaire.

Ces nombres sont ceux que les normes composent réellement : si les titres
ne sont pas numérotés, un lien `"chapitre"` n'a pas de numéro à afficher.

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

Un graphique se déclare par une balise `{chart}` suivie de ses données en
CSV. La forme concise est :

    {chart bar "Évolution" 10cm×6cm centre}
    Jan,10
    Fév,20
    Mar,15

Le premier mot est le type : `bar` (par défaut), `pie` (secteurs), `line`
(lignes), `scatter` (nuage de points) ou `box` (boîtes à moustaches). Le
titre, entre guillemets, est facultatif. La taille `largeur×hauteur` et le
placement — `centre`, `gauche`, `droite` ou `pleine` — le sont aussi ;
`{chart pie "Répartition"}` suffit pour un graphique simple. `sans-legende`
et `sans-valeurs` désactivent les deux options quand le type les connaît.
La forme longue `type=… titre=… largeur=…` reste lue pour les documents
existants.

Les colonnes dépendent du type :

- `bar`, `pie`, `line` : `étiquette,valeur` ;
- `scatter` : `x,y` ;
- `box` : `étiquette,min,q1,médiane,q3,max`.

Les documents qui portent encore l'ancien bloc `{graphique}` restent lus : le
générateur les traduit vers primaviz, sans changer leur source.

Une ligne d'en-tête est reconnue et sautée. La virgule et le point-virgule
sont acceptés comme séparateurs, et les valeurs entre guillemets gardent
leurs virgules. Le bouton **Insérer un graphique** de la barre d'outils
ouvre le panneau complet : type, titre, taille, placement, légende, valeurs
et un vrai champ CSV où coller plusieurs lignes ou importer un fichier. Il
n'y a pas d'aperçu séparé : le document composé est la prévisualisation.

Les graphiques utilisent le paquet Typst
[primaviz](https://typst.app/universe/package/primaviz), vendu avec
l'application et installé localement — aucun téléchargement au rendu.

## Voir les erreurs de compilation

En écriture, les erreurs sont aussi listées dans le panneau gauche sous
**n erreur(s) de compilation** et soulignées dans le texte, à la ligne et à la
colonne signalées. Chaque ligne du panneau montre le message du compilateur ;
un clic place le curseur à l'endroit fautif. Le panneau de rendu garde sa
propre liste détaillée quand il est ouvert.

## Les callouts

Un callout encadre une citation sans la recopier. La balise se pose devant
le bloc de citation :

    {callout note "À retenir"}
    > Une distinction utile pour la suite.

Le premier mot après `callout` est le ton : `citation` (neutre), `note`,
`important`, `attention` ou `astuce`. Le titre, entre guillemets, est
facultatif ; sans lui, le cadre garde sa couleur mais n'a pas de bande de
titre. Les alias anglais `quote`, `warning` et `tip` sont aussi lus.

Le bouton **Insérer un callout** ouvre le panneau : ton, titre, puis
« Encadrer cette citation » si le curseur est déjà dans une citation ;
sinon « Insérer le callout » pose une citation vide et y place le curseur.
Le texte reste toujours dans le document — le panneau ne le recopie pas.

Les callouts utilisent le paquet Typst
[showybox](https://typst.app/universe/package/showybox), vendu avec
l'application et installé localement : le rendu ne dépend d'aucun
téléchargement. Cinq tons sobres sont fournis ; le type inconnu retombe sur
`note`, jamais sur une erreur de compilation.

## Lire l'aperçu

Le panneau de rendu est un lecteur du PDF exporté. Les pages sont centrées
dans la largeur disponible ; l'aperçu précharge une bande de la page voisine
pour qu'un défilement rapide ne montre pas de page blanche. Les erreurs de
compilation occupent toute la largeur du panneau, au-dessus du document, et
aucun message de compilation ne s'affiche dans la page.

Le texte peut y être sélectionné au glisser, au double-clic pour un mot et au
triple-clic pour une ligne ; `Ctrl+C` copie la sélection. `Maj+clic` étend la
sélection à partir de son point de départ.

Dans une vue de rendu, `Ctrl+F` cherche dans le **document composé** — titres,
notes et bibliographie compris — et non dans la source. La barre compte les
occurrences, `F3` et `Maj+F3` passent au résultat suivant ou précédent, et
les résultats sont surlignés dans la page. `Échap` ferme la recherche. Dans
l'éditeur, `Ctrl+F` garde son sens habituel.

Dans la barre de recherche de l'éditeur, le bouton **Document / Projet**
change la portée. En mode **Projet**, la recherche parcourt tous les fichiers
`.tw` et `.typ` du projet, affiche le chemin, la ligne et un extrait, et un
clic ouvre le fichier à l'occurrence. Le remplacement reste limité au document
courant : une correction globale n'écrit jamais dans un fichier qu'on n'a pas
sous les yeux.

La barre de recherche est un **outil**, pas une fenêtre modale : elle reste
ouverte sans empêcher de cliquer ou de faire défiler le texte. Les boutons
**Précédent** et **Suivant** amènent l'occurrence dans la fenêtre de
l'éditeur, pas seulement dans la mémoire du curseur. En vue de rendu,
`Ctrl+F` ouvre la recherche du PDF ; en vue d'édition, celle du document —
jamais les deux à la fois.

Le bouton des **numéros de ligne** de la barre d'outils les affiche ou les
masque dans l'éditeur. C'est une préférence de lecture, conservée d'une
session à l'autre : elle ne change rien au document composé.

## Revenir au même endroit

Changer de mode ne renvoie plus au début. L'édition retrouve son défilement
exact, le rendu sa position exacte. À la première ouverture du rendu, la page
choisie est celle du sommaire qui couvre le haut de l'édition : c'est la
correspondance que Typst donne entre une ligne et une page. Les deux ancres
vivent dans la vue, pas dans le fichier.

Les liens du document sont actifs : un lien interne amène à la page visée,
un lien externe ouvre l'adresse avec l'outil du système. Un pied de page
discret indique la page courante et le nombre total de pages du document.

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
