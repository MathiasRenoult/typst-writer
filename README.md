# Typst Writer

**Le studio d'écriture des documents qui comptent.**

Typst Writer est un éditeur pour écrire de longs textes — une thèse, un
mémoire, un livre, un article — sans jamais perdre de vue la page finale.
Le texte reste simple et lisible ; la mise en page est composée par Typst,
le moteur qui produit aussi le PDF.

Version 0.6.0 · Linux.

## L'idée

Un document de recherche n'est pas une suite de paragraphes : c'est un
assemblage de chapitres, de citations, de notes, de références bibliques,
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
les liens internes et externes.

### Un vrai espace de travail pour un long document

L'éditeur se divise en panneaux indépendants, avec des onglets. Le même
chapitre peut être ouvert en écriture dans un panneau et en lecture dans
un autre ; la disposition est retrouvée au lancement. La bibliothèque de
projets se parcourt comme une étagère, avec des couvertures et des
vignettes de page.

Les chapitres, le sommaire, les références et les projets se déplacent
ensemble : on change de chapitre sans perdre le fil.

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

### Une mise en page qui suit le document

Les mesures ne sont pas écrites au milieu du texte. Elles vivent dans les
normes du document : papier, marges, en-tête et pied de page, typographie,
titres, couleurs, numérotation. Un projet peut ainsi changer de mise en
page sans être réécrit.

Les encadrés, les tableaux, les graphiques, les dates, les couleurs et les
métadonnées sont insérés depuis une même palette, avec des formulaires
simples qui montrent toujours le texte qui sera écrit.

### Une interface qui parle la même langue que vous

La palette de commandes s'ouvre avec `Ctrl+K` ou `/` ; `..` propose les
éléments à insérer. Une recherche de livre biblique filtre la liste au fil
de la frappe, les commandes inutiles disparaissent quand leur module est
désactivé, et le curseur revient naturellement dans le document après une
insertion.

Les raccourcis sont réunis dans une carte (`F1`), et l'aide indique aussi
où vivent vos données et comment les sauvegarder.

## À qui s'adresse l'application

Aux personnes qui écrivent longtemps : théologiens, chercheurs, étudiants,
auteurs de mémoires et de livres. Plus généralement, à celles et ceux qui
veulent garder la main sur leur texte tout en obtenant une mise en page
soignée.

## Démarrer

Le paquet Linux s'installe sans `sudo` :

```sh
tar -xzf typst-writer-0.6.0-linux-x86_64.tar.gz
cd typst-writer-0.6.0-linux-x86_64
./install.sh
```

L'installateur vérifie chaque fichier par son empreinte. La désinstallation
se fait avec `./desinstaller.sh` et ne touche jamais aux documents.

Pour compiler l'application vous-même, une chaîne Rust à jour suffit :

```sh
cargo build --release
./target/release/typst-writer
```

L'application demande Linux avec Wayland ou X11 et un pilote graphique
récent. L'assistance à la frappe est facultative : écrire, composer et
exporter fonctionnent sans elle.

## Premiers pas

1. Créez un projet depuis la bibliothèque.
2. Écrivez dans le chapitre ouvert.
3. Ouvrez l'aperçu pour voir la page composée.
4. Ajoutez vos sources depuis la bibliographie ou tapez `@` dans le texte.
5. Exportez en PDF, en Markdown ou en DOCX quand le document est prêt.

La [prise en main](docs/guide/prise-en-main.md) détaille chaque geste.

## Un aperçu de la syntaxe

La source reste volontairement simple :

```text
# Un chapitre

Un paragraphe avec du **gras**, une citation @dupont[p. 45] et une note.^[Une précision.]

{{Jn 3.16 ; "cf. aussi" ; Gn 1.1}}

{{e type=note titre="À retenir"|Une idée mise en valeur.}}
```

Les références bibliques, les citations, les notes et les blocs de mise en
page sont décrits dans le [guide d'écriture](docs/guide/ecrire.md).

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
- les diagnostics du compilateur sont comptés, mais pas encore soulignés
  dans le texte ;
- Zotero est consulté en lecture seule ;
- l'historique d'annulation peut repartir après certains remplacements
  de symboles.

Les sauvegardes automatiques et les copies précédentes sont là pour les
accidents ; elles ne remplacent pas une sauvegarde personnelle des travaux
importants.

## Documentation

Le manuel d'usage est dans [`docs/guide/`](docs/guide/index.md) : prise en
main, écriture, bibliographie, glossaire, normes, recherche, raccourcis,
fichiers et installation. Les notes de version sont dans
[`CHANGELOG.md`](CHANGELOG.md) et les temps forts de cette version dans
[`RELEASE_NOTES.md`](RELEASE_NOTES.md).

## Licence

MIT — voir [`LICENSE`](LICENSE).
