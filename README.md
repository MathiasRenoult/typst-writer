# Typst Writer

Un éditeur pour écrire des documents en Typst — thèses, mémoires, livres.
L'aperçu affiché est celui du document final : c'est le vrai compilateur
Typst qui le compose.

**Version 0.3.0-alpha.** Utilisable au quotidien — c'est avec elle
qu'elle s'écrit — mais elle change encore.

## Ce qu'elle fait

- **Écrire** dans une couche markdown lisible, `document.tw` — titres,
  emphase, citations, notes, liens, images, tableaux, formules,
  graphiques, décor — qui se compare ligne à ligne, et dont le Typst
  reste le maître du rendu ;
- **Composer** dès que l'écriture s'arrête : la page se montre telle que
  le lecteur la verra, et l'aperçu suit le chapitre sous le curseur ;
- **Travailler en espace** : panneaux divisibles, onglets par panneau,
  même document dans plusieurs vues (édition / rendu / Typst), disposition
  restaurée au lancement ;
- **Citer** : une bibliographie par projet, une fiche complète (éditeur,
  lieu, titre court, type de document), et un projet lié à une collection
  **Zotero** — lue, jamais réécrite ;
- **Mettre en page** par des normes — papier, marges, titres, couleurs,
  typographie par langue, décor, en-tête et pied de page — jamais par des
  mesures portées dans le texte ;
- **Chercher** en expressions régulières, avec remplacement à rappel de
  groupes ;
- **Personnaliser** l'apparence : onze thèmes livrés, un éditeur de thème
  intégré, des étagères pour classer les projets ;
- **Exporter** en PDF, et travailler au clavier d'un bout à l'autre —
  `F1` montre toutes les touches.

## Écrire

Le document se compose en markdown :

    # Le titre

    Un paragraphe, avec du **gras** et une citation — @dupont[p. 45].

    | Mois | Ventes |
    | --- | :-: |
    | Jan | 10 |

    Soit $x^2$ dans le texte.

    {graphique type=barre titre="Évolution"}
    Jan | 10
    Fév | 20

Les mesures de mise en page vivent dans les normes du document ; la
source dit l'intention. La syntaxe entière est dans
[la documentation](docs/guide/ecrire.md).

## L'espace de travail

L'éditeur se scinde en panneaux (glisser un bord), chacun avec ses
onglets. Le même document peut s'ouvrir dans plusieurs vues — texte
partagé, aperçus indépendants. La disposition se retrouve au lancement.
La bibliothèque de projets se parcourt en galerie plein écran, avec
couvertures et étagères.

## Installation

Les versions se téléchargent dans les **publications** : un binaire Linux
x86_64, à décompresser et à lancer tel quel. Il faut un Linux avec
Wayland ou X11, et un pilote graphique offrant Vulkan.

Ou, avec une chaîne Rust à jour :

```sh
cargo build --release
./target/release/typst-writer
```

L'assistance à la frappe s'appuie sur
[tinymist](https://github.com/Myriad-Dreamin/tinymist), qui est
**facultatif** : sans lui, tout le reste fonctionne.

## Documentation

Le manuel d'usage est dans [`docs/guide/`](docs/guide/index.md) — prise
en main, écriture, bibliographie, normes, recherche, raccourcis, fichiers.

Il devient un site d'une commande, sans dépendance :

```sh
python3 site/construire.py
```

Le site se lit dans `site/dist/`.

## Qualité

- **395 tests**, clippy à zéro, `cargo deny` sans erreur ;
- **zéro `unsafe`** dans le code applicatif ;
- les licences des dépendances sont auditées (`deny.toml`) : MIT,
  Apache-2.0, BSD, ISC, Zlib — toutes permissives, compatibles avec la
  licence du projet ;
- les écritures sont sûres : corbeille pour les suppressions, verrou
  d'instance, sauvegarde automatique.

## Limites connues

- les diagnostics du compilateur ne sont pas soulignés dans le texte :
  ils sont comptés dans la barre d'état ;
- le renommage d'un symbole écrit par remplacement les fichiers visés, et
  l'histoire d'annulation du document ouvert repart avec ;
- l'aller à la définition d'une citation ne mène nulle part : la
  bibliographie n'est pas un fichier éditable ici ;
- les graphiques utilisent le paquet `cetz` de Typst — la première
  compilation le télécharge.

## Licence

MIT — voir `LICENSE`.
