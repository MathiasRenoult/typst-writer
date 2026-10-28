# Langage intermédiaire — conception

> **Statut : syntaxe gelée ; pipeline `.tw` raccordé au projet, au rendu et à l'export.**
> Ce document est la référence de `document.tw`, l'AST et les deux traductions
> qui l'encadrent. Le moteur Typst existant reste le seul moteur de composition
> et de rendu.

## Rôle

L'éditeur travaille sur un **Document structuré** : une suite de blocs dont le
texte est une suite d'éléments inline. Sur disque, ce document est représenté
par un fichier texte lisible et diffable, écrit dans notre syntaxe.

```text
`document.tw` ──parse──▶ Document ──génère──▶ corps Typst
                                             │
`main.typ` (normes) ──assemble avec le corps ─┴─compile──▶ rendu / PDF
```

- **Parse** : la syntaxe de l'utilisateur devient l'AST.
- **Sérialise** : l'AST devient sa forme canonique en syntaxe utilisateur.
- **Génère** : l'AST devient le corps d'un chapitre Typst.
- **Assemble** : `main.typ` garde les normes et inclut le corps généré en
  mémoire avant `#bibliography`. Les anciens assemblages sans directive récupèrent
  `#bibliography("references.bib")` en mémoire si le fichier `.bib` existe. Le
  fichier intermédiaire généré n'est jamais la source de vérité et n'est jamais
  écrit sur disque.

Le même `crate::typst::render` compile l'assemblage pour l'aperçu et l'export
PDF. Il n'y a pas de second moteur de rendu.

## Invariants

1. Le Document structuré est la source de vérité ; le texte est sa représentation
   lisible.
2. La syntaxe canonique est déterministe :
   `parse(serialiser(document)) == document`.
3. Les lignes vides séparent les blocs et sont ignorées par le parseur. Une
   ligne vide n'est jamais un bloc.
4. Le même Document génère toujours le même corps Typst.
5. Ce qui n'est pas reconnu lors de la migration reste un bloc Typst brut, sans
   réécriture approximative.
6. `main.typ`, les chapitres sources et la bibliographie existants ne sont pas
   détruits par une migration.

## Syntaxe gelée de `document.tw`

Elle ne l'est pas pour toujours : chaque ajout suit la procédure de
`extension-langage.md` — les blocs et les propriétés y ont leur route,
leur recette et leurs preuves.

### Blocs

| Bloc | Syntaxe | Typst généré |
|---|---|---|
| Titre de niveau 1–6 | `# Titre`, `## Titre`, … | `= Titre`, `== Titre`, … |
| Paragraphe | texte libre ; ligne vide entre les blocs | paragraphe Typst |
| Citation de bloc | `> texte` | `#quote(block: true)[texte]` |
| Liste à puces | `- élément` | liste Typst à puces |
| Liste numérotée | `1. élément`, `2. élément` | liste Typst numérotée |
| Bloc Typst brut | bloc clôturé par ` ```typst` et ` ``` ` | contenu inséré tel quel |

Les lignes d'un paragraphe qui se suivent sont jointes par une espace. Les
espaces de fin de ligne et les lignes blanches supplémentaires sont normalisés
par la sérialisation. Si le contenu d'un bloc brut contient déjà un fence, le
sérialiseur allonge son délimiteur pour préserver le round-trip.

L'éditeur `.tw` utilise la grammaire de coloration Markdown (pas Typst). Les
crochets, parenthèses et accolades se ferment automatiquement, sans doubler
l'apostrophe française. Entrée poursuit les listes à puces/numérotées et les
citations de bloc ; Entrée sur un marqueur vide quitte la structure.

### Inline

| Inline | Syntaxe | Typst généré |
|---|---|---|
| Gras | `**texte**` | `*texte*` |
| Italique | `*texte*` ou `_texte_` | `_texte_` |
| Gras + italique | `***texte***` | `*_texte_*` |
| Code | `` `texte` `` | ` `texte` `` |
| Littéral | `\x` | le caractère nu, sans interprétation |
| Formule en ligne | `$x^2$` | `$x^2$` — math Typst |
| Couleur | `{couleur:texte}` | `#text(fill: couleur)[texte]` |
| Citation bibliographique | `@clé`, `@clé[p. 45]`, `@clé; @autre` | une note, jamais deux |
| Lien | `[texte](cible)` | `#link("cible")[texte]` |
| Note de bas de page | `^[texte]` | `#footnote[texte]` |

La couleur est une valeur Typst (par exemple `red`, `#ff0000` ou une expression
`rgb(...)`). La clé bibliographique est une suite de lettres, chiffres, tirets
et soulignements. Le style par défaut du projet est `chicago-notes` : Typst rend
les citations bibliographiques en notes de bas de page, puis imprime la
bibliographie. Un style défini explicitement dans `main.typ` reste prioritaire.

### Échappement

La barre oblique inverse échappe le caractère qui suit : `\*`, `\@` et `\{`
restent du texte littéral. Elle permet également à un paragraphe de commencer
par un marqueur de bloc sans devenir un titre, une citation ou une liste.

Le sérialiseur échappe les marqueurs inline et les débuts de bloc. Le texte nu
est lui aussi échappé pour la syntaxe Typst : une étoile, un `#` ou un crochet
littéral ne devient pas une commande du document compilé.

**L'escapement est uniforme** : les dix marqueurs —
`\ * ` @ [ { } ] ^ _` — sont échappés partout où du texte est littéral, y
compris les mentions de page et les cibles de lien. Une région littérale
s'écrit comme du texte nu, sinon ses caractères feraient signe aux balayages
qui l'entourent : un `*` de page refermerait l'italique du voisinage. La
lecture déséchappe génériquement.

**Un échappement fait un `Litteral`** : le caractère qui suit ne reçoit
**aucune interprétation** — ni syntaxique, ni **typographique**. C'est la
porte de sortie universelle : `\;` en français donne un point-virgule nu,
sans l'espace insécable que la langue y mettrait. Les échappements qui se
suivent forment un seul littéral, et le sérialiseur leur redonne leur barre.

### Typographie selon la langue

Les normes du document portent une **typographie** — `aucune`, `française`,
`anglaise` — que le générateur applique au texte composé :

- **française** : une espace insécable (`~` en Typst) devant `;`, `:`, `!`,
  `?` et les guillemets `« »`, une espace normale après ;
- **anglaise** : la ponctuation colle — aucune espace devant `;`, `:`, `!`,
  `?`, une espace normale après ;
- en toute langue, la virgule et le point ne s'ouvrent jamais — `3.14`
  tient — et les suites d'espaces n'en font qu'un.

Le code inline, les blocs bruts et les **littéraux** ne reçoivent aucune
règle.

### Grammaire formelle

**Priorité de lecture.** Le parseur inline lit dans cet ordre, à chaque
position : échappement `\x`, gras-italique `***`, gras `**`, italique `*`
ou `_`, code `` ` ``, note `^[`, appel `@`, lien `[`, couleur `{`. Le premier
marqueur reconnu l'emporte ; ce qui n'en est pas un s'accumule en texte nu,
et les textes adjacents n'en font qu'un.

**Imbrication.** Les conteneurs se lisent par descente récursive — un gras
peut contenir une note, un lien, une couleur, un appel —, à une réserve près :
l'emphase est **plate**. Aucune emphase ne descend dans une emphase, à aucun
niveau — le balayage d'une emphase est plat, une étoile de la profondeur la
refermerait avant l'heure. Le gras-italique se dit en `GrasItalique`
(`***…***`), jamais en deux.

**Délimiteurs incomplets.** Un marqueur jamais fermé n'en est pas un : `*x`
reste du texte, `^[note` reste du texte, `@` sans clé reste un `@` nu, un
fence jamais clôturé va jusqu'à la fin du document. Le parseur ne renvoie
jamais d'erreur : ce qui ne se lit pas reste du texte, et un fichier abîmé ne
doit pas empêcher d'ouvrir le projet.

**Bandes opaques.** Un code inline est une bande opaque : ce qui s'y trouve
ne ferme ni une emphase, ni une note, ni une couleur. Le balayage des
délimiteurs saute les codes d'un bond — un `` `]` `` dans une note ne ferme
rien du tout.

**Listes.** Une ligne de liste est un élément : il n'y a pas de suite de
ligne, et une ligne qui n'est plus un marqueur referme la liste et devient
paragraphe. Les listes numérotées se réécrivent à la suite — `1.`, `2.`, …
— quel que soit le numéro lu.

**Forme canonique.** Le round-trip `parse(serialiser(AST)) == AST` est garanti
pour tout AST **canonique**, et pour lui seul — c'est `Document::est_canonique`
(`src/langage/schema.rs`) qui les énonce et les vérifie, règle par règle. Les
dix règles, en résumé :

1. un texte nu n'est jamais vide, ne contient pas de saut de ligne **ni de
   marqueur** — les marqueurs vivent dans des littéraux échappés, et un
   littéral est non vide ;
2. une emphase est plate, non vide, sans blanc aux bords ;
3. un code ne contient ni backtick ni saut de ligne ;
4. le nom d'une couleur est simple : ni deux-points, ni marqueur ;
5. une référence a une clé alphanumérique (tirets et underscores admis), et
   sa page, si elle est, est non vide et déjà rognée ;
6. un paragraphe, une citation de bloc et une liste ne sont pas vides ; un
   titre est de niveau 1 à 6 ;
7. deux textes nus ne sont jamais adjacents — le parseur les rassemble ;
8. un lien ne suit jamais immédiatement un appel sans page — son crochet
   ouvrant serait lu comme sa page ;
9. un texte ne suit jamais immédiatement un appel sans page par un caractère
   de clé — il grossirait la clé ;
10. le `; ` d'un appel groupé est réservé : un texte « ; » ne sépare jamais
    deux appels ;
11. le premier élément d'un paragraphe n'est pas un `Texte` qui commence par
    un marqueur de bloc : la barre de protection qui l'écrit se relirait
    comme un littéral. Le caractère se prend donc en `Litteral`.

Ces règles ne sont pas des manques : elles décrivent ce que la syntaxe peut
dire sans ambiguïté. Tout autre AST est toléré à la lecture, mais repris à
l'écriture — et la réécriture est idempotente : rouvrir puis réécrire ne
reformate jamais deux fois.

La propriété est éprouvée, non démontrée : un générateur produit cinq cents
documents canoniques au hasard, que `parse` et `serialiser` doivent
reconduire à l'identique (`src/langage/serialiser.rs`, tests
`le_round_trip_tient_sur_tout_ast_canonique` et
`la_canonicalisation_est_idempotente`).

## Schéma du Document

`src/langage/schema.rs` définit :

- `Document { blocs }` ;
- les blocs `Titre`, `Paragraphe`, `Citation`, `ListePuce`, `ListeNumero` et
  `Brut` ;
- les inline `Texte`, `Gras`, `Italique`, `GrasItalique`, `Code`, `Couleur`,
  `Citation`, `Note` et `Lien`.

Le texte nu du document est calculé depuis l'AST. Le compte de mots ignore le
balisage, les espaces et les signes seuls.

## Fichiers d'un projet

```text
~/.typst-writer/projets/<nom>/
  main.typ              normes et en-tête Typst
  document.tw           document structuré, source de vérité
  references.bib        bibliographie du projet
  chapitres/*.typ       sources héritées, conservées après migration
```

`document.generated.typ` est un nom virtuel utilisé pour l'include en mémoire ;
il n'est pas un fichier utilisateur.

## Migration des projets Typst

À l'ouverture d'un ancien projet sans `document.tw`, les chapitres qui figurent
dans `main.typ` sont lus dans l'ordre de l'assemblage et copiés sous forme d'un
seul document structuré. Les titres, paragraphes, emphases simples et listes
sont convertis. Un chapitre qui contient des commandes, des étiquettes, des
formules ou une syntaxe non reconnue devient un bloc brut Typst. Les chapitres
originaux et `main.typ` sont conservés ; le rendu du document intermédiaire
écarte leurs anciens includes pour ne pas imprimer les deux versions.

## Décisions actées

- Langage maison, surface Markdown-like, AST comme vérité.
- Extension : **`.tw`**.
- Fidélité visée : Standard — titres, paragraphes, listes, gras, italique,
  couleurs, citations, liens, notes et blocs Typst bruts.
- Inspiration neo : écriture dans une page proche du rendu, balisage invisible,
  plan de navigation, un fichier d'écriture par projet et bibliothèque en
  étagères. Le moteur de rendu Paged.js/HTML de neo n'est pas repris.
- Le moteur Typst réel reste le maître du rendu final.
- Le serveur tinymist ne sert pas à éditer `document.tw` ; l'analyse, les
  commandes de formatage et les références seront prises en charge par notre
  AST. Tinymist reste disponible pour les documents Typst hérités et leurs
  diagnostics.

## Progression

- **Jalon 0** : parseur, sérialiseur, générateur, migration prudente des sources
  Typst, et intégration de `document.tw` à l'ouverture, la sauvegarde, l'aperçu
  et l'export PDF. Le Typst généré reste un override mémoire.
- **Jalon 1 (partiel)** : `.tw` est coloré en Markdown, Entrée poursuit les
  listes/citations, l'éditeur maintient un AST synchronisé, fournit des actions
  de mise en forme et complète localement les citations `@clé`. Le remplacement
  du widget texte par un éditeur de blocs AST/WYSIWYG reste à faire.
- **Jalon 2** : `document.tw` est le document actif unique ; les chapitres hérités
  restent conservés en archives. Le plan est indenté et navigue vers les lignes
  source comme vers les pages ; le compteur suit le document unique.
- **Jalon 3** : ajustement largeur/hauteur et zoom libre par boutons ; la densité
  de rasterisation suit le rapport affiché, les tâches d'une ancienne échelle
  sont invalidées et le cache d'images reste borné.
