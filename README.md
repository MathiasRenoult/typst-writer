# Typst Writer

Un éditeur pour **écrire des documents en Typst** — travaux, mémoires, livres.
Le rendu affiché est celui du document final, parce que c'est le vrai compilateur
Typst qui le produit, et non un aperçu approximatif.

L'ambition tient en une phrase : que la mise en page ne demande jamais d'y
penser. On écrit, le document se compile dès que l'écriture s'arrête, et l'on voit
ce que le lecteur verra.

> **Statut : version 0.2 — alpha.** L'application est utilisable au quotidien —
> c'est avec qu'elle s'écrit — mais elle change encore beaucoup. Les limites
> connues sont écrites plus bas, sans fard.

## Écrire dans `document.tw`

L'écriture se fait dans une **couche simple**, `document.tw`, qui se lit et se
compare ligne à ligne. Le Typst reste le maître du rendu : tout ce que la couche
ne sait pas exprimer passe par un bloc brut.

- titres `# Titre`, `## Titre` ; les blocs se séparent par des lignes vides ;
- **citations** : `@cle[p. 45]`, et plusieurs d'un geste — `@dupont[p. 45];
  @martin` — qui deviennent autant de `#cite` à la génération ;
- **notes libres** : `^[ce que l'on veut]` ;
- le reste — tableaux, formules, Typst à la main — dans un bloc brut ;
- ce qui n'est pas reconnu à la migration d'un ancien projet `.typ` reste bloc
  brut, sans réécriture approximative.

La conception complète — la syntaxe, l'AST et les deux traductions — est écrite
dans `docs/design/langage-intermediaire.md`.

## Les citations en glyphes

Une citation est un **glyphe numéroté**, dans le fil du texte — ¹, ², ³ — qui
suit le compteur du document, appels de notes et notes libres mélangés dans
l'ordre. La note est un **caractère entier** : elle se déplace, se sélectionne et
se supprime d'un seul geste, jamais lettre à lettre.

- **cliquer sur un glyphe** est la seule porte d'édition : la fenêtre du
  composeur s'ouvre, et la note se déplie au pied de la page sous le curseur ;
- le composeur se conduit **aux flèches** : la note ligne par ligne, puis les
  clés ; `Échap` abandonne sans laisser de trace — le document ne reçoit la note
  qu'à la fin de la composition ;
- remplacer une citation ne touche jamais au texte d'alentour, et une seule
  annulation (`Ctrl+Z`) reprend tout le geste.

## Les références

Le panneau latéral les liste **par auteur, prénom et titre** — jamais par clé —
et abrège ce qui dépasse. Le crayon d'une ligne ouvre la fenêtre d'édition :
nom, prénom, titre, année.

- une **bibliothèque locale** s'écrit : la fenêtre réécrit les champs de l'entrée
  dans `references.bib`, et rien d'autre ;
- une collection **Zotero** pointée se lit, jamais : la règle du projet est que
  seules les bibliothèques locales s'écrivent.

## La mise en page

La fenêtre des normes tient le papier, les marges, le texte, les titres, les
numéros de page et les couleurs. Ce qu'elle montre est une copie de travail :
seul « Appliquer » touche le projet, et les valeurs fautives sont dites champ par
champ avant que quoi que ce soit ne soit écrit.

- les choix finis sont des **listes déroulantes** : format, orientation, langue,
  numérotation des chapitres, forme et position des numéros, casse ;
- chaque paramètre porte son **infobulle** : laisser la souris dit ce qu'il
  règle et sous quelle forme l'écrire — toute mesure Typst est acceptée
  (`2cm`, `1in`, `24pt`).

## Les thèmes

Onze thèmes sont livrés avec l'application — Everforest et sa nuit, Moka et sa
nuit, Gruvbox crème et nuit, Pin rosé et sa nuit, Solarized lumière et pénombre,
Kanagawa — auxquels s'ajoutent le clair et le sombre d'origine. Ils sont semés
une fois dans le dossier des thèmes, puis appartiennent à l'utilisateur : une
mise à jour ne réécrit jamais un thème modifié.

Un thème est un petit fichier TOML — papier, encre, accent, lisière, couleurs de
l'éditeur et de la coloration. La sélection se teinte par la **bordure**, jamais
par le bloc de couleur : l'interface reste discrète, et ce qui est actif se voit
au trait.

## Rendu et assistance

- rendu fidèle, page par page, avec le compilateur Typst 0.15.1 ; mise à jour
  **après deux secondes sans frappe**, ou dès un retour à la ligne — une seule
  compilation par mise à jour, jamais de clignotement ;
- export PDF ;
- mode focus (F11) : le texte, rien que le texte ;
- [tinymist](https://github.com/Myriad-Dreamin/tinymist) en assistance :
  complétion après `#` et après `@` (les clés de la bibliographie), survol
  (`Ctrl+I`), aller à la définition (`F12`), renommage d'un symbole avec
  l'aperçu de **tout** ce que cela changerait, fichier par fichier, avant que
  rien ne s'écrive ; diagnostics comptés dans la barre d'état. tinymist est
  **facultatif** : sans lui, tout le reste fonctionne, et son absence se dit
  sans alarme.

## Installation

Les versions se téléchargent dans les **publications** : un binaire Linux
x86_64, à décompresser et à lancer tel quel. Il faut un Linux avec Wayland ou
X11, et un pilote de carte graphique offrant Vulkan. Rien d'autre : les
bibliothèques graphiques se chargent à l'exécution.

Sinon, la chaîne Rust à jour suffit (édition 2024) :

```sh
cargo build --release
./target/release/typst-writer
```

Pour le développement, `cargo run` est utilisable au quotidien : le profil de
développement optimise les **dépendances**, dont le travail est lourd — compiler
Typst et rasteriser une page y coûte 334 ms et 91 ms sans optimisation, contre
15 ms et 29 ms avec. Les assertions et les points d'arrêt du code de
l'application sont conservés.

## Les raccourcis

| Touche | Effet |
| --- | --- |
| `Ctrl+N` | nouveau chapitre |
| `Ctrl+O` | ouvrir un fichier `​.typ` (il rejoint le projet) |
| `Ctrl+S` | enregistrer |
| `Ctrl+E` | exporter en PDF |
| `F11` | mode focus — `Échap` en sort |
| `F12` | aller à la définition de ce qui est sous le curseur |
| `Ctrl+I` | ce que le serveur sait du symbole sous le curseur |
| `Échap` | fermer ce qui s'est ouvert en dernier |

Dans le composeur de citations : les flèches parcourent la note puis les clés,
`Échap` abandonne sans rien écrire. Dans les listes de complétion : les flèches
parcourent, `Entrée` ou `Tab` accepte, `Échap` ferme ; une liste sans résultat
se ferme d'elle-même.

## Où vivent vos fichiers

    ~/.typst-writer/
    ├── session.json          l'état de la dernière session (projet ouvert, thème…)
    ├── themes/               les thèmes : livrés une fois, puis à vous
    └── projets/
        └── mon-travail/
            ├── document.tw   ce que vous écrivez : la couche simple
            ├── main.typ      les normes de mise en page, et l'assemblage
            ├── references.bib la bibliographie
            └── chapitres/    les sources .typ héritées du projet

## Notes pour les curieux

- **`vendor/iced-code-editor`** : le crate d'édition est livré **modifié**. Le
  diff par rapport à l'amont est documenté dans
  `vendor/iced-code-editor/PATCHES.md`, et sa licence dans le même dossier. Aucun
  correctif n'y est gratuit : chacun répare une chose qu'aucun réglage ne permet
  (le retour à la ligne au mot, la citation comme caractère entier, le clic qui
  tombe où l'on clique, la frappe qui reste du texte après un glyphe, et les
  touches réservées à la liste de complétion).
- **Tout ce qui touche au serveur de langage est éprouvé contre le vrai
  tinymist**, par des épreuves qui ne tournent qu'à la demande :

  ```sh
  cargo test --release -- --ignored
  ```

## Limites connues

Elles sont écrites ici parce qu'elles sont réelles, et parce qu'une application
qui se tait sur ses manques se rend inutilisable.

- Les diagnostics de tinymist ne sont **pas** soulignés dans le texte : ils sont
  comptés dans la barre d'état. Le crate d'édition n'expose encore aucune API de
  diagnostic, et dessiner demande une retouche de plus.
- Le renommage d'un symbole **écrit par remplacement** les fichiers que le
  serveur désigne : le document ouvert est rouvert sur son contenu neuf, et son
  histoire d'annulation part avec. Tant que l'aperçu n'a pas été montré, le
  bouton d'application n'est pas même actif.
- tinymist ne résout pas une référence d'un chapitre vers un **autre** chapitre :
  sa résolution s'arrête au fichier ouvert (c'est mesuré). L'application cherche
  donc elle-même les étiquettes `<nom>`, ce qui fait fonctionner le geste.
- L'aller à la définition d'une **citation** ne mène nulle part : la
  bibliographie appartient à l'assemblage, et son entrée n'est pas un fichier
  éditable ici.

## Licence

MIT — voir `LICENSE`. Le crate `iced-code-editor` est distribué sous MIT
également : voir `vendor/iced-code-editor/LICENSE`.
