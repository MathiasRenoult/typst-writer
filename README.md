# Typst Writer

**Le studio d'écriture des documents qui comptent.**

Typst Writer est un éditeur pour écrire de longs textes — une thèse, un
mémoire, un livre, un article — sans jamais perdre de vue la page finale.
Le texte reste simple et lisible ; la mise en page est composée par Typst,
le moteur qui produit aussi le PDF.

Version 0.6.3 · Linux.

## L'idée

Un document de recherche n'est pas une suite de paragraphes : c'est un
assemblage de sections, de citations, de notes, de références bibliques,
d'index et de règles de présentation. Typst Writer réunit ces gestes dans
un seul espace, sans imposer un format fermé.

La source du document est un fichier texte, `document.tw`, que l'on peut
lire, comparer, sauvegarder et rouvrir dans dix ans. L'aperçu est le
document réellement composé, pas une approximation.

## Ce qui rend l'application différente

### Le texte reste à vous

Le document n'est pas enfermé dans une base de données ni dans un format
propriétaire. Il vit dans un fichier lisible, proche de Markdown, que vous
pouvez ouvrir avec n'importe quel éditeur. Les projets, les bibliographies
et les normes restent sur votre machine ; aucun compte, aucun serveur,
aucun envoi de vos travaux.

### On écrit déjà dans la page

L'aperçu n'est pas une image à côté du texte : c'est le rendu Typst du
document, avec ses marges, ses polices, ses notes et sa pagination. On
peut écrire, regarder la page se recomposer et vérifier immédiatement une
citation, un titre ou un tableau.

Le lecteur de PDF intégré permet aussi de sélectionner et copier le texte
composé, de chercher une expression dans le document final et de suivre
les liens internes et externes. Un simple clic ne sélectionne rien : il
faut glisser pour sélectionner un passage. Un lien interne peut afficher la
numérotation réelle du document — `[[titre-adoption-coutume|"chapitre"]]`
donne « chapitre 2.3 », `[[figure-planche|"figure"]]` donne « Figure 4 »,
et `"page"` donne la page composée.

### Un vrai espace de travail pour un long document

L'éditeur se divise en panneaux indépendants, avec des onglets. Le même
document peut être ouvert en écriture dans un panneau et en lecture dans
un autre ; la disposition est retrouvée au lancement. La bibliothèque de
projets se parcourt comme une étagère, avec des couvertures et des
vignettes de page.

Le document unique, son sommaire, ses références et les projets se
déplacent ensemble : on garde le fil sans changer de fichier. Le panneau
gauche montre le **Sommaire** complet ; les titres structurent le texte,
ils ne créent pas de chapitres séparés.

Une **page de garde** se délimite par `{{garde}}` … `{{/garde}}` : elle est
composée avant le sommaire, sans numéro, et n'entre pas dans la table des
matières. On peut en mettre plusieurs ; le menu `,,` insère le bloc.

### Les citations et les sources, sans friction

Taper `@` ouvre le composeur de citation : on choisit une référence, une
page, puis une autre référence si nécessaire. La note reste visible dans
le texte et prend sa forme finale au rendu. Les références viennent d'une
bibliographie locale, d'un fichier `.bib` ou d'une collection Zotero lue
en lecture seule ; les styles de citation de Zotero sont reconnus.

Chaque source peut être corrigée dans une fiche, et la bibliographie
finale se place où vous le décidez.

### La Bible traitée comme une source savante

Une référence biblique s'écrit simplement `{{Jn 3.16}}`. Le panneau
d'insertion propose un livre, un chapitre, des versets et un texte libre,
puis compose **une seule** citation avec plusieurs références.

Les formes de livres sont réglables — nom complet, abréviation courante ou
abréviations de la TOB — et les livres deutérocanoniques comme les
apocryphes (Tobie, Judith, Sagesse, Hénoch, Jubilés, Testaments des
patriarches, apocalypses) sont citables et indexables. Un clic sur une
référence montre le verset dans une bulle ; l'index biblique donne les
pages réelles du document, et les statistiques disent quels livres ont été
cités.

Une version biblique supplémentaire s'ajoute localement, à partir d'un
fichier JSON dont vous détenez les droits : elle peut contenir autant de
livres que son édition, dans son ordre propre, et se choisit comme les deux
traductions embarquées. Le panneau des versions les liste avec leur nombre
de livres et de chapitres, leur provenance, un accès au dossier, une copie
du JSON pour le transport et une suppression réversible. Un convertisseur
USFM/JSONL accompagne le format. Voir
[les versions bibliques](docs/guide/bibles.md).

### Une mise en page qui suit le document

Les mesures ne sont pas écrites au milieu du texte. Elles vivent dans les
normes du document : papier, marges, en-tête et pied de page, typographie,
titres, couleurs, numérotation. Un projet peut ainsi changer de mise en
page sans être réécrit.

Les encadrés, les tableaux, les graphiques, les dates, les couleurs et les
métadonnées sont insérés depuis une même palette, avec des formulaires
simples qui montrent toujours le texte qui sera écrit.

Les tableaux peuvent porter un **titre**, numéroté comme une figure et
référençable par `[[tableau-…]]` ou `[[tableau-…|"tableau"]]`. Leur légende
suit le style des figures, avec un compteur de tableaux indépendant. Sans
titre, ils gardent une ancre de rang ; un lien peut demander le titre avec
`show-title=true`. La section bibliographique n'apparaît que si `{{bib}}`
est écrit dans le texte — jamais par une directive cachée de `main.typ`.

Le panneau de travail sépare les mots du corps des notes de bas de page et
affiche la part des citations de bloc. Les blocs de citation (`>`) ont leur
police, leur taille, leur couleur et leur décalage dans les normes. Deux
champs donnent aussi une police prioritaire au script hébreu ou grec — par
exemple Ezra SIL SR et Galatia SIL — sans toucher au texte latin.

### Une interface qui parle la même langue que vous

La palette de commandes s'ouvre avec `Ctrl+K` ou `/` ; `,,` propose les
éléments à insérer. Elle donne aussi accès aux formats du texte (gras,
italique, souligné, barré, code, formules, titres), aux teintes orange et
verte ou à la dernière couleur choisie, à tous les imports et exports, aux
thèmes, aux normes de la bibliothèque et aux bascules de plugins. Une
recherche de livre biblique filtre la liste au fil de la frappe, les
commandes inutiles disparaissent quand leur module est désactivé, et le
curseur revient naturellement dans le document après une insertion.

La page **Paramètres** (`Ctrl+,`) règle l'application elle-même : les boutons
de mise en page peuvent disparaître pour qui écrit déjà le markdown, les
numéros de ligne s'affichent à la demande, et les thèmes de l'interface s'y
choisissent. Les normes du document restent indépendantes.

Les raccourcis sont réunis dans une carte (`F1`), et l'aide indique aussi
où vivent vos données et comment les sauvegarder.

## À qui s'adresse l'application

Aux personnes qui écrivent longtemps : théologiens, chercheurs, étudiants,
auteurs de mémoires et de livres. Plus généralement, à celles et ceux qui
veulent garder la main sur leur texte tout en obtenant une mise en page
soignée.

## Démarrer

La version 0.6.3 est publiée **sans paquet binaire ni artefact de test**.
Le code source est la référence ; une chaîne Rust à jour suffit :

```sh
git clone https://github.com/MathiasRenoult/typst-writer.git
cd typst-writer
cargo build --release
./target/release/typst-writer
```

Les paquets Linux des versions précédentes restent disponibles dans leurs
versions GitHub. L'application demande Linux avec Wayland ou X11 et un pilote
graphique récent.

L'application demande Linux avec Wayland ou X11 et un pilote graphique
récent. L'assistance à la frappe est facultative : écrire, composer et
exporter fonctionnent sans elle.

## Premiers pas

1. Créez un projet depuis la bibliothèque.
2. Écrivez dans le document.
3. Ouvrez l'aperçu pour voir la page composée.
4. Ajoutez vos sources depuis la bibliographie ou tapez `@` dans le texte.
5. Exportez en PDF, en Markdown ou en DOCX quand le document est prêt.

La [prise en main](docs/guide/prise-en-main.md) détaille chaque geste.

## Un aperçu de la syntaxe

La source reste volontairement simple :

```text
# Un titre

Un paragraphe avec du **gras**, une citation @dupont[p. 45] et une note.^[Une précision.]

{{Jn 3.16 ; "cf. aussi" ; Gn 1.1}}

{{e type=note titre="À retenir"|Une idée mise en valeur.}}
```

Les références bibliques, les citations, les notes et les blocs de mise en
page sont décrits dans le [guide d'écriture](docs/guide/ecrire.md). Le
plugin **Personnes citées** ajoute `{{p clé}}` et `{{index-personnes}}` :
les personnes sont déclarées dans `personnes.toml`, jamais devinées dans la
prose. La balise ouvre un panneau qui filtre le registre et remplit la fiche ;
sans expression, elle suit les formes réglées dans les normes (nom simple par défaut). Voir
[Les personnes citées](docs/guide/personnes.md).

## Ce que l'application sait produire

- un PDF composé par Typst ;
- un Markdown lisible ;
- un DOCX pour les personnes qui travaillent encore avec Word ;
- un projet portable ou un fichier `.tw` unique à transmettre.

Chaque export est accompagné de ce qu'il sait préserver. Les notes, les
citations et les références bibliques sont expliquées dans la
documentation.

## Limites actuelles

L'application est encore jeune, et certaines limites sont assumées :

- l'export DOCX passe par LibreOffice et rend les notes comme du texte ;
- l'import DOCX n'est pas disponible ;
- les diagnostics du compilateur sont soulignés dans le texte et listés dans
  le panneau gauche, sans correction automatique proposée ;
- Zotero est consulté en lecture seule ;
- les versions bibliques ajoutées ne sont jointes à l'archive que sur demande
  explicite, avec un avertissement sur les droits ; sinon, exportez leur JSON
  depuis le panneau des versions ;
- l'historique d'annulation peut repartir après certains remplacements
  de symboles.

Les sauvegardes automatiques et les copies précédentes sont là pour les
accidents ; elles ne remplacent pas une sauvegarde personnelle des travaux
importants. L'écran **Restaurer une version** (Aide, `F1`) les montre avec
leur date, leur taille, leur empreinte et un aperçu, et restaure la copie
choisie sans perdre le texte courant.

## Documentation

Le manuel d'usage est dans [`docs/guide/`](docs/guide/index.md) : prise en
main, écriture, bibliographie, glossaire, personnes citées, normes,
recherche, raccourcis, fichiers et installation. Les notes de version sont dans
[`CHANGELOG.md`](CHANGELOG.md) et les temps forts de cette version dans
[`RELEASE_NOTES.md`](RELEASE_NOTES.md). Les commandes de vérification et la
classification des épreuves ignorées sont dans
[`docs/verification.md`](docs/verification.md).

## Licence

MIT — voir [`LICENSE`](LICENSE).
