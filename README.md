# Typst Writer

Un éditeur pour **écrire des documents en Typst** — travaux, mémoires, livres.
Le rendu affiché est celui du document final, parce que c'est le vrai compilateur
Typst qui le produit, et non un aperçu approximatif.

L'ambition tient en une phrase : que la mise en page ne demande jamais d'y
penser. On écrit, le document se compile dès que l'écriture s'arrête, et l'on voit
ce que le lecteur verra.

> **Statut : alpha fermée 0.1.** L'application est utilisable au quotidien —
> c'est avec qu'elle s'écrit — mais elle change encore beaucoup. Les limites
> connues sont écrites plus bas, sans fard.

## Ce que sait faire l'application

**Écrire**

- rendu fidèle, page par page, avec le compilateur Typst 0.15.1 ;
- mise à jour **après deux secondes sans frappe**, ou dès un retour à la ligne —
  une seule compilation par mise à jour, jamais de clignotement ;
- mise en page de rédaction : corps 17 px, interligne 1,6, retour à la ligne aux
  frontières de mots, pas de numéros de ligne, pas de repliage, pas de parenthèses
  ou d'apostrophes automatiques ;
- mode focus (F11) : le texte, rien que le texte ;
- thèmes clair, sombre, ou celui du système — aucune couleur n'est décidée en dur.

**Se faire assister** (avec [tinymist](https://github.com/Myriad-Dreamin/tinymist))

- liste de complétion après `#` (commandes, fonctions, champs) **et** après `@`,
  qui propose les clés de votre bibliographie ;
- survol (Ctrl+I) et aller à la définition (F12) ;
- renommage d'un symbole, avec l'aperçu de **tout** ce que cela changerait,
  fichier par fichier, avant que rien ne s'écrive ;
- diagnostics du document, comptés dans la barre d'état ;
- tinymist est **facultatif** : sans lui, tout le reste fonctionne, et son absence
  se dit sans alarme.

**Tenir un projet**

- un projet est un dossier : ses chapitres dans `chapitres/`, et un assemblage
  `main.typ` qui les réunit ;
- un chapitre par fichier, import d'un `.typ` existant, export PDF ;
- icônes Lucide, infobulles, panneaux que l'on redimensionne.

## Installation

L'alpha se télécharge dans les **publications** : un binaire Linux x86_64, à
décompresser et à lancer tel quel. Il faut un Linux avec Wayland ou X11, et un
pilote de carte graphique offrant Vulkan. Rien d'autre : les bibliothèques
graphiques se chargent à l'exécution.

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

Dans la liste de complétion : les flèches parcourent, `Entrée` ou `Tab` accepte,
`Échap` ferme. Une liste sans résultat se ferme d'elle-même.

## Où vivent vos fichiers

    ~/.typst-writer/
    ├── session.json          l'état de la dernière session (projet ouvert, thème…)
    └── projets/
        └── mon-travail/
            ├── main.typ      l'assemblage : c'est lui qui est compilé
            └── chapitres/
                ├── 01-intro.typ
                └── 02-suite.typ

## Notes pour les curieux

- **`vendor/iced-code-editor`** : le crate d'édition est livré **modifié**. Le
  diff par rapport à l'amont est documenté dans
  `vendor/iced-code-editor/PATCHES.md`, et sa licence dans le même dossier. Aucun
  correctif n'y est gratuit : chacun répare une chose qu'aucun réglage ne permet
  (le retour à la ligne au mot, les entrées tinymist, une palette de coloration
  que son cache suit, et les touches réservées à la liste de complétion).
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
